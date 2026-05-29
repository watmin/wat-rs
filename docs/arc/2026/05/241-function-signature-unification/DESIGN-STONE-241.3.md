# DESIGN — Stone 241.3 — migrate A4 defclause parser through canonical

**Status:** READY (sub-DESIGN). Phase 1 third stone. Single substrate site. Phase 1 CLOSES with this stone.

## Why this stone

Stones 241.1 + 241.1.fix shipped the canonical `parse_argspec_triples` vigilia-CONVERGED; 241.2 migrated A1/A2/A3 (fn-form parsers). The fourth and final triple walker is **A4** `parse_defclause_args` at `src/runtime.rs:6827`:

- Signature: `fn parse_defclause_args(args_vec: &[WatAST], head: &str, form_span: &Span) -> Result<Vec<(String, TypeExpr)>, RuntimeError>`
- 69 lines of inline triple-walking logic
- Same canonical `name <- :T` shape as A1/A2/A3
- NO ret-clause (defclause has no `-> :Ret`; the clause's ret-type lives ELSEWHERE — in a flexible scan after the args-vector)

After this stone, the parser-divergence class CLOSES: all 4 fn/defclause parsers route through one canonical parser; the same structural failure produces ONE error variant (`ArgSpecError`); per-site error conversion happens at the call boundary via `From<>` impls.

## What this stone delivers

ONE migration. ~7 lines replacing ~69 inline. No new types. No new helpers. Public API unchanged. Caller (`parse_defclause_clause` at runtime.rs:6947) untouched.

### M1 — A4 `parse_defclause_args` at `src/runtime.rs:6827`

Replace the entire body of `parse_defclause_args` (lines 6831-6894 — `let mut result = Vec::new();` through `Ok(result)`) with:

```rust
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

The `?` converts `ArgSpecError → RuntimeError` via `From<ArgSpecError> for RuntimeError` (shipped Stone 241.1.fix). `spec.fixed_params: Vec<(String, TypeExpr)>` is RETURNED DIRECTLY — no `.into_iter().unzip()` needed (defclause's return shape IS the canonical's `fixed_params` shape).

## Locked decisions

### D1 — A4 PUBLIC API UNCHANGED

Signature stays `fn parse_defclause_args(args_vec: &[WatAST], head: &str, form_span: &Span) -> Result<Vec<(String, TypeExpr)>, RuntimeError>`. The caller (`parse_defclause_clause`) doesn't know the migration happened.

### D2 — Single-file substrate refactor

Diff confined to `src/runtime.rs` (body of A4 only). NO other files touched (`src/check.rs` not in scope; `src/argspec/*` exceptional and stays). NO new exports. NO new helpers.

### D3 — Error-message regression EXPECTED and HONEST

A4's inline messages carry arc-lineage citations (e.g., "literal patterns are not permitted (arc 159/169/234 binding contract requires a plain symbol name)"). The canonical's domain-neutral messages strip that lineage citation: *"name slot must be a plain symbol (not a keyword, literal, or nested form)"*.

The arc-lineage citation was load-bearing PEDAGOGY in the old A4 body — defclause uses A4 explicitly to enforce the binding contract; the message taught the user "this is by design per arc 159/169/234." The canonical message doesn't carry this teaching.

**Verdict (per Stone 241.1.fix Path Y discipline)**: the canonical message is structurally honest. The pedagogy CAN live elsewhere (USER-GUIDE / doctrine inscription / etc.) but doesn't need to live in every error message at every site. Error reports point at the bug; documentation explains the doctrine.

Tests asserting against the old A4 lineage-citation messages WILL need updating. Per Stone 241.2's observed zero-cascade, this MAY ALSO be zero (the substrate's tests may not assert against message strings). If non-zero: HONEST DELTA per `docs/SUBSTRATE-AS-TEACHER.md` cascade discipline.

### D4 — `head` parameter is variable, not hardcoded

A4 takes `head: &str` as a parameter — defclause callers pass the specific clause head per call site. The canonical parser takes `head: &str` — directly compatible. Forward the parameter unchanged:

```rust
parse_argspec_triples(args_vec, head, form_span, options)  // pass head verbatim
```

NOT hardcoded `":wat::core::defclause"` — defclause's caller decides the head string per clause.

### D5 — `form_span` parameter is already correct

A4 takes `form_span: &Span` from the caller (`parse_defclause_clause` passes `&form_span`). The canonical parser takes `form_span: &Span`. Forward verbatim.

### D6 — `spec.fixed_params` returned DIRECTLY (no unzip)

Unlike A1/A2/A3 which needed `.into_iter().unzip()` for the (Vec<String>, Vec<TypeExpr>) shape, A4 returns `Vec<(String, TypeExpr)>` — IDENTICAL to `spec.fixed_params`. Just `Ok(spec.fixed_params)`. Zero repackaging.

### D7 — `spec.rest_param` is always None (verified)

`ParseOptions { allow_rest_binder: false }` means `parse_argspec_triples` rejects any `&` via `RestBinderNotSupported`. So `spec.rest_param.is_none()` after a successful parse — always. A4 doesn't need to handle Some(rest_param); the field is permanently None at this caller until Stone 241.4+241.5 ship defclause rest-binder opt-in.

### D8 — No new probe-internal types; the canonical surface is complete

No new types, no new variants, no new `ParseOptions` fields. The canonical surface shipped in Stone 241.1.fix is sufficient.

### D9 — Vigilia-gate doctrine does NOT apply

Same as Stone 241.2 D9: `src/runtime.rs` is pre-existing flat substrate, not a namespaced home. Per `feedback_namespaced_home_vigilia_gate` + `feedback_ward_zone_comms_only`: gate doctrine applies to `src/<noun>/` homes only. Stone 241.3 commits on SCORE-green; no vigilia cast.

### D10 — Phase 1 closure inscribed in SCORE

Stone 241.3 closes the parser-divergence class. The SCORE doc inscribes this milestone — Phase 1 (parser unification) complete; Phase 2 (metadata-map mechanism) and Phase 3 (form-collapse + renames) open thereafter. Stone 241.4 (`&` rest-binder extension) lands on the settled canonical API before Phase 1 truly closes (241.4 is still Phase 1 by stone-chain ordering); it unblocks 237.8b.

---

## Trap-door audit

### T1 — A4's pedagogical error messages cite arc lineage

A4's current `NameNotSymbol`-equivalent message says: *"defclause arg-vector triple at position {} must be `name <- :T`; got {} at name slot — literal patterns are not permitted (arc 159/169/234 binding contract requires a plain symbol name)"*.

The arc-lineage citation is GONE post-migration; the canonical message: *"name slot must be a plain symbol (not a keyword, literal, or nested form)"*.

Per D3: the canonical wording is structurally honest; doctrine pedagogy belongs in documentation, not in every error site. Tests asserting on the OLD lineage citation must update or be rewritten to assert structural properties.

### T2 — A4's position-tracking ("triple at position N")

A4's current error messages include a `triple_pos` index (the position of the triple that failed). The canonical parser does NOT track triple position in error variants (each error has a span, but no positional index).

Loss of position is a minor diagnostic regression. Possible additions:
- (α) Add a `triple_pos: usize` field to `ArgSpecError` variants (across-the-board substrate change)
- (β) Accept the loss — the span points at the offending element; the user reads the source at that span
- (γ) Reconstruct position post-conversion via `From<ArgSpecError> for RuntimeError` (no — From impls don't have iteration context)

**Verdict (β)**: span pointer is sufficient; positional index is convenience. Per `feedback_refuse_easy_solutions`: don't ship surface for an audience that hasn't demanded it. If user complains about lost positions, add via follow-up arc.

### T3 — A4's strict NameNotSymbol enforcement matches canonical

A4 explicitly rejects non-Symbol at the name slot per arc 159/169/234 binding contract. The canonical parser ALSO rejects non-Symbol at the name slot. The behaviors align; the migration preserves the enforcement.

### T4 — No new `From<>` impl needed

`From<ArgSpecError> for RuntimeError` shipped in Stone 241.1.fix. A4 uses it via `?`. No additional From impl needed.

### T5 — `parse_defclause_clause` caller is UNTOUCHED

`parse_defclause_clause` at runtime.rs:6947 calls `parse_defclause_args(args_vec, head, &form_span)`. The caller's call site stays identical; the migration is INTERNAL to A4's body.

### T6 — Test cascade may be small or zero (per Stone 241.2 calibration)

Stone 241.2's test-assertion cascade was ZERO. Stone 241.3 may also be zero if defclause tests don't assert against A4's specific lineage messages. Honest-delta documentation in SCORE.

### T7 — No struct/enum changes; type-system unchanged

A4's body is a pure body-replacement. No struct field changes, no enum variant changes, no trait additions. Compiler check is `cargo build`; tests are `cargo test`.

### T8 — Probe shape is behavioral parity (mirror Stone 241.2)

Behavioral-parity probe verifies well-formed defclause forms parse cleanly + malformed forms produce errors (don't silently succeed). Same FM 2-bis discipline as Stone 241.2.

---

## STOP triggers (REJECTION)

1. **STOP-1** — Unexpected compile errors not traced to A4's body
2. **STOP-2** — Lib baseline regression below 834 (after any necessary assertion updates)
3. **STOP-3** — 30 min elapsed (single-site mechanical migration)
4. **STOP-4** — `holon-rs` touched (frozen)
5. **STOP-5** — Files outside `src/runtime.rs`, test files with message-assertion updates (if any), and the SCORE doc touched. `src/argspec/*` MUST stay unchanged; `src/lib.rs` MUST stay unchanged; `src/check.rs` MUST stay unchanged (A2/A3 are 241.2 territory); Stone 241.1 + 241.2 probes MUST stay 9/9 + 10/10 respectively.
6. **STOP-6** — Scope creep:
   - Migrating other parsers — out of scope
   - Adding `triple_pos` index to `ArgSpecError` (T2 verdict β; don't add)
   - Adding NEW types / fields / variants
   - Changing A4's public signature
   - Touching the caller (`parse_defclause_clause`)
7. **STOP-7** — Behavioral-parity probe < N/N PASS
8. **STOP-8** — Stone 241.1 probe regresses (9/9 → less); Stone 241.2 probe regresses (10/10 → less); arc 237 probes regress
9. **STOP-9** — Clippy warnings > 905
10. **STOP-10** — Behavior subtly differs (not just messages — different parsed result for valid inputs)

---

## FM 2-bis evidence

`tests/probe_arc241_stone3_defclause_parser_migration.rs` (NEW). Behavioral-parity probe — passes at HEAD AND post-migration. Same FM 2-bis discipline as Stone 241.2's probe.

Contracts (~6):

| # | Contract | Path |
|---|---|---|
| 1 | defclause empty argspec succeeds | `(defclause :name [] body)` |
| 2 | defclause single-arg succeeds | `(defclause :name [x <- :i64] x)` |
| 3 | defclause multi-arg succeeds | `(defclause :name [x <- :i64 y <- :i64] (+ x y))` |
| 4 | defclause non-Symbol name errors | `(defclause :name [:kw <- :i64] body)` |
| 5 | defclause missing arrow errors | `(defclause :name [x = :i64] body)` |
| 6 | defclause incomplete triple errors | `(defclause :name [x <-] body)` |

Each contract drives the wat source through `startup_from_source` and asserts on the err/ok boundary.

**Pre-stone**: probe passes via existing A4 inline walker.
**Post-stone**: probe passes via canonical-routed A4.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.3.md` (NEW). Mirror Stone 241.2's SCORE shape; smaller. Include:

- Header (Mode A/B; runtime; one-line summary)
- Phase A scorecard ~10 rows
- Structural verification ~5 rows
- Migration audit (A4 line count delta)
- Final post-migration code shape (verbatim)
- Error-message changes inventory (if any; expect zero or near-zero)
- **PHASE 1 CLOSURE NOTE**: all 4 parsers (A1/A2/A3/A4) route through canonical; parser-divergence class is structurally CLOSED
- Honest deltas
- NO Vigilia Convergence section (per D9)

---

## Calibration

**Target band:** 15–30 min Mode A.
**Upper bound:** 30 min (STOP-3).

**Surface estimate:**

| File | Pre-migration | Post-migration | Delta |
|---|---|---|---|
| `src/runtime.rs` (A4 body lines 6831-6894) | ~64 lines | ~7 lines | **-57** |
| `tests/probe_arc241_stone3_defclause_parser_migration.rs` (NEW) | 0 | ~80 lines | **+80** |
| (test files with assertion updates) | N | N | depends; expect 0 per 241.2 calibration |
| **Net delta** | — | — | **~+23 lines + N assertion updates** |

**Confidence: VERY HIGH.** Simplest stone in Phase 1. Single site. Zero new types. Direct return of `spec.fixed_params`. Per Stone 241.2 calibration: cascade likely zero or small.

**Per `feedback_stone_briefs_cite_prior_score`**: BRIEF cites Stone 241.2 SCORE for migration shape; Stone 241.3 is a sub-case (no ret-clause; no unzip).

---

## What this unblocks

Stone 241.4 — extend canonical `parse_argspec_triples` with `&` rest-binder logic. The `allow_rest_binder: true` path becomes wired (no longer `unreachable!` post-Stone 241.1.fix's branching collapse — wait, the unreachable IS gone; 241.4 will just add the rest-binder PARSING when `allow_rest_binder: true`). This unblocks probe 237.8b Gate 1 (defclause arithmetic + rest-binder).

**Phase 1 closure** (parser-divergence class eliminated): four parsers (A1/A2/A3/A4) now route through ONE canonical. The substrate carries ONE triple-walking implementation. Same structural failures produce same `ArgSpecError` variants; per-site error conversion at the call boundary.

---

## Cross-references

- `SCORE-STONE-241.2.md` — migration shape precedent (A1/A2/A3); Stone 241.3 is the sub-pattern
- `SCORE-STONE-241.1.fix.md` § Vigilia Convergence — the structural foundation
- `AUDIT.md` § A4 row + per-site invariants — A4 has `include_ret_type=false` (legacy framing); now means "no ret-clause concerns"
- `DESIGN.md` § Scope expansion 2026-05-28 — arc-level framing
- `feedback_namespaced_home_vigilia_gate` — D9; gate doesn't apply (flat substrate)
- `feedback_refuse_easy_solutions` — T2 verdict β rationale (no position-index field)
- `feedback_substrate_diagnostics_are_brief` + `docs/SUBSTRATE-AS-TEACHER.md` — error-message changes surface as cascade
