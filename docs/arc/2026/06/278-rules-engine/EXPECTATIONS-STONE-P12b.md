# EXPECTATIONS — Stone P12b (independent scorecard, fixed BEFORE the strike)

Additive: a new `Why` record + a new `explain` wat fn. Greens the P12 north-star's structural assertions
(via-counts). Everything else UNCHANGED.

| # | what | command | expected |
|---|------|---------|----------|
| 1 | the EXPLAIN north-star greens | `cargo test --release -p wat --test probe_arc278_P12_explain_walk` | **2 passed; 0 failed; 0 ignored** |
| 2 | P12a substrate still green | `cargo test --release -p wat --test probe_arc278_P12a_explain_substrate` | 3 passed |
| 3 | rete differential UNCHANGED | `cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules --test probe_arc278_P4c_native_retraction` | green |
| 4 | rete suite UNCHANGED | `cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy -- --include-ignored` | same as HEAD (still RED-ignored) |
| 5 | lib floor | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | **940 / 36** (no new failure) |
| 6 | deftest floor | `cargo test --release --test test \| grep "test result"` | **264 / 1** |
| 7 | deporder floor | `cargo test --release --test test_stdlib_load_order \| grep result` | **1 / 0** |
| 8 | nursery floor | `cargo test --release -p wat --test nursery -- --test-threads=1 \| grep result` | **~893 / 4** (±3) |
| 9 | build clean | `cargo build --release` | compiles; 25 warnings (baseline, unchanged) |

## Runtime prediction
~20–30 min. The work is one wat Record def + one recursive wat fn (≈15–25 lines) using idioms already in
rete.wat. Most of the clock is release builds + floor runs.

## Trap-door risks (named)
- **`via` collection type.** If `map` yields a std `Vector` not a `PersistentVector<Why>`, the `Why`
  constructor (field typed `PersistentVector<wat::rete::Why>`) rejects it → use `foldl` + `PersistentVector/conj`
  or the PV-producing map. Row 1 catches it (won't compile / wrong type).
- **Empty-PV footgun.** `(:wat::core::PersistentVector :wat::rete::Why)` can capture the type's constructor as
  an ELEMENT (the documented footgun). Use the corpus's empty-typed-PV idiom; row 1 catches a wrong leaf shape
  (via-count would be 1 not 0 at a leaf → cascade count wrong).
- **Recursion non-termination.** Only if a base fact were wrongly looked up as derived; the `None` branch must
  return a leaf. Row 1 would hang/overflow — if so, the base/derived test is inverted.
- **Over-building.** Adding `:met`/edge types to chase the probe's DOC (which describes the P12c rich form) when
  the ASSERTIONS only need via-counts. STOP-3. The doc is the eventual contract; the assertions are the gate.

## Acceptance
All rows met, weighed against the orchestrator's OWN re-run + a read of the diff: the diff shows ONLY the `Why`
record def + the `explain` fn in `wat/rete.wat` + 2 removed `#[ignore]`s — no Rust, no change to existing rete
paths. Commit on green + push.
