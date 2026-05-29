# BRIEF — Stone 241.3 — migrate A4 defclause parser through canonical

You are sonnet (the Shadowdancer). The simplest stone in Phase 1. ONE substrate site. Phase 1 CLOSES with this stone — all 4 parsers route through canonical.

## What to do

Single migration. Replace A4's 69-line inline triple walker with a 7-line canonical call. Return `spec.fixed_params` directly (no unzip).

### M1 — `src/runtime.rs:6827` `parse_defclause_args`

Replace the function BODY (lines 6831-6894 — from `let mut result = Vec::new();` through `Ok(result)`) with:

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

The function SIGNATURE stays unchanged. The doc comment ABOVE the function (lines 6822-6826) can be UPDATED to reflect the canonical routing, OR left alone — sonnet's choice (intueri-call: does the existing doc misrepresent post-migration behavior?).

The `?` operator converts `ArgSpecError → RuntimeError` via `From<ArgSpecError> for RuntimeError` (Stone 241.1.fix). `spec.fixed_params: Vec<(String, TypeExpr)>` is RETURNED DIRECTLY — no `.into_iter().unzip()` needed. Defclause's return shape IS `spec.fixed_params`.

**Key difference from Stone 241.2**: A4's caller already passes `form_span` AND `head` as parameters; we don't need to capture them from the AST. Just forward them. The pattern is simpler than M1/M2/M3 in Stone 241.2.

## Discipline

- **A4 PUBLIC SIGNATURE unchanged.** `fn parse_defclause_args(args_vec: &[WatAST], head: &str, form_span: &Span) -> Result<Vec<(String, TypeExpr)>, RuntimeError>` stays.
- **`head` parameter is variable** — forward verbatim to canonical; DO NOT hardcode `:wat::core::defclause`.
- **`spec.fixed_params` returned directly** — no unzip; no repackaging.
- **`spec.rest_param` is always None** (allow_rest_binder=false). Don't check it; the parser guarantees it.
- **NO new helpers minted.** No parse_ret_clause (not applicable). No split helpers.
- **Caller `parse_defclause_clause` UNTOUCHED.** The caller's call site at runtime.rs:6947 stays identical.
- **`src/argspec/*` UNCHANGED.**
- **`src/lib.rs` UNCHANGED.**
- **`src/check.rs` UNCHANGED.** (A2/A3 were 241.2 territory; don't touch.)
- **Stone 241.1 probe UNCHANGED.** (`tests/probe_arc241_stone1_argspec_canonical.rs` stays 9/9.)
- **Stone 241.2 probe UNCHANGED.** (`tests/probe_arc241_stone2_fn_parser_migration.rs` stays 10/10.)
- **Use `crate::argspec::*`** from inside the crate.
- **No `cargo run`; no wrapper scripts; just `cargo test/build/clippy`.**

## Error-message regression is EXPECTED and HONEST (and likely zero per Stone 241.2 calibration)

After migration, A4's inline messages (including the arc-159/169/234 lineage citation) are REPLACED by canonical-domain-neutral wording from `classify()`. Examples:

- A4 today: `"defclause arg-vector triple at position 0 must be \`name <- :T\`; got Keyword at name slot — literal patterns are not permitted (arc 159/169/234 binding contract requires a plain symbol name)"`
- Canonical post-migration: `"name slot must be a plain symbol (not a keyword, literal, or nested form)"`

The arc-lineage citation was pedagogical inside the error string. The doctrine WILL persist elsewhere (USER-GUIDE, doctrine docs); the error message becomes structurally honest about the failure shape.

**Per Stone 241.2 calibration**: zero test-assertion cascade was observed. Stone 241.3 may also be zero or near-zero. If lib tests assert against A4's specific old strings, UPDATE the assertions to match the canonical messages; treat as HONEST DELTA in SCORE.

If a test fails STRUCTURALLY (variant changed, behavior differs) — STOP.

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md` — FM catalog
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.3.md` — this doc
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.3.md` — D1-D10 + T1-T8 + STOP
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.2.md` — migration shape precedent (A1/A2/A3)
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` § Vigilia Convergence — canonical foundation
6. `/home/watmin/work/holon/wat-rs/src/argspec/parse.rs` — the canonical parser signature
7. `/home/watmin/work/holon/wat-rs/src/argspec/error.rs` — the `From<ArgSpecError>` impls
8. `/home/watmin/work/holon/wat-rs/src/runtime.rs` lines 6820-6900 — A4 current body + signature + caller context
9. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone3_defclause_parser_migration.rs` — 6-contract behavioral-parity probe; passes 6/6 at HEAD
10. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.3.md` — Phase A scorecard

## Implementation sketch

1. Read A4 (runtime.rs:6820-6900) + canonical parser surface
2. Baseline check:
   - `cargo test --release --lib -p wat` (expect 834 PASS)
   - `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` (expect 6 PASS)
3. **M1**: replace A4 body with the 7-line canonical call; update doc comment if intueri suggests
4. Run lib tests; check for assertion failures; update if any (expect zero)
5. Final verification:
   - `cargo test --release --lib -p wat` (≥834 PASS)
   - `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` (6/6)
   - `cargo test --release --test probe_arc241_stone2_fn_parser_migration` (10/10 preserved)
   - `cargo test --release --test probe_arc241_stone1_argspec_canonical` (9/9 preserved)
   - `cargo build --release --tests --workspace` (clean)
   - `cargo clippy --release` (≤ 905)
6. Write `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.3.md`
7. **DO NOT COMMIT.** Orchestrator commits after verification.

## STOP triggers — each is REJECTION

1. **STOP-1** — Unexpected compile errors not traced to A4's body
2. **STOP-2** — Lib baseline regression below 834 (after any assertion updates)
3. **STOP-3** — 30 min elapsed
4. **STOP-4** — `holon-rs` touched (frozen)
5. **STOP-5** — Rust files outside `src/runtime.rs` touched (`src/check.rs`, `src/argspec/*`, `src/lib.rs` MUST stay unchanged); Stone 241.1 + 241.2 probes MUST stay 9/9 + 10/10
6. **STOP-6** — Scope creep: NEW types/fields/variants; A4 signature change; touching the caller `parse_defclause_clause`; adding position-index fields (DESIGN T2 verdict β)
7. **STOP-7** — Stone 241.3 probe < 6/6 PASS
8. **STOP-8** — Stone 241.1 / 241.2 / arc 237 probes regress
9. **STOP-9** — Clippy > 905
10. **STOP-10** — Subtle behavior difference (not just messages); surface as finding

## SCORE doc spec — write `SCORE-STONE-241.3.md`

Mirror SCORE-STONE-241.2.md structural shape; smaller scope. Include:

- Header (Mode A/B; runtime; one-line summary)
- Phase A scorecard ~10 rows
- Structural verification (~5 rows): A4 inline walker gone; canonical routing present; signatures unchanged
- Migration audit (A4 line delta — expect ~-57)
- Final post-migration A4 body (verbatim)
- Error-message changes inventory (zero or N — per Stone 241.2's zero-cascade calibration, likely small)
- **PHASE 1 CLOSURE NOTE**: explicit inscription that all 4 parsers (A1/A2/A3/A4) now route through canonical; parser-divergence class structurally closed
- Honest deltas (T2's position-index loss; arc-lineage citation removed from error messages)
- NO Vigilia Convergence section (per D9)

## Post-strike

When SCORE-STONE-241.3.md is written and verification passes, return with a one-paragraph status summary explicitly noting the Phase 1 closure (all 4 parsers unified through canonical).

Three stones shipped in one session before this. Strike clean. Phase 1 closes here.
