---
name: mcp-registry-publish
description: Authoritative procedure for publishing an MCP server to the official MCP registry at registry.modelcontextprotocol.io — package metadata, server.json schema, authentication, validation, publish, and verification. Preloaded by mcp-publisher.
user-invocable: false
---

## Reference docs (read locally before going to the network)

The `references/` directory next to this SKILL.md contains the upstream MCP registry docs as of the last refresh:

| Local file | Upstream source | Use when |
|---|---|---|
| `references/reference-overview.md` | `docs/reference/README.md` | Orienting — top-level index of the registry's reference material |
| `references/cli-commands.md` | `docs/reference/cli/commands.md` | Looking up `mcp-publisher` subcommands, flags, exit codes |
| `references/official-registry-requirements.md` | `docs/reference/server-json/official-registry-requirements.md` | Validating identity rules, namespace ownership, what the official registry will accept vs. reject |
| `references/generic-server-json.md` | `docs/reference/server-json/generic-server-json.md` | The full `server.json` schema with every field documented in prose |
| `references/server.schema.json` | `https://static.modelcontextprotocol.io/schemas/<version>/server.schema.json` | The actual JSON Schema — feed to `jq`/`ajv` for offline `server.json` validation |
| `references/.schema-version` | (derived) | One-line file recording which schema version is currently checked in |

Read those before falling back to the live URLs. The upstream quickstart lives at `https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/quickstart.mdx`.

### Refreshing the references

```bash
.claude/skills/mcp-registry-publish/scripts/refresh-references.sh
```

The script auto-detects the current schema version from the upstream markdown — no manual editing needed when the schema bumps. Run it whenever the upstream MCP registry docs change. After running, commit the refreshed files so the team is on the same version.

### Validating `server.json` locally (REQUIRED before publishing)

```bash
.claude/skills/mcp-registry-publish/scripts/validate-server-json.sh [path/to/server.json]
```

Catches the documented top-3 publish failures BEFORE the irreversible `npm publish` / `pypi upload` / `docker push` step:

1. **Schema violations** — validates against the local `server.schema.json` using whichever validator is available (Python `jsonschema`, `check-jsonschema`, or `npx ajv-cli` as zero-install fallback)
2. **Name mismatch** — `server.json` `name` ↔ package manifest `mcpName` (or `[tool.mcp].name` for PyPI)
3. **Version mismatch** — `server.json` `version` ↔ package manifest `version`
4. **Namespace rule** — `name` matches `io.github.<user>/<server>` for GitHub auth

Exit codes: 0 pass · 1 schema fail · 2 cross-manifest mismatch · 3 namespace violation · 4 file missing · 5 no validator available.

Run this at every iteration of `server.json` and **always** as the gate before `mcp-publisher publish`. A failed publish doesn't roll back an already-published npm package.

The registry distinguishes a server's **identity** (`name` like `io.github.user/server-name`) from its **distribution package** (the npm/pypi/docker/etc. artifact). Both need to exist before publishing.

---

## §0 — Pre-flight checks

Before any publishing attempt, verify:

- The MCP server actually runs locally (`mcp-publisher publish` won't catch broken servers — it only validates metadata)
- A GitHub repo exists for the server source (the registry uses GitHub-based auth, and the server name namespace is tied to the GitHub username)
- The user has decided which registry type backs the package: **npm**, **PyPI**, **Docker/OCI**, or **NuGet**. The procedure differs at the package-publish step but the registry-publish step is universal.
- The user is logged in to whichever package registry they're using (`npm whoami`, `pip` credentials, `docker login`, etc.)

If any of these are missing, surface the gap before scaffolding `server.json`.

## §1 — Identity rules (critical)

The `name` field in `server.json` and the equivalent metadata field in the package manifest **must agree**. They take the form:

```
io.github.<github-username>/<server-name>
```

- The `io.github.` prefix is required for the default GitHub-based authentication path
- `<github-username>` must match the GitHub account that authenticates with `mcp-publisher login github`
- `<server-name>` is the actual server identifier (lowercase, dash-separated)

**Mismatches between these manifests are the #1 cause of publish failures.** Always verify before invoking the publisher.

| Package registry | Where to put the matching name | Field |
|---|---|---|
| npm | `package.json` | `mcpName` |
| PyPI | `pyproject.toml` | `[project] name` (must equal `name` in `server.json`) or use `mcp.name` extra-metadata field |
| Docker / OCI | image labels — see registry docs for the current label key |
| NuGet | `.nuspec` — see registry docs for the current property name |

If publishing a Rust binary (rmcp + cargo-dist), there's no npm-style metadata field — instead, the `server.json` `packages[]` entry references the GitHub release artifact directly with the appropriate `registryType`.

## §2 — Procedure (npm-backed example)

This is the documented happy path. Adapt §3 for other registry types.

### 2a. Prepare `package.json`

```json
{
  "name": "@<github-username>/<server-name>",
  "version": "1.0.1",
  "mcpName": "io.github.<github-username>/<server-name>",
  "description": "Brief description",
  "repository": {
    "type": "git",
    "url": "https://github.com/<github-username>/<repo>.git"
  }
}
```

The `mcpName` field is the contract surface — without it the registry will reject the publish with `"Registry validation failed for package"`.

### 2b. Publish the package

```bash
npm install
npm run build
npm adduser              # only on first publish
npm publish --access public
```

Verify the package landed: `https://www.npmjs.com/package/@<github-username>/<server-name>`.

The package must be public — the registry pulls metadata from the registry's public API, not from authenticated endpoints.

### 2c. Install `mcp-publisher` CLI

macOS/Linux:
```bash
curl -L "https://github.com/modelcontextprotocol/registry/releases/latest/download/mcp-publisher_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/').tar.gz" \
  | tar xz mcp-publisher \
  && sudo mv mcp-publisher /usr/local/bin/
```

Windows (PowerShell):
```powershell
$arch = if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq "Arm64") { "arm64" } else { "amd64" }
Invoke-WebRequest -Uri "https://github.com/modelcontextprotocol/registry/releases/latest/download/mcp-publisher_windows_$arch.tar.gz" -OutFile "mcp-publisher.tar.gz"
tar xf mcp-publisher.tar.gz mcp-publisher.exe
rm mcp-publisher.tar.gz
```

### 2d. Generate and edit `server.json`

```bash
mcp-publisher init
```

The generated stub:

```json
{
  "$schema": "https://static.modelcontextprotocol.io/schemas/<date>/server.schema.json",
  "name": "io.github.<github-username>/<server-name>",
  "description": "Brief description",
  "repository": {
    "url": "https://github.com/<github-username>/<repo>",
    "source": "github"
  },
  "version": "1.0.1",
  "packages": [
    {
      "registryType": "npm",
      "identifier": "@<github-username>/<server-name>",
      "version": "1.0.1",
      "transport": { "type": "stdio" }
    }
  ]
}
```

Field-by-field:

| Field | Required | Notes |
|---|---|---|
| `$schema` | yes | Pin to the URL `mcp-publisher init` emits — this version-locks validation |
| `name` | yes | Identity (`io.github.<user>/<server>`); must equal package metadata `mcpName` |
| `description` | yes | One-line, shown in registry listings |
| `repository.url` | yes | Public GitHub URL |
| `repository.source` | yes | `github` (other sources supported but undocumented in quickstart) |
| `version` | yes | Semver; must match the published package version |
| `packages[]` | yes | At least one entry |
| `packages[].registryType` | yes | `npm`, `pypi`, `oci`, `nuget`, etc. — see latest schema |
| `packages[].identifier` | yes | Package id within that registry (e.g. `@scope/name`, `pip-name`, `docker-image:tag`) |
| `packages[].version` | yes | Must match the published package version |
| `packages[].transport.type` | yes | `stdio`, `http`, or `sse` (transport the server speaks) |
| `packages[].transport.url` | conditional | Required when `transport.type` is `http` or `sse` |

### 2e. Authenticate

```bash
mcp-publisher login github
```

Follow the GitHub device-flow prompts (the CLI prints a URL + code; user opens browser, enters code, authorizes the OAuth app). Credentials cache locally — only the first publish needs the flow.

### 2f. Publish

```bash
mcp-publisher publish
```

The publisher validates `server.json` against the schema, verifies the package metadata's `mcpName` matches, and submits the entry. Successful runs print the registry entry URL.

### 2g. Verify

```bash
curl "https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.<github-username>/<server-name>"
```

Should return the published entry within seconds. Registry indexing is near-real-time.

## §3 — Adaptations per registry type

The §2c–§2g steps are universal. Only §2a–§2b changes.

### PyPI

Replace `package.json` with `pyproject.toml`:

```toml
[project]
name = "<your-pypi-name>"
version = "1.0.1"
description = "..."

[tool.mcp]
name = "io.github.<github-username>/<server-name>"
```

(Check the latest schema — the PyPI metadata key has been in flux. `mcp-publisher init` is authoritative.)

Publish via `python -m build && twine upload dist/*`. The `packages[].registryType` becomes `pypi`, `identifier` is the PyPI package name.

### Docker / OCI

Image must be public. Tag and push to a registry (Docker Hub, GHCR, etc.). The `packages[].registryType` becomes `oci`, `identifier` is the full image reference (`ghcr.io/<user>/<server>:<tag>`).

### Rust binary (cargo-dist)

The MCP registry doesn't have a first-class "Rust binary" registry type. Two viable paths:

1. **Wrap the binary in npm**: publish a thin npm package that downloads the appropriate platform binary from your GitHub release on `npm install`. Several MCP servers use this pattern (e.g. `@modelcontextprotocol/server-filesystem`).
2. **Use the OCI path**: ship a Docker image with the binary inside, reference it as `oci`.

Path 1 keeps the user-facing install simple (`npm install` works). Path 2 keeps it pure-Rust but requires Docker on the user's side.

On rmcp (the default for this repo), path 1 is usually the right call — and the npm wrapper can be auto-generated by cargo-dist's `npm-installer` feature if enabled.

## §4 — Common failure modes

| Symptom | Cause | Fix |
|---|---|---|
| `Registry validation failed for package` | `mcpName` (or equivalent) missing from package metadata | Add the field, republish the package, retry |
| `Invalid or expired Registry JWT token` | Auth cache expired | Re-run `mcp-publisher login github` |
| `You do not have permission to publish` | Server name doesn't match GitHub username | Server names using GitHub auth MUST start with `io.github.<your-github-username>/` |
| Publish succeeds but server doesn't show in search | Indexing lag (rare) or wrong search query | Wait 30s, query by exact name not partial |
| `Schema validation failed: ...` | `server.json` missing required field or bad `$schema` URL | Re-run `mcp-publisher init`, copy the schema URL it generates |
| `Package version mismatch` | `server.json` version ≠ published package version | Bump `server.json` to match, or republish package at the version `server.json` claims |

## §5 — Validation checklist (REQUIRED before declaring done)

- [ ] Package is published and publicly accessible (browse the registry URL, not just `npm publish` exit code)
- [ ] `mcpName` (or registry-specific equivalent) in package metadata matches `server.json` `name` exactly
- [ ] `server.json` parses (`jq . server.json`)
- [ ] `server.json` `version` matches the published package version
- [ ] GitHub repository URL in `server.json` is public and resolves
- [ ] `mcp-publisher publish` exited 0 and printed the registry entry URL
- [ ] Verification curl returns the entry within 30 seconds
- [ ] The transport type and any required transport URL match what the server actually speaks

Failing any of these means the publish isn't done.
