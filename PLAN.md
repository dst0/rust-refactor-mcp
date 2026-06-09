# Refactoring Roadmap & Polishing Cycle

## Polishing Cycle (Applied to all tools)
Every tool or feature undergoes this mandatory cycle before being considered complete:
1.  **Implement**
2.  **Test Manually** (on the refactor-mcp project)
3.  **Improve** (based on findings)
4.  **Fix** (compilation/logic bugs)
5.  **Polish** (DX, logging, docs, auto-formatting)
6.  **Push** (commit and push to `main`)

## Task Queue

- [x] `extract_entity` (entity extraction)
- [x] `SPLIT` command (single file)
- [x] `SPLIT_DIR` command (recursive)
- [x] `FORMAT` command (`rustfmt` integration)
- [x] `RENAME` command (global renaming)
- [x] `FIX_CARGO` command (`cargo fix` wrapper)
- [x] `OPTIMIZE_IMPORTS` command (`rustfmt` wrapper)
- [x] `SSR` tool (Structural Search & Replace)
- [x] `EXPAND` tool (`cargo expand` wrapper)
- [x] `split_folder_entities` (batch refactor)
- [x] Entity type filtering
- [x] **Wiring MCP to Pi CLI**
- [x] `macro_expander` improvements (robustness)
- [x] `preflight_validator` (Verification Loop)
- [x] `dependency_graph_analyzer` (Coupling awareness)
- [x] `semantic_dead_code_finder` (Safe deletions)
- [ ] `llm-orc` UI Rework (Resume work)
