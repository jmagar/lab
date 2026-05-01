# Neo4j Onboarding Coverage

Neo4j is wired as a first-class non-HTTP service using the Bolt protocol through `neo4rs`.

Implemented actions:

- `cypher.read`
- `cypher.write`
- `schema.labels`
- `schema.relationships`
- `schema.constraints`
- `schema.indexes`
- `db.list`
- `db.info`
- `server.status`
- `txn.run`

Security and bounds:

- Only `neo4j://`, `neo4j+s://`, `bolt://`, and `bolt+s://` URLs are accepted.
- `neo4j+ssc://` and `bolt+ssc://` are rejected; use `NEO4J_CA_CERT_PATH` for self-signed deployments.
- Passwords in connection URLs are redacted before diagnostic display.
- Cypher parameters are passed through Bolt parameters, not string interpolation.
- Read actions use Neo4j read execution; write actions are marked destructive.
- Result materialization is capped at 500 rows and sets `truncated: true` when exceeded.
- Bolt calls are wrapped in a 30 second timeout.
