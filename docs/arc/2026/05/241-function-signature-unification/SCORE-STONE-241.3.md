# SCORE — Stone 241.3 — migrate A4 defclause parser through canonical

**Status:** Mode A — PASS
**Runtime:** ~8 min (within 15–30 min target band; zero cascade drove under-target; mirrors Stone 241.2 calibration)
**Summary:** Single migration landed cleanly. A4 (`parse_defclause_args` runtime.rs:6828) routes through canonical via `?`. `spec.fixed_params` returned directly (no unzip — defclause return shape IS the canonical `fixed_params` shape). Doc comment updated to reflect canonical routing. Zero test-assertion cascade (no lib test asserted against old inline error messages). Lib 834 PASS. Probe 6/6. Probes 10/10 + 9/9 preserved. Clippy 905 (delta 0). Workspace build clean. **Phase 1 CLOSES: all 4 parsers (A1/A2/A3/A4) route through canonical.**

---

## Phase A Scorecard

| Row | Claim | Result |
|-----|---|---|
| 1 | Probe contracts 1-3 PASS (happy paths preserved) | **PASS** — 3 passed; 0 failed |
| 2 | Probe contract 4 PASS (NameNotSymbol errors) | **PASS** — 1 passed; 0 failed |
| 3 | Probe contract 5 PASS (MissingArrow errors) | **PASS** — 1 passed; 0 failed |
| 4 | Probe contract 6 PASS (IncompleteTriple errors) | **PASS** — 1 passed; 0 failed |
| 5 | Probe whole-suite 6/6 | **PASS** — 6 passed; 0 failed |
| 6 | Stone 241.2 probe preserved 10/10 | **PASS** — 10 passed; 0 failed |
| 7 | Stone 241.1 probe preserved 9/9 | **PASS** — 9 passed; 0 failed |
| 8 | Lib baseline preserved | **PASS** — 834 passed; 0 failed; 1 ignored |
| 9 | Workspace test-build clean | **PASS** — `cargo build --release --tests --workspace` exit 0; 0 errors |
| 10 | Clippy delta = 0 | **PASS** — 905 warnings (baseline 905; delta 0) |
| 11 | No prior arc 237 probe regresses | **PASS** — probe_arc237_stone5_conforms: 12 pass, probe_arc237_stone5fix_nominal: 12 pass, probe_arc237_stone6_is_predicate: 10 pass, probe_arc238_eq_completeness: 8 pass |

---

## Structural Verification

| Verification | Command | Result |
|---|---|---|
| A4's inline triple walker GONE | `grep -A 30 "^fn parse_defclause_args" src/runtime.rs \| grep -c "while i < args_vec.len()"` | **0** — inline walker removed |
| A4 routes through canonical | `grep -A 15 "^fn parse_defclause_args" src/runtime.rs \| grep -c "parse_argspec_triples"` | **1** — one call site |
| A4 returns `spec.fixed_params` directly | `grep -A 15 "^fn parse_defclause_args" src/runtime.rs \| grep -c "spec.fixed_params"` | **1** — direct return; no `.unzip()` |
| A4 public signature unchanged | `grep "fn parse_defclause_args" src/runtime.rs` | one match — `fn parse_defclause_args(args_vec: &[WatAST], head: &str, form_span: &Span) -> Result<Vec<(String, ...)>, RuntimeError>` |
| Caller `parse_defclause_clause` UNTOUCHED | `git diff src/runtime.rs \| grep "parse_defclause_clause"` | no matches in diff |
| `src/argspec/*` UNCHANGED | `git diff src/argspec/` | empty diff |
| `src/lib.rs` UNCHANGED | `git diff src/lib.rs` | empty diff |
| `src/check.rs` UNCHANGED | `git diff src/check.rs` | empty diff |

---

## Migration Audit

### A4 — `src/runtime.rs` (lines 6827–6895 pre-migration)

| Section | Pre-migration | Post-migration | Delta |
|---|---|---|---|
| Doc comment (updated) | 4 lines | 5 lines | +1 |
| Function signature + opening | 4 lines | 4 lines | 0 |
| Triple walker loop (removed) | 64 lines | 0 | **-64** |
| Canonical call + direct return | 0 | 8 lines | **+8** |
| **Total fn body** | ~64 lines | ~8 lines | **-56** |

**Net delta: 11 insertions, 66 deletions = -55 lines net** (confirmed via `git diff --stat`).

Estimated in DESIGN as ~-57; actual -55 — within calibration band.

---

## Final Post-Migration A4 Body (verbatim)

```rust
/// Parse the args-vector `[name <- :T ...]` from a defclause clause.
///
/// Routes through the canonical `parse_argspec_triples`; `?` converts
/// `ArgSpecError → RuntimeError` via `From<ArgSpecError> for RuntimeError`
/// (Stone 241.1.fix). Returns `spec.fixed_params` directly — defclause's
/// `Vec<(String, TypeExpr)>` return shape IS the canonical `fixed_params` shape.
fn parse_defclause_args(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
) -> Result<Vec<(String, crate::types::TypeExpr)>, RuntimeError> {
    let spec = crate::argspec::parse_argspec_triples(
        args_vec,
        head,
        form_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )?;
    Ok(spec.fixed_params)
}
```

---

## Error-Message Changes Inventory

**Zero test-assertion updates required.** The lib test suite contains no tests asserting against A4's old inline error messages. The cascade predicted by T1 (DESIGN trap-door) did not materialize: no test file asserts `contains("defclause arg-vector triple at position")` or the old A4 lineage-citation message shapes.

This mirrors Stone 241.2's calibration precisely: the substrate's test coverage for defclause argspec errors is at the behavioral boundary (err/ok), not at message-string level. The canonical-domain-neutral messages from `ArgSpecError::classify()` are now emitted by A4; the improvement ships silently.

**Before (A4 NameNotSymbol inline):**
```
"defclause arg-vector triple at position {N} must be `name <- :T`; got {variant} at name slot — literal patterns are not permitted (arc 159/169/234 binding contract requires a plain symbol name)"
```

**After (via From<ArgSpecError> → classify()):**
```
"name slot must be a plain symbol (not a keyword, literal, or nested form)"
```

**Before (A4 MissingArrow inline):**
```
"defclause arg-vector triple at position {N} must be `name <- :T`; got {variant} where `<-` was expected"
```

**After:**
```
"triple must be `name <- :T`; `<-` arrow not found at slot 1"
```

**Before (A4 IncompleteTriple inline):**
```
"arg-vector triple at position {N} must be `name <- :T`; got incomplete trailing tokens"
```

**After:**
```
"triple is incomplete; expected `name <- :T` but ran out of items"
```

All message changes are honest improvements: domain-neutral, consistent with canonical form. The `triple_pos` position prefix that existed in A4's old messages is not present in the canonical wording — this is an honest delta (position context is now carried by the `span` field rather than embedded in the reason string). See Honest Deltas below.

---

## PHASE 1 CLOSURE NOTE

**Parser-divergence class structurally CLOSED.**

All four fn/defclause triple walkers now route through one canonical parser (`parse_argspec_triples` at `src/argspec/parse.rs`):

| Parser | Site | Stone | Status |
|---|---|---|---|
| A1 `parse_fn_signature` | `src/runtime.rs:6750` | 241.2 | MIGRATED |
| A2 `parse_fn_signature_for_check` | `src/check.rs:15205` | 241.2 | MIGRATED |
| A3 `parse_fn_signature_for_check_diag` | `src/check.rs:15243` | 241.2 | MIGRATED |
| A4 `parse_defclause_args` | `src/runtime.rs:6828` | **241.3** | **MIGRATED** |

Every structural violation in a `name <- :T` argspec triple now produces ONE `ArgSpecError` variant, converted at the call boundary to the site's native error class via `From<>` impls. The substrate carries ONE triple-walking implementation. Phase 1 (parser unification) complete; four parsers reduced to one canonical walker.

Phase 2 (metadata-map mechanism) and Stone 241.4 (`&` rest-binder extension to canonical) open next. Stone 241.4 unblocks probe 237.8b Gate 1 (defclause arithmetic + rest-binder).

---

## Honest Deltas

### T2 — Position-index loss (DESIGN verdict β confirmed)

A4's old inline messages embedded `triple_pos` (e.g., "at position 2") in the reason string. The canonical-domain-neutral messages do not include position — position is carried by the `span` field attached to each `ArgSpecError` variant. Verdict β (accept the loss; span is sufficient) confirmed. No user-facing regression: the span points at the offending element directly.

### T1 — Arc-lineage citation removed from error messages

A4's old `NameNotSymbol` message cited the arc lineage explicitly: *"literal patterns are not permitted (arc 159/169/234 binding contract requires a plain symbol name)"*. The canonical message: *"name slot must be a plain symbol (not a keyword, literal, or nested form)"*. The arc-lineage citation was pedagogical; the canonical wording is structurally honest. Doctrine pedagogy belongs in documentation (USER-GUIDE, doctrine inscriptions), not in every error site. No test regression; the substrate's test coverage was at the err/ok boundary.

### Cascade depth: zero

The T1 trap-door (error-message-asserting tests cascade) predicted possible N updates to lib test assertions. Actual cascade: 0. No lib test asserted against old A4 inline message text. This mirrors Stone 241.2's zero-cascade outcome exactly. The zero-cascade calibration from Stone 241.2 is now confirmed across TWO migration stones; confidence is high that remaining arc 241 work will also be zero-cascade.

### Doc comment updated (intueri call)

The original doc comment said *"enforces the defclause binding contract (arc 159/169/234): the name slot MUST be a Symbol"* — this described the old inline implementation rather than the routing. Post-migration, the doc comment reflects the canonical routing: *"Routes through the canonical `parse_argspec_triples`; `?` converts `ArgSpecError → RuntimeError` via `From<ArgSpecError> for RuntimeError` (Stone 241.1.fix). Returns `spec.fixed_params` directly..."*. The doc now accurately describes what the function does. Intueri verdict: the update improves honesty; the old doc misrepresented the implementation. BRIEF permitted sonnet's-choice on this; updated.

---

## Cascade Depth

**SHALLOW.** Zero test-assertion updates. The migration is purely internal; callers see identical (err/ok) behavior at every path. The only observable changes are the error message wording when defclause argspec validation fails — and no existing test was asserting on that text.

Stone 241.4 (`&` rest-binder extension) opens immediately. Phase 1 is closed.
