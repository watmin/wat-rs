# EXPECTATIONS — Arc 233 Stone 233.2.j — migrate 5 producers + eval_inner TrackedValue cascade

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **233.2.j probe FLIPS 2/5 → 5/5** | `cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Stone 233.2.i eval signature probe still passes | `cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 5 | Stone 233.2.h TrackedValue mint probe still passes | `cargo test --release --test probe_tracked_value_mint_contract 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | Stone 233.2.d substrate-symmetry probe still passes | `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 \| tail -3` | `1 passed; 0 failed` |
| 7 | Stone 233.1 ValueSnapshot probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 8 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 9 | Stone 232.0 dynamic-keyword probes still pass | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 90–150 min Mode A
**Upper bound:** 240 min (STOP-3) — per Stone 233.2.j sub-DESIGN
**Confidence:** medium-low — substrate-as-teacher iteration shape proven (arc 163 slice 3e + Stone 233.2.d + 233.2.i precedents); call-site volume is 3.6× the 233.2.i cascade; producer migration adds shape decisions for the recv/try-recv special case

**Rationale:**
- eval_inner signature flip + ~30 leaf-arm `.into_tracked()` wraps: ~10 min
- Cascade through 383 internal eval_inner call sites in runtime.rs: ~50-80 min mechanical sweep (substrate-as-teacher)
- 5 producer constructor swaps (~18 wrap sites — keyword/from-string + 14 from-holon arms + edn::read + recv/try-recv special case): ~15-20 min
- eval boundary simplification: ~3 min
- ValueSnapshot::of_tracked addition: ~5 min
- Verification cascade + SCORE writing: ~15 min

**Risks:**
- Volume risk: 383 eval_inner call sites is 3.6× the 233.2.i cascade
- Borrow checker friction on `.value()` vs `.value_owned()` choices (sonnet picks per call site)
- recv/try-recv special case requires explicit honest-delta documentation (provenance dropped; arc 233.2.e revisits)
- Helper functions called from eval_inner that take `Value` continue working (they receive the extracted value via `.value_owned()`); but if helper takes `&Value` borrow, may need adjustment
- If cascade exceeds 240 min: STOP-3 fires; orchestrator decides sub-slice or extend

## Honest delta — recv/try-recv provenance regression

**Planned and documented.** The Value::Tracked wrap at `runtime.rs:19788` + `19865` is inside `Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(tagged))))))`. The `tagged` slot is structurally `Value`-typed (it's inside Option<T>); converting to TrackedValue would require flipping Option's inner type, which is out of scope.

This stone REMOVES the wrap entirely at recv/try-recv (the `tagged` becomes bare `v`). Producer provenance is lost at these two sites. **Arc 233 Stone 233.2.e revisits via AST-derived provenance mechanism** that doesn't depend on Value-side carriers.

SCORE must explicitly document this regression and the recovery plan.

## Out-of-scope rows (REJECTED)

- Value::Tracked variant retirement (Stone 233.2.k)
- #[wat_value] proc-macro structural seal (Stone 233.2.l)
- AST-derived provenance for let-bindings + literals (Stone 233.2.e)
- Migrating ALL ValueSnapshot::of sites to of_tracked (incremental work; this stone ADDS the constructor only)
- Internal eval_<name> signature flips (they stay returning Value)
- holon-rs touched (STOP-4)
- Parallel API or deprecation alias (HARD CUT)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to cascade
- STOP-2: baseline regress below 827
- STOP-3: 240 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (Value::Tracked variant body, proc-macro, ValueSnapshot::of migration sweep)
- STOP-7: probe still has failures (any of 5 contracts not PASS)
- STOP-8: existing arc 233 probes regress
- STOP-9: cascade exceeds time-box — surface partial state

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.j.md` (new file per `feedback_inscription_immutable`).

SCORE expected to break down:
- eval_inner signature flip + leaf-arm wraps (file diff, line count)
- Cascade count per file (eval_inner call sites updated; .value_owned() vs .value() picks)
- 5 producer constructor swaps (per-producer line count)
- recv/try-recv honest delta (provenance loss; recovery plan in 233.2.e)
- eval boundary simplification (4-line removal)
- ValueSnapshot::of_tracked addition (~5 lines)
- Time breakdown by phase
- Calibration band actual vs predicted (90-150 target; 240 STOP)

## What this unblocks

- **Stone 233.2.k** — Value::Tracked variant + .inner()/.provenance() retirement (final structural class-elimination at the variant layer)
- **Stone 233.2.l** — #[wat_value] proc-macro structural seal (meta-class prevention; ✅✅✅)
- **arc216 stone1 7 probes** (task #496) — auto-resolve once Value::Tracked is structurally absent
- **Stone 233.2.e** — AST-derived provenance on the fully-sealed substrate (restores recv/try-recv provenance via the new mechanism)

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.j.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.j.md` — sub-DESIGN (commit `064df14`)
- `tests/probe_stone_233_2_j_producer_migration.rs` — FM 2-bis probe (commit `cf6d464`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.i.md` — boundary flip precedent
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md` — substrate-as-teacher cascade precedent
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving annihilation-not-patch
