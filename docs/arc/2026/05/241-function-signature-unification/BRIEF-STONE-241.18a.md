# BRIEF — Stone 241.18a — Mint `src/function/` namespaced home (Phase A: substrate migration; Phase B orchestrator-cast vigilia)

You are sonnet. **Stone 241.18a — first stepping stone in the Stone 241.18 chain (a-g).** Mints `src/function/` namespaced home for fn-form parsers + eval + infer. Per `feedback_namespaced_home_vigilia_gate` REMARKABLE bar: commit ONLY after orchestrator-cast vigilia drives L1+L2=0 across 8 spells.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized — read these BEFORE strike)

1. **Home name is `src/function/` per intueri verdict** — NOT `src/fn/`. The Rust keyword constraint resolved per intueri: r#fn fails Honest + UX; `function` carries the domain concept without leaking implementation friction. Module declaration: `pub mod function;`. Imports: `use crate::function::*;`.

2. **Phase A vs Phase B separation.** Your job (Phase A) is mint + migrate + caller cascade + SCORE-green. Phase B (vigilia 8-spell convergence to L1+L2=0 REMARKABLE bar) is ORCHESTRATOR-CAST — NOT your responsibility. Sonnet does NOT cast vigilia per Song #44 wisdom (Stone 241.10's vigilia self-report was INFLATED; only orchestrator-cast independent verification gates the commit).

3. **tests/function/ + Cargo.toml [[test]] entry ALREADY ESTABLISHED.** Orchestrator pre-spawn paperwork. `tests/function/mod.rs` + `tests/function/stone18a.rs` + `Cargo.toml` `[[test]]` entry all in place. Verified `cargo test --release --test function` returns 2/2 PASS at HEAD. Do NOT touch tests/function/ in Phase A; behavioral preservation tests must continue to PASS post-migration.

4. **HARD CUT IS TOTAL** (`feedback_hard_cut_admits_no_bypasses`). No backward-compat re-exports from `crate::runtime::eval_fn` / `crate::check::infer_fn`. All callers update to `crate::function::*`.

5. **Sub-stones 241.18b-g OFF-LIMITS.** Do NOT touch `src/def/`, defmacro, defstruct, defenum, defclause, defalias migrations. Those are later sub-stones.

6. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`).

7. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.18a.md` at the end of Phase A.

8. **FM 16 sonnet bash firewall awareness** — simple bash patterns; vanilla cargo/git/grep.

## What to do (Phase A — sonnet)

### S1 — Mint `src/function/` namespaced home

```
src/function/
├── mod.rs       (module surface; re-exports public API)
├── parse.rs     (parse_fn_signature + parse_fn_signature_for_check + _diag)
├── eval.rs      (eval_fn)
└── infer.rs     (infer_fn)
```

Add `pub mod function;` to `src/lib.rs` (alphabetical placement — between `freeze` and `hash` or wherever fits).

### S2 — Migrate fn-form code

**From `src/runtime.rs`:**
- `parse_fn_signature` (line ~6578; consumes args[..3] returning Vec<String> + Vec<TypeExpr> + TypeExpr) → `src/function/parse.rs`
- `eval_fn` (line ~6479) → `src/function/eval.rs`

**From `src/check.rs`:**
- `parse_fn_signature_for_check` (line ~14984; Result wrapper around canonical) → `src/function/parse.rs`
- `parse_fn_signature_for_check_diag` (line ~15022; diagnostic variant) → `src/function/parse.rs`
- `infer_fn` (line ~14868) → `src/function/infer.rs`

### S3 — Public API in `src/function/mod.rs`

```rust
//! Module home for fn-form parsing, evaluation, and inference.
//!
//! Established at Stone 241.18a per feedback_namespaced_home_vigilia_gate.
//! Depends on src/argspec/ for canonical triple parsing.

pub mod parse;
pub mod eval;
pub mod infer;

pub use parse::{parse_fn_signature, parse_fn_signature_for_check, parse_fn_signature_for_check_diag};
pub use eval::eval_fn;
pub use infer::infer_fn;
```

Adjust per sonnet's actual function signatures + whether helper functions are public/private.

### S4 — Cascade caller updates

Update imports at all call sites:
- `src/runtime.rs` callers of parse_fn_signature, eval_fn → `crate::function::*`
- `src/check.rs` callers of parse_fn_signature_for_check, parse_fn_signature_for_check_diag, infer_fn → `crate::function::*`
- Any other site importing these (grep `parse_fn_signature\|eval_fn\|infer_fn` across src/ + tests/)

NO backward-compat re-exports (D4 violation). Update every consumer.

### S5 — Move co-located helpers if exclusively used by fn

When extracting eval_fn / infer_fn / parsers from runtime.rs / check.rs, helper functions called ONLY by fn-form code SHOULD move too (preserve cohesion). Helpers used by OTHER substrate code STAY in original location.

Sonnet audits + judges per helper:
- Helper called ONLY by eval_fn → move to `src/function/eval.rs`
- Helper called by eval_fn AND other code → KEEP in original; import from there

### S6 — Verify tests/function/ still passes 2/2

After all migrations:
```
cargo test --release --test function
```

Both preservation contracts must continue to PASS (behavior didn't break).

### S7 — Author SCORE-STONE-241.18a.md (Phase A portion)

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.18a.md`. Phase A portion includes:
- Header (Mode A; runtime; migration scope; lib + clippy + tests preserved)
- Phase A scorecard (substrate migration verified)
- Migration audit (each function: source location → new location; line counts; co-located helpers decision)
- Caller cascade audit (each updated site)
- Honest deltas (anything surfaced)

NOTE: Phase B (vigilia attestation) SECTION is added by orchestrator post-vigilia-rounds. Sonnet's SCORE doc has a placeholder/footer noting "Phase B vigilia attestation pending orchestrator cast."

## Discipline

- HARD CUT TOTAL — no backward-compat re-exports
- `src/argspec/`, `src/lib.rs` modified only for adding `pub mod function;`
- Stone 241.x + arc 237/238/242 probes preserved
- tests/function/ probes preserved (must continue to pass 2/2)
- holon-rs NEVER touched (STOP-5)
- DO NOT write to INTERSTITIAL
- SCORE doc authored at end of Phase A (Phase B section added by orchestrator)
- Sub-stones 241.18b-g OFF-LIMITS

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.18a.md` — this
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.18a.md` — D1-D8 + T1-T5 + STOP triggers
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.17.md` — most recent stone's calibration
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — vigilia methodology + 6-round remediation pattern (Phase B reference for orchestrator; you don't cast vigilia)
6. `/home/watmin/work/holon/wat-rs/src/argspec/mod.rs` — feel the established namespaced-home convention
7. `/home/watmin/work/holon/wat-rs/src/comms/mod.rs` — another established home (parallel pattern; tests/comms/ also exists)
8. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — find parse_fn_signature (line ~6578), eval_fn (line ~6479)
9. `/home/watmin/work/holon/wat-rs/src/check.rs` — find parse_fn_signature_for_check (line ~14984), _diag (line ~15022), infer_fn (line ~14868)
10. `/home/watmin/work/holon/wat-rs/src/lib.rs` — pub mod declarations (add `pub mod function;`)
11. `/home/watmin/work/holon/wat-rs/tests/function/stone18a.rs` — existing probe contracts (must continue to pass post-migration)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test function 2>&1 | tail -3` (expect 2/2 PASS)
2. **S1:** mint src/function/{mod.rs, parse.rs, eval.rs, infer.rs}; add `pub mod function;` to lib.rs
3. **S2:** move parsers + eval + infer into respective files
4. **S3:** wire public API in mod.rs
5. **S4:** cascade caller updates (use grep to find sites; update each)
6. **S5:** move co-located private helpers as appropriate
7. **Cascade iteration:** `cargo test --release --lib` + `cargo build --release --tests --workspace` after each phase
8. **S6:** verify tests/function/ still 2/2 PASS
9. **Final verification:** lib ≥ 890; workspace test-build clean; clippy ≤ 945
10. **S7:** author SCORE-STONE-241.18a.md (Phase A section)
11. **DO NOT COMMIT.** Orchestrator commits + pushes after Phase B vigilia attestation.

## STOP triggers — REJECTION

1. Compile errors not traced to migration cascade
2. Lib < 890
3. **180 min elapsed for Phase A**
4. holon-rs touched (STOP-5)
5. `src/fn/` named (D1 violation — must be `src/function/`)
6. `r#fn` syntax used anywhere
7. Backward-compat re-exports added in runtime.rs/check.rs (D4 + HARD CUT violation)
8. tests/function/ touched (orchestrator pre-spawn; must continue to pass 2/2)
9. Files outside permitted scope (`src/function/` mint + caller updates in runtime.rs/check.rs/maybe a few others + `src/lib.rs` for pub mod + SCORE doc)
10. Stone 241.x or arc 237/238/242 probes regress
11. Clippy > 945
12. Auto-fixer crate survives commit
13. Sonnet writes to INTERSTITIAL → `feedback_sonnet_never_drafts_interstitial` violation
14. SCORE-STONE-241.18a.md NOT authored at end → `feedback_score_present_check_before_closure` violation
15. Sub-stone 241.18b+ scope touched → D8 violation
16. Sonnet casts vigilia (Phase B is orchestrator-cast; sonnet does NOT)

## Post-strike return (Phase A only)

Return one paragraph: src/function/ minted at <files>; pub mod added to lib.rs; parsers + eval + infer migrated (per-function source location → new location); co-located helpers decision (moved vs kept); caller cascade count + sites updated; tests/function/ preserved 2/2 PASS; lib 890/0; workspace test-build clean; clippy count; SCORE doc Phase A section authored.

Phase B vigilia attestation will be cast by orchestrator post-strike — sonnet's work is complete after Phase A SCORE. Strike clean.
