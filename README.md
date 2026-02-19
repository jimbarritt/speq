# speq

A keyboard-driven TUI for navigating and inspecting OpenAPI specifications. Schema-first, read-only, nvim-style keybindings.

See `doc/initial-specification.md` for the full design spec and `doc/todo.md` for the build plan.

---

## Build

Requires Rust (stable). Install via [rustup](https://rustup.rs) if needed.

```bash
cargo build           # debug build
cargo build --release # optimised build → target/release/speq
```

---

## Run

```bash
# Open a spec file directly
cargo run -- path/to/spec.yaml
cargo run -- path/to/spec.json

# With no argument, falls back to fixtures/petstore.yaml (dev fixture)
cargo run

# Release binary
./target/release/speq path/to/spec.yaml
```

---

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `gg` | Jump to top |
| `G` | Jump to bottom |
| `l` | Expand node |
| `h` | Collapse node |
| `zo` | Expand node |
| `zc` | Collapse node |
| `zR` | Expand all |
| `zM` | Collapse all |
| `Tab` | Switch focus between left and right pane |
| `Ctrl-d` / `Ctrl-u` | Scroll detail pane down / up |
| `q` / `Ctrl-c` | Quit |

---

## Tests

```bash
cargo test
```

---

## Project layout

```
src/
  main.rs          entry point, CLI arg parsing, event loop
  app.rs           App state, navigation methods
  spec.rs          LoadedSpec + SpecVersion types
  tree.rs          TreeNode, NodeKind, NodeInfo, TreeState
  parser/
    mod.rs         version detection, dispatch to v2/v3
    v3.rs          OpenAPI 3.x parser + schema→tree conversion
  ui/
    mod.rs         top-level render function, layout
    schema_list.rs left pane — expandable schema tree
    detail.rs      right pane — schema/property metadata
    statusbar.rs   bottom bar — keybind hints
fixtures/
  petstore.yaml    OpenAPI 3.0 dev fixture (4 schemas)
doc/
  initial-specification.md  full design spec
  todo.md                   phased build plan + progress
```
