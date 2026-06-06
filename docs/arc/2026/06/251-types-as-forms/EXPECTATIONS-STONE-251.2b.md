# EXPECTATIONS — Stone 251.2b — signal + observe lift

Written BEFORE the strike. Pure lift — load-bearing expectation is baseline-identical.

## Scorecard

| # | What | Command | Expected |
|---|---|---|---|
| 1 | new files | `ls src/value/` | + `signal.rs` + `observe.rs` (now: encoding_ctx, frame, mod, observe, signal) |
| 2 | signal gone from runtime.rs | `grep -c 'enum EvalSignal\|struct RuntimeError\|enum RuntimeErrorKind' src/runtime.rs` | `0` |
| 3 | observe gone from runtime.rs | `grep -c 'struct TrackedValue\|struct ValueSnapshot' src/runtime.rs` | `0` |
| 4 | render_value moved | `grep -c 'fn render_value' src/runtime.rs` then `src/value/observe.rs` | `0` then `1` |
| 5 | 251.2a dead pubs dropped | `grep -c 'pub use crate::value::EncodingCtx\|pub use crate::value::{FrameInfo' src/runtime.rs` | `0` |
| 6 | lib builds | `cargo build --release` | clean |
| 7 | **lib tests IDENTICAL** | `cargo test --release --lib -p wat` | **923 / 0 / 1** |
| 8 | corpus IDENTICAL | `./scripts/integration-run.sh` | no new failures |
| 9 | clippy clean in-home | `cargo clippy --release -p wat 2>&1 \| grep 'src/value/'` | nothing |
| 10 | external API intact | `grep -n 'RuntimeError' src/lib.rs` | re-exported from `crate::value` |

## Independent prediction

- Runtime: **25–40 min** (larger than 251.2a: RuntimeErrorKind is ~450 lines / ~30 variants; the
  per-type re-export judgment + render_value move add steps).
- Mode A plausible but this is the first MEDIUM stone — watch for the ValueSnapshot/RuntimeErrorKind
  Box coupling + the render_value move.

## Trap-doors

1. **render_value full body** — confirmed clean in the first ~90 lines; sonnet must read the WHOLE
   body before moving (STOP-2 if a back-reference appears deeper).
2. **RuntimeErrorKind carries Box<ValueSnapshot>** — observe must resolve before/with signal; both
   land same stone so intra-stone ordering is the compiler's job, but the `use` must be right.
3. **EvalSignal::TailCall carries Arc<Function>** — Function still in runtime.rs → `use crate::runtime::Function`
   (transitional). NOT a cycle.
4. **The re-export judgment** — the score must show, per moved type, re-export-vs-repoint + the count.
   ValueSnapshot (74) MUST be a re-export; if sonnet repoints 74 sites that's churn (flag it).
5. **lib.rs multi-name `pub use` list** — move only the signal/observe names, leave the rest.

## Scoring method

Orchestrator re-runs rows 2–9 independently. Row 7 (923/0/1) is load-bearing — any delta = behavior
moved = reject. Verify the re-export principle was applied correctly per type (not blanket pub-use,
not 74 repoints). Confirm render_value moved with no eval back-reference. Commit on green; SCORE
records the per-type re-export/repoint decisions.
