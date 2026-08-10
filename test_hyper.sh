#!/bin/bash
set -e

TOOL="/home/dst/dev/rust-refactor-mcp/target/release/rust-refactor-mcp"
REPO="/home/dst/dev/rust-refactor-mcp/hyper_repo"

echo "======================================"
echo "Testing hyper repo with MCP tools"
echo "======================================"

cd $REPO

echo "1. Baseline check..."
cargo check

echo "2. Running ANALYZE_DEPS..."
$TOOL . ANALYZE_DEPS . > /dev/null

echo "3. Running FIND_DEAD_CODE..."
$TOOL . FIND_DEAD_CODE . > /dev/null

echo "4. Testing EXTRACT (Kind from src/error.rs)..."
# We extract Kind. It should go to src/kind.rs or src/kind_mod.rs
$TOOL src/error.rs Kind src/

echo "5. Testing RENAME (Kind -> HyperKind) in src/kind_mod.rs..."
if [ -f "src/kind_mod.rs" ]; then
    $TOOL src/kind_mod.rs RENAME Kind HyperKind
elif [ -f "src/kind.rs" ]; then
    $TOOL src/kind.rs RENAME Kind HyperKind
else
    echo "kind mod file not found, skipping rename."
fi

echo "6. Testing FORMAT..."
TARGET=""
[ -f "src/kind_mod.rs" ] && TARGET="src/kind_mod.rs"
[ -f "src/kind.rs" ] && TARGET="src/kind.rs"
if [ -n "$TARGET" ]; then
    $TOOL "$TARGET" FORMAT "$TARGET"
fi

echo "7. Testing OPTIMIZE_IMPORTS..."
if [ -n "$TARGET" ]; then
    $TOOL "$TARGET" OPTIMIZE_IMPORTS "$TARGET"
fi

echo "8. Testing SSR..."
if [ -n "$TARGET" ]; then
    $TOOL "$TARGET" SSR "enum HyperKind" "enum HyperKind /* SSR works */"
fi

echo "9. Skipping EXPAND (cargo-expand not installed)..."


echo "10. Testing SPLIT_DIR on src/body..."
$TOOL SPLIT_DIR src/body > /dev/null

echo "11. Final PREFLIGHT check..."
$TOOL . PREFLIGHT Cargo.toml

echo "======================================"
echo "Hyper repo test completed successfully"
echo "======================================"
