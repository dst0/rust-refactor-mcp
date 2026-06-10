# Skill: `rust-refactor-mcp`

The `rust-refactor-mcp` server provides high-precision Rust codebase refactoring tools, specialized for moving entities (structs, enums, functions, traits, constants, statics, types) into dedicated files while managing module dependencies.

## Key Capabilities

- **Single Entity Extraction**: Surgically move one entity to its own file.
- **Automated Dependency Management**: Automatically adjusts `use` statements, preserves `impl` blocks, transforms `self::`/`super::` imports to absolute `crate::` paths, and migrates `#[cfg(test)]` modules.
- **Batch Codebase Refactoring**: Commands to split large, multi-entity files or entire directories into "one entity per file" standard.
- **Analysis & Verification**: Semantic dead code detection, module dependency mapping, and preflight integrity checks (`cargo check`/`test`).

## Available Tools (accessible via MCP)

### 1. `extract_entity`
Extracts a single entity from a source file into a new file.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `file_path` | yes | Path to source `.rs` file. |
| `entity_name` | yes | Name of entity (e.g., `MyStruct`). |
| `target_folder` | yes | Output directory. |
| `entity_types` | no | Filter by type (e.g., `["struct", "fn"]`). |
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

### 5. `optimize_imports`
Optimizes and sorts imports in a Rust file using `rustfmt`.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `file_path` | yes | Path to source `.rs` file. |

### 6. `ssr`
Performs structural search and replace on Rust source code.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `file_path` | yes | Path to source `.rs` file. |
| `pattern` | yes | syn-style pattern to match. |
| `replacement` | yes | Replacement code. |

### 7. `expand_macros`
Expands procedural macros for a given target using `cargo expand`.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `target` | yes | Target module or item to expand. |

### 8. `analyze_dependencies`
Analyzes module coupling and dependencies within a crate.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `dir_path` | yes | Path to directory to scan. |

### 9. `find_dead_code`
Identifies potentially dead code (unused functions, structs, etc.) using semantic analysis.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `dir_path` | yes | Path to directory to scan. |

### 10. `preflight_validator`
Runs `cargo check` and `cargo test` to verify project integrity.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `manifest_path` | yes | Path to `Cargo.toml`. |

### 11. `split_folder_entities`
Recursively scans a directory and splits all multi-entity files into single-entity files.

| Parameter | Required | Description |
| :--- | :--- | :--- |
| `dir_path` | yes | Path to directory to scan. |
| `generate_reexport` | no | Default: `true`. |

---

## CLI-Only Commands (Bulk Automation)

The CLI executable provides bulk refactoring commands to enforce the "one entity per file" standard.

### Usage
```bash
cargo run -- <target> <COMMAND> [args...] [--options]
```

- **Split File**: `cargo run -- <file.rs> SPLIT <target_dir>`
- **Split Directory**: `cargo run -- SPLIT_DIR <dir_path>`
- **Analyze Deps**: `cargo run -- . ANALYZE_DEPS <dir_path>`
- **Find Dead Code**: `cargo run -- . FIND_DEAD_CODE <dir_path>`
- **Preflight**: `cargo run -- . PREFLIGHT <Cargo.toml>`
- **Rename**: `cargo run -- <file.rs> RENAME <old> <new>`
- **SSR**: `cargo run -- <file.rs> SSR <pattern> <replacement>`
- **Expand**: `cargo run -- . EXPAND <target>`

---

## Refactoring Guidelines for PI Agent

1. **Topological Order**: When performing batch refactors, prioritize entities with fewer dependencies first. The `SPLIT` commands handle this automatically.
2. **Preserve Public API**: By default, the tool adds `pub use` re-exports if an entity was previously public.
3. **Compilability**: These tools are AST-based and designed to keep the project compilable before and after each refactoring step.
4. **Verification Loop**: Always run `PREFLIGHT` after a major refactor to ensure no regressions were introduced.
