# LoggiFly Source Contract

Retrieved: 2026-05-01

Sources:
- https://clemcer.github.io/LoggiFly/
- https://clemcer.github.io/LoggiFly/guide/config_sections/
- https://clemcer.github.io/LoggiFly/guide/healthcheck

LoggiFly monitors Docker logs/events and sends notifications or triggers actions from config files, environment variables, and Docker labels. No stable safe HTTP API was found in public docs. Lab therefore classifies implementation as deferred.

## V1 Actions

| Action | Contract | Hosted posture |
|---|---|---|
| `contract.status` | returns deferral and deferred surfaces | safe |

## Deferred

Config parsing, config validation, heartbeat file reads, Docker socket access, raw logs, Docker labels, remote Docker hosts, OliveTin triggers, container actions, notification tests, and notification sends are deferred until an allowlisted config/health contract is approved.

## Security

Notification URLs, webhook headers, Docker host addresses, OliveTin credentials, labels, keyword matches, and log snippets are sensitive.

