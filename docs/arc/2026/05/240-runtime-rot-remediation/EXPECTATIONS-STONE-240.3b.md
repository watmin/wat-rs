# EXPECTATIONS — Stone 240.3b — telemetry + sqlite `.wat` drift sweep. Orchestrator scores on independent re-run.

## Independent runtime prediction

**25–40 min Mode A.** ~20 Atom sites in WorkUnit.wat (test) + a handful of HashMap + the
sqlite files, each a mechanical per-site recipe application from a proven exemplar, iterating
from the test diagnostic stream. Possible extra drift layers (atom-value/from-holon) surface
mid-loop, same as they did on WorkUnitLog. Wakeup time-box: **2× upper = 80 min.**

## Scorecard (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | **wat-telemetry green (LOAD-BEARING)** | `cargo test --release -p wat-telemetry 2>&1 \| grep -c "FAILED"` | 0 |
| 2 | **wat-telemetry-sqlite green (LOAD-BEARING)** | `cargo test --release -p wat-telemetry-sqlite 2>&1 \| grep -c "FAILED"` | 0 |
| 3 | **build gate** | `cargo build --release --tests --workspace 2>&1 \| grep -c "^error"` | 0 |
| 4 | lib baseline unaffected | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` |
| 5 | scope | `git status --short` | only `.wat` under `crates/wat-telemetry/` + `crates/wat-telemetry-sqlite/` + the SCORE; NO src/*.rs, NO holon-rs, NO lru |
| 6 | no retired verbs left | `grep -rc ":wat::holon::Atom\|:wat::core::atom-value" crates/wat-telemetry crates/wat-telemetry-sqlite \| grep -v ":0"` | only comments-recording-history, if any (live calls = 0) |

**FM-9:** independently re-run rows 1 + 2 + 3. The telemetry crates' deftests are thread-based
(no process leak) — targeted `-p` runs are safe and leak-free. Do NOT run the full workspace
suite (arc 170 leak; out of scope).

## Mode classification
- **Mode A:** rows 1–4 green; all drift was the 4-element recipe; ≤ STOP-2.
- **Mode B:** any crate still FAILED for a non-recipe reason, src/holon-rs/lru touched, full
  suite run, or a STOP-trigger (logic/deadlock/leak) surfaced and was worked around. Any → re-brief.
- **STOP-and-report (NOT failure):** a genuinely novel error (logic mismatch, in-flight-arc
  coupling) → sonnet stops + surfaces; orchestrator classifies (may be a new DEFER, not a fix).

## On green
Commit the swept `.wat` files + `SCORE-STONE-240.3b.md` as one commit. Then 240.4: re-confirm
the wat-cli A-cascade cleared (prod telemetry wat now loads clean → CLI startup no longer
carries WorkUnitLog/WorkUnit errors) + reconcile the DEFER ledger + INSCRIPTION.
