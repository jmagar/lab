# Neo4j Upstream Contract

Neo4j is not an HTTP service in Lab. The integration uses Bolt through `neo4rs 0.9.0-rc.9`.

## Connection

Environment:

- `NEO4J_URL`: `neo4j://`, `neo4j+s://`, `bolt://`, or `bolt+s://` URL.
- `NEO4J_USER`: basic auth username.
- `NEO4J_PASSWORD`: basic auth password.
- `NEO4J_DB`: optional default database.
- `NEO4J_POOL_SIZE`: optional connection pool size, default `16`.
- `NEO4J_CA_CERT_PATH`: optional CA certificate for self-signed TLS.

`neo4j+ssc://` and `bolt+ssc://` are intentionally rejected so insecure TLS bypass does not become a default path.

## Actions

| Action | Upstream operation | Destructive |
| --- | --- | --- |
| `cypher.read` | `Graph::execute_read` | no |
| `cypher.write` | `Graph::run` | yes |
| `schema.labels` | `SHOW LABELS` | no |
| `schema.relationships` | `SHOW RELATIONSHIP TYPES` | no |
| `schema.constraints` | `SHOW CONSTRAINTS` | no |
| `schema.indexes` | `SHOW INDEXES` | no |
| `db.list` | `SHOW DATABASES` | no |
| `db.info` | `SHOW DATABASE` or named database | no |
| `server.status` | `CALL dbms.components()` | no |
| `txn.run` | sequential read calls or one write transaction | yes |

## Bounds

Cypher results are returned as JSON rows keyed by column name. Lab caps materialized rows at 500 and marks the response as truncated when additional rows are observed. Every Bolt operation has a 30 second timeout.
