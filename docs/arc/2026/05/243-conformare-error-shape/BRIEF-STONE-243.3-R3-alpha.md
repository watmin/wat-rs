# BRIEF — Stone 243.3 R3-α sweep — close 13 obvious vigilia findings

You are sonnet. Stone 243.3 R3-α sweep. 13 mechanical fixes closing obvious/trivial findings from the R2 vigilia round (8 spells parallel: intueri + solvere + purgare + struere + sequi + temperare + exigere + conformare). The remaining 16 findings (architectural + spell-conflict + cross-file-untouched) get debated with orchestrator and either fixed in a follow-up R-round or deferred via attested-arc mechanism. This sweep is the OBVIOUS subset.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Pre-spawn state (working tree)

Working tree carries R3 main 5 fixes + R3 addendum 4 fixes (Stone 243.3 R3.1-R3.9 + R3.13-R3.16 from prior sonnet rounds; uncommitted). Your 13 fixes JOIN them; orchestrator commits Stone 243.3 atomic after R-rounds converge + SCORE Phase B.

**Gates baseline (verified post-addendum):**
- Lib: 890 PASS / 0 FAIL
- tests/function: 8 / 0
- probe_arc243_stone3_typeerror_pattern_a: 3 / 0
- Workspace test-build: clean (exit 0)
- Clippy: 897

## The 13 fixes (one cluster per file as helpful)

### Fix 1 — exigere L1 strip (check.rs:11376-11377)

Current (pre-existing future-arc deferral):
```rust
"Future arcs may generalise to Vector<HolonAST> when integer index or
tuple steps are needed."
```

Action: delete the sentence. The PRESENT-state behavior is documented on the prior line ("Path is `Vector<keyword>`"). The future-arc speculation has no named tracker; per `feedback_pre_existing_is_not_exemption` + `feedback_dont_document_non_fixes` it goes.

### Fix 2 — intueri L1 displaced doc (check.rs:5092-5116)

Inspect `is_primitive_type_keyword_in_value_position` doc comment (lines 5092-5102) and `infer()` doc comment (line 5116+).

The displaced doc lies: `is_primitive_type_keyword_in_value_position`'s doc OPENS with `infer()`'s sentence ("Infer the type of an expression. Returns `CheckResult::ok(type)` when a type can be assigned cleanly,"). At line 5095 the comment switches mid-paragraph to "Stone 242.2 — Doctrine 1 guard: returns true when `k` is a primitive scalar type keyword..." without a paragraph break.

Action:
- `is_primitive_type_keyword_in_value_position` doc starts at the Stone 242.2 sentence (delete the displaced "Infer the type..." opening).
- Verify `infer()`'s doc at line 5116+ is complete without the displaced opening (it likely is; the displaced sentence was orphaned). If `infer()`'s doc reads incomplete after the displacement removal, prepend the displaced opening sentence to `infer()`'s doc.

Report which path you took.

### Fix 3 — intueri L2 variable renames (types.rs)

Inside `parse_defstruct` metadata parsing, rename:
- `fi` (line 2076 — field index into `fm_pairs`, spans ~125 lines) → `field_pair_idx`
- `fpi` (line 2142 — field-pair index into `fpairs`, spans ~54 lines) → `inner_key_idx`
- `idx` (line 1992 — meta-pair index, spans ~216 lines) → `meta_pair_idx`

Mechanical sed-equivalent within their lexical scopes only (NOT global).

### Fix 4 — intueri L2 `_expected`/`_got` convention WHY (check.rs ~1697-1830)

Seven `arc_109_*_migration_hint` functions take `_expected: &str, _got: &str` parameters they don't consume (underscore-prefixed). The pattern is a CONVENTION of the migration-hint dispatch family — `collect_hints` threads expected/got uniformly to all entries; per-hint bodies may or may not consume them.

Action: add a single WHY comment IMMEDIATELY ABOVE the `arc_109_vec_verb_migration_hint` (the first of the family, around line 1695):

```rust
// CONVENTION: migration-hint functions take (callee, expected, got);
// hints that only need `callee` underscore-prefix the others. `collect_hints`
// dispatches uniformly so the trait surface stays consistent across the family.
```

One WHY comment covers all 7 sibling functions.

### Fix 5 — intueri L2 `_head_span` convention WHY (check.rs)

`infer_let` (line 9337), `infer_string_concat` (line 14713), `infer_boolean_shortcircuit` (line 14867) take `_head_span` they don't consume. Per intueri: the parameter is a uniform calling convention of the `infer_*` family.

Action: add a single WHY comment IMMEDIATELY ABOVE the first `infer_*` function declaration (sift through check.rs to find the family-introducing site; if none exists, place at `infer_let`'s docstring as the first instance):

```rust
// CONVENTION: infer_* functions take (..., head_span) for structural
// uniformity; functions that don't emit errors at the head site
// underscore-prefix the parameter (`_head_span`) — the signature stays
// consistent across the family.
```

One WHY comment covers the 3 underscore-prefixed sites.

### Fix 6 — intueri L2 `rename` doc WHY (check.rs:15310-15313)

`rename`'s doc says WHAT (replaces Path occurrences); not WHY (HM-style instantiation to fresh unification variables).

Action: add to `rename`'s doc:
```
/// Called from `instantiate` to convert each rigid type-variable name
/// (`:T`, `:K`, `:V`) into a fresh unification variable for a call site,
/// so independent call sites don't alias.
```

### Fix 7 — solvere L1-1 move `span_prefix` to `src/span.rs` (cross-file)

Currently `span_prefix(span: &Span) -> String` is duplicated in src/types.rs:~1547 AND src/check.rs (the comment at types.rs:1549 confesses: `"Mirrors src/check.rs::span_prefix exactly."`).

Action:
1. Read src/span.rs (the `Span` type's home; leaf module — no import cycle risk).
2. Add `pub fn span_prefix(span: &Span) -> String { ... }` at appropriate location in src/span.rs, preserving the exact body from the current duplicates.
3. Delete the duplicate in src/types.rs (with its "Mirrors" comment).
4. Delete the duplicate in src/check.rs.
5. Update both files to import: `use crate::span::span_prefix;` (or `use super::span::span_prefix;` if path differs).

Verify both Display impls (TypeError + CheckError) now route through the single function.

### Fix 8 — solvere L2-1 inline 5 private `validate_X` wrappers (check.rs)

Five PRIVATE `validate_*` functions are 1-line wrappers over `walk_for_*` siblings:
- `validate_legacy_stream_path` (line ~3444) → `walk_for_legacy_stream`
- `validate_legacy_telemetry_service_path` (~3482) → `walk_for_legacy_telemetry_service`
- `validate_legacy_lru_cache_service_path` (~3520) → `walk_for_legacy_lru_cache_service`
- `validate_legacy_kernel_queue_path` (~3571) → `walk_for_legacy_kernel_queue`
- `validate_bare_legacy_console_path` (~3645) → `walk_for_bare_legacy_console`

Both ends are `fn` (not `pub fn`) — no visibility boundary justifies the split.

Action: for each of the 5 pairs:
1. Find the call site (in `check_program` or wherever) calling `validate_X`.
2. Rewrite the caller to call `walk_for_X` directly (same args).
3. Delete the `validate_X` wrapper function.

**Do NOT touch the 2 `pub fn validate_*`** (`validate_bare_legacy_primitives` ~3014; `validate_arc170_legacy_callsites` ~3043) — those are public-surface; the wrapper is justified.

### Fix 9 — solvere L2-2 extract `register_types_impl<F>` (types.rs:1671-1736)

`register_types` (line 1671-1706) and `register_stdlib_types` (line 1717-1736) replicate the same loop body, differing only in which `env` method gets called (`register_with_span` vs `register_stdlib_with_span`). The `splice_type_decls<F>` extraction (line 1751) already applied this pattern one level down; mirror it one level up.

Action: extract `fn register_types_impl<F>(forms, env, register: &dyn Fn(&mut TypeEnv, TypeDef, Span) -> Result<(), TypeError>) -> Result<(), TypeError>` containing the shared loop body. Both `register_types` and `register_stdlib_types` become thin wrappers passing the appropriate closure. (NB: use `&dyn Fn(...)` directly to also satisfy Fix 11 pattern simultaneously — no `&F` generic indirection.)

### Fix 10 — struere F1 + temperare T-L1-1 `subtype_parents` (types.rs:448)

`subtype_parents(&self, name: &str) -> Vec<&str>` allocates a Vec on every call; callers immediately `.map(|s| s.to_string()).collect()` — pure allocation churn.

Action: change signature to `fn subtype_parents(&self, name: &str) -> Option<&[String]>` returning the borrowed slice from `self.subtype_edges.get(name).map(|v| v.as_slice())`. Update `is_subtype` (lines 3458 + 3468) to iterate the optional slice directly: `if let Some(parents) = self.subtype_parents(name) { for p in parents { ... } }`. The `.to_string()` ownership transfer at downstream push sites can stay where it's needed; the intermediate Vec hop is what gets eliminated.

### Fix 11 — struere F2 `splice_type_decls` drop `&F` indirection (types.rs:1751)

Current signature: `fn splice_type_decls<F>(form, env, register: &F) where F: Fn(...)`. The `&F` reference-to-generic-closure is unidiomatic Rust — `&dyn Fn(...)` or by-value `F: Fn(...)` is the proper shape.

Action: change to `register: &dyn Fn(&mut TypeEnv, TypeDef, Span) -> Result<(), TypeError>` (drop the `<F>` generic). Update the 2 wrappers (`splice_type_decls_user` + `splice_type_decls_stdlib`) to pass `&(|env, def, span| ...)` (the `&` stays; the value-side is the closure).

If you applied this style ALREADY in Fix 9, the patterns are consistent across both extraction layers.

### Fix 12 — purgare L2 `format_type_inner` visibility downgrade (check.rs:15375)

`pub fn format_type_inner` has zero external callers (grep confirms only `check.rs` references it).

Action: change `pub fn format_type_inner` → `fn format_type_inner` (crate-private; same visibility as its only callers).

### Fix 13 — temperare L2-6 `binding_metadata` Arc wrap (check.rs:2038)

`CheckEnv::from_symbols` clones `sym.binding_metadata` (`HashMap<String, HashMap<String, WatAST>>`) at line 2038. The outer HashMap allocation grows with restricted-binding count.

Action: change `CheckEnv` field `binding_metadata` type from `HashMap<...>` to `Arc<HashMap<...>>`. Construction at line 2038 becomes `binding_metadata: Arc::new(sym.binding_metadata.clone())` (still ONE clone, but downstream `CheckEnv` clones share the Arc). Update the `get_binding_metadata` accessor to dereference the Arc transparently.

**If this turns out to require touching many call sites** (more than 5-10 sites updating type signatures), STOP and surface to orchestrator — there may be a simpler approach (e.g., leave clone for now, document its bounded cost).

## Cadence

1. Baseline gate (expect 890/0 lib, 8/0 tests/function, 3/0 probe_arc243_stone3).
2. Apply Fix 1, Fix 2 — cargo test --release --lib (expect 890/0).
3. Apply Fixes 3-6 (intueri renames + WHY comments) — cargo build --release --tests (expect clean).
4. Apply Fix 7 (span_prefix move) — cargo test --release --lib + tests/function (expect 890/0 + 8/0).
5. Apply Fix 8 (inline 5 wrappers) — cargo test --release --lib (expect 890/0).
6. Apply Fix 9 (register_types_impl<F>) — cargo test --release --lib (expect 890/0).
7. Apply Fix 10 (subtype_parents return type) — cargo test --release --lib (expect 890/0); subtype/conforms tests exercise this path.
8. Apply Fix 11 (splice_type_decls drop &F) — cargo test --release --lib (expect 890/0).
9. Apply Fix 12 (format_type_inner visibility) — cargo build --release --tests (expect clean — no external consumers).
10. Apply Fix 13 (binding_metadata Arc) — cargo test --release --lib (expect 890/0). If scope balloons per Fix 13's STOP clause, surface to orchestrator.
11. Final gates: lib ≥ 890; tests/function 8/0; probe_arc243_stone3 3/0; workspace test-build clean; clippy ≤ 897.
12. DO NOT COMMIT — orchestrator commits atomic after debating remainders + R-round convergence + SCORE Phase B.
13. Return paragraph ≤ 200 words: which fixes landed (1-13 confirmed); final gates; any trap-doors encountered + how absorbed; any ADDITIONAL findings surfaced honestly per `feedback_pre_existing_is_not_exemption`.

## STOP triggers (REJECTION)

1. Lib < 890
2. tests/function < 8
3. probe_arc243_stone3 < 3
4. Workspace test-build fails
5. Clippy > 897
6. 60 min elapsed (mechanical scope)
7. holon-rs touched (STOP-5)
8. Scope creep into the 16 DEBATABLE remainders (those are post-debate; do NOT touch them)
9. New deferral language anywhere
10. INTERSTITIAL touched
11. Vigilia or conformare cast attempted by sonnet
12. New runes for the 13 fixes — all are solvable + perf-OK; all FIX
13. Fix 13 (binding_metadata Arc) cascades into many call-site signature changes — STOP per the Fix-13 internal cap; surface to orchestrator
14. Fix 7 (span_prefix move) reveals import-cycle (it shouldn't — span.rs is leaf — but if it does) — STOP, surface verbatim

## Critical doctrine (read before strike)

1. **NO skip-pre-existing** per `feedback_pre_existing_is_not_exemption`
2. **NO deferral language** new
3. **Affirmative-out-of-scope** is acceptable shape
4. **Sonnet writes substrate** per `feedback_sonnet_writes_substrate`
5. **DO NOT commit** — orchestrator commits atomic Stone 243.3 closure after debate
6. **DO NOT cast vigilia or conformare** — orchestrator-cast post-debate
7. **The 16 DEBATABLE remainders are OUT OF SCOPE for this sweep** — do NOT touch them under any circumstance

## Predicted band

**45-75 min Mode A.** 13 mechanical fixes spanning intueri renames + cross-file moves (span_prefix to span.rs) + 5-function inline + 2 closure-pattern extractions + 1 return-type refactor + 1 visibility downgrade + 1 Arc wrap.
