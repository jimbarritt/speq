# speq — Implementation Plan

## What's Next

- **Next:** Task 1 — Breadcrumb history stack (Delta: Ref Following)
- **Sub-doc:** (none)
- **Blockers:** None

## Summary

| Delta | Task | Status |
|-------|------|--------|
| [Delta: Core Scaffold](#delta-core-scaffold) | [1. Blank ratatui window](#task-1-blank-ratatui-window) | ✓ DONE |
| | [2. Parse a hardcoded YAML spec](#task-2-parse-a-hardcoded-yaml-spec) | ✓ DONE |
| [Delta: Schema Browser UI](#delta-schema-browser-ui) | [1. Split-pane layout](#task-1-split-pane-layout) | ✓ DONE |
| | [2. Tree expand/collapse](#task-2-tree-expandcollapse) | ✓ DONE |
| | [3. Detail pane rendering](#task-3-detail-pane-rendering) | ✓ DONE |
| [Delta: Ref Following](#delta-ref-following) | [1. Breadcrumb history stack](#task-1-breadcrumb-history-stack) | TODO |
| | [2. Keymap action layer](#task-2-keymap-action-layer) | TODO |
| | [3. Resolve and expand $ref inline](#task-3-resolve-and-expand-ref-inline) | TODO |
| | [4. Breadcrumb path in detail header](#task-4-breadcrumb-path-in-detail-header) | TODO |
| [Delta: Search](#delta-search) | [1. Search state and fuzzy matcher](#task-1-search-state-and-fuzzy-matcher) | TODO |
| | [2. Search bar rendering](#task-2-search-bar-rendering) | TODO |
| | [3. Match navigation and cancel](#task-3-match-navigation-and-cancel) | TODO |
| [Delta: SQLite Indexer](#delta-sqlite-indexer) | [1. CLI subcommand split](#task-1-cli-subcommand-split) | TODO |
| | [2. Spec cache walker](#task-2-spec-cache-walker) | TODO |
| | [3. Schema upsert and FTS5 index](#task-3-schema-upsert-and-fts5-index) | TODO |
| | [4. Incremental reindex](#task-4-incremental-reindex) | TODO |
| | [5. TUI startup from the index](#task-5-tui-startup-from-the-index) | TODO |
| [Delta: Lua Config](#delta-lua-config) | [1. Load config.lua](#task-1-load-configlua) | TODO |
| | [2. Expose the speq global table](#task-2-expose-the-speq-global-table) | TODO |
| | [3. nvim key notation parser](#task-3-nvim-key-notation-parser) | TODO |
| | [4. Failure handling](#task-4-failure-handling) | TODO |

## Delta: Core Scaffold

### Task 1: Blank ratatui window
- ✓ DONE — `cargo run` opens a terminal window with a `speq` title block and exits cleanly on `q`.
  - crossterm raw mode + alternate screen, restored on every exit path including error.

### Task 2: Parse a hardcoded YAML spec
- ✓ DONE — Petstore fixture loads at startup and its schema names render.
  - `fixtures/petstore.yaml` — OpenAPI 3.0, 4 schemas (`Error`, `NewPet`, `Pet`, `Pets`).
  - `src/spec.rs` — `LoadedSpec` struct, `SpecVersion` enum.
  - `src/parser/mod.rs` — version detection and dispatch; `src/parser/v3.rs` extracts `components.schemas`.
  - First CLI arg is the spec path, falling back to the fixture.
  - 2 unit tests in `parser/v3.rs`.

## Delta: Schema Browser UI

### Task 1: Split-pane layout
- ✓ DONE — header / body / statusbar vertically, left 35% / right 65% horizontally.
  - `src/app.rs` — `App` with `selected`, `spec`, `focused_pane`, `should_quit`, and movement methods.
  - `j`/`k` and arrows navigate, `gg`/`G` jump, `Tab` switches pane focus with a cyan rounded border on the focused pane.

### Task 2: Tree expand/collapse
- ✓ DONE — schemas expand in the left pane to reveal properties as a tree.
  - `src/tree.rs` — `TreeNode` enum (Object, Array, Primitive, Ref, Combiner) and `TreeState` flat visible-node list.
  - `▶`/`▼` prefix, depth indent, type badge, required `*`.
  - `Enter`/`Space` toggle; `zo`/`zc`/`zR`/`zM` open, collapse, expand all, collapse all.

### Task 3: Detail pane rendering
- ✓ DONE — right pane shows full metadata for the selected node.
  - Name, type, format, word-wrapped description, required flag, constraints, enum values, example, default.
  - `$ref` nodes show the target name with `→`.
  - `Ctrl-d`/`Ctrl-u` scroll the detail pane independently.

## Delta: Ref Following

### Task 1: Breadcrumb history stack
- TODO — `app.rs`: `Vec<CursorState>` holding the selected node path and scroll offset.

### Task 2: Keymap action layer
- TODO — `keymap.rs`: `Action` enum with `gd` → `FollowRef`, `Ctrl-o` → `JumpBack`, `Ctrl-i` → `JumpForward`.

### Task 3: Resolve and expand $ref inline
- TODO — on `FollowRef`, resolve the target schema and expand it under the current node.
  - Push the current cursor onto the stack, then move the cursor to the expanded node.
  - On `JumpBack`, pop the stack and restore the cursor position.
  - Done when `gd` on `items →Pet` expands Pet's properties inline and `Ctrl-o` returns.

### Task 4: Breadcrumb path in detail header
- TODO — render the path as `Pets › items › Pet` in the detail pane header.

## Delta: Search

### Task 1: Search state and fuzzy matcher
- TODO — `search.rs`: `SearchState` with query string, match indices, current match cursor.
  - Substring match or a lightweight crate (`nucleo` / `fuzzy-matcher`).

### Task 2: Search bar rendering
- TODO — `/` opens an inline search bar rendered as `/query█` in the status bar area.
  - Left pane highlights matching names; non-matching schemas dimmed or hidden.

### Task 3: Match navigation and cancel
- TODO — `Enter` confirms, `n`/`N` cycle matches, `Esc` cancels and restores the prior position.
  - Done when `/pet` highlights `Pet` and `Pets`, `n` cycles, `Esc` restores.

## Delta: SQLite Indexer

### Task 1: CLI subcommand split
- TODO — `clap` CLI: `speq [file]` for TUI mode, `speq index [--force] [service]` for the indexer.
  - `clap` is already a dependency; add `rusqlite` and `tokio`.

### Task 2: Spec cache walker
- TODO — `indexer.rs` walks `~/.cache/speq/specs/*/`, reading `meta.yaml` and `openapi.yaml`.

### Task 3: Schema upsert and FTS5 index
- TODO — upsert `services`, `schemas`, `properties`, `paths` rows per spec; rebuild the FTS5 `search_index` per service.
  - SQL schema is in `doc/initial-specification.md`.

### Task 4: Incremental reindex
- TODO — skip a spec when `openapi.yaml` mtime is unchanged, unless `--force`.
  - Progress to stdout: `indexing payments-service ... ok (47 schemas, 123 paths)`.

### Task 5: TUI startup from the index
- TODO — with no file argument, open from `~/.cache/speq/index.db` in multi-service mode.

## Delta: Lua Config

### Task 1: Load config.lua
- TODO — `config.rs` loads `~/.config/speq/config.lua` at startup into an `mlua::Lua` context.
  - Add `mlua` with the `lua54` feature.

### Task 2: Expose the speq global table
- TODO — `speq.keymap.set(mode, lhs, action)` and `speq.set(key, value)`.
  - Defaults register first; config bindings apply on top.
  - Done when `speq.keymap.set("n", "L", "expand_node")` makes `L` expand nodes.

### Task 3: nvim key notation parser
- TODO — support `<CR>`, `<Esc>`, `<C-d>`, `<leader>` and similar.
  - `"noop"` as the action disables a default binding.

### Task 4: Failure handling
- TODO — warn to stderr on unknown action strings and continue; proceed silently with defaults when the config file is absent.

## Implementation Notes

### Architecture

- Compile after every logical unit — write a little, `cargo build`, fix, continue.
- No async in the TUI hot path. The event loop stays sync; async is for the indexer and remote URL fetching only.
- SQLite is the runtime source for multi-service mode. A direct file-path invocation parses in memory and bypasses the index.
- Lua config is deliberately last so the core UX shape settles first.
- Remote URL fetching (`reqwest` / `tokio`) is only needed for the full SQLite Indexer delta.
- A multi-service browser (switching between indexed services inside the TUI) is a future feature beyond the SQLite Indexer delta.

### Fixtures

- `fixtures/petstore.yaml` — hand-written OpenAPI 3.0 Petstore with 4 schemas. The dev fixture for everything up to the SQLite Indexer delta.

### Source of this plan

- Seeded from `doc/todo.md` (phase-based). Phases 1–5 map to the Core Scaffold and Schema Browser UI deltas; phases 6–9 map to Ref Following, Search, SQLite Indexer and Lua Config.
