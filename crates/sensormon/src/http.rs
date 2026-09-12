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
    pub chirps: Option<Arc<Mutex<crate::chirps::ChirpStore>>>,
    /// Maintained by the watchdog thread in `main::run`.
    pub stalled: Arc<Mutex<crate::runtime::Stalled>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/events", get(events))
        .route("/stats", get(stats))
        .route("/health", get(health))
        .route("/chirps", get(chirps_list))
        .route("/chirps/stats", get(chirps_stats))
        .route("/chirps/groups", get(chirps_groups))
        .route("/chirps/:id", get(chirp_get))
        .route("/chirps/:id/iq", get(chirp_iq))
        .with_state(state)
}

/// 200 "ok" while every receiver delivers samples; 503 naming the stalled
/// receivers otherwise (the watchdog exits the process shortly after, so this
/// mostly matters for external monitoring).
async fn health(State(st): State<AppState>) -> Response {
    let stalled = st.stalled.lock().unwrap().clone();
    if stalled.receivers.is_empty() {
        "ok\n".into_response()
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, format!("stalled: {}\n", stalled.receivers.join(", "))).into_response()
    }
}

#[derive(serde::Deserialize, Default)]
pub struct ChirpQuery {
    pub group: Option<i64>,
    pub receiver: Option<String>,
    pub since: Option<f64>,
    pub limit: Option<i64>,
    pub min_count: Option<i64>,
}

fn no_store() -> Response {
    (axum::http::StatusCode::NOT_FOUND, "chirp store not configured ([chirps] in sensormon.toml)\n").into_response()
}

async fn chirps_stats(State(st): State<AppState>) -> Response {
    let Some(store) = st.chirps else { return no_store() };
    let r = store.lock().unwrap().stats();
    match r {
        Ok(s) => Json(serde_json::to_value(s).unwrap()).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}\n")).into_response(),
    }
}

async fn chirps_groups(State(st): State<AppState>, axum::extract::Query(q): axum::extract::Query<ChirpQuery>) -> Response {
    let Some(store) = st.chirps else { return no_store() };
    let r = store.lock().unwrap().groups(q.receiver.as_deref(), q.min_count.unwrap_or(1));
    match r {
        Ok(g) => Json(serde_json::to_value(g).unwrap()).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}\n")).into_response(),
    }
}

async fn chirps_list(State(st): State<AppState>, axum::extract::Query(q): axum::extract::Query<ChirpQuery>) -> Response {
    let Some(store) = st.chirps else { return no_store() };
    let r = store.lock().unwrap().list(q.group, q.receiver.as_deref(), q.since, q.limit.unwrap_or(200).min(10_000));
    match r {
        Ok(l) => Json(serde_json::to_value(l).unwrap()).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}\n")).into_response(),
    }
}

async fn chirp_get(State(st): State<AppState>, axum::extract::Path(id): axum::extract::Path<i64>) -> Response {
    let Some(store) = st.chirps else { return no_store() };
    let r = store.lock().unwrap().get_iq(id);
    match r {
        Ok(Some((m, _))) => Json(serde_json::to_value(m).unwrap()).into_response(),
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "no such chirp\n").into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}\n")).into_response(),
    }
}

/// Raw baseband IQ of one chirp: int8 interleaved I/Q (rtl_433 "cs8"), peak-normalised,
/// already mixed to the burst's center; `X-Sample-Rate` and `X-Center-Hz` headers.
async fn chirp_iq(State(st): State<AppState>, axum::extract::Path(id): axum::extract::Path<i64>) -> Response {
    let Some(store) = st.chirps else { return no_store() };
    let r = store.lock().unwrap().get_iq(id);
    match r {
        Ok(Some((m, iq))) => {
            let mut h = axum::http::HeaderMap::new();
            h.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
            h.insert("x-sample-rate", m.sample_rate.to_string().parse().unwrap());
            h.insert("x-center-hz", format!("{:.0}", m.center_hz).parse().unwrap());
            h.insert("x-format", "cs8".parse().unwrap());
            h.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"chirp_{}_{:.0}Hz_{}sps.cs8\"", m.id, m.center_hz, m.sample_rate).parse().unwrap());
            (h, iq).into_response()
        }
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "no such chirp\n").into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}\n")).into_response(),
    }
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
