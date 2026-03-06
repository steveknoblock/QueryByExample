use serde_json::Value;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A parsed query document. Each field in the JSON object maps to a
/// FieldEntry describing what to do with that field.
#[derive(Debug, Clone)]
pub struct QueryNode {
    pub fields: HashMap<String, FieldEntry>,
    pub collection_opts: CollectionOpts,
}

/// What a single field in the query document means.
#[derive(Debug, Clone)]
pub enum FieldEntry {
    /// "*" — return this field's value in the response.
    Projection,

    /// A literal scalar value — match documents where this field equals this value.
    Match(Value),

    /// A typographical operator object — e.g. { ">": 100 }.
    Operator(Operator),

    /// A nested object — recurse into this sub-document.
    Nested(QueryNode),
}

/// Comparison and membership operators expressed as typographical keys.
#[derive(Debug, Clone)]
pub enum Operator {
    Gt(Value),
    Gte(Value),
    Lt(Value),
    Lte(Value),
    Ne(Value),
    In(Vec<Value>),
    MatchOnly(Box<FieldEntry>),
}

/// Collection control options extracted from operator keys within a node.
#[derive(Debug, Clone, Default)]
pub struct CollectionOpts {
    pub order_by: Option<String>,
    pub order_dir: Option<OrderDir>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum OrderDir {
    Asc,
    Desc,
}

// ---------------------------------------------------------------------------
// Parse errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ParseError {
    /// The input is not a JSON object at the top level.
    NotAnObject,
    /// An operator key was found but its value has the wrong type.
    InvalidOperatorValue { key: String, reason: String },
    /// An unrecognised operator key was encountered.
    UnknownOperator(String),
    /// A collection operator had a value of the wrong type.
    InvalidCollectionOperator { key: String, reason: String },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NotAnObject =>
                write!(f, "Query document must be a JSON object"),
            ParseError::InvalidOperatorValue { key, reason } =>
                write!(f, "Invalid value for operator \"{key}\": {reason}"),
            ParseError::UnknownOperator(key) =>
                write!(f, "Unknown operator \"{key}\""),
            ParseError::InvalidCollectionOperator { key, reason } =>
                write!(f, "Invalid collection operator \"{key}\": {reason}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a JSON value into a QueryNode.
/// The top-level value must be a JSON object.
pub fn parse(value: &Value) -> Result<QueryNode, ParseError> {
    match value {
        Value::Object(map) => parse_object(map),
        _ => Err(ParseError::NotAnObject),
    }
}

/// Parse a JSON string into a QueryNode.
pub fn parse_str(input: &str) -> Result<QueryNode, ParseError> {
    let value: Value = serde_json::from_str(input)
        .map_err(|_| ParseError::NotAnObject)?;
    parse(&value)
}

fn parse_object(
    map: &serde_json::Map<String, Value>,
) -> Result<QueryNode, ParseError> {
    let mut fields = HashMap::new();
    let mut collection_opts = CollectionOpts::default();

    for (key, value) in map {
        match key.as_str() {
            // Collection operators — consumed here, not stored as fields.
            "^" => {
                let field = value.as_str().ok_or_else(|| {
                    ParseError::InvalidCollectionOperator {
                        key: "^".into(),
                        reason: "value must be a field name string".into(),
                    }
                })?;
                collection_opts.order_by = Some(field.to_string());
                collection_opts.order_dir = Some(OrderDir::Asc);
            }
            "v" => {
                let field = value.as_str().ok_or_else(|| {
                    ParseError::InvalidCollectionOperator {
                        key: "v".into(),
                        reason: "value must be a field name string".into(),
                    }
                })?;
                collection_opts.order_by = Some(field.to_string());
                collection_opts.order_dir = Some(OrderDir::Desc);
            }
            "#" => {
                let n = value.as_u64().ok_or_else(|| {
                    ParseError::InvalidCollectionOperator {
                        key: "#".into(),
                        reason: "value must be a non-negative integer".into(),
                    }
                })?;
                collection_opts.limit = Some(n);
            }
            "@" => {
                let n = value.as_u64().ok_or_else(|| {
                    ParseError::InvalidCollectionOperator {
                        key: "@".into(),
                        reason: "value must be a non-negative integer".into(),
                    }
                })?;
                collection_opts.offset = Some(n);
            }

            // Regular fields.
            field_name => {
                let entry = parse_field_entry(value)?;
                fields.insert(field_name.to_string(), entry);
            }
        }
    }

    Ok(QueryNode { fields, collection_opts })
}

fn parse_field_entry(value: &Value) -> Result<FieldEntry, ParseError> {
    match value {
        // "*" — projection
        Value::String(s) if s == "*" => Ok(FieldEntry::Projection),

        // Nested object — could be an operator object or a nested query node.
        Value::Object(map) => {
            // An operator object has exactly one key that is a typographical operator.
            if map.len() == 1 {
                let (op_key, op_val) = map.iter().next().unwrap();
                if let Some(op) = try_parse_operator(op_key, op_val)? {
                    return Ok(FieldEntry::Operator(op));
                }
            }
            // Otherwise treat it as a nested query node.
            Ok(FieldEntry::Nested(parse_object(map)?))
        }

        // Any other scalar — literal match value.
        other => Ok(FieldEntry::Match(other.clone())),
    }
}

/// Attempt to parse a key-value pair as a comparison operator.
/// Returns Ok(None) if the key is not a known operator symbol.
fn try_parse_operator(
    key: &str,
    value: &Value,
) -> Result<Option<Operator>, ParseError> {
    match key {
        ">" => Ok(Some(Operator::Gt(value.clone()))),
        ">=" => Ok(Some(Operator::Gte(value.clone()))),
        "<" => Ok(Some(Operator::Lt(value.clone()))),
        "<=" => Ok(Some(Operator::Lte(value.clone()))),
        "!=" => Ok(Some(Operator::Ne(value.clone()))),
        "|" => {
            let arr = value.as_array().ok_or_else(|| {
                ParseError::InvalidOperatorValue {
                    key: "|".into(),
                    reason: "value must be an array".into(),
                }
            })?;
            Ok(Some(Operator::In(arr.clone())))
        },
        "?" => {
            let inner = parse_field_entry(value)?;
            Ok(Some(Operator::MatchOnly(Box::new(inner))))
        },
        // Not an operator key — let the caller treat it as a nested node.
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_projection() {
        let q = parse(&json!({ "name": "*", "email": "*" })).unwrap();
        assert!(matches!(q.fields["name"], FieldEntry::Projection));
        assert!(matches!(q.fields["email"], FieldEntry::Projection));
    }

    #[test]
    fn test_literal_match() {
        let q = parse(&json!({ "status": "pending", "id": 1 })).unwrap();
        assert!(matches!(&q.fields["status"], FieldEntry::Match(Value::String(s)) if s == "pending"));
        assert!(matches!(&q.fields["id"], FieldEntry::Match(Value::Number(_))));
    }

    #[test]
    fn test_operator_gt() {
        let q = parse(&json!({ "total": { ">": 100 } })).unwrap();
        assert!(matches!(&q.fields["total"], FieldEntry::Operator(Operator::Gt(_))));
    }

    #[test]
    fn test_operator_in() {
        let q = parse(&json!({ "status": { "|": ["pending", "processing"] } })).unwrap();
        assert!(matches!(&q.fields["status"], FieldEntry::Operator(Operator::In(_))));
    }

    #[test]
    fn test_nested() {
        let q = parse(&json!({
            "user": {
                "id": 1,
                "name": "*",
                "address": {
                    "city": "*",
                    "country": "US"
                }
            }
        })).unwrap();

        let user = match &q.fields["user"] {
            FieldEntry::Nested(node) => node,
            _ => panic!("expected nested node"),
        };

        assert!(matches!(user.fields["name"], FieldEntry::Projection));
        assert!(matches!(&user.fields["id"], FieldEntry::Match(Value::Number(_))));

        let address = match &user.fields["address"] {
            FieldEntry::Nested(node) => node,
            _ => panic!("expected nested node for address"),
        };

        assert!(matches!(address.fields["city"], FieldEntry::Projection));
        assert!(matches!(&address.fields["country"], FieldEntry::Match(Value::String(s)) if s == "US"));
    }

    #[test]
    fn test_collection_opts() {
        let q = parse(&json!({
            "^": "total",
            "#": 10,
            "@": 20,
            "status": "pending"
        })).unwrap();

        assert_eq!(q.collection_opts.order_by.as_deref(), Some("total"));
        assert!(matches!(q.collection_opts.order_dir, Some(OrderDir::Asc)));
        assert_eq!(q.collection_opts.limit, Some(10));
        assert_eq!(q.collection_opts.offset, Some(20));
        assert!(matches!(&q.fields["status"], FieldEntry::Match(Value::String(s)) if s == "pending"));
    }

    #[test]
    fn test_not_an_object() {
        assert!(matches!(parse(&json!([1, 2, 3])), Err(ParseError::NotAnObject)));
    }

    #[test]
    fn test_full_example() {
        let input = r##"
        {
            "user": {
                "id": 1,
                "name": "*",
                "email": "*",
                "orders": {
                    "status": "pending",
                    "total": { ">": 100 },
                    "^": "total",
                    "#": 10,
                    "id": "*",
                    "total": "*"
                }
            }
        }"##;

        let q = parse_str(input).unwrap();
        let user = match &q.fields["user"] {
            FieldEntry::Nested(n) => n,
            _ => panic!("expected nested user"),
        };
        let orders = match &user.fields["orders"] {
            FieldEntry::Nested(n) => n,
            _ => panic!("expected nested orders"),
        };

        assert!(matches!(orders.fields["id"], FieldEntry::Projection));
        assert!(matches!(orders.fields["total"], FieldEntry::Projection));
        assert!(matches!(&orders.fields["status"], FieldEntry::Match(_)));
        assert!(matches!(&orders.fields["total"], FieldEntry::Projection));
        assert!(matches!(orders.collection_opts.limit, Some(10)));
        assert!(matches!(orders.collection_opts.order_by.as_deref(), Some("total")));
    }
}
