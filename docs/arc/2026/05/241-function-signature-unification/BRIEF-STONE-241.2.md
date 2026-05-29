# BRIEF — Stone 241.2 — migrate A1/A2/A3 fn parsers through canonical

You are sonnet (the Shadowdancer). You strike on a three-site internal migration. The canonical `parse_argspec_triples` at `src/argspec/` shipped vigilia-CONVERGED in Stone 241.1.fix; it is exceptional. Now route A1/A2/A3's inline triple walkers through it.

## What to do

Three migrations (M1/M2/M3). Same shape, different error semantics per site. NO public API changes; A1/A2/A3 keep their existing signatures.

### M1 — `src/runtime.rs:6750` `parse_fn_signature` (A1; RuntimeError path)

Replace the inline triple walker (the `while i < args_vec.len()` loop and its body, roughly lines 6807-6850) with:

```rust
let spec = wat::argspec::parse_argspec_triples(
    args_vec,
    ":wat::core::fn",
    args_vec_span,                                         // the Vector's own span
    wat::argspec::ParseOptions { allow_rest_binder: false },
)?;
let (params, param_types): (Vec<String>, Vec<crate::types::TypeExpr>) =
    spec.fixed_params.into_iter().unzip();
```

(Use `crate::argspec::...` if the module is `pub(crate)` not `pub` — sonnet decides based on what's in `src/lib.rs`. Hint: `pub mod argspec;` at line 62 per Stone 241.1's commit. Use `crate::argspec::*`.)

The `?` operator converts `ArgSpecError` → `RuntimeError` via the `From<ArgSpecError> for RuntimeError` impl shipped in Stone 241.1.fix.

**Critical**: capture the args-vector's span when destructuring. Today's A1 does:
```rust
let args_vec = match args_vec_node {
    WatAST::Vector(items, _) => items,
    other => { return Err(...); }
};
```

You need to widen this to capture the span:
```rust
let (args_vec, args_vec_span) = match args_vec_node {
    WatAST::Vector(items, span) => (items, span),
    other => { return Err(...); }
};
```

Then pass `args_vec_span` as the `form_span: &Span` parameter to `parse_argspec_triples`.

**Ret-clause stays inline** — the existing `match arrow_node` block + `match ret_type_node` block at lines 6783-6805 are UNCHANGED. The canonical parser handles the argspec; the fn-form parser handles the ret-clause inline.

### M2 — `src/check.rs:15205` `parse_fn_signature_for_check` (A2; silent `()` path)

Same shape; silence via `.map_err(|_| ())`:

```rust
let spec = crate::argspec::parse_argspec_triples(
    args_vec,
    ":wat::core::fn",
    args_vec_span,
    crate::argspec::ParseOptions { allow_rest_binder: false },
).map_err(|_| ())?;
let (names, types): (Vec<String>, Vec<crate::types::TypeExpr>) =
    spec.fixed_params.into_iter().unzip();
```

Replace the existing `while i < args_vec.len()` loop (roughly lines 15226-15248) with this.

### M3 — `src/check.rs:15258` `parse_fn_signature_for_check_diag` (A3; diagnostic-push path)

A3 pushes errors into `&mut Vec<CheckError>` and returns `None`. Use a match to push-and-return:

```rust
let spec = match crate::argspec::parse_argspec_triples(
    args_vec,
    ":wat::core::fn",
    args_vec_span,
    crate::argspec::ParseOptions { allow_rest_binder: false },
) {
    Ok(s) => s,
    Err(e) => {
        errors.push(e.into());                             // From<ArgSpecError> for CheckError
        return None;
    }
};
let (names, types): (Vec<String>, Vec<crate::types::TypeExpr>) =
    spec.fixed_params.into_iter().unzip();
```

Replace the existing `while i < args_vec.len()` loop (roughly lines 15299-15330) with this.

## Discipline

- **A1/A2/A3 PUBLIC SIGNATURES unchanged.** Return types stay (Vec<String>, Vec<TypeExpr>, TypeExpr) tuple wrapped in Result/Option as before.
- **Ret-clause stays inline** at all three sites. DO NOT extract a shared `parse_ret_clause` helper; per-site error semantics differ; sharing forces a shim.
- **Form-span sourcing**: capture the args-vector's span when destructuring the WatAST::Vector at args[0]. Pass it as `form_span` to `parse_argspec_triples`.
- **`spec.rest_param` is always None** post-canonical-call. Don't add explicit handling.
- **`spec` is consumed via `.into_iter().unzip()`** — type-annotate `(Vec<String>, Vec<TypeExpr>)` to prevent inference ambiguity.
- **`src/argspec/*` UNCHANGED.** The canonical home is exceptional; don't touch it.
- **`src/lib.rs` UNCHANGED.** The `pub mod argspec;` line stays.
- **`tests/probe_arc241_stone1_argspec_canonical.rs` UNCHANGED.** Stone 241.1's probe stays at 9/9.
- **Use the `wat::argspec::*` re-exports** if calling from outside the crate; use `crate::argspec::*` from inside (you're inside; `crate::` is correct).
- **No `cargo run`; no wrapper scripts; just `cargo test/build/clippy`.**

## Error-message regression is EXPECTED and HONEST

After migration, the `?` and `e.into()` conversions produce error messages from Stone 241.1.fix's `classify()` method — domain-neutral wording: *"name slot must be a plain symbol (not a keyword, literal, or nested form)"*, *"triple must be `name <- :T`; `<-` arrow not found at slot 1"*, etc.

These DIFFER from A1's current inline messages. Tests asserting against the OLD message strings will fail. Per `docs/SUBSTRATE-AS-TEACHER.md` + `feedback_substrate_diagnostics_are_brief`: the substrate's diagnostic cascade IS the migration brief.

**Your job when a test fails with an error-message-mismatch**: read the test, see what message it asserts, update the assertion to match the canonical-domain-neutral message that A1/A3 now emits. Treat each as an HONEST DELTA in SCORE, not a regression.

If a test asserts on something STRUCTURAL (not message text — e.g., variant matching, span position, exit code) and BREAKS: STOP. Surface as finding; that's a real regression.

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md` — FM catalog (FM 2-bis, FM 5, FM 11, FM 16)
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.2.md` — this doc (the strike path)
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.2.md` — locked decisions D1-D10 + trap-door T1-T10 + STOP triggers
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` § Vigilia Convergence — the structural foundation (canonical parser shape; `From<ArgSpecError>` impls)
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/AUDIT.md` — per-site invariants (Section "Per-site invariants table")
6. `/home/watmin/work/holon/wat-rs/src/argspec/parse.rs` — the canonical parser signature you're routing through
7. `/home/watmin/work/holon/wat-rs/src/argspec/error.rs` — the `From<ArgSpecError>` impls you're triggering via `?` and `.into()`
8. `/home/watmin/work/holon/wat-rs/src/runtime.rs` lines 6750-6870 — A1 current body
9. `/home/watmin/work/holon/wat-rs/src/check.rs` lines 15205-15330 — A2 + A3 current bodies
10. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone2_fn_parser_migration.rs` — the behavioral-parity probe (10 contracts; passes at HEAD; must STAY at 10 PASS post-migration)
11. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.2.md` — what completion looks like (Phase A scorecard)

## Implementation sketch (order of operations)

1. Read the substrate sites (runtime.rs:6750-6870; check.rs:15205-15330) + canonical parser surface
2. Baseline check: `cargo test --release --lib -p wat` (expect 834 PASS); `cargo test --release --test probe_arc241_stone2_fn_parser_migration` (expect 10 PASS)
3. **M1 — runtime.rs**: capture args-vector span; replace triple walker with `parse_argspec_triples` call; unzip into `(params, param_types)`
4. Run lib tests; identify failing tests with message-string assertions; update assertions; re-run; iterate
5. **M2 — check.rs (silent)**: same pattern, `.map_err(|_| ())`
6. **M3 — check.rs (diag)**: match-and-push pattern
7. Run lib tests AGAIN; identify failing tests; update; iterate until clean
8. Final verification:
   - `cargo test --release --lib -p wat` (≥834 PASS)
   - `cargo test --release --test probe_arc241_stone2_fn_parser_migration` (10 PASS — preserved)
   - `cargo test --release --test probe_arc241_stone1_argspec_canonical` (9 PASS — preserved)
   - `cargo build --release --tests --workspace` (clean)
   - `cargo clippy --release` (≤ 905 warnings)
9. Write `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.2.md`
10. **DO NOT COMMIT.** Orchestrator commits after SCORE-green verification.

## STOP triggers — each is REJECTION (ship NOTHING; surface as finding)

1. **STOP-1** — Unexpected compile errors NOT traced to the migration call sites
2. **STOP-2** — Lib baseline regression below 834 PASS (after test-assertion updates for message changes — the 834 baseline is post-update; updated assertions are part of the work)
3. **STOP-3** — 90 min elapsed
4. **STOP-4** — `holon-rs` touched (frozen)
5. **STOP-5** — Rust files outside `src/runtime.rs`, `src/check.rs`, and existing test files touched. `src/argspec/*` MUST stay unchanged. `src/lib.rs` MUST stay unchanged. `tests/probe_arc241_stone1_argspec_canonical.rs` MUST stay 9 PASS unchanged.
6. **STOP-6** — Scope creep:
   - Migrating A4 (defclause) — that is Stone 241.3
   - Minting `parse_ret_clause` helper — out of scope per BRIEF discipline
   - Changing A1/A2/A3 public signatures — D1 violation
   - Unifying type-keyword helpers (parse_type_keyword vs parse_type_expr vs parse_type_expr_with_span) — T6 finding; queue for follow-up
   - Adding new types / fields / variants anywhere
7. **STOP-7** — Behavioral-parity probe regresses (< 10/10 PASS)
8. **STOP-8** — Any prior arc 237 probe regresses
9. **STOP-9** — Clippy warnings increase above 905
10. **STOP-10** — Migration produces SUBTLY different behavior (not just different error messages — different parsed result for valid inputs OR a previously-valid input now errors OR a previously-invalid input now succeeds). Surface as finding.

## SCORE doc spec — write `SCORE-STONE-241.2.md`

Mirror `SCORE-STONE-241.1.fix.md`'s structural shape:

- **Header**: status (Mode A/B); runtime; one-line summary
- **Phase A scorecard** ~12-15 rows: lib baseline, probe stone2 10/10, probe stone1 9/9 preserved, workspace test-build, clippy delta, file discipline, arc 237 regression
- **Migration audit** — per-site (A1/A2/A3) line counts before/after; identified deltas
- **Final post-migration code shapes** (verbatim) at each of M1/M2/M3 sites
- **Error-message changes inventory** — every test assertion updated; before/after string per case
- **Honest deltas** — T6 type-keyword-helper inconsistency surfaced; any unexpected substrate friction
- **NO Vigilia Convergence section** — Stone 241.2 commits on SCORE-green per D9 of DESIGN (flat-file substrate; vigilia-gate doctrine doesn't apply here)
- **Cascade depth** — should be SHALLOW (the migration is internal); document any deep cascades as honest deltas

## Post-strike

When SCORE-STONE-241.2.md is written and verification passes, return with a one-paragraph status summary. Orchestrator will:
1. Independently verify the scorecard
2. Read the Honest Deltas; ensure they're framed as forward-progress, not deferral
3. Commit atomically; push
4. Open Stone 241.3 (A4 defclause migration)

Stone 241.1.fix proved vigilia + dungeon-crawl work; Stone 241.2 proves migration after foundation is exceptional. Strike clean.
