//! Transport sessions for JSON-RPC SSE streaming.
//!
//! Semantic boundary: a session here is a *transport channel* — an id mapped
//! to an SSE sender for pushing serialized notifications to one client
//! connection. It intentionally carries no identity or authentication
//! state. Identity sessions (who is logged in, token lifecycle, revocation)
//! belong to `kirino-session` in the kirino L0 auth layer; do not conflate
//! the two concepts.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::Query,
    response::sse::{Event, Sse},
};
use futures::channel::mpsc;
use futures::StreamExt;
use serde::Deserialize;
use std::convert::Infallible;
use uuid::Uuid;

pub type SessionId = String;
type SessionSender = mpsc::UnboundedSender<String>;

#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<SessionId, SessionSender>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new session and return its ID. The session is not yet
    /// ready for streaming until the client opens the SSE endpoint.
    pub fn create_id(&self) -> SessionId {
        let id = Uuid::new_v4().to_string();
        self.sessions
            .lock()
            .unwrap()
            .insert(id.clone(), mpsc::unbounded().0);
        id
    }

    /// Send a message to a session. Returns true if delivered. A session
    /// whose receiver is gone (client disconnected / never subscribed) is
    /// RECLAIMED on the spot: without this the map grew once per request
    /// forever (round-8's leak — `create_id` inserts, nothing ever removed).
    pub fn send(&self, session_id: &str, msg: &str) -> bool {
        let mut guard = self.sessions.lock().unwrap();
        if let Some(tx) = guard.get(session_id) {
            let ok = tx.unbounded_send(msg.to_string()).is_ok();
            if !ok {
                guard.remove(session_id);
            }
            ok
        } else {
            false
        }
    }

    /// Explicitly drop a session entry (callers that know a session is
    /// finished). Idempotent.
    pub fn close(&self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);
    }

    pub fn exists(&self, session_id: &str) -> bool {
        self.sessions.lock().unwrap().contains_key(session_id)
    }
}

#[derive(Deserialize)]
pub struct EventsQuery {
    pub session: String,
}

/// SSE endpoint: `GET /api/rpc/events?session=<uuid>`
pub async fn sse_events_handler(
    Query(q): Query<EventsQuery>,
    sessions: axum::extract::State<SessionManager>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    sse_events_handler_impl(sessions.0, q.session).await
}

/// Direct implementation for backends that hold SessionManager in their own state.
pub async fn sse_events_handler_impl(
    sessions: SessionManager,
    session_id: String,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::unbounded();
    sessions
        .sessions
        .lock()
        .unwrap()
        .insert(session_id.clone(), tx);
    // END-OF-STREAM RECLAMATION (round-9's V9-3): the send-failure
    // path (round-8) only reclaims when something pushes NEXT; a
    // stream that completes normally (last push delivered, client
    // disconnects after) left its entry resident forever. When the SSE
    // stream drops — client gone or response finished — the entry
    // goes with it.
    let janitor = SessionJanitor {
        sessions: std::sync::Arc::clone(&sessions.sessions),
        session_id: session_id.clone(),
    };
    let stream = rx
        .map(move |msg| Ok(Event::default().data(msg)))
        .chain(futures::stream::unfold(janitor, |_j| async {
            // Runs when the source rx is EXHAUSTED (the tx dropped);
            // the janitor then drops with the chain, closing the entry.
            None::<(Result<Event, Infallible>, SessionJanitor)>
        }));
    Sse::new(stream)
}

/// Closes a session entry when dropped — the end-of-stream reclamation
/// for the SSE subscription path.
struct SessionJanitor {
    sessions: std::sync::Arc<std::sync::Mutex<HashMap<SessionId, SessionSender>>>,
    session_id: String,
}

impl Drop for SessionJanitor {
    fn drop(&mut self) {
        self.sessions.lock().unwrap().remove(&self.session_id);
    }
}

#[cfg(test)]
mod reclamation_tests {
    use super::*;
    use futures::FutureExt;

    /// Round-8's leak gate: a session whose receiver is gone (never
    /// subscribed, or disconnected) must be reclaimed on the failed
    /// push — the map may not grow once per request forever.
    #[test]
    fn a_dead_receiver_is_reclaimed_on_the_next_push() {
        let mgr = SessionManager::default();
        let sid = mgr.create_id();
        // No receiver was ever attached (create_id drops the rx) — the
        // first push fails and must reclaim the entry.
        assert!(!mgr.send(&sid, "x"), "no receiver: undeliverable");
        assert!(!mgr.exists(&sid), "the dead entry is reclaimed");
        // A second push is a plain miss, never a panic.
        assert!(!mgr.send(&sid, "y"));
    }

    #[test]
    fn a_live_receiver_stays_registered() {
        let mgr = SessionManager::default();
        let sid = mgr.create_id();
        // Attach a live receiver the way the SSE handler does.
        let (tx, mut rx) = futures::channel::mpsc::unbounded();
        mgr.sessions.lock().unwrap().insert(sid.clone(), tx);
        assert!(mgr.send(&sid, "hello"));
        assert!(mgr.exists(&sid), "live sessions stay");
        use futures::StreamExt;
        assert_eq!(rx.next().now_or_never().unwrap().unwrap(), "hello");
        // close() removes explicitly and idempotently.
        mgr.close(&sid);
        mgr.close(&sid);
        assert!(!mgr.exists(&sid));
    }
}

#[cfg(test)]
mod janitor_tests {
    use super::*;

    /// Round-9's V9-3 gate: a NORMALLY-completed subscription (the tx
    /// dropped after the last message, no failed push ever) must still
    /// reclaim its entry — the send-failure path alone left these
    /// resident forever.
    #[tokio::test]
    async fn a_completed_subscription_reclaims_its_entry() {
        let mgr = SessionManager::default();
        let sid = mgr.create_id();
        let sse = sse_events_handler_impl(SessionManager::clone(&mgr), sid.clone()).await;
        // The subscription is live: the entry exists.
        assert!(mgr.exists(&sid));
        // Drop the SSE response (the client disconnects / the stream
        // finishes) — the janitor must close the entry.
        drop(sse);
        assert!(
            !mgr.exists(&sid),
            "a completed subscription must not outlive its stream"
        );
    }
}
