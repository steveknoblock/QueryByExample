mod parser;
mod executor;

use serde_json::json;

fn main() {
    let query = json!({
        "user": {
            "id": 1,
            "name": "*",
            "email": "*",
            "orders": {
                "status": "pending",
                "total": { ">": 100 },
                "^": "total",
                "id": "*",
                "total": "*"
            }
        }
    });

    let document = json!({
        "user": {
            "id": 1,
            "name": "Alice",
            "email": "alice@example.com",
            "orders": [
                { "id": 42, "status": "pending",  "total": 150 },
                { "id": 43, "status": "pending",  "total": 50  },
                { "id": 44, "status": "complete", "total": 200 },
                { "id": 45, "status": "pending",  "total": 200 },
            ]
        }
    });

    let parsed = match parser::parse(&query) {
        Ok(q)  => q,
        Err(e) => { eprintln!("Parse error: {}", e); return; }
    };

    match executor::execute(&parsed, &document) {
        Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
        Err(e)     => eprintln!("Execute error: {}", e),
    }
}

