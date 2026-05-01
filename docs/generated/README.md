# Generated Docs

This directory contains generated service catalogs.

| File | Source |
| --- | --- |
| [cli-help.md](./cli-help.md) | Recursive `lab` CLI help output |
| [mcp-help.md](./mcp-help.md) | Runtime service/action catalog rendered for humans |
| [mcp-help.json](./mcp-help.json) | Runtime service/action catalog JSON |

## Refreshing

There is not yet a checked-in `just` recipe that refreshes every generated
catalog. Until one exists, regenerate these files from a freshly built
all-features binary and compare against the current checkout before committing.

Known current commands:

```bash
cargo run --quiet --package lab@0.12.1 --all-features -- --json help > docs/generated/mcp-help.json
cargo run --quiet --package lab@0.12.1 --all-features -- help > docs/generated/mcp-help.md
```

`cli-help.md` is recursive CLI output and should be refreshed with the same
binary whenever command flags or subcommands change.

