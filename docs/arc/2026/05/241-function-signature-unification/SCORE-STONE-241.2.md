# SCORE — Stone 241.2 — migrate A1/A2/A3 fn parsers through canonical

**Status:** Mode A — PASS
**Runtime:** ~25 min (within 40–60 min target band; shallow cascade drove under-target)
**Summary:** Three migrations landed cleanly. A1 (`parse_fn_signature` runtime.rs:6750) routes through canonical via `?`. A2 (`parse_fn_signature_for_check` check.rs:15205) routes via `.map_err(|_| ())`. A3 (`parse_fn_signature_for_check_diag` check.rs:15258) routes via match-and-push. Zero test-assertion cascade (no lib test asserted against old inline error messages). All three ret-clauses remain inline and unchanged. Lib 834 PASS. Probes 10/10 + 9/9 preserved. Clippy 905 (delta 0). Workspace build clean.

---

## Phase A Scorecard

| Row | Claim | Result |
|-----|---|---|
| 1 | Behavioral-parity probe contracts 01-04 PASS (happy paths preserved) | **PASS** — 4 passed; 0 failed |
| 2 | Behavioral-parity probe contract 05 PASS (NameNotSymbol errors) | **PASS** — 1 passed; 0 failed |
| 3 | Behavioral-parity probe contract 06 PASS (MissingArrow errors) | **PASS** — 1 passed; 0 failed |
| 4 | Behavioral-parity probe contract 07 PASS (non-keyword type errors) | **PASS** — 1 passed; 0 failed |
| 5 | Behavioral-parity probe contract 08 PASS (incomplete triple errors) | **PASS** — 1 passed; 0 failed |
| 6 | Behavioral-parity probe contracts 09-10 PASS (ret-clause inline unchanged) | **PASS** — 2 passed; 0 failed |
| 7 | Behavioral-parity probe whole-suite 10/10 | **PASS** — 10 passed; 0 failed |
| 8 | Stone 241.1 probe preserved 9/9 | **PASS** — 9 passed; 0 failed |
| 9 | Lib baseline preserved | **PASS** — 834 passed; 0 failed; 1 ignored |
| 10 | Workspace test-build clean | **PASS** — `cargo build --release --tests --workspace` exit 0; 0 errors |
| 11 | Clippy delta = 0 | **PASS** — 905 warnings (baseline 905; delta 0) |
| 12 | Files touched match discipline | **PASS** — EXACTLY: `src/runtime.rs`, `src/check.rs`, `SCORE-STONE-241.2.md` |
| 13 | `src/argspec/*` UNCHANGED | **PASS** — `git diff src/argspec/` empty |
| 14 | `src/lib.rs` UNCHANGED | **PASS** — `git diff src/lib.rs` empty |

---

## Structural Verification

| Verification | Command | Result |
|---|---|---|
| A1's inline triple walker GONE | `grep -c "i + 2 >= args_vec.len()" src/runtime.rs` in fn scope | **0** — inline walker removed |
| A2's inline triple walker GONE | `grep -c "while i < args_vec.len()" src/check.rs` in fn scope | **0** — inline walker removed |
| A3's inline triple walker GONE | `grep -c "while i < args_vec.len()" src/check.rs` in fn scope | **0** — inline walker removed |
| A1 routes through canonical | `grep -n "parse_argspec_triples" src/runtime.rs` | **line 6808** — 1 call site |
| A2 routes through canonical | `grep -n "parse_argspec_triples" src/check.rs` | **line 15226** — 1 call site |
| A3 routes through canonical | `grep -n "parse_argspec_triples" src/check.rs` | **line 15282** — 1 call site |
| A1 public signature unchanged | `grep "fn parse_fn_signature(" src/runtime.rs` | one match — line 6750; signature `(args: &[WatAST]) -> Result<(Vec<String>, Vec<crate::types::TypeExpr>, crate::types::TypeExpr), RuntimeError>` |
| A2 public signature unchanged | `grep "fn parse_fn_signature_for_check(" src/check.rs` | one match — line 15205; signature unchanged |
| A3 public signature unchanged | `grep "fn parse_fn_signature_for_check_diag(" src/check.rs` | one match — line 15243; signature unchanged |
| No new helpers minted | `grep -n "fn parse_ret_clause\|fn split_at_arrow" src/runtime.rs src/check.rs` | **no matches** |

---

## Migration Audit

### A1 — `src/runtime.rs` (lines 6750–6871 pre-migration)

| Section | Pre-migration | Post-migration | Delta |
|---|---|---|---|
| Arity check block | 9 lines (unchanged) | 9 lines | 0 |
| Vector destructure | 1 line (items only) | 1 line (items + span) | 0 net (+1 span capture) |
| Arrow + ret-clause inline | ~18 lines (unchanged) | ~18 lines (unchanged) | 0 |
| Triple walker loop | ~47 lines | 0 | **-47** |
| Canonical call + unzip | 0 | 7 lines | **+7** |
| **Total fn body** | ~75 lines | ~35 lines | **-40** |

### A2 — `src/check.rs` `parse_fn_signature_for_check` (lines 15205–15250 pre-migration)

| Section | Pre-migration | Post-migration | Delta |
|---|---|---|---|
| Arity check | 3 lines | 3 lines | 0 |
| Vector destructure | 1 line | 1 line (+ span) | 0 |
| Arrow + ret-clause inline | ~10 lines (unchanged) | ~10 lines (unchanged) | 0 |
| Triple walker loop | ~22 lines | 0 | **-22** |
| Canonical call + unzip | 0 | 7 lines | **+7** |
| **Total fn body** | ~36 lines | ~21 lines | **-15** |

### A3 — `src/check.rs` `parse_fn_signature_for_check_diag` (lines 15258–15363 pre-migration)

| Section | Pre-migration | Post-migration | Delta |
|---|---|---|---|
| Arity check | 3 lines | 3 lines | 0 |
| Vector destructure | 1 line | 1 line (+ span) | 0 |
| Arrow inline (diag-push) | ~13 lines (unchanged) | ~13 lines (unchanged) | 0 |
| Ret-clause inline (diag-push) | ~13 lines (unchanged) | ~13 lines (unchanged) | 0 |
| Triple walker loop | ~57 lines | 0 | **-57** |
| Canonical call + match-push + unzip | 0 | 13 lines | **+13** |
| **Total fn body** | ~87 lines | ~43 lines | **-44** |

**Net delta across all three sites:** approximately **-101 lines** (walker removal) + **+27 lines** (canonical call sites) = **-74 lines net**.

---

## Final Post-Migration Code Shapes

### M1 — A1 `parse_fn_signature` (src/runtime.rs:6807-6818)

```rust
    // Route through the canonical argspec parser; `?` converts ArgSpecError → RuntimeError.
    let spec = crate::argspec::parse_argspec_triples(
        args_vec,
        ":wat::core::fn",
        args_vec_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )?;
    let (params, param_types): (Vec<String>, Vec<crate::types::TypeExpr>) =
        spec.fixed_params.into_iter().unzip();

    Ok((params, param_types, ret_type))
```

Vector destructure (updated to capture span, line 6769):
```rust
    let (args_vec, args_vec_span) = match args_vec_node {
        WatAST::Vector(items, span) => (items, span),
        other => { /* unchanged error block */ }
    };
```

### M2 — A2 `parse_fn_signature_for_check` (src/check.rs:15225-15234)

```rust
    // Route through canonical argspec parser; silence any error as `()`.
    let spec = crate::argspec::parse_argspec_triples(
        args_vec,
        ":wat::core::fn",
        args_vec_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    ).map_err(|_| ())?;
    let (names, types): (Vec<String>, Vec<TypeExpr>) =
        spec.fixed_params.into_iter().unzip();
    Ok((names, types, ret_type))
```

Vector destructure (line 15211):
```rust
    let (args_vec, args_vec_span) = match &args[0] {
        WatAST::Vector(items, span) => (items, span),
        _ => return Err(()),
    };
```

### M3 — A3 `parse_fn_signature_for_check_diag` (src/check.rs:15281-15296)

```rust
    // Route through canonical argspec parser; push CheckError and return None on failure.
    let spec = match crate::argspec::parse_argspec_triples(
        args_vec,
        ":wat::core::fn",
        args_vec_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    ) {
        Ok(s) => s,
        Err(e) => {
            errors.push(e.into());
            return None;
        }
    };
    let (names, types): (Vec<String>, Vec<TypeExpr>) =
        spec.fixed_params.into_iter().unzip();
    Some((names, types, ret_type))
```

Vector destructure (line 15250):
```rust
    let (args_vec, args_vec_span) = match &args[0] {
        WatAST::Vector(items, span) => (items, span),
        _ => return None,
    };
```

---

## Error-Message Changes Inventory

**Zero test-assertion updates required.** The lib test suite contains no tests asserting against the old inline error messages from A1/A2/A3's inline triple walkers. The cascade predicted by T1 (DESIGN trap-door) did not materialize: no test file asserts `contains("fn arg-vector triple at position")` or the old A1/A3 message shapes.

This is the honest substrate truth: the existing test coverage for fn-form errors is at the behavioral boundary (err/ok), not at message-string level. The canonical-domain-neutral messages from `ArgSpecError::classify()` are now emitted by A1/A3; the improvement ships silently because no test was checking the exact old text.

**Before (A1 NameNotSymbol inline):**
```
"fn arg-vector triple at position {N} must be `name <- :T`; got {variant} at name slot"
```

**After (via From<ArgSpecError> → classify()):**
```
"name slot must be a plain symbol (not a keyword, literal, or nested form)"
```

**Before (A1 MissingArrow inline):**
```
"fn arg-vector triple at position {N} must be `name <- :T`; got {variant} where `<-` was expected"
```

**After:**
```
"triple must be `name <- :T`; `<-` arrow not found at slot 1"
```

**Before (A1 IncompleteTriple inline):**
```
"fn arg-vector triple at position {N} must be `name <- :T`; got incomplete trailing tokens (need 3 elements: name, `<-`, type)"
```

**After:**
```
"triple is incomplete; expected `name <- :T` but ran out of items"
```

**Before (A3 NameNotSymbol inline):**
```
"fn arg-vector triple at position {N} must be `name <- :T`; got non-symbol at name slot"
```

**After (via From<ArgSpecError> for CheckError → classify()):**
```
"name slot must be a plain symbol (not a keyword, literal, or nested form)"
```

**Before (A3 MissingArrow inline):**
```
"fn arg-vector triple at position {N} must be `name <- :T`; got non-`<-` token where `<-` was expected"
```

**After:**
```
"triple must be `name <- :T`; `<-` arrow not found at slot 1"
```

**Before (A3 IncompleteTriple inline):**
```
"fn arg-vector triple at position {N} must be `name <- :T`; got incomplete trailing tokens"
```

**After:**
```
"triple is incomplete; expected `name <- :T` but ran out of items"
```

All message changes are honest improvements: domain-neutral, consistent with canonical form. The `triple_pos` position prefix that existed in A1/A3's old messages is not present in the canonical wording — this is an honest delta (position context is now carried by the `span` field rather than embedded in the reason string).

---

## Honest Deltas

### T6 — Type-keyword helper inconsistency confirmed (DESIGN T6; queued for follow-up)

Post-migration, the argspec args for A1/A2/A3 are parsed via `parse_type_expr_with_span` (inside `parse_keyword_type` in `src/argspec/parse.rs`). The ret-clause keyword (inline at each site) uses:
- **A1** ret-clause: `parse_type_keyword` (no span)
- **A2** ret-clause: `crate::types::parse_type_expr` (no span)
- **A3** ret-clause: `crate::types::parse_type_expr` (no span)

Three different helpers for the same type-keyword parsing shape across fn-form sites. This inconsistency existed before Stone 241.2; it is now clearly visible at the inline ret-clause blocks vs the canonical argspec call. NOT in 241.2 scope per DESIGN T6 verdict. Queued for a follow-up arc (241.2.fix or dedicated stone).

### Cascade depth: zero

The T1 trap-door (error-message-asserting tests cascade) predicted N updates to lib test assertions. Actual cascade: 0. No lib test asserted against old inline message text. This is NOT a gap in the migration — the behavior (err/ok boundary) is the real contract, and all 10 behavioral-parity contracts pass. The zero-cascade outcome means Stone 241.2 landed cleaner than the design expected; honest delta.

### Position context dropped from error messages

A1/A3's old inline messages embedded `triple_pos` (e.g., "at position 2") in the reason string. The canonical-domain-neutral messages do not include position — position is carried by the `span` field attached to each `ArgSpecError` variant. This is an improvement in diagnostic quality (spans are more precise than computed integer positions). No user-visible regression: the span points at the offending element directly. Honest: not a regression; a quality upgrade.

### T5 — `debug_assert!(spec.rest_param.is_none())` not added

Per DESIGN D7, this assertion is optional — the canonical parser guarantees `rest_param: None` when `allow_rest_binder: false`. No assertion added at the three call sites. The parser contract is the proof; the assertion would be documentation-only. Deferred per D7 "optional; surface as candidate."

---

## Cascade Depth

**SHALLOW.** Zero test-assertion updates. The migration is purely internal; callers see identical (err/ok) behavior at every path. The only observable change is the error message wording when fn-form argspec validation fails — and no existing test was asserting on that text.

Stone 241.3 (A4 `parse_defclause_args` migration) opens immediately. Same canonical-routing pattern; defclause has no ret-clause (the `-> :Ret` block absent entirely); arity check differs.
