# Lab — Development Commands

default:
    @just --list

# Check all crates compile
check:
    cargo check --workspace --all-features

# Run all tests
test:
    cargo nextest run --workspace --all-features

# Run integration tests (requires running services)
test-integration:
    cargo nextest run --workspace --all-features -- --ignored

# Lint
lint:
    cargo clippy --workspace --all-features -- -D warnings
    cargo fmt --all -- --check

# License and vulnerability audit
deny:
    cargo deny check

# Build debug binary with all features
build:
    cargo build --workspace --all-features

# Build release binary with all features
build-release:
    cargo build --workspace --all-features --release

# Run with args
run *ARGS:
    cargo run --all-features -- {{ARGS}}

# Run the binary-served static chat UI in local ACP mode
chat-local:
    #!/usr/bin/env bash
    set -euo pipefail
    export LAB_WEB_UI_DISABLE_AUTH=true
    export LAB_MCP_HTTP_TOKEN="${LAB_MCP_HTTP_TOKEN:-dev-token}"
    export LAB_CORS_ORIGINS="${LAB_CORS_ORIGINS:-http://dookie:3000,http://127.0.0.1:3000,http://localhost:3000}"
    export LAB_CHAT_LOCAL_PORT="${LAB_CHAT_LOCAL_PORT:-8766}"
    cargo run --all-features --bin lab -- serve --host 0.0.0.0 --port "${LAB_CHAT_LOCAL_PORT}"

# Format all code
fmt:
    cargo fmt --all

# Clean build artifacts
clean:
    cargo clean

# Release (version bump + tag + push)
release *ARGS:
    cargo release {{ARGS}}

# Generate a secure MCP HTTP bearer token and write it to .env
mcp-token:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f .env ]; then
        echo "error: .env not found — copy .env.example first" >&2
        exit 1
    fi
    token=$(openssl rand -hex 32)
    if grep -q '^LAB_MCP_HTTP_TOKEN=' .env; then
        # macOS/BSD sed compat: write to tmp then move
        tmp=$(mktemp)
        awk -v t="$token" '/^LAB_MCP_HTTP_TOKEN=/{print "LAB_MCP_HTTP_TOKEN=" t; next} {print}' .env > "$tmp"
        mv "$tmp" .env
        echo "✓ rotated LAB_MCP_HTTP_TOKEN in .env"
    else
        echo "LAB_MCP_HTTP_TOKEN=$token" >> .env
        echo "✓ appended LAB_MCP_HTTP_TOKEN to .env"
    fi
    echo "  $token"
