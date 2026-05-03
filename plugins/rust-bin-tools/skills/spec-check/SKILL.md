---
name: spec-check
description: Run the xtask enforcement gates before declaring work complete. Use before finishing any Rust code change.
user-invocable: false
---

Before completing any Rust code change, run these checks:

```bash
cargo xtask check-no-mod-rs       # no mod.rs files
cargo xtask check-actions         # action catalog integrity
cargo xtask check-tracing-fields  # log field constants used (not literals)
cargo xtask check-error-mapping   # HttpError → ToolError mapping complete
cargo xtask check-file-size       # no monolithic files
cargo xtask check-env-allowlist   # no unauthorized env vars
```

If any fail, fix before reporting complete. Do not claim success without seeing all checks pass.
