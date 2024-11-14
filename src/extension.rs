use crate::buffer::{PutRecord, StreamAggregator};
use crate::sink::RecordSink;
use crate::sinks::kinesis::KinesisSink;
use axum::extract::State;
use axum::http::header::CONTENT_LENGTH;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{routing::get, Json, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};

pub struct ExtensionApp {
    app: Router,
    agg: Arc<Mutex<StreamAggregator<KinesisSink>>>,
}

impl ExtensionApp {
    pub async fn new() -> Self {
        let sink = KinesisSink::new(None).await;
        let agg = Arc::new(Mutex::new(StreamAggregator::new(1, sink)));

        let app = Router::new().route("/", get(ExtensionApp::root)).route(
            "/add",
            post({
                let client = Arc::clone(&agg);
                move |headers: HeaderMap, payload: Json<PutRecord>| {
                    ExtensionApp::put_records(State(client), payload, headers)
                }
            }),
        );

        ExtensionApp { app, agg }
    }

    async fn root() -> &'static str {
        "OK"
    }

    async fn put_records<S>(
        State(aggregator): State<Arc<Mutex<StreamAggregator<S>>>>,
        Json(payload): Json<PutRecord>,
        headers: HeaderMap,
    ) -> StatusCode
    where
        S: RecordSink + Send + Sync + 'static,
    {
        match headers
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            None => StatusCode::BAD_REQUEST,
            Some(content_length) => {
                aggregator
                    .lock()
                    .await
                    .insert(payload.clone().stream_name, payload, content_length)
                    .await;
                StatusCode::NO_CONTENT
            }
        }
    }

    pub async fn listen(self, shutdown_rx: oneshot::Receiver<()>) -> std::io::Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                match axum::serve(listener, self.app.clone())
                    .with_graceful_shutdown(async {
                        shutdown_rx
                            .await
                            .expect("Failed to receive shutdown signal");
                    })
                    .await
                {
                    Ok(_) => {
                        self.shutdown().await;
                    }
                    Err(e) => eprintln!("Server encountered an error: {}", e),
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to bind address {}: {}", addr, e);
                Err(e)
            }
        }
    }

    pub async fn shutdown(self) {
        println!("Shutting down...");
        self.agg.lock().await.close().await;
        println!("Shutdown complete");
    }
}
