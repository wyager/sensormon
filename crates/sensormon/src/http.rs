//! HTTP API: `GET /events` streams JSON lines (one `Event` per line, kept alive
//! with a blank line every few seconds); `GET /stats`; `GET /health`.

use crate::runtime::Runtime;
use axum::body::Body;
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use axum::Router;
use futures::stream::StreamExt;
use sensormon_core::Event;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<Mutex<Runtime>>,
    pub events: broadcast::Sender<Arc<Event>>,
}

pub fn router(state: AppState) -> Router {
    Router::new().route("/events", get(events)).route("/stats", get(stats)).route("/health", get(|| async { "ok" })).with_state(state)
}

async fn events(State(st): State<AppState>) -> Response {
    let rx = st.events.subscribe();
    let lines = BroadcastStream::new(rx).filter_map(|item| async move {
        match item {
            Ok(ev) => serde_json::to_string(&*ev).ok().map(|s| Ok::<_, std::io::Error>(format!("{s}\n"))),
            Err(_) => None, // lagged subscriber: skip
        }
    });
    let keepalive = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(std::time::Duration::from_secs(5))).map(|_| Ok::<_, std::io::Error>("\n".to_string()));
    let body = Body::from_stream(futures::stream::select(lines, keepalive));
    ([(header::CONTENT_TYPE, "application/x-ndjson"), (header::CACHE_CONTROL, "no-cache")], body).into_response()
}

async fn stats(State(st): State<AppState>) -> Json<serde_json::Value> {
    let status = st.runtime.lock().unwrap().status();
    Json(serde_json::json!({ "receivers": status }))
}
