# SCOPE Diff Format

SCOPE Diff is a selector-based patch format for `scope-engine`. It is meant to
be the semantic counterpart to raw file patches: the diff describes **what to do**
with a selected code object, while SCOPE selectors describe **where the object is**.

The format intentionally has only three top-level actions:

- `Add`
- `Delete`
- `Update`

Operations such as "insert before", "insert after", "replace", "move", and
"rename" are not separate diff actions. Insert positioning belongs in selector
semantics, replacement is expressed by `Update`, and semantic refactorings belong
in dedicated refactoring APIs outside SCOPE Diff.

## Design goals

1. **Selector-first positioning**
   - The action target is always a SCOPE selector.
   - A selector answers where the operation applies and what range or insertion
     point is selected.
   - The diff action does not encode symbol lookup, search, propagation policy, or
     refactoring intent.

2. **Small action surface**
   - `Add`, `Delete`, and `Update` are enough for textual changes.
   - Higher-level behavior must not be smuggled into action names.

3. **Guarded edits**
   - Destructive changes carry old-side text or context so stale selectors fail
     instead of silently changing the wrong target.
   - Selectors must resolve uniquely before applying an edit.

4. **Propagation-aware execution**
   - After a successful source edit, SCOPE reports affected symbols and
     diagnostics in the same propagation result shape used by semantic editing
     tools.

## File envelope

A complete selector-based patch is wrapped in the same explicit envelope name as raw patches:

```text
*** Begin Patch
*** Add: <selector>
+new content

*** Delete: <selector>
-old content guard

*** Update: <selector>
@@
 context
-old content
+new content
 context
*** End Patch
```

Each action starts with a header line:

```text
*** Add: <selector>
*** Delete: <selector>
*** Update: <selector>
```

The selector is parsed by SCOPE, not by the diff body parser. This keeps the diff
format independent from language-specific symbol resolution details.

## Selectors as operation objects

The object of every action is a selector:

```text
*** Update: crates/scope-engine/src/selector.rs::Selector::parse
```

The action says which primitive change is requested. The selector says what that
primitive applies to.

### Target selectors

Target selectors select existing ranges or symbols. They are valid for `Delete`
and `Update`, and can be used as guards for `Add` when the selector denotes a
parent or containing object.

Examples:

```text
src/foo.rs::Foo::new
src/foo.rs#L120-L180
src/foo.rs#match:/ProjectInstructions/
src/foo.rs#enclosing:L150
```

Rules:

- A symbol selector resolves to the current symbol range.
- A line-range selector resolves to the exact file range.
- A match selector must be unique. If it is not unique, SCOPE must reject the edit
  and return candidates instead of guessing.
- An enclosing selector resolves to the innermost containing symbol, then follows
  normal symbol edit semantics.
- An outline selector is read-only and must not be accepted by mutating actions.

### Insertion selectors

`Add` needs an insertion or creation target. Inserting before or after something
is selector semantics, not a separate action.

Insertion selector shapes:

```text
src/foo.rs#file:start
src/foo.rs#file:end
src/foo.rs#before:L120
src/foo.rs#after:L120
src/foo.rs#match:/needle/#before
src/foo.rs#match:/needle/#after
src/foo.rs::impl Foo#body:start
src/foo.rs::impl Foo#body:end
```

These forms still answer only "where is the insertion point?". They do not change
the action taxonomy.

## Actions

### Add

`Add` inserts new text at the selector-designated insertion point, or creates a
new selected object when the selector denotes a creation location.

```text
*** Begin Patch
*** Add: src/user.rs::impl User#body:end
+    pub fn display_name(&self) -> &str {
+        &self.display_name
+    }
*** End Patch
```

Rules:

- Body lines must start with `+`.
- The payload is the body with the leading `+` prefixes removed.
- The selector must resolve to exactly one insertion point or creation target.
- `Add` must not overwrite existing text. If replacement is intended, use
  `Update`.
- For language-aware targets, SCOPE normalizes indentation only when the caller
  explicitly enables such a policy. The base format itself is byte-preserving.

### Delete

`Delete` removes the selector-designated range.

```text
*** Begin Patch
*** Delete: src/user.rs::User::legacy_name
-    pub fn legacy_name(&self) -> &str {
-        &self.name
-    }
*** End Patch
```

Rules:

- Body lines, when present, must start with `-`.
- The old-side body is a guard. After removing the leading `-` prefixes, it must
  match the selected range according to the selected matching policy.
- If the body is omitted, SCOPE deletes the full resolved selector range only for
  stable selectors such as unique symbols or exact line ranges.
- If the selector is stale, ambiguous, or the guard does not match, the edit must
  fail without modifying files.

### Update

`Update` applies one or more guarded hunks inside the selector-designated range.

```text
*** Begin Patch
*** Update: src/user.rs::User::display_name
@@
     pub fn display_name(&self) -> &str {
-        &self.name
+        &self.display_name
     }
*** End Patch
```

Rules:

- Hunks use stripped unified-diff body lines:
  - space-prefixed lines are context
  - `-` lines are old text
  - `+` lines are new text
- A bare `@@` hunk header is allowed when the selector already narrows the target
  enough for unambiguous matching.
- Numbered hunk headers are supported. Line numbers are relative to the resolved
  selector range unless the header explicitly says otherwise.
- Old-side text plus context must match exactly one location inside the resolved
  selector range.
- `Update` can express replacement by deleting all old-side lines in the selected
  range and adding the new body. A separate `Replace` action is intentionally not
  needed.

## Grammar

This is the normative grammar for the base SCOPE Diff format:

```text
patch           := begin action+ end
begin           := "*** Begin Patch" newline
end             := "*** End Patch" newline?

action          := add | delete | update
add             := "*** Add: " selector newline add_line+
delete          := "*** Delete: " selector newline delete_line*
update          := "*** Update: " selector newline hunk+

add_line        := "+" text newline
delete_line     := "-" text newline
hunk            := hunk_header hunk_line+
hunk_header     := "@@" text? newline
hunk_line       := (" " | "-" | "+") text newline
selector        := <parsed by SCOPE selector parser>
```

Parsers must reject mixed body prefixes for `Add` and `Delete`. `Update` is the
only action whose body accepts context, old, and new lines together.

## Execution model

An implementation applies a SCOPE diff in this order:

1. Parse the envelope and action headers.
2. Resolve every selector against the current project state.
3. Reject the whole diff if any mutating selector is invalid, read-only,
   ambiguous, or stale.
4. Check old-side guards and hunk contexts.
5. Apply actions in file order with conflict detection, or apply them transactionally
   through an equivalent edit plan.
6. Reparse modified files.
7. Report affected symbols, diagnostics, and any propagation review events.

The preferred failure mode is all-or-nothing. Partial application is allowed only
if the caller explicitly requests it and the result reports which actions were
applied.

## Interaction with propagation analysis

SCOPE Diff is not only a text patch. The selector resolution step gives SCOPE the
semantic anchor needed to calculate impact after edits.

For source files, a successful edit returns at least:

- modified files
- changed selector ranges
- affected enclosing symbols
- candidate references or call sites when LSP data is available
- parser or type diagnostics when available

For non-source files, SCOPE applies the textual edit without semantic
propagation, and the result states that propagation analysis was skipped.

## Relationship to raw `apply_patch`

Raw `apply_patch` is file-coordinate based. SCOPE Diff is selector-coordinate
based.

Use SCOPE Diff when:

- the target can be expressed as a symbol, range, unique match, enclosing symbol,
  or insertion selector
- stale-location detection matters
- propagation review must remain attached to the edit

Use raw `apply_patch` when:

- editing non-source files outside SCOPE responsibility
- creating or deleting files that SCOPE cannot model yet
- making cross-file structural changes before SCOPE has a safe transactional model
- recovering from a SCOPE parser or selector limitation

When raw patches are used on source files from the Coding app, the host still
runs the propagation bridge after the edit. SCOPE Diff is the native path.

## Examples

### Update a method body

```text
*** Begin Patch
*** Update: src/config.rs::Config::load
@@
         let text = std::fs::read_to_string(path)?;
-        toml::from_str(&text)
+        toml::from_str(&text).map_err(Into::into)
*** End Patch
```

### Add a method at the end of an impl block

```text
*** Begin Patch
*** Add: src/config.rs::impl Config#body:end
+    pub fn is_empty(&self) -> bool {
+        self.entries.is_empty()
+    }
*** End Patch
```

### Delete a stale field with a guard

```text
*** Begin Patch
*** Delete: src/config.rs#match:/legacy_timeout: Duration,/
-    legacy_timeout: Duration,
*** End Patch
```

### Replace a selected line range through `Update`

```text
*** Begin Patch
*** Update: src/config.rs#L42-L45
@@
-    let timeout = Duration::from_secs(30);
-    let retries = 3;
+    let retry_policy = RetryPolicy::default();
*** End Patch
```

## Final decisions

- Insertion selector modifiers are part of the selector grammar, not the diff
  action grammar.
- Multi-file application uses an edit-plan representation and defaults to
  all-or-nothing transaction semantics.
- Formatting and indentation policy is explicit. The base format does not silently
  format code.
- Refactoring operations such as rename, move, extract, and inline are separate
  semantic APIs rather than extra SCOPE Diff action names.
