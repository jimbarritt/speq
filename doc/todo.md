# speq — Build Plan

Reference spec: `doc/initial-specification.md`

Phases are ordered by dependency. Each phase should compile and produce a runnable binary before moving on.

---

## Phase 1 — Scaffold: blank ratatui window ✅

**Goal:** `cargo run` opens a terminal window, displays a placeholder title bar, and exits cleanly on `q`.

- [x] `cargo init --name speq`
- [x] Add dependencies: `ratatui`, `crossterm`
- [x] `main.rs`: init crossterm raw mode, create ratatui `Terminal`
- [x] Event loop: poll crossterm events, quit on `q`
- [x] Render a single full-screen block with title `speq`
- [x] Restore terminal on exit (even on error — restores raw mode + alternate screen in all exit paths)

**Done when:** `cargo run` opens a window with a title and closes cleanly. ✅

---

## Phase 2 — Parse a single hardcoded YAML spec ✅

**Goal:** Load the Petstore fixture at startup and print schema names to the title bar / a list widget.

- [x] Add `fixtures/petstore.yaml` (hand-written Petstore in OpenAPI 3.0 — 4 schemas: `Error`, `NewPet`, `Pet`, `Pets`)
- [x] Add dependencies: `openapiv3`, `serde_yaml`, `serde_json`, `serde`
- [x] `src/spec.rs`: `LoadedSpec` struct + `SpecVersion` enum
- [x] `parser/mod.rs`: detect version from raw value; dispatch to v2 or v3 parser
- [x] `parser/v3.rs`: load `openapiv3::OpenAPI`, extract `components.schemas` names (sorted)
- [x] `main.rs`: first CLI arg used as spec path; falls back to `fixtures/petstore.yaml`
- [x] 2 unit tests in `parser/v3.rs` — confirms schema names and version detection

**Done when:** Running the binary shows `Error`, `NewPet`, `Pet`, `Pets` from the petstore spec. ✅

---

## Phase 3 — Basic split-pane layout ✅

**Goal:** Two-pane layout: left pane shows schema names, right pane shows a placeholder. Title bar and status bar visible.

- [x] `app.rs`: `App` struct with `selected` index, `spec: LoadedSpec`, `focused_pane: Pane`, `should_quit`
- [x] `app.rs`: `move_up`, `move_down`, `goto_top`, `goto_bottom`, `toggle_pane` methods
- [x] `ui/mod.rs`: vertical layout — header (1 line) / body (fill) / statusbar (1 line)
- [x] `ui/mod.rs`: horizontal split of body — left 35% / right 65%
- [x] `ui/schema_list.rs`: renders schema names as `ratatui::List` with cyan selection highlight; focused pane gets rounded cyan border
- [x] `ui/detail.rs`: placeholder — shows selected schema name; same focus border logic
- [x] `ui/statusbar.rs`: static keybind hint string
- [x] `main.rs`: `j`/`k` + arrow keys navigate list; `gg`/`G` jump to top/bottom; `Tab` switches pane focus

**Done when:** Left pane scrolls through schema names with `j`/`k`; right pane echoes the selected name. ✅

---

## Phase 4 — Tree expand/collapse with keyboard nav

**Goal:** Each schema can be expanded in the left pane to reveal its properties as a tree.

- [ ] `tree.rs`: define `TreeNode` enum (Object, Array, Primitive, Ref, Combiner)
- [ ] `tree.rs`: `TreeState` — flat list of visible nodes with expand/collapse state per node
- [ ] `parser/v3.rs`: convert parsed schema into `TreeNode` hierarchy
- [ ] `ui/schema_list.rs`: render tree with `▶`/`▼` prefix, indent for depth, type badge, required `*`
- [ ] Keyboard: `Enter`/`Space` expand/collapse; `gg`/`G` jump to top/bottom
- [ ] Keyboard: `zo` open node, `zc` collapse node, `zR` expand all, `zM` collapse all

**Done when:** Expanding `Pet` reveals `id (integer*)`, `name (string*)`, `tag (string)`; expanding `Pets` reveals `items →Pet`.

---

## Phase 5 — Detail pane rendering

**Goal:** Right pane shows full metadata for the currently selected node.

- [ ] `ui/detail.rs`: render schema/property detail from selected `TreeNode`
- [ ] Show: name, type, format, description (word-wrapped), required, constraints
- [ ] Show: enum values, example, default if present
- [ ] For `$ref` nodes: show target name with `→` indicator
- [ ] `Ctrl-d`/`Ctrl-u` scroll the detail pane independently
- [ ] `Tab` switches focus between left and right panes (highlight active pane border)

**Done when:** Selecting `id` on `Pet` shows `type: integer, format: int64, required: yes` in the right pane.

---

## Phase 6 — `gd` ref-following + breadcrumb history

**Goal:** `gd` on a `$ref` node expands the referenced schema inline; `Ctrl-o` goes back.

- [ ] `app.rs`: breadcrumb stack — `Vec<CursorState>` (selected node path + scroll offset)
- [ ] `keymap.rs`: define `Action` enum; map `gd` → `Action::FollowRef`, `Ctrl-o` → `Action::JumpBack`, `Ctrl-i` → `Action::JumpForward`
- [ ] On `FollowRef`: resolve the `$ref` target schema, expand it inline under the current node, push current cursor to stack, move cursor to the expanded node
- [ ] On `JumpBack`: pop stack, restore cursor position
- [ ] Detail pane breadcrumb path: show `Pets › items › Pet` in the header

**Done when:** `gd` on `items →Pet` expands Pet's properties inline; `Ctrl-o` collapses them and returns to prior position.

---

## Phase 7 — `/` search

**Goal:** `/` opens an inline search bar; typing filters schema names with fuzzy match.

- [ ] `search.rs`: `SearchState` — query string, list of match indices, current match cursor
- [ ] Fuzzy match: simple substring or `nucleo`/`fuzzy-matcher` crate — keep it lightweight
- [ ] `ui/statusbar.rs`: when search active, render `/query█` in the status bar area
- [ ] Left pane: highlight matching names; non-matching schemas dimmed or hidden
- [ ] `Enter` confirms selection; `n`/`N` cycle through matches; `Esc` cancels and restores position

**Done when:** Typing `/pet` highlights `Pet` and `Pets`; `n` cycles between them; `Esc` restores.

---

## Phase 8 — SQLite indexer (`speq index`)

**Goal:** `speq index` walks `~/.cache/speq/specs/`, parses each spec, and writes to `~/.cache/speq/index.db`.

- [ ] Add dependencies: `rusqlite`, `tokio`
- [ ] `main.rs`: `clap` CLI — `speq [file]` for TUI mode, `speq index [--force] [service]` for indexer (clap already in deps)
- [ ] `indexer.rs`: walk `~/.cache/speq/specs/*/`, read `meta.yaml` + `openapi.yaml`
- [ ] `indexer.rs`: upsert `services`, `schemas`, `properties`, `paths` rows per spec (see SQL schema in spec)
- [ ] FTS5 `search_index` rebuild per service
- [ ] Incremental: skip if `openapi.yaml` mtime unchanged (unless `--force`)
- [ ] Stdout progress: `indexing payments-service ... ok (47 schemas, 123 paths)`
- [ ] TUI startup: if no file arg given, open from SQLite index (multi-service mode)

**Done when:** `speq index` runs without error on a test `~/.cache/speq/specs/` directory and populates the DB.

---

## Phase 9 — Lua config + keybinding layer

**Goal:** `~/.config/speq/config.lua` can override or extend keybindings using `speq.keymap.set`.

- [ ] Add dependency: `mlua` (feature `lua54`)
- [ ] `config.rs`: load `~/.config/speq/config.lua` at startup; build `mlua::Lua` context
- [ ] Expose `speq` global table with `keymap.set(mode, lhs, action)` and `set(key, value)`
- [ ] Apply config bindings on top of defaults (defaults registered first)
- [ ] Support nvim key notation: `<CR>`, `<Esc>`, `<C-d>`, `<leader>`, etc.
- [ ] `"noop"` action disables a default binding
- [ ] Warn to stderr on unknown action strings; ignore gracefully
- [ ] If config file missing, proceed with defaults silently

**Done when:** Adding `speq.keymap.set("n", "L", "expand_node")` in config makes `L` expand nodes.

---

## Fixtures

- `fixtures/petstore.yaml` — hand-written Petstore spec in OpenAPI 3.0 format with 4 schemas (`Error`, `NewPet`, `Pet`, `Pets`). Used in Phases 2–7 as the dev fixture.

---

## Notes

- Compile after every logical unit. Don't write 200 lines before checking it builds.
- Lua config (Phase 9) is intentionally last — core UX shape should be stable first.
- Remote URL fetching (`reqwest`/`tokio`) is only needed when implementing Phase 8 fully. Skip for earlier phases.
- Multi-service browser (switching between indexed services in the TUI) is a future feature beyond Phase 8.
- `clap` is already in `Cargo.toml` — no need to add it again in Phase 8.
