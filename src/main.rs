use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/add", post(put_records))
        .route("/users", post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };
    println!("Hello, world!");
    println!("{}", user.username);
    (StatusCode::CREATED, Json(user))
}

async fn put_records(
    Json(payload): Json<PutRecord>,
) -> StatusCode {
    println!("{}", payload.stream_name);
    println!("{}", payload.partition_key);
    println!("{}", match payload.explicit_hash_key {
        Some(x) => x,
        None    => "nu".to_string(),
    });
    StatusCode::NO_CONTENT
}

#[derive(Deserialize)]
struct PutRecord {
    stream_name: String,
    partition_key: String,
    explicit_hash_key: Option<String>,
    data: Vec<u8>,
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}