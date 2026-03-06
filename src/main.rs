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
    let store = match std::env::args().nth(1) {
        Some(path) => {
            let p = Path::new(&path);
            if p.is_dir() {
                match store::Store::new(p) {
                    Ok(s) => {
                        println!("Serving documents from directory: {path}");
                        s
                    }
                    Err(e) => {
                        eprintln!("Failed to open data directory: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Expected a directory, got: {path}");
                eprintln!("Usage: jqbe <data-directory>");
                std::process::exit(1);
            }
        }
        None => {
            println!("No data directory provided — using built-in example document at /users.");
            store::Store::from_value("users", serde_json::json!({
                "users": [
                    {
                        "id": 1,
                        "name": "Alice",
                        "email": "alice@example.com",
                        "status": "active",
                        "address": { "city": "New York", "state": "NY", "country": "US" },
                        "orders": [
                            { "id": 42, "status": "pending",  "total": 150, "product": "Widget A" },
                            { "id": 43, "status": "pending",  "total": 50,  "product": "Widget B" },
                            { "id": 44, "status": "complete", "total": 200, "product": "Widget C" },
                            { "id": 45, "status": "pending",  "total": 200, "product": "Widget D" }
                        ]
                    },
                    {
                        "id": 2,
                        "name": "Bob",
                        "email": "bob@example.com",
                        "status": "active",
                        "address": { "city": "Los Angeles", "state": "CA", "country": "US" },
                        "orders": [
                            { "id": 46, "status": "pending",  "total": 75,  "product": "Widget A" },
                            { "id": 47, "status": "complete", "total": 300, "product": "Widget E" },
                            { "id": 48, "status": "complete", "total": 125, "product": "Widget B" }
                        ]
                    },
                    {
                        "id": 3,
                        "name": "Carol",
                        "email": "carol@example.com",
                        "status": "inactive",
                        "address": { "city": "Chicago", "state": "IL", "country": "US" },
                        "orders": [
                            { "id": 49, "status": "pending",  "total": 400, "product": "Widget F" },
                            { "id": 50, "status": "pending",  "total": 100, "product": "Widget A" }
                        ]
                    }
                ]
            }))
        }
    };

    let state = AppState {
        store: Arc::new(store),
    };

    let app = server::router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
