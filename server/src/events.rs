use actix_web::{Error, HttpResponse, web};
use bytes::Bytes;
use futures_util::stream::{self};
use serde::Serialize;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error};

use crate::server::AppState;

#[derive(Debug, Serialize, Clone)]
pub struct ProgressEvent {
    pub event_type: String,
    pub message: String,
    pub progress: Option<f32>,
}

impl ProgressEvent {
    pub fn new(event_type: &str, message: &str, progress: Option<f32>) -> Self {
        Self {
            event_type: event_type.to_string(),
            message: message.to_string(),
            progress,
        }
    }

    pub fn to_sse_format(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_else(|e| {
            error!("Failed to serialize event: {}", e);
            "{}".to_string()
        });
        format!("data: {}\n\n", json)
    }
}

pub async fn sse_events(data: web::Data<AppState>) -> HttpResponse {
    debug!("Client connected to SSE events endpoint");
    let rx = data.event_channel.subscribe();

    // Create a stream that combines events with keepalive pings
    // We need to wrap the state in Arc<Mutex<>> to avoid ownership issues
    let stream = stream::unfold(
        (rx, interval(Duration::from_secs(15))),
        |(mut rx, mut keepalive)| async move {
            tokio::select! {
                _ = keepalive.tick() => {
                    debug!("Sending SSE keepalive");
                    Some((Ok::<_, Error>(Bytes::from(":\n\n")), (rx, keepalive)))
                }
                event = rx.recv() => {
                    match event {
                        Ok(progress_event) => {
                            debug!("Sending event: {:?}", progress_event);
                            let bytes = Bytes::from(progress_event.to_sse_format());
                            Some((Ok::<_, Error>(bytes), (rx, keepalive)))
                        }
                        Err(e) => {
                            error!("SSE channel error: {}", e);
                            None
                        }
                    }
                }
            }
        },
    );

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}
