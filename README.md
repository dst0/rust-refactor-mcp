# Rust Refactor MCP Server

MCP server that extracts named entities from Rust source into dedicated module files.
One entity per file. Zero re-export index files. Cross-file `use` updates included.

## Features

- **Entity Extraction**: Surgically extract a single `struct`, `enum`, `fn`, `trait`, `const`, `static`, or `type` from a file.
- **Dependency Aware**: Preserves associated `impl` blocks and necessary `use` statements.
- **Intelligent Routing**: Converts `self::` and `super::` imports automatically into absolute `crate::` paths when shifting module structures. Module declarations are correctly placed in the root (`lib.rs`, `main.rs`, or `mod.rs`) to ensure valid crate structure.
- **Automated Batch Splitting**: Commands to completely split a file or an entire directory recursively into a "one entity per file" architecture.
- **Fast Global Updates**: Automatically updates usage (`use crate::...`) in sibling and descendant files.
- **Tests Follow Code**: `#[cfg(test)]` modules that reference an extracted entity are automatically moved to a corresponding `{name}_tests.rs` file.
- **Analysis & Validation**:
    - **Dead Code Finder**: Semantic analysis to find unused code.
    - **Dependency Graph**: Map module coupling across the crate.
    - **Preflight Validator**: Integrated `cargo check` and `cargo test --lib` loop (skips brittle doc-tests in large repos).
- **Transformation Tools**:
    - **SSR**: Structural Search & Replace.
    - **Rename**: Global entity renaming.
    - **Macro Expansion**: `cargo expand` integration.
    - **Import Optimization**: Sorting and cleaning imports.

## Tools (MCP)

### `extract_entity`
Extracts a single entity from a source file.
- `file_path`: Path to source file.
- `entity_name`: Name of entity to extract.
- `target_folder`: Output directory.
- `entity_types`: (Optional) Filter by type (e.g., `["struct", "fn"]`).
- `generate_reexport`: (Optional) Default `true`.

### `format_code`
Formats a Rust file using `rustfmt`.
- `file_path`: Path to file.

### `rename_entity`
Renames an entity across a file.
- `file_path`: Path to file.
- `old_name`: Existing name.
- `new_name`: New name.

### `fix_cargo_errors`
Runs `cargo fix` on a project.
- `manifest_path`: Path to `Cargo.toml`.

### `optimize_imports`
Optimizes and sorts imports.
- `file_path`: Path to file.

### `ssr`
Structural search and replace.
- `file_path`: Path to file.
- `pattern`: syn-style pattern.
- `replacement`: Replacement code.

### `expand_macros`
Expands macros for a target.
- `target`: Target item/module.

### `analyze_dependencies`
Map module dependencies.
- `dir_path`: Directory to scan.

### `find_dead_code`
Identify potentially unused code.
- `dir_path`: Directory to scan.

### `preflight_validator`
Verify project integrity.
- `manifest_path`: Path to `Cargo.toml`.

### `split_folder_entities`
Recursively split all multi-entity files in a folder.
- `dir_path`: Directory to scan.

### `discover_multi_entity_files`
List files that contain more than one entity.
- `dir_path`: Directory to scan.

## CLI Usage

The executable can be used directly as a CLI.

### Extraction
```bash
cargo run -- <file.rs> <EntityName> <target_dir> [--types=struct,fn] [--no-reexport]
```

### Batch Tools
- **Split File**: `cargo run -- <file.rs> SPLIT <target_dir>`
- **Split Directory**: `cargo run -- SPLIT_DIR <target_dir>`
- **Analyze Deps**: `cargo run -- . ANALYZE_DEPS <dir>`
- **Find Dead Code**: `cargo run -- . FIND_DEAD_CODE <dir>`
- **Preflight**: `cargo run -- . PREFLIGHT <Cargo.toml>`
- **Rename**: `cargo run -- <file.rs> RENAME <old> <new>`
- **SSR**: `cargo run -- <file.rs> SSR <pattern> <replacement>`
- **Expand**: `cargo run -- . EXPAND <target>`

## Conventions Enforced

- **One entity per file** — no bundling.
- **Snake_case filenames** — e.g. `MyStruct` becomes `my_struct.rs`.
- **No re-export index files** — never create mod.rs that only re-exports.
- **Tests travel with entities** — extracted to `{entity}_tests.rs`.
- **Cross-file updates** — `use` paths updated in all affected files.
