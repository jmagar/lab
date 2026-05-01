# Generated Docs

Files in this directory are generated from code-owned metadata.

Regenerate all artifacts with:

```bash
just docs-generate
```

Verify tracked artifacts are fresh with:

```bash
just docs-check
```

`docs-check` compares the declared generated artifact manifest and enforces generated-doc invariants such as feature-matrix consistency and safety linting. It does not run Markdown link checks, live service health, or onboarding audit policy gates.

| Artifact | Source |
| --- | --- |
| `service-catalog.md/json` | `lab docs generate` |
| `action-catalog.md/json` | `lab docs generate` |
| `env-reference.md/json` | `lab docs generate` |
| `api-routes.md/json` | `lab docs generate` |
| `openapi.json` | `lab docs generate` |
| `feature-matrix.md/json` | `lab docs generate` |
| `onboarding-audit.md/json` | `lab docs generate` |
| `mcp-help.md/json` | `lab docs generate` |
| `cli-help.md` | `lab docs generate` |
