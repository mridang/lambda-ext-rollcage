mod reporter;

use crate::reporter::ErrorReporter;
use lambda_extension::{
    service_fn, Error, Extension, LambdaTelemetry, LambdaTelemetryRecord, SharedService, Status,
};
use std::sync::Arc;

#[allow(clippy::collapsible_match)]
#[allow(clippy::single_match)]
async fn handler(
    events: Vec<LambdaTelemetry>,
    crash_reporter: Arc<ErrorReporter>,
) -> Result<(), Error> {
    for event in events {
        match event.record {
            LambdaTelemetryRecord::PlatformRuntimeDone {
                error_type, status, ..
            } => match status {
                Status::Error => match error_type.as_deref() {
                    Some("Runtime.HandlerNotFound") => {
                        crash_reporter.report("Runtime.HandlerNotFound").await;
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
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

    let crash_reporter = Arc::new(ErrorReporter::new());
    let telemetry_processor = SharedService::new(service_fn({
        let crash_reporter = crash_reporter.clone();
        move |events| handler(events, crash_reporter.clone())
    }));
    Extension::new()
        .with_telemetry_processor(telemetry_processor)
        .run()
        .await?;
    Ok(())
}
