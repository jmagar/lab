# ─── Stage 1: Build ───────────────────────────────────────────────────────────
# Requires Rust 1.90+ (Cargo.toml: rust-version = "1.90", edition = "2024")
FROM rust:1-slim AS builder

WORKDIR /build

# System build deps:
#   pkg-config     — lets -sys crates find system headers
#   build-essential — C compiler for rusqlite's bundled SQLite + other -sys crates
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      pkg-config \
      build-essential \
 && rm -rf /var/lib/apt/lists/*

# ── Dependency-caching layer ──────────────────────────────────────────────────
# Copy only manifests so the heavy dep-compile layer is invalidated only when
# Cargo.toml / Cargo.lock changes, not on every src edit.
COPY Cargo.toml Cargo.lock ./
COPY crates/lab/Cargo.toml      crates/lab/Cargo.toml
COPY crates/lab-apis/Cargo.toml crates/lab-apis/Cargo.toml
COPY crates/lab-auth/Cargo.toml crates/lab-auth/Cargo.toml

# Create stub source + test files so cargo can parse ALL manifests without error.
#
# lab-apis/Cargo.toml declares explicit [[test]] targets with required-features:
#   plex_client, gotify_client → need tests/plex_client.rs, tests/gotify_client.rs
# Cargo validates file existence at manifest-parse time even when the required
# feature is disabled, so we must stub them.
#
# lab/ has both [[bin]] (main.rs) and [lib] (lib.rs) targets — stub both.
#
# We also need apps/gateway-admin/out/ to exist for include_dir!() in web.rs,
# but we copy the real one below — for the dep-cache step a stub dir suffices.
RUN mkdir -p \
      crates/lab/src \
      crates/lab-apis/src \
      crates/lab-apis/tests \
      crates/lab-auth/src \
      apps/gateway-admin/out \
 && echo 'fn main(){}' > crates/lab/src/main.rs \
 && touch \
      crates/lab/src/lib.rs \
      crates/lab-apis/src/lib.rs \
      crates/lab-apis/tests/plex_client.rs \
      crates/lab-apis/tests/gotify_client.rs \
      crates/lab-auth/src/lib.rs \
 && cargo build --release -p 'lab@0.11.1' \
 && rm -rf \
      crates/*/src \
      crates/lab-apis/tests \
      apps/gateway-admin/out \
      target/release/lab \
      target/release/liblab_apis* \
      target/release/liblab_auth* \
      "target/release/.fingerprint/lab-"* \
      "target/release/.fingerprint/lab_apis-"* \
      "target/release/.fingerprint/lab_auth-"*

# ── Real source + frontend assets ─────────────────────────────────────────────
# The include_dir!() macro in crates/lab/src/api/web.rs embeds the Next.js
# static export at compile time. The out/ dir must exist before `cargo build`.
# (.dockerignore excludes apps/gateway-admin/{node_modules,.next,...} — only
# apps/gateway-admin/out is sent in the build context.)
COPY apps/gateway-admin/out apps/gateway-admin/out
COPY crates/ crates/
RUN cargo build --release -p 'lab@0.11.1'

# ─── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# ca-certificates: reqwest uses rustls for TLS but still validates against the
# system CA bundle — required for outbound HTTPS calls.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Non-root service account (uid 1000 keeps volume perms sane).
RUN useradd --system --create-home --uid 1000 --shell /usr/sbin/nologin lab

# Pre-create XDG directories so the binary never needs mkdir at runtime.
#   ~/.local/state/lab  — master.lock and runtime state
#   ~/.local/share/lab  — SQLite databases (node log, registry, auth)
#   ~/.config/lab       — config.toml (mounted read-only from host)
RUN mkdir -p \
      /home/lab/.config/lab \
      /home/lab/.local/state/lab \
      /home/lab/.local/share/lab \
 && chown -R lab:lab /home/lab

COPY --from=builder /build/target/release/lab /usr/local/bin/lab

USER lab
WORKDIR /home/lab

# Default HTTP port.  Override with LAB_MCP_HTTP_PORT env var or config.toml.
EXPOSE 8765

ENTRYPOINT ["/usr/local/bin/lab"]
# Master HTTP mode by default.
# Override CMD to ["serve","mcp","--stdio"] for the stdio-only MCP node.
CMD ["serve"]
