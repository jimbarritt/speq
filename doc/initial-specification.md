# speq — Agent Build Spec

**speq** — OpenAPI Spec Explorer. Part of the Ubiqtek/Ubiq toolchain.

Binary name: `speq`  
Crate name: `speq`  
Config dir: `~/.config/speq/`  
Cache dir: `~/.cache/speq/`

## Overview

A Rust TUI application for navigating and inspecting OpenAPI specifications at organisational scale. The core problem it solves: when you have 100 microservices each with their own OpenAPI spec, understanding the API landscape — finding schemas, spotting inconsistencies, tracing references across services — is extremely difficult with existing tools. Flat text editors and web viewers handle single specs poorly, and nothing handles the multi-service case at all.

speq indexes all specs locally into SQLite and provides a fast, keyboard-driven TUI for browsing and searching across the entire org's API surface.

---

## Prior Art & Differentiation

**`openapi-tui`** (github.com/zaghaghi/openapi-tui) is the closest existing tool — written in Rust, uses ratatui, actively maintained. However it targets a fundamentally different use case:

- Single spec at a time, no multi-service concept
- Primary focus is **calling** APIs (Postman-in-the-terminal)
- No local index or cross-service search
- No nvim keybindings or Lua config

speq goes in the opposite direction: read-only, schema-first, multi-service, nvim-native. The goal is **comprehension at scale**, not API execution.

---

## Language Decision

**Chosen: Rust**

TypeScript was considered as an alternative given the existing cache tooling is in that ecosystem and agent iteration speed is faster. However Rust was chosen for the following reasons:

- `ratatui` is best-in-class for TUI rendering — smooth, no flicker, instant keypress response. The Node TUI ecosystem (`ink`, `blessed`) is either architecturally limited for complex layouts or unmaintained.
- This is a daily-use reading tool where feel matters. The difference in rendering quality is noticeable.
- Single binary distribution with no runtime dependency.
- The core data structures (tree state, keybinding dispatch, breadcrumb history) are well suited to Rust's ownership model once the initial build is done.

The tradeoff accepted is that agentic builds in Rust are slower to iterate due to borrow checker friction. The agent should prefer compiling frequently and fixing errors incrementally rather than writing large amounts of code before checking.

---

## Technical Foundation

- **Language:** Rust
- **TUI framework:** `ratatui` (with `crossterm` backend)
- **OpenAPI parsing:** `openapiv3` crate for 3.x; raw `serde_json`/`serde_yaml` deserialization for Swagger 2.0; version auto-detected before selecting parser
- **Lua config:** `mlua` crate with Lua 5.4
- **Async runtime:** `tokio` (needed for remote URL fetching)
- **HTTP client:** `reqwest` (for remote specs)
- **Database:** `rusqlite` with FTS5 extension for the local index

---

## Input

The tool accepts an OpenAPI spec from any of the following sources, auto-detected:

- **Local YAML file** — path argument, detected by extension or content
- **Local JSON file** — path argument, detected by extension or content
- **Remote URL** — if the argument begins with `http://` or `https://`, fetches via HTTP
- **Stdin** — if no argument is provided (or `-` is passed), reads from stdin; auto-detects YAML vs JSON by content

### Version Auto-Detection

After parsing, the tool inspects the document to determine the OpenAPI version:

- Presence of `swagger: "2.0"` → OpenAPI 2.0 (Swagger)
- Presence of `openapi: "3.0.x"` → OpenAPI 3.0
- Presence of `openapi: "3.1.x"` → OpenAPI 3.1

Schema definitions are sourced from:
- `#/definitions` for Swagger 2.0
- `#/components/schemas` for OpenAPI 3.x

---

## Global Layout

```
┌─────────────────────────────────────────────────────────┐
│  speq  ·  petstore.yaml  ·  OpenAPI 3.0          │
├──────────────────────┬──────────────────────────────────┤
│  [Schema list/tree]  │  [Detail pane]                   │
│                      │                                  │
│                      │                                  │
│                      │                                  │
├──────────────────────┴──────────────────────────────────┤
│  [Status bar / keybind hints]                           │
└─────────────────────────────────────────────────────────┘
```

- Left pane: ~35% width, scrollable list/tree of items
- Right pane: ~65% width, detail view for selected item
- Top bar: filename, detected spec version, current mode
- Bottom bar: contextual keybind hints

---

## Feature 1: Schema Definition Browser

### Purpose
Allow the user to explore all schema definitions in the spec, understand their structure, follow references, and inspect field-level metadata.

### Left Pane — Schema List

Displays all top-level schema names as a navigable list. Each schema can be expanded in-place to reveal its properties as a tree. Nodes in the tree:

- **Object schema** → expandable, shows properties as children
- **Array schema** → shows `items` type inline or as expandable child
- **Primitive** (string, number, boolean, integer) → leaf node, shown with type badge
- **`$ref`** → shown with a `→` indicator and the target name; can be followed
- **`allOf` / `oneOf` / `anyOf`** → shown as a combiner node, children are the constituent schemas (resolved)
- **`additionalProperties`** → shown as a special child node

Each tree node shows: field name, type badge, required indicator (`*`), and a short description excerpt if available.

### Right Pane — Schema Detail

When a schema or property node is selected, the detail pane shows the full resolved metadata:

- Schema name (or field path for nested properties)
- Type, format
- Description (full, word-wrapped)
- Required / optional
- Constraints: `minimum`, `maximum`, `minLength`, `maxLength`, `pattern`, `enum` values
- `example` value if present
- `default` value if present
- For `$ref`: the resolved target name with an indication it is a reference
- For combiners: lists each branch

### Interactions

Default keybindings follow nvim conventions throughout. All bindings are remappable via config (see Keybinding Configuration below).

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` / `Space` | Expand / collapse node |
| `gd` | Follow `$ref` inline (expand in place, push to breadcrumb history) |
| `Ctrl-o` | Go back (pop breadcrumb history — nvim jump list convention) |
| `Ctrl-i` | Go forward (push breadcrumb history) |
| `gg` | Jump to top of list |
| `G` | Jump to bottom of list |
| `zc` | Collapse current node |
| `zo` | Open (expand) current node |
| `zR` | Expand all top-level schemas |
| `zM` | Collapse all |
| `/` | Open fuzzy search over schema names |
| `n` | Next search match |
| `N` | Previous search match |
| `Esc` | Close search / cancel / clear |
| `Ctrl-d` | Scroll detail pane down half page |
| `Ctrl-u` | Scroll detail pane up half page |
| `Tab` | Switch focus between left and right pane |
| `q` / `ZZ` | Quit |

### Search

`/` opens an inline search bar at the bottom of the left pane, identical to nvim's `/` search. Typing filters the schema list in real time using fuzzy matching. `Enter` confirms and selects; `n`/`N` cycle through matches; `Esc` cancels and restores previous position.

---

## Keybinding Configuration

### Config file location

```
~/.config/speq/config.lua
```

The config file is a Lua file loaded at startup. If it does not exist, all defaults apply. Unknown keys are ignored with a warning printed to stderr.

### API style

Keybindings use a `vim.keymap.set`-inspired API. The global config object is `speq` (analogous to `vim` in nvim config):

```lua
-- ~/.config/speq/config.lua

-- Remap gd to Enter for $ref following
speq.keymap.set("n", "<CR>", "follow_ref")

-- Remap quit
speq.keymap.set("n", "<leader>q", "quit")

-- Add an extra binding alongside the default
speq.keymap.set("n", "L", "expand_node")

-- Use leader key (default leader is Space, same as many nvim setups)
speq.set("leader", " ")
```

### Signature

```
speq.keymap.set(mode, lhs, action)
```

- `mode` — currently only `"n"` (normal) is supported; reserved for future modes
- `lhs` — key string using nvim notation: `<CR>`, `<Esc>`, `<C-d>`, `<leader>`, etc.
- `action` — string action identifier (see action registry below)

### Action registry

The following action identifiers are available for binding:

| Action | Default |
|--------|---------|
| `move_down` | `j` |
| `move_up` | `k` |
| `expand_node` | `<CR>` |
| `collapse_node` | `zc` |
| `open_node` | `zo` |
| `expand_all` | `zR` |
| `collapse_all` | `zM` |
| `follow_ref` | `gd` |
| `jump_back` | `<C-o>` |
| `jump_forward` | `<C-i>` |
| `goto_top` | `gg` |
| `goto_bottom` | `G` |
| `search` | `/` |
| `search_next` | `n` |
| `search_prev` | `N` |
| `scroll_down` | `<C-d>` |
| `scroll_up` | `<C-u>` |
| `focus_next_pane` | `<Tab>` |
| `quit` | `q` |

### Implementation notes

- Use the `mlua` crate to embed a Lua runtime
- Config is loaded once at startup before the TUI initialises
- Default bindings are registered first; `config.lua` bindings are applied on top and may override or extend them
- Multiple keys can be bound to the same action
- Bindings cannot be removed, only overridden (to remove a default, bind the original key to `"noop"`)

---

## UI Layout Examples

Using the Swagger Petstore spec (OpenAPI 3.0) as reference — schemas: `Pet`, `Pets`, `Error`.

### State 1: Initial view — all schemas collapsed

```
speq · petstore.yaml · OpenAPI 3.0                    [schemas]
┌─────────────────────┬─────────────────────────────────────────────────┐
│ Schemas (3)         │ Pet                                             │
│                     │ ────────────────────────────────────────────── │
│ ▶ Error             │ type    object                                  │
│ ▶ Pet           ◀── │ source  components/schemas/Pet                  │
│ ▶ Pets              │                                                 │
│                     │ required                                        │
│                     │   id, name                                      │
│                     │                                                 │
│                     │ properties                                      │
│                     │   id       integer (int64)  *                   │
│                     │   name     string           *                   │
│                     │   tag      string                               │
│                     │                                                 │
├─────────────────────┴─────────────────────────────────────────────────┤
│ j/k navigate · Enter expand · gd follow $ref · / search · q quit     │
└───────────────────────────────────────────────────────────────────────┘
```

### State 2: `Pets` expanded — array with `$ref` child

```
speq · petstore.yaml · OpenAPI 3.0                    [schemas]
┌─────────────────────┬─────────────────────────────────────────────────┐
│ Schemas (3)         │ Pets                                            │
│                     │ ────────────────────────────────────────────── │
│ ▶ Error             │ type      array                                 │
│ ▶ Pet               │ maxItems  100                                   │
│ ▼ Pets          ◀── │                                                 │
│   └ items  →Pet     │ items                                           │
│                     │   $ref → Pet                                    │
│                     │                                                 │
│                     │ [gd] follow →Pet                                │
│                     │                                                 │
├─────────────────────┴─────────────────────────────────────────────────┤
│ j/k navigate · Enter expand · gd follow $ref · C-o back · q quit     │
└───────────────────────────────────────────────────────────────────────┘
```

### State 3: `gd` on `→Pet` — ref expanded inline, breadcrumb shown

```
speq · petstore.yaml · OpenAPI 3.0                    [schemas]
┌─────────────────────┬─────────────────────────────────────────────────┐
│ Schemas (3)         │ Pets › items › Pet                              │
│                     │ ────────────────────────────────────────────── │
│ ▶ Error             │ (expanded from $ref)                            │
│ ▶ Pet               │                                                 │
│ ▼ Pets              │ type      object                                │
│   ▼ items  →Pet     │ required  id, name                              │
│     ├ id   int64 *  │                                                 │
│     ├ name str   *  │ id                                              │
│     └ tag  str  ◀── │   type    integer                               │
│                     │   format  int64                                 │
│                     │   required  yes                                 │
│                     │                                                 │
├─────────────────────┴─────────────────────────────────────────────────┤
│ j/k navigate · Enter expand · C-o back · / search · q quit           │
└───────────────────────────────────────────────────────────────────────┘
```

### State 4: `/` search active

```
speq · petstore.yaml · OpenAPI 3.0                    [schemas]
┌─────────────────────┬─────────────────────────────────────────────────┐
│ Schemas (3)         │                                                 │
│                     │                                                 │
│   Error             │                                                 │
│ ▸ Pet               │                                                 │
│   Pets              │                                                 │
│                     │                                                 │
│                     │                                                 │
│                     │                                                 │
├─────────────────────┴─────────────────────────────────────────────────┤
│ /pet█                                                    [Esc] cancel │
└───────────────────────────────────────────────────────────────────────┘
```

---

## Feature 2: Local Spec Cache & SQLite Indexer

### Cache Directory Layout

The canonical cache root is `~/.cache/speq/`. This is not configurable.

```
~/.cache/speq/
├── specs/
│   ├── payments-service/
│   │   ├── openapi.yaml
│   │   └── meta.yaml
│   ├── order-service/
│   │   ├── openapi.yaml
│   │   └── meta.yaml
│   └── ...
└── index.db
```

Each service lives in its own subdirectory under `specs/`. The spec file is always named `openapi.yaml`. A `meta.yaml` sidecar file sits alongside it.

### `meta.yaml` Schema

Written by the external gh cli cache script at the time the spec is fetched. The indexer reads this alongside the spec.

```yaml
service: payments-service          # required — canonical service name
repo: git@github.com:acme/payments # required — git remote URL
team: payments                     # optional — owning team
domain: finance                    # optional — business domain / grouping
cached_at: 2026-02-18T17:00:00Z    # required — ISO8601, set by cache script
source_url: https://raw.github...  # optional — URL the spec was fetched from
```

All fields except `service`, `repo`, and `cached_at` are optional. The indexer tolerates missing optional fields gracefully.

### Indexer CLI

The indexer is a subcommand of the main binary:

```
speq index          # index all services in ~/.cache/speq/specs/
speq index --force  # re-index all, ignoring mtime checks
speq index payments-service   # re-index a single service by name
```

**Incremental indexing:** by default, the indexer skips a service if the `openapi.yaml` mtime has not changed since it was last indexed (recorded in the DB). `--force` overrides this.

Output is printed to stdout:

```
indexing payments-service ... ok (47 schemas, 123 paths)
indexing order-service    ... ok (12 schemas, 34 paths)
indexing legacy-api       ... skipped (unchanged)
indexing broken-api       ... error: failed to parse openapi.yaml: missing required field 'info'
indexed 2 services, 1 skipped, 1 error
```

### SQLite Schema

Database at `~/.cache/speq/index.db`.

```sql
CREATE TABLE services (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,   -- from meta.yaml service field
  repo        TEXT,
  team        TEXT,
  domain      TEXT,
  source_url  TEXT,
  cached_at   TEXT,                   -- ISO8601
  indexed_at  TEXT,                   -- ISO8601, set by indexer
  spec_path   TEXT NOT NULL,          -- absolute path to openapi.yaml
  openapi_version TEXT,               -- e.g. "3.0", "3.1", "2.0"
  spec_title  TEXT,                   -- from info.title
  spec_version TEXT                   -- from info.version
);

CREATE TABLE schemas (
  id          INTEGER PRIMARY KEY,
  service_id  INTEGER NOT NULL REFERENCES services(id),
  name        TEXT NOT NULL,          -- schema name as defined in spec
  description TEXT,
  type        TEXT,                   -- object, array, string, etc.
  raw_json    TEXT                    -- full schema serialised as JSON for runtime use
);

CREATE TABLE properties (
  id          INTEGER PRIMARY KEY,
  schema_id   INTEGER NOT NULL REFERENCES schemas(id),
  service_id  INTEGER NOT NULL REFERENCES services(id),
  name        TEXT NOT NULL,          -- property field name
  type        TEXT,
  format      TEXT,
  description TEXT,
  required    INTEGER,                -- 0 or 1
  ref_target  TEXT                    -- if $ref, the target schema name
);

CREATE TABLE paths (
  id          INTEGER PRIMARY KEY,
  service_id  INTEGER NOT NULL REFERENCES services(id),
  path        TEXT NOT NULL,          -- e.g. /payments/{id}
  method      TEXT NOT NULL,          -- get, post, put, delete, etc.
  operation_id TEXT,
  summary     TEXT,
  description TEXT,
  tags        TEXT                    -- JSON array of tag strings
);

-- FTS5 virtual table for full-text search across schemas and properties
CREATE VIRTUAL TABLE search_index USING fts5(
  service_name,
  schema_name,
  property_name,
  description,
  content='',                         -- contentless, populated by indexer
  tokenize='unicode61'
);
```

### Indexing Process (per service)

1. Read and parse `meta.yaml` — extract service metadata
2. Read and parse `openapi.yaml` — detect version, deserialise
3. Upsert the `services` row
4. Delete existing `schemas`, `properties`, `paths` rows for this service
5. For each schema in `components/schemas` (or `definitions`):
   - Insert a `schemas` row with `raw_json` containing the full schema
   - For each property, insert a `properties` row
   - Recursively handle `allOf`/`oneOf`/`anyOf` — flatten into properties with a `combiner` annotation
6. For each path + method, insert a `paths` row
7. Rebuild the `search_index` FTS5 entries for this service
8. Update `indexed_at` on the `services` row

### Runtime Usage by the TUI

The TUI reads **only from SQLite** at runtime — it never parses YAML directly during browsing. When a user expands a schema node, the `raw_json` column is deserialised on demand and rendered into the tree. This keeps startup instant regardless of how many services are indexed.

The one exception is when the user opens a spec via a direct file path argument (`speq ./path/to/spec.yaml`) — in this case the file is parsed in memory and not written to the index.

---

## Future Features (to be specced)

- **Path / endpoint browser** — navigate routes, methods, parameters, request bodies, responses
- **Request body inspector** — drill into request body schemas with the same tree view
- **Response inspector** — per-status-code response schemas
- **Dependency graph view** — which schemas reference which (visual or list-based)
- **Validation mode** — highlight missing required fields, broken `$ref`s, inconsistencies
- **Export** — copy a schema or field path to clipboard
- **Diff mode** — compare two versions of a spec
