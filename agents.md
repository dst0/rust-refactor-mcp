# Agents — Rust Refactor MCP

## User Updates

While actively working, reread `user-updates.md` for new instructions at least once per minute and incorporate any new guidance before continuing.

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

## Commands

- Poll running background tasks with reasonable intervals that approximately equal to ETA or reasonably smaller when closer progress monitoring is absolutely necessary. But not repeatedly in tight loops. Rely on reactive completion messages instead.

## Test Quality & Adversarial Review

- Tests must never be added solely as mechanical line-fillers to pass coverage gates. Tests must meaningfully verify domain logic, invariant preservation, realistic crash recovery, positive cases, negative cases, and edge cases.
- Bug fixes must start with a reproducible failing regression test before writing the fix.
- For non-trivial features, bug fixes, or test additions, automatically spawn an adversarial test-critic subagent to review the tests. The critic must evaluate whether the suite verifies real behavior vs artificial line coverage, identifies missing edge cases, and flags fragile/vacuous tests before work is completed.
- Never use coverage bypass comments (e.g. `/* v8 ignore */`, `#[cfg(not(coverage))]`, `# pragma: no cover`) to bypass coverage gates. All code in the repository must be reachable and exercised by tests; dead or unreachable code must be deleted rather than kept or suppressed (except rare compiler/type-exhaustiveness edge cases where a branch is syntactically required but provably unreachable at runtime).
- If you create or modify a test file, run it and iterate on test or implementation until it passes.

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

## Issues and PRs

When creating PRs:

- Always include a `## Bug Fixes` section in the PR description detailing any bugs uncovered and resolved during the task, with references to their regression tests.

## Mandatory Learning Log

- Maintain the repository-wide append-only learning collection in `docs/leanings/`.
- Create exactly one Markdown file per learning in the same change whenever work reveals a resolved bug or regression, failed or misleading experiment, unexpected behavior, setup or environment trap, non-obvious constraint, important workaround, or rejected approach with reusable rationale.
- Routine successful work does not need an entry unless it produces a reusable insight.
- Follow the filename convention and exact entry structure documented in `docs/leanings/README.md`. Include the task/context, observation or failure, evidence, approaches tried and their outcomes, root cause, resolution, verification, prevention or follow-up, and the reusable learning.
- Mark uncertainty honestly. If root cause or resolution is incomplete, record the entry as `Partial` or `Open` and state what evidence is still missing.
- Keep learning files append-only by default: do not delete or rewrite older files merely to make the history cleaner. Put later discoveries in a new file that links the earlier learning.
- Exception for confirmed falsehoods: when authoritative evidence proves that an entry itself was fabricated, hallucinated, or factually false, correct or remove the false content so future agents do not reuse it.
- A confirmed-falsehood correction must never be silent. Mark the affected file `Corrected` and add a dated correction note stating what was wrong, the authoritative evidence used, and what was changed. Do not repeat removed sensitive content.
- If the evidence is incomplete or disputed, do not rewrite the original file; add a dated `Partial` or `Open` learning file that links it.
- Link relevant issues, commits, logs, or regression tests when safe and useful.
- Never place credentials, tokens, private keys, customer data, sensitive payloads, or unsanitized production evidence in learning files.
