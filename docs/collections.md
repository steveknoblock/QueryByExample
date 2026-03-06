# JQBE — What Constitutes a Collection

A collection is any node in the document tree whose value is a JSON array.

In the `serde_json` type system that means any `Value::Array`. When the
executor encounters a `Nested` field entry and finds that the corresponding
document value is a `Value::Array` rather than a `Value::Object`, it treats
it as a collection and runs `execute_collection` instead of recursing with
`execute_node`.

## Document Determines the Collection

The distinction is entirely determined by the document, not the query. The
query does not declare whether a field is a collection or a single document
— it just describes the shape and constraints for that field. The executor
looks at what is actually in the document at that position and decides which
path to take.

## Example

```json
{
  "users": [
    {
      "id": 1,
      "name": "Alice",
      "address": {
        "city": "New York"
      },
      "orders": [
        { "id": 42, "total": 150 }
      ]
    }
  ]
}
```

| Field | Type | Treated As |
|-------|------|------------|
| `users` | `Value::Array` | Collection |
| `id` | `Value::Number` | Scalar |
| `name` | `Value::String` | Scalar |
| `address` | `Value::Object` | Nested document |
| `city` | `Value::String` | Scalar |
| `orders` | `Value::Array` | Collection |

Both `users` and `orders` are collections because they are arrays. `address`
is a nested document because it is an object. `id`, `name`, and `city` are
scalars.

## Collection Contents

A collection can contain any JSON value in principle, but in practice a
useful collection contains objects — because the query needs fields to match
and project against. A collection of bare scalars like:

```json
["pending", "complete", "pending"]
```

would not be queryable with the current model since there are no named fields
to constrain or project.
