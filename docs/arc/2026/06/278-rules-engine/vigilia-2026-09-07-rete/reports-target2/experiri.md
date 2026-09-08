## EXPERIRI — Cast Report (TARGET 2 — the `RETE_OPS` declared surface)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> ⭐ **The only ward in this vigilia that EXECUTED.** Every other cast was read-only.

## The instrument, and what I actually did

The prototype named in the brief (`src/rete/reachability.rs`, 2180 lines) is real, live, and already on the release floor. I drove it — I did not build it. My own construction work was limited to the one position its own header names as absent from that live gate (`:then`), using the exact program shape a shelved reconnaissance harness had already established, replayed through `./target/release/wat` directly rather than through Rust (no edits under `src/`, `wat/`, or `tests/`).

## CALIBRATION

**PASSED — two calibrations, 8 drives total, both mixed.**

1. **The live ledger's own calibration** (`the_ledger_reproduces_every_known_cell_before_it_reports_an_unknown_one`): `i64::>` fires in both positions, `keyword::=` fires in both positions, `law_a_core_head` (`:wat::core::>`, a non-rete head) is refused in both positions. Companion gates also passed — verbatim:
   ```
   PASS wat rete::reachability::the_ledger_reproduces_every_known_cell_before_it_reports_an_unknown_one
   PASS wat rete::reachability::the_two_positions_render_differently_and_now_agree
   PASS wat rete::reachability::a_refusal_that_does_not_name_the_op_is_a_template_defect_not_a_reachability_finding
   ```
2. **My own calibration for `:then`** (2 pinned cells × fire/refuse), driven fresh today outside `src/`:
   ```
   $ ./target/release/wat wat-scripts/scratch-pad/experiri-then/then-calib-i64gt-fire.wat      → 1
   $ ./target/release/wat wat-scripts/scratch-pad/experiri-then/then-calib-i64gt-refuse.wat    → 0
   $ ./target/release/wat wat-scripts/scratch-pad/experiri-then/then-calib-stringeq-fire.wat   → 1
   $ ./target/release/wat wat-scripts/scratch-pad/experiri-then/then-calib-stringeq-refuse.wat → 0
   ```

## Roster re-derivation (not trusted from the header)

`RETE_OPS` measured **79 rows**, three independent ways: `grep -c 'rete_name: ":wat::rete::' src/rete/vocabulary.rs` → 79; a count of `ReteOp {` struct openings → 79; `operands_for`'s own two builders (27 `special_for` arms + 52 uniform-table arms) → 79. This matches the lint gate's own *"measured 2026-09-01: 79 rows"* and **contradicts** `reachability.rs`'s header prose (*"a 74-row x N-kind ledger"*) — same stale-count class as the already-rowed `vocabulary.rs` defect (2I1); reporting the delta, not re-filing it.

## COVERAGE

```
COVERAGE: driven inline-constraint, where-fence (79/79 rows, live gate)
        | driven acc-head-fold (1/1 eligible row, live gate)
        | driven :then — PARTIAL (4/79 rows, ad-hoc probe this cast; 75 rows not driven)
        | untestable —
        | unadjudicated —
```

**Positions swept in full by the live, floor-gated ledger:**
- `InlineConstraint` / `WhereFence` — all 6 shards, all 79 rows, every cell `FIRES`/`FIRES`. Zero unclassified, zero `MatchesNothing`, zero `TemplateDefect`; both exclusion lists (`NOT_YET_GENERABLE`, `COMPILED_EXECUTOR_CANNOT_RUN`) are empty.
- `AccHeadFold` — exactly one `RETE_OPS` row is admissible by shape (`PersistentVector/length`, the only row whose params are `[PersistentVectorOf(_)]`); it fires.

**`:then` — the position the live ledger declares out of scope.** The header's stated reason is *"remains deliberately unmodelled, with its own separate defect. An un-calibrated position would add a column of findings nobody can trust."* I judge this a **deferral, not a disposal**, and drove around it rather than accepting it as `position-not-modelled`:

- A calibrated harness for exactly this position **already exists** — `docs/arc/2026/06/278-rules-engine/harness-experiri/positions-3-4.rs.txt`, with its own `experiri_then_calibration` (2 pinned cells, 4 drives, mixed outcomes) — built, driven once (2026-08-30), and shelved as `.rs.txt` specifically so tooling won't load it. It found two real L1s, both since fixed (acc-head dispatch, 2026-08-31; `match`-arm-is-a-call, 2026-09-02), but the harness's own **general 79-row `:then` sweep was never converted to an assertion and never re-run** — its README says so explicitly (*"only `experiri_then_calibration` can fail… nothing compares the matrix to anything"*).
- I reproduced that calibration fresh today (PASSED, mixed) and additionally drove 4 declarations at `:then`:

| declaration | position | verdict | caller | system said |
|---|---|---|---|---|
| `i64::>` | `:then` | drove-and-discriminated | `then-calib-i64gt-fire.wat` / `-refuse.wat` | `1` / `0` |
| `string::=` | `:then` | drove-and-discriminated | `then-calib-stringeq-fire.wat` / `-refuse.wat` | `1` / `0` |
| `match`, bare-kw arm | `:then` | drove-and-discriminated | `then-match-bare-arm.wat` | `1` — the 2026-09-02 cure holds at current HEAD |
| `match`, paren arm | `:then` | drove-and-discriminated | `then-match-paren-arm.wat` | `1` — agrees with bare spelling |
| `PersistentVector/length` | `:then` | drove-and-discriminated | `then-pv-length.wat` | `1` |
| `:wat::core::>` (Law A control) | `:then` | **Refused** (named, via a *different* mechanism) | `then-law-a-core-head.wat` | `"compile-condition: then expr is not total — ':wat::core::>' is not total"` |
| `:wat::core::not` (Law A control) | `:then` | **Refused**, named | `then-law-a-core-not.wat` | `"then expr is not a rete primitive — ':wat::core::not' is not a rete primitive; a then admits only :wat::rete:: ops"` |

All 4 sampled `RETE_OPS` declarations fire cleanly in `:then`, agreeing with their `InlineConstraint`/`WhereFence` verdicts (no new asymmetry). Law A (rete-primitives-only) holds in `:then` too — confirmed by a genuine, op-naming refusal — though `:wat::core::>` is caught one step earlier by a totality check than by the rete-primitive check `:wat::core::not` hits; both correctly refuse, so this is a curiosity, not a bypass.

**This does not make `:then` "position-not-modelled" in the ward's sense.** The reason on file is falsified by direct, fresh evidence that the position is drivable, was already calibrated once, and found real defects when driven — a shelved instrument, not an impossible one. I have **not** swept the remaining 75 rows, so this stays a **partial, not-driven** entry on the coverage line rather than a closed one.

## CELLS IN GRID / ATTEMPTED / FINDINGS

```
CELLS IN GRID:   163 = attempted 163 | exempt 0 | never reached 0
ATTEMPTED:       163 = drove-and-discriminated 163 | unreachable 0 | inert 0 | driver defects 0
FINDINGS:          0 cell-level (L1/L2) + 1 CAST-LEVEL finding (below)
DECLARATIONS: asymmetric 0
COST: ~236s wall-clock (~69s cold release compile + ~25s nextest + ~135s re-running the two wat-scripts lint gates for hygiene + ~3s direct wat-CLI drives)
```

163 = 158 (79 rows × {InlineConstraint, WhereFence}, live gate) + 1 (`AccHeadFold`, live gate) + 4 (`:then`, sampled this cast).

**The one cast-level finding (Level 1, per the spell's "cast that cannot finish" clause):**

> **`:then` (79 declarations)** — position **not driven** by any standing gate. Its exclusion is currently justified in-code as `position-not-modelled` (*"would add a column of findings nobody can trust"*), but a working, mixed-outcome calibration for exactly this position already exists (`harness-experiri/positions-3-4.rs.txt`, built and driven 2026-08-30), and I reproduced it clean today. **The reason on file disposes of nothing — it defers to a harness that was already built and then shelved.** `:then` is one of exactly three positions the vocabulary's own module doc declares admissible for every rete verb; today the live ledger tests zero of that column.
> caller: `./target/release/wat wat-scripts/scratch-pad/experiri-then/then-calib-i64gt-fire.wat` (and siblings)
> system: `1` / `0` / `1` / `1` / `1` — every sampled cell fires and discriminates; no defect found in the sample, only an un-swept column.

## VERDICT

**VERDICT: the declared surface is NOT fully reachable** — not because any driven cell failed, but because `COVERAGE`'s *not driven* slot is non-empty (`:then`, 75/79 rows unsampled). Every cell this cast drove — 158 from the live gate, 1 from the live acc-head gate, 4 sampled at `:then` — fired and discriminated cleanly, with zero asymmetric declarations. **The gap is a coverage gap this ward is disclosing, not a reachability defect this ward found.**

## Files touched

New, untracked, under the mandated scratch location (no edits under `src/`, `wat/`, or `tests/`): nine `.wat` files in `wat-scripts/scratch-pad/experiri-then/`. Both `wat-scripts/` lint gates were re-run after adding these and pass clean.
