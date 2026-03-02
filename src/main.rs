mod executor;
mod parser;
mod server;
mod store;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use server::AppState;

#[tokio::main]
async fn main() {
    // Load the document store from a JSON file.
    // Creates a default document if no file path is provided.
    let store = match std::env::args().nth(1) {
        Some(path) => {
            match store::Store::load(Path::new(&path)) {
                Ok(s) => {
                    println!("Loaded store from {path}");
                    s
                }
                Err(e) => {
                    eprintln!("Failed to load store: {e}");
                    std::process::exit(1);
                }
            }
        }
        None => {
            println!("No store file provided — using built-in example document.");
            store::Store::from_value(serde_json::json!({
                "user": {
                    "id": 1,
                    "name": "Alice",
                    "email": "alice@example.com",
                    "orders": [
                        { "id": 42, "status": "pending",  "total": 150 },
                        { "id": 43, "status": "pending",  "total": 50  },
                        { "id": 44, "status": "complete", "total": 200 },
                        { "id": 45, "status": "pending",  "total": 200 }
                    ]
                }
            }))
        }
    };

    let state = AppState {
        store: Arc::new(store),
    };

    let app = server::router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Listening on http://{addr}");
    println!("Send QUERY requests to http://{addr}/");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
