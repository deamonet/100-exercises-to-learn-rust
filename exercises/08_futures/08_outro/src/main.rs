// This is our last exercise. Let's go down a more unstructured path!
// Try writing an **asynchronous REST API** to expose the functionality
// of the ticket management system we built throughout the course.
// It should expose endpoints to:
//  - Create a ticket
//  - Retrieve ticket details
//  - Patch a ticket
//
// Use Rust's package registry, crates.io, to find the dependencies you need
// (if any) to build this system.

pub mod data;
pub mod store;

use crate::data::{Ticket, TicketDraft, TicketPatch};
use crate::store::{TicketId, TicketStore};
use axum::debug_handler;
use axum::extract::Path;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::sync::OnceLock;
use tokio::sync::Mutex;

static STORE: OnceLock<Mutex<TicketStore>> = OnceLock::new();

async fn get_store() -> &'static Mutex<TicketStore> {
    STORE.get_or_init(|| Mutex::new(TicketStore::new()))
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/ticket/{id}", get(get_ticket))
        .route("/ticket", post(add_ticket).patch(patch_ticket));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.expect("TODO: panic message");
}

async fn get_ticket(Path(id): Path<u64>) -> Result<Json<Ticket>, StatusCode> {
    let store = get_store().await.lock().await;
    match store.get(TicketId(id)) {
        Some(t) => Ok(Json(t.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[debug_handler]
async fn add_ticket(Json(draft): Json<TicketDraft>) -> &'static str {
    let mut store = get_store().await.lock().await;
    store.add_ticket(draft);
    "Done"
}

#[debug_handler]
async fn patch_ticket(Json(patch): Json<TicketPatch>) -> &'static str {
    let mut store = get_store().await.lock().await;
    let ticket = store.get_mut(patch.id);
    if let Some(t) = ticket {
        if let Some(title) = patch.title {
            t.title = title
        }
        if let Some(status) = patch.status {
            t.status = status
        }
        if let Some(description) = patch.description {
            t.description = description
        }
    }
    "Done"
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
