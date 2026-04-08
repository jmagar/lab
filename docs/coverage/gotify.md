# Gotify API Coverage

**Last updated:** 2026-04-08
**OpenAPI spec:** docs/api-specs/gotify.openapi.json
**OpenAPI version:** -
**API version:** 2.0.2
**Paths:** 32

## Legend

| Symbol | Meaning |
|--------|---------|
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Endpoint Inventory

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /application | getApps | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /application | createApp | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /application/{id} | deleteApp | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /application/{id} | updateApplication | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /application/{id}/image | removeAppImage | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /application/{id}/image | uploadAppImage | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /application/{id}/message | deleteAppMessages | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /application/{id}/message | getAppMessages | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /client | getClients | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /client | createClient | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /client/{id} | deleteClient | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /client/{id} | updateClient | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /current/user | currentUser | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /current/user/password | updateCurrentUser | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /health | getHealth | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /message | deleteMessages | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /message | getMessages | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /message | createMessage | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /message/{id} | deleteMessage | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /plugin | getPlugins | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /plugin/{id}/config | getPluginConfig | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /plugin/{id}/config | updatePluginConfig | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /plugin/{id}/disable | disablePlugin | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /plugin/{id}/display | getPluginDisplay | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /plugin/{id}/enable | enablePlugin | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /stream | streamMessages | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user | getUsers | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user | createUser | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /user/{id} | deleteUser | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{id} | getUser | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{id} | updateUser | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /version | getVersion | ⬜ | ⬜ | ⬜ | ⬜ |
