mod buffer;

use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use buffer::Person;
use aws_config;
use std::error::Error;
use aws_sdk_kinesis::error::ProvideErrorMetadata;
use aws_sdk_kinesis::primitives::Blob;
use std::sync::Arc;
use aws_sdk_kinesis::Client;
use axum::extract::State;

#[tokio::main]
async fn main() {
    let config = aws_config::load_from_env().await;
    let client = Arc::new(aws_sdk_kinesis::Client::new(&config));

    let app = Router::new()
        .route("/", get(root))
        .route("/add", post({
            let client = Arc::clone(&client); // Clone Arc for the closure
            move |payload: Json<PutRecord>| put_records(State(client), payload) // Pass the client to the handler
        }))
        .route("/users", post(create_user));


    let output = client.put_record()
        .stream_arn("arn:aws:kinesis:us-east-1:188628773952:stream/mytest")
        .partition_key("moomoo")
        .data(Blob::new("dd"))
        .send()
        .await;

    match output {
        Ok(response) => {
            println!("Successfully put record with Sequence Number: {:?}", response.sequence_number);
            Ok::<(), Box<dyn Error>>(());
        },
        Err(err) => {
            eprintln!("Error putting record: {}", err);
            println!("{}", err.to_string());
            println!("{}", err.message().unwrap());
            println!("{}", err.code().unwrap());
            Err::<(), Box<dyn Error>>(Box::new(err));
        },
    }

    let mut person = Person::new("Alice".to_string(), 30);
    println!("Name: {}", person.get_name());
    println!("Age: {}", person.get_age());

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
    State(client): State<Arc<Client>>,
    Json(payload): Json<PutRecord>,
) -> StatusCode {
    println!("{}", payload.stream_name);
    println!("{}", payload.partition_key);
    println!("{}", match payload.explicit_hash_key {
        Some(x) => x,
        None    => "nu".to_string(),
    });

    let output = client.put_record()
        .stream_arn("arn:aws:kinesis:us-east-1:188628773952:stream/mytest")
        .partition_key("moomoo")
        .data(Blob::new("dd"))
        .send()
        .await;

    match output {
        Ok(response) => {
            println!("Successfully put record with Sequencex Number: {:?}", response.sequence_number);
            Ok::<(), Box<dyn Error>>(());
        },
        Err(err) => {
            eprintln!("Error putting record: {}", err);
            println!("{}", err.to_string());
            println!("{}", err.message().unwrap());
            println!("{}", err.code().unwrap());
            Err::<(), Box<dyn Error>>(Box::new(err));
        },
    }

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