# Rust Refactor MCP Server

MCP server that extracts named entities from Rust source into dedicated module files.
One entity per file. Zero re-export index files. Cross-file `use` updates included.

## Tool: `extract_entity`

| Parameter       | Required | Description                             |
|-----------------|----------|-----------------------------------------|
| `file_path`     | yes      | Path to source `.rs` file               |
| `entity_name`   | yes      | Entity to extract                       |
| `target_folder` | yes      | Output directory for new module         |
| `entity_type`   | no       | Hint: `struct`, `enum`, `fn`, `trait`  |

### What it does

1. Parse source → find entity by name via `syn` AST
2. Collect entity + all related `impl` blocks
3. Extract `#[cfg(test)]` modules that reference the entity → `{name}_tests.rs`
4. Write `{name}.rs` (and `_tests.rs` if any) to `target_folder`
5. Scan sibling files for `use` references → update module paths
6. Surgically remove extracted byte spans from source
7. Return: new file paths, items extracted, usage files updated

## Usage

```bash
# CLI
cargo run -- file.rs MyStruct ./src

# MCP stdio
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"extract_entity","arguments":{"file_path":"file.rs","entity_name":"MyStruct","target_folder":"./src"}}}' \
  | cargo run --release
```

## Conventions

- **One entity per file** — no bundling
- **No re-export index files** — never create mod.rs that only re-exports
- **Tests travel with entities** — extracted to `{entity}_tests.rs`
- **Cross-file updates** — `use` paths updated in all affected files
- **Compilable before & after** — fixture project survives refactoring