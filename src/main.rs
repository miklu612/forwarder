
use axum::{response::Html, routing::get, Router};
use std::env;
use std::fs;
use serde_json::{Value};

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);

    // TODO: Load this into memory always and don't read it evertime.
    let file_name = args[1].clone();
    let json_raw = fs::read_to_string(file_name).expect("Failed to read file");
    println!("{}", json_raw);

    let config_data: Value = serde_json::from_str(&json_raw).expect("Failed to parse json");
    let config = config_data["config"].clone();

    let mut app = Router::new();
    app = app.fallback(get(error));

    for path in config.as_array().unwrap() {
        app = app.route(path["path"].as_str().unwrap(), get(handler));
    }





    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn error() -> Html<&'static str> {
    Html("<h1> Error </h1>")
}

async fn handler() -> Html<&'static str> {
    Html("<h1> Forwarder </h1>")
}
