# JQBE — Operator Symbols

## Field Operators

| Symbol | Function |
|--------|----------|
| `*` | Project — include this field in the response |
| `?` | Match only — apply constraint but do not include field in response |

## Comparison Operators

| Symbol | Function |
|--------|----------|
| `>` | Greater than |
| `>=` | Greater than or equal |
| `<` | Less than |
| `<=` | Less than or equal |
| `!=` | Not equal |
| `\|` | In — matches any value in the array |

## Collection Operators

| Symbol | Function |
|--------|----------|
| `^` | Order ascending by the specified field |
| `v` | Order descending by the specified field |
| `#` | Limit — return at most n members |
| `@` | Offset — skip the first n members |

## Planned Operators

| Symbol | Function |
|--------|----------|
| `~` | Pattern match — filter where field matches a wildcard or regex pattern |
| `!` | NOT — negate a constraint |
