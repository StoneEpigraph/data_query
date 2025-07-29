use axum::{routing, Router};

pub fn init() -> Router {
    let router = Router::new().route("/", routing::get(home()));
    router
}

pub fn home() -> &'static str {
    "Hello, world!"
}
