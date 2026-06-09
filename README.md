# Rust Refactor MCP Server

MCP server that extracts named entities from Rust source into dedicated module files.
One entity per file. Zero re-export index files. Cross-file `use` updates included.

## Features

- **Entity Extraction**: Surgically extract a single `struct`, `enum`, `fn`, `trait`, `const`, `static`, or `type` from a file.
- **Dependency Aware**: Preserves associated `impl` blocks and necessary `use` statements.
- **Intelligent Routing**: Converts `self::` and `super::` imports automatically into absolute `crate::` paths when shifting module structures. Iterative AST parsing prevents stack overflows.
- **Automated Batch Splitting**: Commands to completely split a file or an entire directory recursively into a "one entity per file" architecture.
- **Fast Global Updates**: Automatically updates usage (`use crate::...`) in sibling and descendant files using pre-filtered fast-string matching to scale to large codebases instantly.
- **Tests Follow Code**: `#[cfg(test)]` modules that reference an extracted entity are automatically moved to a corresponding `{name}_tests.rs` file.

## Tools (MCP)

### `extract_entity`

| Parameter       | Required | Description                             |
|-----------------|----------|-----------------------------------------|
| `file_path`     | yes      | Path to source `.rs` file               |
| `entity_name`   | yes      | Entity to extract                       |
| `target_folder` | yes      | Output directory for new module         |
| `entity_type`   | no       | Hint: `struct`, `enum`, `fn`, `trait`  |

## CLI Usage

The executable can be used directly as a CLI for immediate codebase refactoring.

### Single Entity Extraction
```bash
cargo run -- <file.rs> <EntityName> <target_dir> [entity_type]
# Example:
# cargo run -- src/repos/requests.rs RequestsRepo ./src/repos
```

### Auto-Split a Single File
Automatically finds all entities in a file, determines the safest topological dependency order, and extracts them one by one into individual files.
```bash
cargo run -- <file.rs> SPLIT <target_dir>
# Example:
# cargo run -- src/repos/leases.rs SPLIT src/repos/
```

### Auto-Split an Entire Directory
Recursively scans a directory for `.rs` files and runs the `SPLIT` operation on all of them, ignoring crate/module roots (`lib.rs`, `mod.rs`, `main.rs`). Progress is tracked in-place in the terminal.
```bash
cargo run -- SPLIT_DIR <target_dir>
# Example:
# cargo run -- SPLIT_DIR src/repos/
```

## Conventions Enforced

- **One entity per file** — no bundling
- **Snake_case filenames** — e.g. `MyStruct` becomes `my_struct.rs`
- **No re-export index files** — never create mod.rs that only re-exports
- **Tests travel with entities** — extracted to `{entity}_tests.rs`
- **Cross-file updates** — `use` paths updated in all affected files
- **Compilable before & after** — fixture project survives refactoring
