# EXPECTATIONS — Stone P12a (independent scorecard, fixed BEFORE the strike)

The stone is **purely additive** behind a new verb + new ephemeral type. The fast path takes the `None` support
param → byte-identical behavior. So: the P12a probe flips 3-ignored → 3-green, and everything else is UNCHANGED
(floors + the rete differential).

| # | what | command | expected |
|---|------|---------|----------|
| 1 | P12a probe green | `cargo test --release -p wat --test probe_arc278_P12a_explain_substrate` | **3 passed; 0 failed; 0 ignored** |
| 2 | rete differential UNCHANGED (the load-bearing guard) | `cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules --test probe_arc278_P4c_native_retraction --test probe_arc278_P2_native_fire_once` | all green, same as HEAD |
| 3 | rete suite UNCHANGED | `cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy -- --include-ignored` + the 5a/5b/4b probes | same pass/fail as HEAD (north-star still RED-ignored; P12 north-star still RED-ignored) |
| 4 | lib floor unchanged | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | **940 passed / 36 failed** (no new failure) |
| 5 | deftest floor unchanged | `cargo test --release --test test \| grep "test result"` | **264 / 1** |
| 6 | nursery floor unchanged | `cargo test --release -p wat --test nursery -- --test-threads=1 \| grep result` | **~893 / 4** (±3 fork flake) |
| 7 | deporder floor unchanged | `cargo test --release --test test_stdlib_load_order \| grep result` | **1 / 0** |
| 8 | build clean | `cargo build --release` | compiles; warning count ~unchanged |

## Runtime prediction
~20–35 min wall. The work: one param + one branch in `fire_fixpoint_delta`, a new `eval_fire_rules_explain`
entry building the `Explained`/`Support` Values, one dispatch arm, one TypeScheme, two wat Record defs. Most of
the clock is the release builds + the floor/differential runs.

## Trap-door risks (named)
- **Differential drift (the real risk).** If `fire_fixpoint_delta` gets *copied* instead of parameterized, the
  two engines drift. Row 2 is the guard; STOP-4 forbids the copy.
- **`token_to_value` mismatch.** If the support index's tokens aren't converted to proper wat `Token` Values,
  the probe's `Token/matches` read fails. Row 1 assertion 3 catches it.
- **`seen`-dedup vs first-producer.** Recording at `if !seen` with `or_insert` is first-producer-wins (v1,
  correct). If recorded outside that guard, a fact derived twice double-counts → row 1 assertion 2/3 catches it.
- **Fast-path leak.** If the `None` path accidentally allocates/records, the differential (row 2) flags it.

## Acceptance
All rows met, weighed against the orchestrator's OWN re-run (not the executor's report) + a read of the diff:
the diff shows ONLY additive changes (the `support` param + `if !seen` recording + `eval_fire_rules_explain` +
`Explained`/`Support` defs + one dispatch arm + one TypeScheme + 3 removed `#[ignore]`s) and NOTHING in the fast
`fire-rules'`/`fire-rules-spec`/`fire-once'` bodies. Commit on green + push.
