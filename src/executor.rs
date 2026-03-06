use serde_json::{Value, Map};
use crate::parser::{QueryNode, FieldEntry, Operator, OrderDir};

// ---------------------------------------------------------------------------
// Execution errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ExecuteError {
    /// A field in the query does not exist in the document.
    MissingField { path: String },
    /// A comparison operator was applied to a field whose type does not
    /// support ordering (e.g. comparing a string with ">").
    IncomparableType { path: String, operator: String },
    /// A field in the query was expected to be an object but was a scalar.
    NotAnObject { path: String },
    /// A field in the query was expected to be an array but was not.
    NotAnArray { path: String },
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecuteError::MissingField { path } =>
                write!(f, "Field not found in document: {path}"),
            ExecuteError::IncomparableType { path, operator } =>
                write!(f, "Cannot apply operator \"{operator}\" to field: {path}"),
            ExecuteError::NotAnObject { path } =>
                write!(f, "Expected an object at: {path}"),
            ExecuteError::NotAnArray { path } =>
                write!(f, "Expected an array at: {path}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

/// Execute a query against a JSON document.
/// Returns a new Value shaped and filtered according to the query.
pub fn execute(query: &QueryNode, document: &Value) -> Result<Value, ExecuteError> {
    execute_node(query, document, "")
}

fn execute_node(
    query: &QueryNode,
    document: &Value,
    path: &str,
) -> Result<Value, ExecuteError> {
    let obj = document.as_object().ok_or_else(|| {
        ExecuteError::NotAnObject { path: path_or_root(path) }
    })?;

    let mut result = Map::new();

    for (field_name, entry) in &query.fields {
        let field_path = build_path(path, field_name);

        let doc_value = obj.get(field_name).ok_or_else(|| {
            ExecuteError::MissingField { path: field_path.clone() }
        })?;

        match entry {
            // Return this field as-is.
            FieldEntry::Projection => {
                result.insert(field_name.clone(), doc_value.clone());
            }

            // Match — check equality and include in output.
            FieldEntry::Match(expected) => {
                if doc_value != expected {
                    return Ok(Value::Null); // signals no match to caller
                }
                result.insert(field_name.clone(), doc_value.clone());
            }

            FieldEntry::Operator(op) => {
                match op {
                    Operator::MatchOnly(inner) => {
                        // Evaluate the inner entry but do not project.
                        match inner.as_ref() {
                            FieldEntry::Match(expected) => {
                                if doc_value != expected {
                                    return Ok(Value::Null);
                                }
                            }
                            FieldEntry::Operator(inner_op) => {
                                if !evaluate_operator(inner_op, doc_value, &field_path)? {
                                    return Ok(Value::Null);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        if !evaluate_operator(op, doc_value, &field_path)? {
                            return Ok(Value::Null);
                        }
                        result.insert(field_name.clone(), doc_value.clone());
                    }
                }
            }

            // Nested object or collection.
            FieldEntry::Nested(sub_query) => {
                match doc_value {
                    Value::Array(arr) => {
                        let matched = execute_collection(
                            sub_query,
                            arr,
                            &field_path,
                        )?;
                        result.insert(field_name.clone(), Value::Array(matched));
                    }
                    Value::Object(_) => {
                        let nested = execute_node(sub_query, doc_value, &field_path)?;
                        if nested.is_null() {
                            return Ok(Value::Null); // propagate no-match
                        }
                        result.insert(field_name.clone(), nested);
                    }
                    _ => {
                        return Err(ExecuteError::NotAnObject {
                            path: field_path,
                        });
                    }
                }
            }
        }
    }

    Ok(Value::Object(result))
}

/// Execute a query node against an array of documents.
/// Applies matching, projection, ordering, limit, and offset.
fn execute_collection(
    query: &QueryNode,
    arr: &[Value],
    path: &str,
) -> Result<Vec<Value>, ExecuteError> {
    // Match and project each element.
    let mut results: Vec<Value> = arr
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let item_path = format!("{}[{}]", path, i);
            execute_node(query, item, &item_path)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|v| !v.is_null()) // null signals no match
        .collect();

    // Ordering.
    if let Some(order_field) = &query.collection_opts.order_by {
        let descending = matches!(query.collection_opts.order_dir, Some(OrderDir::Desc));
        results.sort_by(|a, b| {
            let av = a.get(order_field);
            let bv = b.get(order_field);
            let ord = compare_values(av, bv);
            if descending { ord.reverse() } else { ord }
        });
    }

    // Offset.
    let offset = query.collection_opts.offset.unwrap_or(0) as usize;
    if offset < results.len() {
        results = results[offset..].to_vec();
    } else {
        results = vec![];
    }

    // Limit.
    if let Some(limit) = query.collection_opts.limit {
        results.truncate(limit as usize);
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Operator evaluation
// ---------------------------------------------------------------------------

fn evaluate_operator(
    op: &Operator,
    value: &Value,
    path: &str,
) -> Result<bool, ExecuteError> {
    match op {
        Operator::In(candidates) => {
            return Ok(candidates.contains(value));
        }
        _ => {}
    }

    // Numeric comparisons.
    let lhs = value.as_f64().ok_or_else(|| ExecuteError::IncomparableType {
        path: path.to_string(),
        operator: op_symbol(op).to_string(),
    })?;

    match op {
        Operator::Gt(rhs) => {
            let r = numeric(rhs, op, path)?;
            Ok(lhs > r)
        }
            Operator::Gte(rhs) => {
            let r = numeric(rhs, op, path)?;
            Ok(lhs >= r)
        }
        Operator::Lt(rhs) => {
            let r = numeric(rhs, op, path)?;
            Ok(lhs < r)
        }
        Operator::Lte(rhs) => {
            let r = numeric(rhs, op, path)?;
            Ok(lhs <= r)
        }
        Operator::Ne(rhs) => Ok(value != rhs),
        Operator::In(_) => unreachable!(),
        Operator::MatchOnly(_) => Ok(true),
    }
}

fn numeric(value: &Value, op: &Operator, path: &str) -> Result<f64, ExecuteError> {
    value.as_f64().ok_or_else(|| ExecuteError::IncomparableType {
        path: path.to_string(),
        operator: op_symbol(op).to_string(),
    })
}

fn op_symbol(op: &Operator) -> &'static str {
    match op {
        Operator::Gt(_)  => ">",
        Operator::Gte(_) => ">=",
        Operator::Lt(_)  => "<",
        Operator::Lte(_) => "<=",
        Operator::Ne(_)  => "!=",
        Operator::In(_)  => "|",
        Operator::MatchOnly(_) => "?",
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_path(parent: &str, field: &str) -> String {
    if parent.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", parent, field)
    }
}

fn path_or_root(path: &str) -> String {
    if path.is_empty() {
        "<root>".to_string()
    } else {
        path.to_string()
    }
}

/// Compare two optional JSON values for sorting purposes.
fn compare_values(a: Option<&Value>, b: Option<&Value>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => {
            let xf = x.as_f64().unwrap_or(0.0);
            let yf = y.as_f64().unwrap_or(0.0);
            xf.partial_cmp(&yf).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Some(Value::String(x)), Some(Value::String(y))) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use serde_json::json;

    fn run(query: &Value, doc: &Value) -> Result<Value, ExecuteError> {
        let q = parse(query).unwrap();
        execute(&q, doc)
    }

    #[test]
    fn test_projection() {
        let result = run(
            &json!({ "name": "*", "email": "*" }),
            &json!({ "name": "Alice", "email": "alice@example.com", "age": 30 }),
        ).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["email"], "alice@example.com");
        assert!(result.get("age").is_none());
    }

    #[test]
    fn test_literal_match_passes() {
        let result = run(
            &json!({ "status": "active", "name": "*" }),
            &json!({ "status": "active", "name": "Alice" }),
        ).unwrap();

        assert_eq!(result["name"], "Alice");
    }

    #[test]
    fn test_literal_match_fails() {
        let result = run(
            &json!({ "status": "active", "name": "*" }),
            &json!({ "status": "inactive", "name": "Alice" }),
        ).unwrap();

        assert!(result.is_null());
    }

    #[test]
    fn test_operator_gt() {
        let result = run(
            &json!({ "total": { ">": 100 }, "id": "*" }),
            &json!({ "total": 150, "id": 1 }),
        ).unwrap();

        assert_eq!(result["id"], 1);

        let no_match = run(
            &json!({ "total": { ">": 100 }, "id": "*" }),
            &json!({ "total": 50, "id": 1 }),
        ).unwrap();

        assert!(no_match.is_null());
    }

    #[test]
    fn test_operator_in() {
        let result = run(
            &json!({ "status": { "|": ["pending", "processing"] }, "id": "*" }),
            &json!({ "status": "pending", "id": 1 }),
        ).unwrap();

        assert_eq!(result["id"], 1);

        let no_match = run(
            &json!({ "status": { "|": ["pending", "processing"] }, "id": "*" }),
            &json!({ "status": "complete", "id": 1 }),
        ).unwrap();

        assert!(no_match.is_null());
    }

    #[test]
    fn test_missing_field_error() {
        let result = run(
            &json!({ "nonexistent": "*" }),
            &json!({ "name": "Alice" }),
        );

        assert!(matches!(result, Err(ExecuteError::MissingField { .. })));
    }

    #[test]
    fn test_nested_object() {
        let result = run(
            &json!({
                "user": {
                    "name": "*",
                    "address": {
                        "city": "*",
                        "country": "US"
                    }
                }
            }),
            &json!({
                "user": {
                    "name": "Alice",
                    "address": {
                        "city": "New York",
                        "country": "US"
                    }
                }
            }),
        ).unwrap();

        assert_eq!(result["user"]["name"], "Alice");
        assert_eq!(result["user"]["address"]["city"], "New York");
    }

    #[test]
    fn test_collection_filtering() {
        let result = run(
            &json!({
                "orders": {
                    "status": "pending",
                    "id": "*",
                    "total": "*"
                }
            }),
            &json!({
                "orders": [
                    { "id": 1, "status": "pending",  "total": 100 },
                    { "id": 2, "status": "complete", "total": 200 },
                    { "id": 3, "status": "pending",  "total": 300 },
                ]
            }),
        ).unwrap();

        let orders = result["orders"].as_array().unwrap();
        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0]["id"], 1);
        assert_eq!(orders[1]["id"], 3);
    }

    #[test]
    fn test_collection_ordering_and_limit() {
        let result = run(
            &json!({
                "orders": {
                    "v": "total",
                    "#": 2,
                    "id": "*",
                    "total": "*"
                }
            }),
            &json!({
                "orders": [
                    { "id": 1, "total": 100 },
                    { "id": 2, "total": 300 },
                    { "id": 3, "total": 200 },
                ]
            }),
        ).unwrap();

        let orders = result["orders"].as_array().unwrap();
        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0]["total"], 300); // descending
        assert_eq!(orders[1]["total"], 200);
    }

    #[test]
    fn test_collection_offset() {
        let result = run(
            &json!({
                "orders": {
                    "^": "total",
                    "@": 1,
                    "id": "*",
                    "total": "*"
                }
            }),
            &json!({
                "orders": [
                    { "id": 1, "total": 100 },
                    { "id": 2, "total": 200 },
                    { "id": 3, "total": 300 },
                ]
            }),
        ).unwrap();

        let orders = result["orders"].as_array().unwrap();
        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0]["total"], 200); // first skipped by offset
    }

    #[test]
    fn test_full_specification_example() {
        let result = run(
            &json!({
                "user": {
                    "id": 1,
                    "name": "*",
                    "email": "*",
                    "orders": {
                        "status": "pending",
                        "total": { ">": 100 },
                        "id": "*",
                        "total": "*"
                    }
                }
            }),
            &json!({
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
            }),
        ).unwrap();

        assert_eq!(result["user"]["name"], "Alice");
        assert_eq!(result["user"]["email"], "alice@example.com");

        let orders = result["user"]["orders"].as_array().unwrap();
        assert_eq!(orders.len(), 2); // id 42 and 45 match
        assert_eq!(orders[0]["id"], 42);
        assert_eq!(orders[1]["id"], 45);
    }
}
