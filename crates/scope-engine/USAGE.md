# SCOPE Usage

SCOPE is the semantic code search, read, edit, and propagation engine behind the
Coding app. The model should not write SCOPE positioning syntax. SCOPE produces
stable read handles from search results; the model copies those handles into
`read_code` and copies read line anchors into `edit_code`.

## `search_code`

`search_code` is the normal entry point for locating code. It accepts a content
query plus optional narrowing fields such as `path`, `include`, and `limit`.
The query defaults to literal matching with smart case, so code fragments such
as `matching_commands(` do not need escaping. Use `mode: "regex"` only when a
regular expression is intended.

Common options mirror the useful parts of `rg`:

```json
{
  "query": "matching_commands(",
  "mode": "literal",
  "path": "src/dashboard",
  "include": ["*.rs"],
  "exclude": ["target/**"],
  "types": ["rust"],
  "case": "smart",
  "word": false,
  "line": false,
  "hidden": false,
  "respect_ignore": true,
  "follow": false,
  "limit": 20
}
```

- `mode: "literal" | "regex"` corresponds to `rg -F` versus regex search.
- `case: "sensitive" | "insensitive" | "smart"` corresponds to `rg -s`, `rg -i`, and `rg -S`.
- `word` and `line` correspond to `rg -w` and `rg -x`; `line` overrides `word`.
- `path` restricts the searched subtree, like passing a path to `rg`.
- `include` and `exclude` are glob arrays; exclusions are separate instead of `!glob`.
- `types` and `type_not` filter by SCOPE language type or known extension.
- `hidden`, `respect_ignore`, and `follow` correspond to `rg --hidden`, default ignore behavior, and `rg -L`.

Search returns compact read targets:

```text
1268#k7Qp|src/dashboard/mod.rs::fn run_tui_dashboard #L1268-L1320
286#b91Z|src/dashboard/mod.rs::trait DashboardHistoryLoader #L286-L302
1#a0F2|src/dashboard/mod.rs#L1-L24
```

The left side is a stable read handle. Its format is `start_line#hash4`.

Rules:

- The handle is a read capability for the canonical target label, not a content
  fingerprint.
- The four-character hash is derived only from the canonical target label.
- The handle must not include target body text, search query, session salt, file
  mtime, read timestamp, line hashes, or freshness data.
- Search results inside an AST symbol point at that symbol's canonical target
  label.
- Search results outside an AST symbol point at a small canonical line range.
- Multiple matches inside the same target are deduplicated.

## `read_code`

The normal read path uses a search handle:

```json
{ "ref": "1268#k7Qp" }
```

Explicit path ranges for imports, top-level code, search misses, and
user-specified locations belong to the runtime `read_file` tool, not to
SCOPE `read_code`.

Read output is source text with per-line edit anchors:

```text
1268#7a|fn run_tui_dashboard(...) {
1269#c1|    ...
```

Do not repeat the search handle, canonical target label, or path in model-facing
read output when the model already obtained them from search. The structured
response may still carry the path for UI and scoped-instruction plumbing.

## `edit_code`

`edit_code` keeps the explicit path plus line-anchor API:

```json
{
  "edits": [
    {
      "path": "src/dashboard/mod.rs",
      "op": "replace",
      "start": "1268#7a",
      "end": "1320#d4",
      "content": "fn run_tui_dashboard(...) {\n    ...\n}"
    }
  ]
}
```

Operations:

- `replace` replaces the inclusive range from `start` to `end`; `content: null`
  deletes the range.
- `append` inserts `content` after `start`.
- `prepend` inserts `content` before `start`.

Line anchors use `line#hash2`, where `hash2` is a two-character hex prefix of
the current line content. Line hashes are stale-edit guards, not target
identity. SCOPE verifies anchors against the current file before writing,
rejects mismatches, applies edits transactionally per call, reparses modified
source, and returns propagation results.

Use raw file tools only for non-source files or cases outside SCOPE engine
responsibility.
