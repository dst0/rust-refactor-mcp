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

---

## CLI-Only Commands (Bulk Automation)

The CLI executable provides bulk refactoring commands to enforce the "one entity per file" standard across the repository.

### Auto-Split Single File
Finds all entities in a file and extracts them sequentially based on dependency order.
```bash
cargo run -- <file.rs> SPLIT <target_dir>
```

### Auto-Split Directory
Recursively scans a directory (skipping `lib.rs`/`mod.rs`/`main.rs`) and splits all entities into their own files.
```bash
cargo run -- SPLIT_DIR <dir_path>
```

---

## Refactoring Guidelines for PI Agent

1. **Topological Order**: When performing batch refactors, prioritize entities with fewer dependencies first. The `SPLIT` commands handle this automatically.
2. **Preserve Public API**: The tool automatically adds `pub use` re-exports if an entity was previously public, ensuring external code does not break.
3. **File Attributes**: The tool preserves file-level attributes (`#![...]`) and shebangs during extraction.
4. **Iterative Refactor**: For large refactors, prefer `SPLIT_DIR` on a directory-by-directory basis to manage throughput and allow for incremental verification.
5. **No Bundling**: Always aim for the "one entity per file" convention enforced by these tools.
6. **Compilability**: These tools are AST-based and designed to keep the project compilable before and after each refactoring step.
