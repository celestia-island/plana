//! Transport sessions for JSON-RPC SSE streaming.
//!
//! Semantic boundary: a session here is a *transport channel* — an id mapped
//! to an SSE sender for pushing serialized notifications to one client
//! connection. It intentionally carries no identity or authentication
//! state. Identity sessions (who is logged in, token lifecycle, revocation)
//! belong to `kirino-session` in the kirino L0 auth layer; do not conflate
//! the two concepts.
//!
//! # The session id is a bearer credential (audit R1-JR-1)
//!
//! Whoever knows the session id can mount the SSE endpoint and receive
//! everything pushed to it — the mount does not (and cannot) prove more
//! than possession of the id, exactly like the DeferredOps registry's
//! bearer model. The ids are UUIDv4 (unguessable); callers hand them to
//! their own client over an authenticated channel and must treat them as
//! secrets (they surface in access logs via the query string). A mount
//! over an id that is already live replaces the sender — legitimate
//! reconnect use that shape — and logs a warning so a hijack attempt is
//! at least visible in the trace.
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use axum::{
    extract::Query,
    response::sse::{Event, Sse},
};
use futures::StreamExt;
use serde::Deserialize;
use std::convert::Infallible;
use uuid::Uuid;

pub type SessionId = String;

/// Per-session send-queue depth (audit R1-JR-2): a slow SSE consumer can
/// no longer grow its queue without bound — once the queue is full the
/// session is treated as undeliverable and reclaimed; the client's
/// EventSource reconnects fresh.
const SESSION_QUEUE_CAPACITY: usize = 256;

/// Soft ceiling on live sessions per manager (audit R1-JR-1): at the
/// ceiling the OLDEST session is evicted to make room, so the map can
/// never grow without bound no matter how many ids are minted. A client
/// whose session is evicted this way gets a fresh id on its next request
/// (its SSE stream's janitor cleans the old entry).
const MAX_SESSIONS: usize = 1_000;

/// The per-session sender is a **tokio** bounded channel, not a futures
/// one: futures' `try_send` ignores the capacity entirely (a probe with
/// buffer=2 accepted sends 3 AND 4), so the queue bound would be
/// unenforceable from the synchronous `send` path. tokio's `try_send`
/// fails for real once the queue is full.
type SessionSender = tokio::sync::mpsc::Sender<String>;

/// Bridge a tokio receiver into a futures stream for axum's SSE
/// (poll_recv is the whole adapter; no tokio-stream dependency).
/// Receiver is Unpin, so this stream is too.
struct RxStream {
    rx: tokio::sync::mpsc::Receiver<String>,
}
impl futures::Stream for RxStream {
    type Item = String;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<String>> {
        self.rx.poll_recv(cx)
    }
}

#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<SessionId, SessionSender>>>,
    /// Insertion order of live session ids — FIFO eviction at
    /// [`MAX_SESSIONS`].
    insertion_order: Arc<Mutex<VecDeque<SessionId>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            insertion_order: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Create a new session and return its ID. The session is not yet
    /// ready for streaming until the client opens the SSE endpoint.
    ///
    /// At [`MAX_SESSIONS`] live sessions the oldest is evicted first —
    /// create never fails, and the map never grows without bound
    /// (audit R1-JR-1: the ceiling used to be absent, so minting ids was
    /// an unbounded-memory lever).
    pub fn create_id(&self) -> SessionId {
        let id = Uuid::new_v4().to_string();
        let mut guard = self.sessions.lock().unwrap();
        if guard.len() >= MAX_SESSIONS {
            let mut order = self.insertion_order.lock().unwrap();
            while guard.len() >= MAX_SESSIONS {
                match order.pop_front() {
                    Some(oldest) => {
                        guard.remove(&oldest);
                    }
                    None => break,
                }
            }
        }
        self.insertion_order.lock().unwrap().push_back(id.clone());
        guard.insert(
            id.clone(),
            tokio::sync::mpsc::channel(SESSION_QUEUE_CAPACITY).0,
        );
        id
    }

    /// Send a message to a session. Returns true if delivered. Two
    /// reclaim paths keep the map honest:
    /// - a session whose receiver is gone (client disconnected / never
    ///   subscribed) is RECLAIMED on the spot (round-8's leak);
    /// - a session whose queue is FULL (slow consumer, audit R1-JR-2) is
    ///   likewise reclaimed — bounded memory beats buffering a wedged
    ///   client forever, and the client's EventSource reconnects fresh.
    pub fn send(&self, session_id: &str, msg: &str) -> bool {
        let mut guard = self.sessions.lock().unwrap();
        if let Some(tx) = guard.get(session_id) {
            let ok = tx.try_send(msg.to_string()).is_ok();
            if !ok {
                guard.remove(session_id);
                self.forget_order(session_id);
            }
            ok
        } else {
            false
        }
    }

    /// Drop a session id from the FIFO order (all removal paths).
    fn forget_order(&self, session_id: &str) {
        self.insertion_order
            .lock()
            .unwrap()
            .retain(|id| id != session_id);
    }

    /// Explicitly drop a session entry (callers that know a session is
    /// finished). Idempotent.
    pub fn close(&self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);
        self.forget_order(session_id);
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
    let (tx, rx) = tokio::sync::mpsc::channel(SESSION_QUEUE_CAPACITY);
    {
        let mut guard = sessions.sessions.lock().unwrap();
        if guard.insert(session_id.clone(), tx.clone()).is_some() {
            // Audit R1-JR-1: the session id is a bearer credential; a
            // mount over a LIVE id is either a reconnect or a hijack —
            // the trace should say which happened and when.
            tracing::warn!(
                target: "plana_jsonrpc::session",
                session = %session_id,
                "SSE mount replaced a live session sender (reconnect or takeover)"
            );
        }
        // Track insertion order for the FIFO ceiling on live sessions.
        // A remounted id replaces its stale order entry first (audit R1
        // follow-up: duplicate pushes would grow the queue per reconnect).
        {
            let mut order = sessions.insertion_order.lock().unwrap();
            order.retain(|id| id != &session_id);
            order.push_back(session_id.clone());
        }
    }
    // END-OF-STREAM RECLAMATION (round-9's V9-3): the send-failure
    // path (round-8) only reclaims when something pushes NEXT; a
    // stream that completes normally (last push delivered, client
    // disconnects after) left its entry resident forever. When the SSE
    // stream drops — client gone or response finished — the entry
    // goes with it.
    // Round-30's F-30-1: the janitor keeps ITS OWN clone of the sender it
    // registered — on drop it evicts the entry ONLY IF the map still holds
    // that same receiver. A client reconnecting under the SAME session id
    // replaces the sender; the old stream's janitor must not evict the new
    // mount (the old shape left the EventSource open with events silently
    // black-holed after every reconnect race).
    let janitor = SessionJanitor {
        sessions: std::sync::Arc::clone(&sessions.sessions),
        insertion_order: std::sync::Arc::clone(&sessions.insertion_order),
        session_id: session_id.clone(),
        own_tx: tx.clone(),
    };
    // Bridge the tokio receiver into a futures stream for axum's SSE
    // via the module-level [`RxStream`] adapter (no tokio-stream
    // dependency).

    let stream = RxStream { rx }
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
    insertion_order: std::sync::Arc<std::sync::Mutex<VecDeque<SessionId>>>,
    session_id: String,
    /// The sender THIS mount registered — the drop guard compares it
    /// against the map (same_receiver) so a reconnected mount's fresh
    /// sender survives the old stream's death.
    own_tx: SessionSender,
}

impl Drop for SessionJanitor {
    fn drop(&mut self) {
        let mut guard = self.sessions.lock().unwrap();
        if let Some(current) = guard.get(&self.session_id) {
            // tokio senders expose channel identity via `same_channel`
            // (the futures counterpart was `same_receiver`).
            if current.same_channel(&self.own_tx) {
                guard.remove(&self.session_id);
                // Audit R1 follow-up (must-fix): the FIFO order queue is
                // part of the manager's state — a zombie id here would be
                // the same unbounded growth JR-1 set out to kill, just
                // moved into the helper structure.
                self.insertion_order
                    .lock()
                    .unwrap()
                    .retain(|id| id != &self.session_id);
            }
        }
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
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        mgr.sessions.lock().unwrap().insert(sid.clone(), tx);
        assert!(mgr.send(&sid, "hello"));
        assert!(mgr.exists(&sid), "live sessions stay");
        use futures::StreamExt;
        let mut rx = RxStream { rx };
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

#[cfg(test)]
mod reconnect_race_tests {
    use super::*;

    /// Round-30's F-30-1 gate: a client reconnecting under the SAME
    /// session id replaces the sender — the OLD stream's janitor must
    /// not evict the NEW mount (the old shape black-holed every event
    /// after the reconnect race; proven red before the fix).
    #[tokio::test]
    async fn an_old_janitor_cannot_evict_a_reconnected_mount() {
        let mgr = SessionManager::default();
        let sid = mgr.create_id();
        let sse1 = sse_events_handler_impl(mgr.clone(), sid.clone()).await;
        assert!(mgr.exists(&sid), "first mount registered");
        let sse2 = sse_events_handler_impl(mgr.clone(), sid.clone()).await;
        assert!(mgr.exists(&sid), "second mount replaced the sender");
        // Audit R2 follow-up: the remount REPLACED its order entry
        // (dedupe) rather than appending — without the dedupe this count
        // is 2 and the queue grows once per reconnect.
        assert_eq!(
            mgr.insertion_order
                .lock()
                .unwrap()
                .iter()
                .filter(|id| *id == &sid)
                .count(),
            1,
            "a remount must replace its stale order entry"
        );
        // The FIRST stream dies (half-open connection finally noticed).
        drop(sse1);
        assert!(
            mgr.exists(&sid),
            "the old janitor must not evict the reconnected session"
        );
        // And the new mount still delivers.
        assert!(mgr.send(&sid, "still-alive"));
        drop(sse2);
        assert!(!mgr.exists(&sid), "the last mount's janitor reclaims");
    }

    /// Audit R1-JR-2: a slow consumer (queue full) must be reclaimed —
    /// bounded memory beats buffering a wedged client forever.
    #[test]
    fn a_full_queue_reclaims_the_session() {
        let mgr = SessionManager::default();
        let sid = mgr.create_id();
        // Mount a sender whose receiver nobody drains.
        let (tx, _rx) = tokio::sync::mpsc::channel::<String>(2);
        mgr.sessions.lock().unwrap().insert(sid.clone(), tx);
        // Two sends fill the queue; the third finds it full.
        assert!(mgr.send(&sid, "1"));
        assert!(mgr.send(&sid, "2"));
        assert!(
            !mgr.send(&sid, "3"),
            "a full queue must count as undeliverable"
        );
        assert!(!mgr.exists(&sid), "the wedged session must be reclaimed");
    }

    /// Audit R1-JR-1: minting ids is bounded — at the ceiling the OLDEST
    /// session is evicted and the NEWEST survives (FIFO direction pinned
    /// both ways; a pop_back-shaped mutation must go red).
    #[test]
    fn the_session_map_stays_under_its_ceiling() {
        let mgr = SessionManager::default();
        let oldest = mgr.create_id();
        for _ in 0..(MAX_SESSIONS + 10) {
            let _ = mgr.create_id();
        }
        assert!(
            mgr.sessions.lock().unwrap().len() <= MAX_SESSIONS,
            "the map must stay at or under the ceiling"
        );
        assert!(
            !mgr.exists(&oldest),
            "FIFO: the oldest session must be the one evicted"
        );
        let newest = mgr.create_id();
        assert!(mgr.exists(&newest), "the newest session must survive");
    }

    /// Audit R1 follow-up (must-fix): the FIFO order queue is manager
    /// state too — a finished stream must not leave zombie order entries
    /// behind (that would be JR-1's unbounded growth, relocated).
    #[tokio::test]
    async fn the_fifo_order_queue_never_holds_zombies() {
        let mgr = SessionManager::default();
        let sid = mgr.create_id();
        let sse = sse_events_handler_impl(mgr.clone(), sid.clone()).await;
        assert!(mgr.exists(&sid));
        drop(sse);
        assert!(!mgr.exists(&sid), "map entry reclaimed");
        assert!(
            mgr.insertion_order.lock().unwrap().is_empty(),
            "the order queue must not hold zombie ids"
        );
    }
}
