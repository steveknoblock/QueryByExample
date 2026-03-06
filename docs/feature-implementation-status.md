# JQBE — Feature Implementation Status

| Feature | Function | Query Syntax | Implemented |
|---------|----------|--------------|-------------|
| Projection | Include a field in the response with no constraint | `"field": "*"` | ✅ |
| Literal Match and Project | Filter documents where field equals value, include field in response | `"field": <value>` | ✅ |
| Match Only Literal | Filter documents where field equals value, do not include field in response | `"field": { "?": <value> }` | ✅ |
| Greater Than | Filter where field is greater than value | `"field": { ">": <value> }` | ✅ |
| Greater Than or Equal | Filter where field is greater than or equal to value | `"field": { ">=": <value> }` | ✅ |
| Less Than | Filter where field is less than value | `"field": { "<": <value> }` | ✅ |
| Less Than or Equal | Filter where field is less than or equal to value | `"field": { "<=": <value> }` | ✅ |
| Not Equal | Filter where field does not equal value | `"field": { "!=": <value> }` | ✅ |
| In | Filter where field matches any value in a list | `"field": { "\|": [<value>, ...] }` | ✅ |
| Match Only Operator | Apply a comparison operator as a filter without projecting the field | `"field": { "?": { <op>: <value> } }` | ✅ |
| Nested Document | Apply query recursively to a nested object | `"field": { ... }` | ✅ |
| Collection | Apply query to each member of an array, returning matching members | `"field": { ... }` against an array | ✅ |
| Order Ascending | Sort collection results ascending by a field | `"^": "field"` | ✅ |
| Order Descending | Sort collection results descending by a field | `"v": "field"` | ✅ |
| Limit | Return at most n members from a collection | `"#": <n>` | ✅ |
| Offset | Skip the first n members of a collection | `"@": <n>` | ✅ |
| Multi-level URL paths | Serve documents at nested URL paths from a directory tree | `QUERY /path/to/resource` | ✅ |
| In-memory caching | Cache documents in memory after first load from disk | Server-side, transparent to client | ✅ |
| 404 for missing documents | Return error when no document exists at the requested path | Server-side, transparent to client | ✅ |
| Strict field checking | Return error when query references a field not present in the document | Server-side, transparent to client | ✅ |
| String Pattern Match | Filter where field matches a wildcard or regex pattern | `"field": { "~": "<pattern>" }` | ❌ |
| Aggregation — Count | Return the count of matching collection members | `"field": { "$count": "*" }` | ❌ |
| Aggregation — Sum | Return the sum of a numeric field across matching members | `"field": { "$sum": "<field>" }` | ❌ |
| Aggregation — Average | Return the average of a numeric field across matching members | `"field": { "$avg": "<field>" }` | ❌ |
| NOT Operator | Negate a constraint | `"field": { "!": <value> }` | ❌ |
| Cache Invalidation | Reload a document when its file changes on disk | Server-side, transparent to client | ❌ |
| Empty Collection Exclusion | Exclude parent documents that have no matching children | Query option — syntax TBD | ❌ |
