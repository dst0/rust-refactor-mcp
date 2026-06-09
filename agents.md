# Agents — Rust Refactor MCP

All requirements, conventions, and workflows for this project.
Read this file before any work session.

## Core Principles

- **One entity per file** — never bundle multiple entities in one extracted file
- **No re-export index files** — never create `mod.rs` or `index.rs` that only re-exports
- **Compilable before AND after** — fixture project must build and pass tests both before and after extraction; no conditions, no manual fixes
- **100% test coverage** — every public function, every branch, every edge case
- **AST-only analysis** — use `syn` for all usage detection, occurrence tracking, and reference resolution; never regex on source text

## Extraction Rules

### What gets extracted

1. **Main entity** — struct, enum, trait, fn, or impl block named `entity_name`
2. **All related `impl` blocks** — `impl Entity` and `impl Trait for Entity`
3. **Attributes** — `#[derive]`, `#[cfg]`, doc comments travel with the entity
4. **Tests** — any `#[cfg(test)]` module whose functions reference the entity is extracted to `{entity_name}_tests.rs` alongside the entity file

### What happens after extraction

1. Entity + impls written to `{entity_name}.rs` in `target_folder`
2. Tests (if any) written to `{entity_name}_tests.rs` in `target_folder`
3. Extracted byte spans surgically removed from source file — whitespace and comments preserved
4. All sibling `.rs` files scanned via `syn` AST for `use` paths referencing the entity — module paths updated to point to new file
5. Resulting source files remain compilable as-is

### File naming

- Entity file: `{entity_name.snake_case()}.rs`
- Test file: `{entity_name.snake_case()}_tests.rs`

## Fixture Project

### Structure

```
fixtures/
  project/
    Cargo.toml
    src/
      lib.rs           # declares all modules
      simple.rs        # Point + greet (2 entities)
      medium.rs        # User, Status, validate_email, UserBuilder (4 entities)
      complex.rs       # Document, Parser, MarkdownParser, Error, Cache, format_html (6 entities)
      usage.rs         # cross-file usage of all entities
```

### Fixture requirements

- Must compile with `cargo build --manifest-path fixtures/project/Cargo.toml`
- Must pass `cargo test --manifest-path fixtures/project/Cargo.toml`
- After any extraction, the fixture project must still compile and pass tests
- Usage files reference entities across module boundaries to test cross-file `use` updates
- Each fixture has matching `expected/` files for verification:
  - `fixtures/expected/{entity_name}.rs` — expected extracted content
  - `fixtures/expected/{entity_name}_tests.rs` — expected extracted tests (if any)
  - `fixtures/expected/{source_file}_after.rs` — expected remaining source

## Tool Specification: `extract_entity`

| Parameter       | Required | Description                              |
|-----------------|----------|------------------------------------------|
| `file_path`     | yes      | Path to source `.rs` file                |
| `entity_name`   | yes      | Name of entity to extract                |
| `target_folder` | yes      | Output directory for new module file     |
| `entity_type`   | no       | Hint: `struct`, `enum`, `fn`, `trait`   |

### Return format (JSON)

```json
{
  "new_file_path": "path/to/entity.rs",
  "test_file_path": "path/to/entity_tests.rs",
  "items_extracted": ["struct: Entity", "impl: Entity"],
  "source_updated": true,
  "usage_files_updated": ["path/to/usage.rs"]
}
```

## Testing Workflow

Every PR/commit must pass:

1. `cargo test` — unit tests on spans + extract modules
2. `cargo build` — binary compiles
3. Fixture project builds: `cargo build --manifest-path fixtures/project/Cargo.toml`
4. Fixture project tests: `cargo test --manifest-path fixtures/project/Cargo.toml`
5. Extraction on fixtures: tool runs, produces output
6. Post-extraction build: fixture project still compiles
7. Post-extraction tests: fixture project still passes tests
8. MCP stdio test: `echo '{...}' | cargo run --release` produces correct JSON-RPC response

## Development Workflow

1. Implement feature
2. Add/update tests (100% coverage)
3. `cargo test` passes
4. `cargo clippy -- -D warnings` passes
5. Manual test with fixtures
6. Test as MCP server (stdio JSON-RPC)
7. Register in pi config
8. Test via pi TTY in new process

## Conventions

- Edition: 2021
- `syn` with `full`, `extra-traits`, `visit` features
- No external refactoring libraries — roll our own with `syn`
- Byte-span based text surgery, not token-stream rewriting
- Snake-case for all generated file names
- No `pub mod` index files — entities are standalone modules
- Usage path updates use AST resolution, never string replacement
