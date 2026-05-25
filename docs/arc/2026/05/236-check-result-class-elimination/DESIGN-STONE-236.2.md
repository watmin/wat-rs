# DESIGN — Stone 236.2 — migrate sibling `infer_*` fn signatures

**Status:** ACTIVE (2026-05-24).

**Scope:** Flip ALL 47 sibling `infer_*` functions in `src/check.rs` from `Option<TypeExpr>` + `&mut Vec<CheckError>` parameter dual-channel to `CheckResult<TypeExpr>` single-channel return. Cascade adapts:

- ~111 internal sibling-call sites (most concentrated in `infer_list` dispatch hub + primary `fn infer()`)
- Primary `fn infer()`'s sibling-calls — currently bridging via `&mut local_errors` (legacy from 236.1) — flip to `.drain_errors_into(&mut local_errors)`
- Sibling↔sibling calls (`infer_list` is the star hub; calls 30+ other siblings)

Substrate empirical disk truth (crawled 2026-05-24 pre-DESIGN):
- **48 fn defs** at `^fn infer_*` or `^fn infer(`: 1 primary (236.1-shipped) + 47 siblings
- **111 sibling-call sites** distributed across the file
- **Topology:** STAR pattern — `infer_list` calls ~30+ siblings (dispatch hub); other siblings rarely call each other
- **Body size:** 2 hubs large (`infer_list`, `infer_match`); remaining 45 small-to-medium

---

## Locked decisions

### D1 — Signature flip: uniform across ALL 47 siblings

```rust
// BEFORE (all 47 siblings, shape uniform):
fn infer_<verb>(
    ast: &WatAST,                     // or &[WatAST] depending on fn
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr>

// AFTER:
fn infer_<verb>(
    ast: &WatAST,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr>
```

The `errors` param is dropped from EVERY sibling. No exceptions. Some siblings may have additional parameters (e.g., `infer_list` takes `head_ident`, `args`); preserve those, only drop the trailing `errors`.

### D2 — Internal body: local_errors pattern (same as 236.1 primary)

At the top of each new sibling body:

```rust
let mut local_errors: Vec<CheckError> = Vec::new();
```

Every existing `errors.push(...)` site INSIDE the sibling body becomes `local_errors.push(...)`. At each return point, package per the constructor table:

| Old pattern | New pattern |
|---|---|
| `Some(ty)` (no errors pushed in this branch) | `CheckResult::ok(ty)` |
| `Some(ty)` (after some `errors.push(...)`) | `CheckResult::partial_with(ty, local_errors)` if local_errors non-empty, else `CheckResult::ok(ty)` |
| `return None` (after error pushed) | `CheckResult::errs(local_errors)` |
| `return None` (NO error pushed) | **HARVEST POINT** — per D3 |

### D3 — The HARVEST is per-fn explicit

Every `return None` in every sibling body must be reviewed + classified. Three honest classifications (same shape as 236.1):

1. **Silent ON PURPOSE** — `CheckResult::ok(fresh.fresh())` (polymorphic placeholder); inline comment documents why
2. **Error case missing diagnostic** — identify the silent failure; add the appropriate CheckError::* push and convert to `CheckResult::errs(local_errors)`
3. **Error case already had diagnostic** — straight `CheckResult::errs(local_errors)` conversion

Each None-replacement gets a brief comment naming which classification was made:

```rust
// HARVEST (236.2): no diagnostic on this path; specific error added.
local_errors.push(CheckError::SomeVariant { ... });
return CheckResult::errs(local_errors);

// HARVEST (236.2): silent-by-intent — polymorphic placeholder.
return CheckResult::ok(fresh.fresh());

// HARVEST (236.2): existing diagnostic; straight conversion.
return CheckResult::errs(local_errors);
```

Aggregate count per classification (across all 47 siblings) is the failure-class harvest data for arc 236.

**Pre-warning from Stone 236.1's SCORE:** The Symbol-arm + sibling-delegation Classification 1 sites that 236.1 noted as "the silent failures live in the sibling functions — 236.2's territory" surface HERE. Expect Classification 2 count > 0 (likely several silent failures get diagnostics minted).

### D4 — Primary's calls to siblings flip to the bridge

The primary `fn infer()` (already CheckResult-returning per 236.1) currently calls siblings using the legacy `&mut local_errors` bridge:

```rust
// CURRENT (236.1 state):
let result = infer_list(args, env, locals, fresh, subst, &mut local_errors);
// `result` is Option<TypeExpr>; legacy
```

After 236.2 flips siblings:

```rust
// AFTER (236.2):
let result = infer_list(args, env, locals, fresh, subst).drain_errors_into(&mut local_errors);
// `result` is Option<TypeExpr>; same shape; bridge handles error draining
```

Behavior preserved; only the call shape changes.

### D5 — Sibling↔sibling calls use the same bridge

Within `infer_list`'s body (the star hub), calls to OTHER siblings now CheckResult-returning:

```rust
// BEFORE (inside infer_list body):
let ty = infer_let(args, env, locals, fresh, subst, errors)?;

// AFTER:
let ty = infer_let(args, env, locals, fresh, subst).drain_errors_into(&mut local_errors)?;
```

The `local_errors` is the calling sibling's own local Vec (from D2). The bridge propagates errors uniformly.

### D6 — No new probe; existing scorecard regression-guards suffice

Stone 236.0's probe (CheckResult contract) + Stone 236.1's scorecard rows (arc 234 + 232 + 233 regression guards + lib baseline) form the load-bearing test surface. No new probe file for 236.2.

The lib baseline (827) IS the audit: harvest classifications that surface silent failures (D3 Classification 2) may produce 1-5 lib-test changes if a test was relying on silent failure. Per FM 11 + arc-236 doctrine: those are EXPECTED behavior shifts, surface in SCORE; orchestrator reviews per case.

### D7 — Clippy stays at 54

No new warnings. Migration pattern is uniform; bridge calls don't introduce new lints.

### D8 — HARD CUT: no transitional dual-channel

NO `infer_<verb>_v2(...)` alongside `infer_<verb>(...)` for incremental migration. All 47 siblings flip in this stone. The `drain_errors_into` bridge IS the migration helper; no parallel function shims.

Same prohibition as 236.1 D8.

### D9 — Internal harvest tracking (inline comments per D3)

Each `return None` replacement gets the brief inline comment. Sonnet's SCORE captures the aggregate count per classification (1/2/3 across all 47 siblings).

### D10 — Single-stone shape (not split)

Considered: split into 236.2.a (first batch, ~15 fns) + 236.2.b (`infer_list` + `infer_match` hubs) + 236.2.c (remaining fns).

**Decision: single stone.** Rationale:
- Pattern proven (236.1 shipped clean; cascade depth 2 vs predicted 3-5; 25 min vs 60-90 band)
- Bridge tool exists (`drain_errors_into` from 236.0)
- Sibling↔sibling calls (concentrated in `infer_list`) cleanest when all flip together; split would leave mixed-mode within `infer_list` body across stones
- HARVEST per-fn is batchable; sonnet classifies per-fn; orchestrator scores aggregate
- Cargo cascade enumerates the work; substrate-as-teacher discipline applies
- **Fallback:** if sonnet hits STOP-3, partial-state-grading (`feedback_partial_state_grading`) lets us slice retroactively — clean fns stay committed; remaining fns become 236.2.continuation

If STOP-3 fires: NOT a failure of the single-stone shape; just a calibration overrun. The work stays clean; the split happens after the fact.

---

## Trap-door audit

### T1 — `let _ = sibling(...)` discard sites

Pattern: caller doesn't care about return type; only side-effects (errors pushed). After migration: `let _ = sibling(...).drain_errors_into(errors);` — must drain to preserve error-propagation behavior. Sonnet doesn't drop the `.drain_errors_into(errors)` even though the value is discarded.

Stone 236.1's SCORE noted "39 sites" of this pattern; expect similar density in sibling bodies.

### T2 — `for arg in args { sibling(...); }` iteration patterns

Some sites iterate over args calling siblings for validity-side-effect. Same as T1; each call inside the loop drains errors.

### T3 — Self-recursive siblings

`infer_list` recurses (it's the dispatch hub for List ASTs which can nest). Some siblings may self-recurse. Same bridge pattern applies: `infer_self(sub_ast, ...).drain_errors_into(&mut local_errors)?;`.

### T4 — Constructor `.unwrap_or_else` patterns

Stone 236.1's SCORE flagged `infer_some_constructor`, `infer_ok_constructor`, `infer_err_constructor` as having `.unwrap_or_else` call chains. Those used `infer(...)?` shortcircuit inside; now those siblings' OWN signatures flip. Each constructor's body needs the same body-level pattern (local_errors + CheckResult returns).

### T5 — `infer_list` is the wide cascade

`infer_list` (1273 lines per awk count; likely 200-500 actual body lines + nested arms) calls 30+ other siblings inside its dispatch arms. When `infer_list`'s body translates: every internal sibling-call gets the bridge. Expect this fn to dominate the diff.

### T6 — Sibling param signature sanity check

Sonnet sanity-checks each sibling for the expected `errors: &mut Vec<CheckError>` parameter BEFORE flipping. If any sibling has a different signature shape (no errors param; different param order; trait impl), flag in SCORE as honest delta — don't force a uniform flip on a non-uniform target.

### T7 — `apply_subst(&ty, subst)` independent

After draining, callers continue to use `apply_subst(&ty, subst)` as before. Subst is independent of the error channel; no migration concern.

### T8 — Behavior preservation principle

The migration MUST preserve current behavior modulo HARVEST diagnostic additions. Every test that passes today passes tomorrow UNLESS a test was relying on silent failure (D3 Classification 2). Substrate-as-teacher catches regressions in real-time.

### T9 — debug_assert in CheckResult::errs / partial_with

If a HARVEST classification accidentally produces `CheckResult::errs(vec![])` (empty), debug build panics. Per Stone 236.0 D3 invariant. Tests catch in debug; sonnet adjusts.

### T10 — Symbol arm in sibling bodies

Sibling bodies may have their own Symbol-arm Classification 1 sites (looking up symbols in `locals` may return None silently). Same treatment as 236.1 primary's Symbol arm: `CheckResult::ok(fresh.fresh())` polymorphic placeholder.

### T11 — `infer_list`'s sibling-delegation None paths

Stone 236.1 SCORE noted: *"infer_list / infer_list_constructor returning None with no errors. Primary cannot distinguish. Translated as Classification 1 fresh-var. **Arc 236.2 will flip sibling signatures and eliminate those silent paths at their source.**"*

→ When `infer_list` flips: every internal arm that returned `None` silently is a HARVEST point. Many will be Classification 2 (missing diagnostics — silent failures finally getting their voice). Expect this fn to dominate Classification 2 count.

### T12 — Primary-side bridge update (the post-flip cleanup)

After siblings flip, the primary's calls to siblings change from legacy `&mut local_errors` to `.drain_errors_into(&mut local_errors)`. Stone 236.1 left those legacy-bridged sites with inline comments naming the 236.2 dependency. Sonnet removes those comments as part of the flip (no longer load-bearing).

---

## STOP triggers

- STOP-1 unexpected compile errors not tracing to signature flip / cascade
- STOP-2 lib baseline regresses below 827 substantively (1-5 expected from harvest is OK; > 5 OR not-tracing-to-harvest is REJECTION)
- STOP-3 **180 min elapsed** (Mode A target is 90 min; STOP-3 is 2× upper-bound)
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside src/check.rs
- STOP-6 primary `fn infer()` signature changed (already done by 236.1; out of scope for 236.2)
- STOP-7 any arc 234 regression
- STOP-8 clippy > 54
- STOP-9 transitional dual-channel shim minted (D8 forbids)

Each STOP REJECTION.

**Special handling — lib baseline:**

If lib baseline drops by 1-5 tests AND the drops trace to HARVEST classifications that surface previously-silent failures (D3 case 2), that's EXPECTED behavior change — surface them in SCORE; orchestrator decides per case. Not auto-STOP-2.

If lib baseline drops > 5 tests OR drops trace to migration errors: STOP-2.

The threshold is wider than 236.1's (1-2) because Stone 236.1 SCORE explicitly noted "the silent failures live in the sibling functions — 236.2's territory." Expect higher harvest yield.

---

## Calibration

**Target:** 90 min Mode A. **Upper:** 180 min (STOP-3).

Surface:
- 47 sibling fn body translations (mostly small; 2 hubs medium-large)
- ~111 sibling-call site bridges (most concentrated in `infer_list` and primary)
- Cascade depth: 3-5 compile rounds expected
- HARVEST: aggregate count across 47 fns; Classification 2 likely > 0 (236.1 SCORE foreshadowed this)
- 0-5 new CheckError variants if harvest surfaces previously-silent failure semantics

Confidence: MEDIUM. Pattern proven by 236.1; mostly mechanical. The hot spots:
- `infer_list` (~1273 line body; many sibling-call sites; HARVEST density)
- `infer_match` (~1279 line body; pattern-matching arms)
- HARVEST decisions across 47 fns (sonnet flags edge cases; orchestrator reviews aggregate)

Pre-emption: Stone 236.1 + 236.0 documents the pattern verbatim in src/check.rs:1040-1206 (the CheckResult migration-pattern docstring uses `infer_legacy` / `infer_new` / `infer_something` / `infer_something_inner` as worked examples). Sonnet mirrors those examples directly.

---

## What this unblocks

Stone 236.3 — failure-class harvest audit + remediation of any silent-failure sites surfaced (likely few; 236.2's HARVEST is the audit-in-flight).

Stone 236.4 — verification + clippy + final regression sweep.

Stone 236.5 — INSCRIPTION + arc 236 close.

When 236.2 + 236.3 + 236.4 + 236.5 ship, silent-error-loss is structurally eliminated across check.rs. Arc 234 resumes (per spawn-block winding).

---

## Cross-references

- `src/check.rs` — single file under treatment
- `src/check.rs` line ~996 area — CheckResult<T> + drain_errors_into helper (Stone 236.0; the migration bridge)
- `src/check.rs` line 1040-1206 — CheckResult migration-pattern docstring (`infer_legacy` / `infer_new` / `infer_something` / `infer_something_inner` worked examples; sonnet mirrors)
- `src/check.rs` line 4868 — primary `fn infer()` definition (236.1-shipped)
- `src/check.rs` line 5056-13164 — sibling `infer_*` defs (47 of them; the target surface)
- `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.1.md` — pattern proof + HARVEST methodology lineage
- `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` — bridge-tool proof
- `docs/arc/2026/05/236-check-result-class-elimination/DESIGN.md` — arc umbrella
- `feedback_no_known_defect_left_unfixed` — discipline driving arc 236
- `feedback_partial_state_grading` — STOP-3 fallback discipline
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites 236.1 SCORE for sonnet to mirror
- `project_failure_engineering` — the pattern arc 236 embodies
