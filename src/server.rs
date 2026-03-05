use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    response::Response,
    routing::any,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::executor::{self, ExecuteError};
use crate::parser::{self, ParseError};
use crate::store::Store;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Store>,
}

// ---------------------------------------------------------------------------
// Custom QUERY method
// ---------------------------------------------------------------------------

fn query_method() -> Method {
    Method::from_bytes(b"QUERY").unwrap()
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", any(dispatch))
        .route("/{*path}", any(dispatch))
        .with_state(state)
}

/// Dispatch incoming requests — accept QUERY, reject everything else.
async fn dispatch(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Response {
    if request.method() == query_method() {
        handle_query(state, request).await
    } else {
        error_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
            "This endpoint only accepts QUERY requests",
            None,
        )
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn handle_query(
    state: AppState,
    request: Request<Body>,
) -> Response {
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "malformed_document",
                "Could not read request body",
                None,
            );
        }
    };

    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "malformed_document",
                "Request body is not valid UTF-8",
                None,
            );
        }
    };

    let query = match parser::parse_str(body_str) {
        Ok(q) => q,
        Err(e) => {
            return parse_error_response(e);
        }
    };

    match executor::execute(&query, state.store.document()) {
        Ok(Value::Null) => {
            json_response(StatusCode::OK, &json!({}))
        }
        Ok(result) => {
            json_response(StatusCode::OK, &result)
        }
        Err(e) => {
            execute_error_response(e)
        }
    }
}

// ---------------------------------------------------------------------------
// Error responses
// ---------------------------------------------------------------------------

fn parse_error_response(e: ParseError) -> Response {
    match e {
        ParseError::NotAnObject => error_response(
            StatusCode::BAD_REQUEST,
            "malformed_document",
            "Query document must be a JSON object",
            None,
        ),
        ParseError::InvalidOperatorValue { key, reason: _ } => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_operator",
            "Invalid value for operator",
            Some(&key),
        ),
        ParseError::UnknownOperator(key) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_operator",
            "Unknown operator",
            Some(&key),
        ),
        ParseError::InvalidCollectionOperator { key, reason: _ } => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_operator",
            "Invalid collection operator",
            Some(&key),
        ),
    }
}

fn execute_error_response(e: ExecuteError) -> Response {
    match e {
        ExecuteError::MissingField { path } => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_field",
            "Field not found in document",
            Some(&path),
        ),
        ExecuteError::IncomparableType { path, operator: _ } => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_value",
            "Cannot apply operator to field type",
            Some(&path),
        ),
        ExecuteError::NotAnObject { path } => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_field",
            "Expected an object at path",
            Some(&path),
        ),
        ExecuteError::NotAnArray { path } => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_field",
            "Expected an array at path",
            Some(&path),
        ),
    }
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: &str,
    detail: Option<&str>,
) -> Response {
    let mut body = json!({
        "error": {
            "code": code,
            "message": message,
        }
    });

    if let Some(d) = detail {
        body["error"]["detail"] = json!(d);
    }

    json_response(status, &body)
}

fn json_response(status: StatusCode, body: &Value) -> Response {
    let json = serde_json::to_string_pretty(body).unwrap_or_default();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .unwrap()
}