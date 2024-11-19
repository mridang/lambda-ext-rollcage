use chrono::Utc;
use gethostname::gethostname;
use reqwest::{Client, Url};
use serde_json::json;
use std::env;
use uuid::Uuid;

pub struct ErrorReporter {
    client: Client,
    app_environment: String,
    project_id: String,
    ingest_scheme: String,
    ingest_host: String,
    ingest_port: u16,
    public_key: String,
    platform_name: String,
    server_name: String,
}

impl ErrorReporter {
    pub fn new() -> ErrorReporter {
        match env::var("SENTRY_DSN") {
            Err(e) => panic!("SENTRY_DSN environment variable is not set: {}", e),
            Ok(val) => ErrorReporter::new_with_env_and_dsn(
                env::var("NODE_ENV").unwrap_or_else(|_| "production".to_string()),
                val,
            ),
        }
    }

    pub fn new_with_env_and_dsn(env_name: String, sentry_dsn: String) -> ErrorReporter {
        match Url::parse(&sentry_dsn) {
            Err(e) => panic!("couldn't interpret the DSN: {}", e),
            Ok(url) => {
                let client = Client::new();
                ErrorReporter {
                    client,
                    server_name: gethostname().to_string_lossy().to_string(),
                    platform_name: "node".to_string(),
                    app_environment: env_name,
                    ingest_scheme: url.scheme().to_string(),
                    ingest_host: url.host_str().expect("URL missing host").to_string(),
                    ingest_port: url.port_or_known_default().expect("Invalid port"),
                    project_id: url
                        .path_segments()
                        .and_then(|segments| segments.last())
                        .expect("Project ID not found")
                        .to_string(),
                    public_key: url.username().to_string(),
                }
            }
        }
    }

    pub async fn report(&self, error_type: &str) {
        let report_url = format!(
            "{}://{}:{}/api/{}/envelope/",
            self.ingest_scheme, self.ingest_host, self.ingest_port, self.project_id
        );

        println!("Sending crash to {}", report_url);

        match self
            .client
            .post(report_url)
            .header("Content-Type", "application/json")
            .header("X-Sentry-Auth", format!("Sentry sentry_version=7, sentry_key={}", self.public_key))
            .body(
                vec![
                    json!({
                        "event_id": Uuid::new_v4().as_simple().to_string(),
                        "sent_at": Utc::now().to_rfc3339(),
                        "sdk": {
                                "name": "sentry.javascript.node",
                                "version": "8.38.0"
                            },
                        "trace": {
                                "environment": self.app_environment,
                                "public_key": self.public_key,
                            }
                    })
                    .to_string(),
                    json!({
                        "type": "event"
                    })
                    .to_string(),
                    json!({
                        "exception": {
                            "values": [
                                {
                                    "type": error_type,
                                    "value": "A lambda crash occurred",
                                    "stacktrace": {},
                                    "mechanism": {
                                        "type": "generic",
                                        "description": "The lambda function crashed",
                                        "handled": false,
                                        "synthetic": true,
                                    }
                                }
                            ]
                        },
                        "platform": self.platform_name,
                        "server_name": self.server_name,
                        "environment": self.app_environment,
                    })
                    .to_string(),
                ]
                .join("\n"),
            )
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Reporter report sent successfully.");
                } else {
                    eprintln!("Failed to send report: {}, {}", response.status(), response.text().await.unwrap());
                }
            }
            Err(e) => {
                eprintln!("Error sending report: {:?}", e);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::reporter::ErrorReporter;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    const ENV_NAME: &str = "test";
    #[tokio::test]
    async fn my_test() {
        let mock_server = MockServer::start().await;
        let endpoint = format!("http://user@{}/1", mock_server.address());

        Mock::given(method("POST"))
            .and(path("/api/1/envelope/"))
            .and(header("Content-Type", "application/json"))
            .and(headers(
                "X-Sentry-Auth",
                vec!["Sentry sentry_version=7", "sentry_key=user"],
            ))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let error_reporter = ErrorReporter::new_with_env_and_dsn(ENV_NAME.to_string(), endpoint);
        error_reporter.report("my_test").await;

        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 1);
    }
}
