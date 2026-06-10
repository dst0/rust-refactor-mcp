#!/bin/bash
set -e

REPOS=(
    "https://github.com/ratatui-org/ratatui"
    "https://github.com/dtolnay/anyhow"
    "https://github.com/uuid-rs/uuid"
    "https://github.com/servo/rust-url"
    "https://github.com/tokio-rs/bytes"
    "https://github.com/hyperium/http"
    "https://github.com/rust-lang/regex"
    "https://github.com/image-rs/image"
    "https://github.com/mitsuhiko/minijinja"
    "https://github.com/env-logger-rs/env_logger"
)


TOOL="/rust-refactor-mcp/target/release/rust-refactor-mcp"
FAILURES=0

for REPO in "${REPOS[@]}"; do
    NAME=$(basename $REPO)
    echo "======================================"
    echo "Testing $NAME"
    echo "======================================"
    
    cd /test
    git clone --depth 1 $REPO
    cd $NAME
    
    echo "Baseline check..."
    if ! cargo check; then
        echo "Baseline failed for $NAME"
        FAILURES=$((FAILURES + 1))
        continue
    fi
    
    echo "Running ANALYZE_DEPS..."
    $TOOL . ANALYZE_DEPS . > /dev/null
    
    echo "Running FIND_DEAD_CODE..."
    $TOOL . FIND_DEAD_CODE . > /dev/null
    
    echo "Running SPLIT_DIR..."
    if [ -d "src" ]; then
        $TOOL SPLIT_DIR src > /dev/null
    fi
    
    echo "Running FIX_CARGO to cleanup unused imports..."
    $TOOL . FIX_CARGO Cargo.toml > /dev/null || true
    
    echo "Running PREFLIGHT..."
    if ! $TOOL . PREFLIGHT Cargo.toml > /dev/null; then
        echo "PREFLIGHT failed after split for $NAME"
        FAILURES=$((FAILURES + 1))
        continue
    fi
    
    echo "$NAME passed analysis tools."
done

echo "Total Failures: $FAILURES"
exit $FAILURES
