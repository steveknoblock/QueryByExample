# JSON Query by Example — curl Examples

These examples assume the server is running locally on port 3000 with the
sample `document.json` loaded.

```bash
cargo run -- document.json
```

---

## Example 1 — Filter by status, project name and email

Return name and email for all active users.

```bash
curl -X QUERY http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "users": {
      "status": "active",
      "name": "*",
      "email": "*"
    }
  }'
```

**Response:**

```json
{
  "users": [
    {
      "email": "alice@example.com",
      "name": "Alice",
      "status": "active"
    },
    {
      "email": "bob@example.com",
      "name": "Bob",
      "status": "active"
    }
  ]
}
```

Carol is excluded because her status is `"inactive"`.

---

## Example 2 — Nested collection with operator constraint

Return all users with their pending orders over 100, projecting name, order id,
product, and total.

```bash
curl -X QUERY http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "users": {
      "name": "*",
      "orders": {
        "status": "pending",
        "total": { ">": 100 },
        "id": "*",
        "product": "*"
      }
    }
  }'
```

**Response:**

```json
{
  "users": [
    {
      "name": "Alice",
      "orders": [
        { "id": 42, "product": "Widget A", "status": "pending", "total": 150 },
        { "id": 45, "product": "Widget D", "status": "pending", "total": 200 }
      ]
    },
    {
      "name": "Bob",
      "orders": []
    },
    {
      "name": "Carol",
      "orders": [
        { "id": 49, "product": "Widget F", "status": "pending", "total": 400 }
      ]
    }
  ]
}
```

`status` and `total` appear in the response because constraints imply
projection. Bob has no matching orders so he appears with an empty array.

---

## Example 3 — Match only operator, constraint without projection

Return id and product for orders over 100. The `total` field is used as a
filter but does not appear in the response.

```bash
curl -X QUERY http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "users": {
      "name": "*",
      "orders": {
        "total": { "?": { ">": 100 } },
        "id": "*",
        "product": "*"
      }
    }
  }'
```

**Response:**

```json
{
  "users": [
    {
      "name": "Alice",
      "orders": [
        { "id": 42, "product": "Widget A" },
        { "id": 44, "product": "Widget C" },
        { "id": 45, "product": "Widget D" }
      ]
    },
    {
      "name": "Bob",
      "orders": [
        { "id": 47, "product": "Widget E" },
        { "id": 48, "product": "Widget B" }
      ]
    },
    {
      "name": "Carol",
      "orders": [
        { "id": 49, "product": "Widget F" }
      ]
    }
  ]
}
```

`total` does not appear in the response. Order 43 (total 50), order 46
(total 75), and order 50 (total 100) are excluded since none are greater
than 100.

---

## Example 4 — Match only literal, filter without projection

Return names and cities of US users without returning the country field.

```bash
curl -X QUERY http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "users": {
      "address": {
        "country": { "?": "US" },
        "city": "*"
      },
      "name": "*"
    }
  }'
```

**Response:**

```json
{
  "users": [
    { "address": { "city": "New York" },    "name": "Alice" },
    { "address": { "city": "Los Angeles" }, "name": "Bob"   },
    { "address": { "city": "Chicago" },     "name": "Carol" }
  ]
}
```

`country` does not appear in the response even though it was used as a
filter constraint.
