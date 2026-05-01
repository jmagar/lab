# NotebookLM Upstream Contract

**Last updated:** 2026-05-01
**Status:** Feature-gated SDK/service contract

NotebookLM does not publish a stable OpenAPI document for the flows used by
`lab`. The current integration follows the community `teng-lin/notebooklm-py`
RPC contract and treats that behavior as the upstream source.

## Source

- Reference implementation: `teng-lin/notebooklm-py`
- API style: private RPC/web contract
- Stability posture: best-effort; upstream behavior may change without a
  versioned schema

## Configuration

NotebookLM service metadata defines profile/auth-oriented environment
variables. Keep browser-derived credentials and profile state out of logs and
generated docs examples.

## Supported Areas

The coverage document owns the exact action inventory. This upstream note only
records the source and stability expectations because there is no vendor schema
to vendor under `docs/upstream-api/`.

## Unsupported Areas

- No official OpenAPI refresh command.
- No guarantee that upstream private RPC shapes are stable across NotebookLM
  UI releases.

