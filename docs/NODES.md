# Nodes

`nodes` is the canonical operator surface for Lab.

## Vocabulary

- `node`: one managed runtime
- `nodes`: the operator control surface
- `controller`: the coordinating node for the runtime
- `node_id`: the only public runtime identifier

## Public Surface

Canonical CLI:

```bash
lab nodes list
lab nodes get squirts
lab nodes enrollments list
```

Canonical HTTP routes:

- `GET /v1/nodes`
- `GET /v1/nodes/{node_id}`
- `GET /v1/nodes/enrollments`
- `POST /v1/nodes/enrollments/{node_id}/approve`
- `POST /v1/nodes/enrollments/{node_id}/deny`
- `POST /v1/nodes/logs/search`
- `POST /v1/nodes/hello`
- `POST /v1/nodes/oauth/relay/start`
- `POST /v1/nodes/syslog/batch`
- `GET /v1/nodes/ws`

Canonical websocket contract:

- initialize metadata key: `lab.node_id`
- method: `nodes/status.push`
- method: `nodes/metadata.push`
- method: `nodes/log.event`

Canonical config:

```toml
[node]
controller = "tootie"
```

## Serialization Boundary

Per `docs/design/SERIALIZATION.md`:

- `lab-apis` owns SDK wire models
- `lab` owns product-surface routes, payloads, envelopes, and presentation

Fresh product output uses `node_id` only.
