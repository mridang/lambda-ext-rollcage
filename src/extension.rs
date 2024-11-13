use crate::buffer::PutRecord;
use aws_config;
use aws_sdk_kinesis::error::ProvideErrorMetadata;
use aws_sdk_kinesis::primitives::Blob;
use aws_sdk_kinesis::Client;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{routing::get, Json, Router};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

pub struct MyAxumApp {
    app: Router,
}

impl MyAxumApp {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Arc::new(aws_sdk_kinesis::Client::new(&config));

        let app = Router::new().route("/", get(MyAxumApp::root)).route(
            "/add",
            post({
                let client = Arc::clone(&client); // Clone Arc for the closure
                move |payload: Json<PutRecord>| MyAxumApp::put_records(State(client), payload)
                // Pass the client to the handler
            }),
        );

        MyAxumApp { app }
    }

    async fn root() -> &'static str {
        "Hello, World!"
    }

    async fn put_records(
        State(client): State<Arc<Client>>,
        Json(payload): Json<PutRecord>,
    ) -> StatusCode {
        println!("{}", payload.stream_name);
        println!("{}", payload.partition_key);
        println!(
            "{}",
            match payload.explicit_hash_key {
                Some(x) => x,
                None => "nu".to_string(),
            }
        );

        let output = client
            .put_record()
            .stream_arn("arn:aws:kinesis:us-east-1:188628773952:stream/mytest")
            .partition_key("moomoo")
            .data(Blob::new("dd"))
            .send()
            .await;

        match output {
            Ok(response) => {
                println!(
                    "Successfully put record with Sequencex Number: {:?}",
                    response.sequence_number
                );
                Ok::<(), Box<dyn Error>>(());
            }
            Err(err) => {
                eprintln!("Error putting record: {}", err);
                println!("{}", err.to_string());
                err.message().unwrap();
                println!("{}", err.message().unwrap());
                println!("{}", err.code().unwrap());
                Err::<(), Box<dyn Error>>(Box::new(err));
            }
        }

        StatusCode::NO_CONTENT
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
    }
}
