# JQBE — JSON Query by Example

A lightweight HTTP server that enables querying of JSON documents using a
query-by-example model. Clients send a `QUERY` request with a JSON document
describing both the constraints and the shape of the data they want back.

---

## Building

Requires Rust 1.78 or later.

```bash
cargo build --release
```

---

## Running

```bash
cargo run -- <data-directory>
```

The server listens on `http://localhost:3000` by default.

If no data directory is provided the server starts with a built-in example
document available at `/users`.

---

## Data Directory Layout

Documents are served from a root data directory. The URL path of a request
maps directly to a `.json` file under that directory.

```
data/
  users.json              →  QUERY /users
  orders.json             →  QUERY /orders
  users/
    orders.json           →  QUERY /users/orders
    addresses.json        →  QUERY /users/addresses
  products/
    categories.json       →  QUERY /products/categories
```

The mapping rule is simple — take the URL path, treat all but the last
segment as directory components, and append `.json` to the last segment.

A `.json` file and a subdirectory may share the same base name at the same
level without conflict. `users.json` serves `QUERY /users` and the `users/`
directory contains documents served at deeper paths. They are independent.

Documents are loaded from disk on first request and cached in memory for
subsequent requests. The cache is held for the lifetime of the server
process. Restart the server to pick up changes to data files.

---

## Making a Query

Send a `QUERY` request with a JSON body describing what you want.

```bash
curl -X QUERY http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{
    "users": {
      "status": "active",
      "name": "*",
      "email": "*"
    }
  }'
```

---

## Query Document Syntax

### Field Semantics

| Value | Meaning |
|-------|---------|
| `"*"` | Project this field — include it in the response |
| `<value>` | Match and project — filter and include in response |
| `{ "?": <value> }` | Match only — filter but do not include in response |
| absent | Do not return this field |

### Comparison Operators

| Operator | Meaning |
|----------|---------|
| `{ ">": n }` | Greater than |
| `{ ">=": n }` | Greater than or equal |
| `{ "<": n }` | Less than |
| `{ "<=": n }` | Less than or equal |
| `{ "!=": n }` | Not equal |
| `{ "\|": [...] }` | Matches any value in the array |
| `{ "?": <op> }` | Match only — apply operator but do not project |

### Collection Operators

| Operator | Meaning |
|----------|---------|
| `"^": "field"` | Order ascending by field |
| `"v": "field"` | Order descending by field |
| `"#": n` | Limit — return at most n members |
| `"@": n` | Offset — skip the first n members |

---

## Query Examples

**Filter by field value, project selected fields:**

```json
{
  "users": {
    "status": "active",
    "name": "*",
    "email": "*"
  }
}
```

**Nested collection with operator constraint:**

```json
{
  "users": {
    "name": "*",
    "orders": {
      "total": { ">": 100 },
      "id": "*",
      "product": "*"
    }
  }
}
```

**Match only — filter without projecting the constraint field:**

```json
{
  "users": {
    "address": {
      "country": { "?": "US" },
      "city": "*"
    },
    "name": "*"
  }
}
```

**Collection ordering and pagination:**

```json
{
  "orders": {
    "^": "total",
    "#": 10,
    "@": 0,
    "id": "*",
    "total": "*",
    "product": "*"
  }
}
```

---

## Error Responses

All errors are returned as JSON with a consistent structure:

```json
{
  "error": {
    "code": "resource_not_found",
    "message": "No document found at this path",
    "detail": "/nonexistent"
  }
}
```

| Code | Meaning |
|------|---------|
| `malformed_document` | Request body is not valid JSON |
| `invalid_operator` | Unrecognised operator in query |
| `invalid_field` | Query references a field not present in the document |
| `invalid_value` | Wrong type for the operator applied |
| `resource_not_found` | No document exists at the requested path |
| `method_not_allowed` | Request method is not QUERY |

---

## Running Tests

```bash
cargo test
```

---

## Project Structure

```
src/
  main.rs       Entry point — parses arguments and starts the server
  server.rs     HTTP layer — routing and request handling
  store.rs      Document store — lazy loading and in-memory caching
  parser.rs     Query document parser
  executor.rs   Query executor — runs parsed queries against documents
data/
  users.json    Example document
```

---

## Specification

The full specification is available in `jqbe-spec.md` and `jqbe-spec.docx`.
