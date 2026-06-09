# Skill: `rust-refactor-mcp`

The `rust-refactor-mcp` server provides high-precision Rust codebase refactoring tools, specialized for moving entities (structs, enums, functions, traits, constants, statics, types) into dedicated files while managing module dependencies.

## Key Capabilities

- **Single Entity Extraction**: Surgically move one entity to its own file.
- **Automated Dependency Management**: Automatically adjusts `use` statements, preserves `impl` blocks, transforms `self::`/`super::` imports to absolute `crate::` paths, and migrates `#[cfg(test)]` modules.
- **Batch Codebase Refactoring**: Commands to split large, multi-entity files or entire directories into "one entity per file" standard.

## Available Tools (accessible via MCP)

### 1. `extract_entity`
Extracts a single entity from a source file into a new file.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `file_path` | yes | Path to source `.rs` file. |
| `entity_name` | yes | Name of entity (e.g., `MyStruct`). |
| `target_folder` | yes | Output directory. |
| `entity_type` | no | Hint: `struct`, `enum`, `fn`, `trait`, `const`, `static`, `type`. |
| `generate_reexport` | no | Default: `true`. Set `false` to disable `pub use` re-exports. |

### 2. `format_code`
Formats a Rust file using `rustfmt`.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `file_path` | yes | Path to source `.rs` file. |

### 3. `rename_entity`
Renames an entity (struct, enum, fn, etc.) across a file.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `file_path` | yes | Path to source `.rs` file. |
| `old_name` | yes | Existing name of entity. |
| `new_name` | yes | New name for entity. |

### 4. `fix_cargo_errors`
Runs `cargo fix` on a project to resolve auto-fixable errors.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `manifest_path` | yes | Path to `Cargo.toml`. |

---

## CLI-Only Commands (Bulk Automation)

The CLI executable provides bulk refactoring commands to enforce the "one entity per file" standard across the repository.

### How Splitting Works
When an entity is extracted, the original source file is surgically modified:
1. The extracted code is removed, preserving surrounding comments and whitespace.
2. If the entity was public, a `pub use crate::<new_module_path>::<entity_name>;` re-export is added to the original file to maintain API compatibility (can be disabled with `--no-reexport`).
3. The new entity file is created and automatically formatted via `rustfmt`.

### Auto-Split Single File
Finds all entities in a file and extracts them sequentially based on dependency order.
```bash
cargo run -- <file.rs> SPLIT <target_dir> [--no-reexport]
```

### Auto-Split Directory
Recursively scans a directory (skipping `lib.rs`/`mod.rs`/`main.rs`) and splits all entities into their own files.
```bash
cargo run -- SPLIT_DIR <dir_path> [--no-reexport]
```

---

## Refactoring Guidelines for PI Agent

1. **Topological Order**: When performing batch refactors, prioritize entities with fewer dependencies first. The `SPLIT` commands handle this automatically.
2. **Preserve Public API**: By default, the tool adds `pub use` re-exports if an entity was previously public, ensuring external code does not break. Use `--no-reexport` to force a clean break in dependency paths if you are performing a major refactor and are prepared to manually update all call sites.
3. **File Attributes**: The tool preserves file-level attributes (`#![...]`) and shebangs during extraction.
4. **Iterative Refactor**: For large refactors, prefer `SPLIT_DIR` on a directory-by-directory basis to manage throughput and allow for incremental verification.
5. **No Bundling**: Always aim for the "one entity per file" convention enforced by these tools.
6. **Compilability**: These tools are AST-based and designed to keep the project compilable before and after each refactoring step.
