# BRIEF — Strike 257.2: destructure-as-Map + `StructPattern` annihilation

Read `DESIGN.md` first, then this. Strike 257.1 (native `WatAST::Map`/`Set` value
literals) is DONE and committed (`e2ffc527`). This strike makes a `Map` in binder/
pattern position a **destructure**, migrates hash-destructure off `StructPattern`,
and **deletes `WatAST::StructPattern` entirely**. After this, the 257.0 probe goes
GREEN and `{x y z}` is a parse error.

## The work (one paragraph)
Today a symbol-head brace (`{x y z}` struct-destructure, `{var :field}` hash-
destructure) still parses to `WatAST::StructPattern`, and 14 binding-context sites
detect destructure by `matches!(b, StructPattern(..))`. Flip it: ALL `{…}` parse to
`WatAST::Map` (delete the `BraceKind` dispatch + both destructure-body parsers); a
single classifier recognises a Map-in-binder-position as a destructure; the 14 sites
read the classifier instead of matching `StructPattern`; then `StructPattern` and its
~75 vanishing arms are deleted (the compiler's non-exhaustive-match errors are your
worklist — the cascade is the progress meter, not a crisis). `{:keys [x y z]}` binds
each named field (the EDN replacement for the old `{x y z}`); `{var :field …}` binds
`var` ← field `:field` (unchanged semantics, new node). `{x y z}` now hits the map-
literal even-arity rule → a clear parse/check error guiding migration to `{:keys […]}`.

## The classifier (THE pinned contract — one authoritative helper)
Add to `src/ast.rs` (or a small `src/destructure.rs`), used by check, runtime, AND
closure_extract — DRY, like `is_metadata_map`:

```rust
/// A Map in binder/pattern position, classified. Both forms reduce to a flat
/// list of (binding_name, field_name, binding_span) — each site iterates it.
pub struct MapDestructure { pub bindings: Vec<(Identifier, String, Span)> }

/// Some(..) iff `pairs` is a valid destructure pattern:
///   keys-destructure: exactly [(Keyword(":keys"), Vector([Symbol, ...]))]
///     -> each symbol binds the SAME-named field (binding_name == field_name).
///   hash-destructure: every pair is (Symbol(var), Keyword(":field"))
///     -> var binds field :field (field_name = the keyword sans leading ':').
/// None -> not a destructure (a plain value-position map literal, or malformed
///   in binder position -> caller raises the existing "not a destructure" error).
pub fn classify_map_destructure(pairs: &[(WatAST, WatAST)]) -> Option<MapDestructure>
```

Field-name normalization: strip the leading `:` from the keyword; keep the rest
verbatim (e.g. `:magnitude` -> `"magnitude"`). The OLD struct-destructure read field
names straight from the bare symbols; keys-destructure reads them from the `:keys`
vector symbols — same downstream `(binding, field)` pairs, so the 14 sites' existing
field-lookup / type-inference / binding logic is PRESERVED, only the source of the
pairs changes.

## Rooms — the 14 load-bearing sites (swap StructPattern-match → classifier)
Each currently branches on `WatAST::StructPattern(items, _)` and inspects `items[1]`
(Keyword = hash, Symbol = struct). Replace with: detect `WatAST::Map(pairs, _)` in
the binder/pattern slot, call `classify_map_destructure(pairs)`, and feed the
resulting `bindings` into the SAME logic that exists today.

Runtime (`src/runtime.rs`):
- `parse_let_binding` (~5772) — the central let-binder dispatch (the `StructPattern`
  arm builds `LetBinding::StructDestructure` / `HashDestructure`). Map arm → classify
  → build the binding list. Keep `LetBinding` variants or collapse to one — your call;
  simpler is better.
- `try_match_pattern` (~11456) — runtime match-arm: hash-destructure field lookup +
  bind against Record/Struct/HashMap. Map arm → classify (hash form).
- `try_match_pattern_ast` (~21807) — stepper AST match: destructure arm returns
  `Ok(None)` (deferred to runtime). Map arm → same.

Check (`src/check.rs`):
- `process_let_binding` (~11325) — check-time let-binder: struct-field type lookup +
  fresh-var assignment. Map arm → classify → same type logic.
- `infer_match` (~5856) — match-arm typing: bind each var to fresh, set wildcard.
- `detect_match_shape` (~6222) — skip destructure arms → `MatchShape::Open`.
- `check_let_for_scope_deadlock_inferred` (~9929) — extract bound names from binder.

Closure (`src/closure_extract.rs`) — **the trap; see hygiene below**:
- `walk_let_form` (~723) — add destructure binding names to `current_locals`.
- `collect_pattern_bindings` (~1045) — match-arm pattern captures.
- `rewrite_let` (~2139) — scope extension for the binder.
- `rewrite_with_scope` (~2096) — **must preserve binder positions verbatim**.
- `walk_free_symbols` (~668) — the binder symbols are BINDINGS, not free refs.

## Parser (`src/parser.rs`)
Delete `parse_struct_destructure_body`, `parse_hash_destructure_body`, the `BraceKind`
enum + the `LBrace` content-shape dispatch (~276). The `LBrace` arm now always calls
`parse_map_literal_body` (→ `WatAST::Map` from 257.1). Delete `MalformedStructPattern`
+ its Display + `ast_variant_label`'s struct-pattern arm. A symbol-head brace is now
just a Map; binder-position interpretation is the runtime/check job.

## `StructPattern` annihilation
Delete the `WatAST::StructPattern` variant from `src/ast.rs` and the `struct_pattern()`
constructor. Then `cargo build` and let non-exhaustive-match errors enumerate the ~75
vanishing arms — delete each (generic `children()`/recursion arms, `variant_name`
labels → gone, diagnostic `"struct-pattern"` strings, the value-position error guards
in runtime/check that said "got a StructPattern"). `hash.rs` `TAG_STRUCT_PATTERN`
deleted. Hash changes for forms that contained the old node — expected, breaking arc.

## The hygiene trap (closure_extract — read carefully)
After this strike a destructure binder is a `Map` node whose `children()` (257.1 made
it `Cow`) flatten the pairs to `[:keys, [x y z]]` or `[var, :field, …]`. The generic
`List`/`Map` walk in `rewrite_with_scope` / `rewrite_let` / `walk_free_symbols` will
descend into the binder. The binding symbols (`x y z` inside `:keys [..]`, or the
`var` names in hash form) MUST be treated as **bindings, not free references**, and
MUST NOT be substituted by the closure rewrite. The let/match handlers add them to
`current_locals` BEFORE walking the body; ensure the binder Map itself is NOT walked
as an expression (or is walked with the binding names already shadowed). Mirror
exactly what the old `StructPattern` arms did (they copied the binder verbatim and
extended locals). If you cannot cleanly prevent binder-symbol substitution, STOP and
report — a silent capture bug here is the worst-case outcome.

## STOP triggers (halt + report; do not improvise)
1. If `classify_map_destructure` cannot cleanly produce the `(binding, field)` pairs
   that the 14 sites' existing logic expects (an impedance mismatch), STOP — do not
   reshape the downstream binding logic to hack around it.
2. If closure hygiene cannot guarantee binder symbols are never substituted, STOP.
3. If deleting `StructPattern` forces a behavior change in match-arm coverage /
   exhaustiveness beyond the mechanical swap, STOP and report the shape.

## Test migration (part of this strike — the cascade includes tests)
- `tests/types/struct_destructure.rs` — migrate `[{outcome} p]` → `[{:keys [outcome]} p]`,
  `[{outcome grace-residue} p]` → `[{:keys [outcome grace-residue]} p]`, etc. Behavior
  (binds the named fields) must be identical. Update the doc comments.
- Hash-destructure probes (`probe_arc234_*`, `[{var :field}]`) — these should pass
  UNCHANGED (same surface, now a Map node). If any breaks, that's a real signal.
- Add a negative test: `{x y z}` in a binder is now a parse/check error (assert it
  fails with a clear message, not a panic).

## Gate (the kill — run it, weigh against the disk, do NOT trust your own summary)
- `cargo build --release` clean.
- `cargo test --release --workspace --no-run` — full test surface COMPILES (every
  binary; a new/deleted variant must compile every exhaustive match).
- **`tests/nursery/probe_arc257_keys_destructure.rs` → 2/2 GREEN** (the arc's whole
  point — `{:keys […]}` now destructures).
- `probe_arc257_native_map_set` → still 3/3 GREEN (value literals unbroken).
- Hash-destructure (`probe_arc234*`) + migrated `struct_destructure` → GREEN.
- Full `cargo test --release --workspace --no-fail-fast`: enumerate every failure;
  for any non-obvious one, **stash-check at HEAD** to prove it is pre-existing, not a
  regression you introduced. Report the pre-existing set explicitly.

## Expectations (scorecard — fixed before the strike)
| what | command | expected |
|---|---|---|
| compiles everywhere | `cargo test --release --workspace --no-run` | 0 errors |
| destructure works | `cargo test --test nursery probe_arc257_keys` | 2/2 pass |
| value literals intact | `cargo test --test nursery probe_arc257_native` | 3/3 pass |
| hash-destructure intact | `cargo test --test nursery probe_arc234` | pass unchanged |
| struct-destructure migrated | `cargo test types::struct_destructure` | pass |
| {x y z} rejected | new negative test | clear error, no panic |
| no new regressions | full workspace + stash-check the odd ones | only pre-existing fail |

Runtime estimate: 60–120 min (the cascade is wide but mechanical; closure hygiene is
the careful part). Return a SCORE: scorecard results, the pre-existing failure set
(stash-verified), honest deltas, files + line counts, any STOP hit. Do NOT commit.
