# EXPECTATIONS — Stone 251.2c — Function + Environment lift

Written BEFORE the strike. Pure lift — load-bearing = baseline-identical. Uniform re-export
(no per-type judgment; purgare sweeps dead pubs at the 251.2e ward).

## Scorecard

| # | What | Command | Expected |
|---|---|---|---|
| 1 | new file | `ls src/value/` | + `environment.rs` |
| 2 | cluster gone from runtime.rs | `grep -c 'struct Function\|struct Environment\|struct EnvBuilder\|struct BoundEntry' src/runtime.rs` | `0` (ignore comment false-positives — verify any hit is a comment) |
| 3 | EnvCell private in new home | `grep -n 'struct EnvCell' src/value/environment.rs` | present, NOT `pub` |
| 4 | lib builds | `cargo build --release` | clean |
| 5 | **lib tests IDENTICAL** | `cargo test --release --lib -p wat` | **923 / 0 / 1** |
| 6 | corpus IDENTICAL | `./scripts/integration-run.sh` | no new failures |
| 7 | clippy clean in-home | `cargo clippy --release -p wat 2>&1 \| grep 'src/value/'` | nothing |
| 8 | external API intact | `grep -n 'Function\|Environment' src/lib.rs` | re-exported from `crate::value` |
| 9 | signal.rs transitional import fixed | `grep -n 'Function' src/value/signal.rs` | `use crate::value::Function` |

## Independent prediction

- Runtime: **15–25 min** (contiguous ~200-line block; uniform re-export = no consumer repointing;
  the only finesse is the TrackedValue/Provenance intra-value imports + signal.rs fix).

## Trap-doors

1. **Environment ↔ observe coupling** — Environment::lookup → TrackedValue, BoundEntry holds
   TrackedValue; both in value/observe.rs → environment.rs imports `crate::value::{TrackedValue, Provenance}`.
2. **Function's TypeExpr fields** — `use crate::types::TypeExpr`; add the `// TRANSFORMS` marker, don't change.
3. **EnvCell visibility** — stays private (no `pub`); moves with Environment; not re-exported.
4. **signal.rs Function import** — must flip to crate::value::Function (was the transitional crate::runtime).
5. **Comment false-positives** in row 2 — "construct Function"/"construct Environment" substrings; verify
   any grep hit is a comment, not a stray def.

## Scoring method

Orchestrator re-runs rows 2–9 independently. Row 5 (923/0/1) load-bearing — any delta = reject.
Verify uniform re-export applied (no consumer repoint churn this stone) + the signal.rs import flipped +
EnvCell private. Commit on green + PUSH. (Dead-pub audit is informational only this stone — the ward
sweeps them; do NOT chase per-stone cleanup.)
