# Superseded Plan

This plan name is retained as a pointer only.

The external-issuer and remote-JWKS design was superseded by the approved internal auth-server plan:

- [2026-04-14-lab-auth-google-mcp-oauth.md](./2026-04-14-lab-auth-google-mcp-oauth.md)

Current direction:

- `LAB_AUTH_MODE=bearer|oauth`
- bearer mode keeps `LAB_MCP_HTTP_TOKEN`
- oauth mode runs `lab`'s own Google-backed authorization server
- the old `LAB_OAUTH_*` external issuer path is removed
