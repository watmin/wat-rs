# BRIEF — Stone 243.6b — `check_program` walker fusion + `collect_hints` triage

Sub-stone of 243.6 (split at 243.6a). Closes the two `rune:temperare(deferred-stone-243.6)` runes — but they resolve DIFFERENTLY (orchestrator four-questions triage).

## Triage verdict (record in SCORE)

- **Rune A — `collect_hints` fold (`check.rs:611`): LEAVE.** The premise was "double-compute." But it runs only on the **error-render path** (cold — errors are rendered rarely, never in steady-state eval), it's 9 cheap string-match fns, and its call sites (fmt_with_span 228/239 vs diagnostic() 620/631) are **alternative** render methods, not both-per-error. The "fix" (a cached `hints` field on the `CheckError` outer struct) would be a **459-site construction cascade** for a 2-variant cold micro-op — fails Simple for an unproven cold gain (`feedback_let_need_reveal_through_work`). **Resolution:** remove the `rune:temperare(deferred-stone-243.6)` comment at `check.rs:611`; leave `collect_hints` and its 4 call sites **unchanged**; add **no** defending comment (`feedback_dont_document_non_fixes`).
- **Rune B — walker fusion (`check.rs:716`): FIX (this stone).** 9 independent pre-inference per-body validator passes run at compile-time, scaling with program size; sequi confirmed them independent accumulator-drains. Fusing → one traversal is a real (non-cold) perf win **and** a structural cleanup.

## What to do (Rune B — the fusion)

`check_program` (check.rs:703+) runs **9 pre-inference validator passes**, each a pair:
```
for func in sym.functions.values() { <validator>(&func.body, ..., &mut errors); }
for form in forms              { <validator>(form, ..., &mut errors); }
```
at lines ~725/728, 758/761, 772/775, 785/788, 834/837, 847/850, 861/864, 873/876, 889/892. Between them are only **retired-walker comments** (Arc 153/154/155) — no live interleaved logic. All run BEFORE the inference loop at ~915 (`for (name, func) in sym.functions.iter()`).

Fuse them into a SINGLE per-body traversal (preserving each validator call + its exact args):
```
for func in sym.functions.values() {
    validate_comm_positions(&func.body, CommCtx::Forbidden, &mut errors);
    <validator 2>(&func.body, ..., &mut errors);
    ... all 9, in their current relative order ...
}
for form in forms {
    validate_comm_positions(form, CommCtx::Forbidden, &mut errors);
    ... all 9 ...
}
```
Keep the fused pass BEFORE inference. Then remove the `rune:temperare(deferred-stone-243.6)` comment at the fusion site.

## Read in order
1. `src/check.rs:703-915` — `check_program`: the 9 pre-inference passes + the inference boundary (~915). Map each pass's validator fn + args.
2. `src/check.rs:611-645` — `collect_hints` (LEAVE unchanged; only delete the rune comment at 611).

## Discipline
- `src/check.rs` ONLY. **Behavior-preserving** — the fused walker MUST produce identical errors (same validators; sequi confirmed order-independent among the 9). No new Value variant; no holon-rs; no touching inference or the post-inference passes (915, 941, 998).
- Do NOT commit. Leave the tree dirty.

## STOP triggers (rejection, not defer)
1. If any of the 9 passes DEPENDS on a prior pass's mutation (not independent) → STOP, name it (sequi said independent; if the code says otherwise, surface it).
2. If a pass reads inferred types (must run AFTER inference) → STOP; it is NOT in the fusable pre-inference set.
3. If fusing changes error ORDER such that a lib test moves → STOP and surface (errors accumulate in a `Vec`; do not blindly reorder to make a test pass).

## Verify (your own commands)
- `cargo test --release --lib -p wat` → **895 / 0 / 1** (behavioral parity — identical errors).
- `cargo build --release --tests` → clean.
- `grep -n "deferred-stone-243.6" src/check.rs` → **0** (both runes removed; A's code otherwise untouched, B fused).

## SCORE
`SCORE-STONE-243.6b.md`: the triage (A=LEAVE with the four-questions reasoning, B=FIX), the fusion (9→1 pre-inference passes), lib parity, both runes closed. Note A is a *reclassified deferral* (the fold was triaged out, not done).

## Calibration
15–30 min Mode A. Focused behavior-preserving refactor + one rune-comment deletion; lib-parity is the gate. No FM 2-bis probe (no new mechanism/capability).
