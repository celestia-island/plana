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

    /// Send a message to a session. Returns true if delivered.
    pub fn send(&self, session_id: &str, msg: &str) -> bool {
        if let Some(tx) = self.sessions.lock().unwrap().get(session_id) {
            tx.unbounded_send(msg.to_string()).is_ok()
        } else {
            false
        }
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
    let stream = rx.map(move |msg| Ok(Event::default().data(msg)));
    Sse::new(stream)
}
