mod buffer;
mod extension;

use std::sync::Arc;
use lambda_extension::{service_fn, Error, LambdaEvent, NextEvent};
use tokio::sync::{oneshot, Mutex};
use tracing_subscriber;
use tracing;

async fn my_extension(event: LambdaEvent, shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(_e) => {
            println!("shutdown {}", _e.shutdown_reason);
            let mut tx = shutdown_tx.lock().await;
            if let Some(tx) = tx.take() {
                let _ = tx.send(()); // Send the shutdown signal once
            }
        }
        NextEvent::Invoke(_e) => {
            println!("invoke {}", _e.request_id)
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let app = extension::MyAxumApp::new().await;
    tokio::task::spawn(async move {
        if let Err(e) = app.listen(shutdown_rx).await {
            eprintln!("Error starting server: {}", e);
        }
    });

    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));
    let func = service_fn(move |event: LambdaEvent| {
        let shutdown_tx = shutdown_tx.clone();
        async move {
            my_extension(event, shutdown_tx).await
        }
    });
    lambda_extension::run(func).await
}