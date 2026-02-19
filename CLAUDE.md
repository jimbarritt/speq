# speq — CLAUDE.md

## What this is

`speq` is a Rust TUI application for navigating and inspecting OpenAPI specifications. Binary name: `speq`. It is a keyboard-driven, read-only, schema-first browser — not an API executor.

Full specification: `doc/initial-specification.md`

## Build & run

```bash
cargo build          # compile
cargo run -- <file>  # run with a spec file
cargo run            # run (reads stdin or uses a hardcoded fixture for dev)
cargo test           # run tests
```

## Project layout (target)

```
src/
  main.rs          # entry point, CLI arg parsing, top-level dispatch
  app.rs           # App state struct, event loop
  ui/
    mod.rs         # top-level render fn
    layout.rs      # split-pane layout
    schema_list.rs # left pane: schema tree
    detail.rs      # right pane: schema detail
    statusbar.rs   # bottom bar
  parser/
    mod.rs         # version detection + dispatch
    v2.rs          # Swagger 2.0
    v3.rs          # OpenAPI 3.x
  tree.rs          # tree node types, expand/collapse state
  keymap.rs        # action enum + default keybinding table
  search.rs        # fuzzy search state
  config.rs        # Lua config loading (Phase 8)
  indexer.rs       # SQLite indexer (Phase 7)
fixtures/
  petstore.yaml    # Swagger Petstore spec for local dev/testing
```

## Key dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI rendering |
| `crossterm` | Terminal backend |
| `openapiv3` | OpenAPI 3.x parsing |
| `serde_yaml` | YAML deserialisation |
| `serde_json` | JSON deserialisation |
| `tokio` | Async runtime |
| `reqwest` | Remote URL fetching |
| `rusqlite` | SQLite / FTS5 index |
| `mlua` | Lua 5.4 config (Phase 8) |

## Architecture decisions

- **Compile frequently** — prefer incremental: write a little, `cargo build`, fix errors, continue.
- **No async in the TUI hot path** — keep the event loop sync; async only for the indexer and remote URL fetch.
- **SQLite at runtime** — TUI reads only from the index DB. Direct file-path invocation parses in memory and bypasses the index.
- **Lua config is Phase 8** — build the full core UX first; Lua is bolted on last.

## Spec conventions

- OpenAPI 3.x schemas live under `#/components/schemas`
- Swagger 2.0 schemas live under `#/definitions`
- Version detected by `openapi:` / `swagger:` field at parse time

## Keybindings (defaults, nvim-style)

`j/k` navigate · `Enter/Space` expand/collapse · `gd` follow $ref · `C-o` back · `gg/G` top/bottom · `zR/zM` expand/collapse all · `/` search · `n/N` next/prev match · `Tab` switch pane · `q/ZZ` quit

## Phases (see doc/todo.md for detail)

1. Scaffold crate, blank ratatui window
2. Parse single hardcoded YAML, print schema names
3. Basic split-pane layout with schema list on left
4. Tree expand/collapse with keyboard nav
5. Detail pane rendering
6. `gd` ref-following + breadcrumb history
7. `/` search
8. SQLite indexer (`speq index` subcommand)
9. Lua config + keybinding layer
