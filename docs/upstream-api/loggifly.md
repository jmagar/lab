# LoggiFly Source Contract

Retrieved: 2026-05-01

Sources:
- https://clemcer.github.io/LoggiFly/
- https://clemcer.github.io/LoggiFly/guide/config_sections/
- https://clemcer.github.io/LoggiFly/guide/healthcheck

LoggiFly monitors Docker logs/events and sends notifications or triggers actions from config files, environment variables, and Docker labels. No stable safe HTTP API was found in public docs. Lab therefore implements the documented local heartbeat-file health contract and a redacted local config summary instead of Docker socket, log, or notification operations.

## V1 Actions

| Action | Contract | Hosted posture |
|---|---|---|
| `contract.status` | returns local contract status and deferred unsafe surfaces | safe |
| `health.status` | inspects the heartbeat file written when `ENABLE_HEALTHCHECK=true` | local-only |
| `config.summary` | summarizes `config.yaml` under `LOGGIFLY_CONFIG_ROOT` without raw config | local-only |

## Deferred

Full config validation, Docker socket access, raw logs, Docker labels, remote Docker hosts, OliveTin triggers, container actions, notification tests, and notification sends are deferred.

## Security

Notification URLs, webhook headers, Docker host addresses, OliveTin credentials, labels, keyword matches, and log snippets are sensitive. `config.summary` returns only file metadata and section booleans; it never returns raw config contents.
