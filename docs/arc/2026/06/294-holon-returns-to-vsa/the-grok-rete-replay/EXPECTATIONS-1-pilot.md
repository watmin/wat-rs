# EXPECTATIONS 1 — the pilot, written BEFORE the strike

| # | what | check | expected |
|---|---|---|---|
| P1 | the chain order is derived, not hand-typed | `scripts/replay/chain-order.sh de827fb4c a3218644d` without overrides vs `bootstrap/landing-order.txt` | identical, 27 lines |
| P2 | the one override is provisional and explained | `scripts/replay/chain-order.overrides` | one move, bare-variant before positional-ctor, marked PROVISIONAL, with its reason |
| P3 | the converter is deterministic and SCOPE-honest | `convert.sh` run twice on one file; a tooling-scoped codemod (e.g. `fmt-head-fqdn-to-clojure`) does not touch a corpus file | byte-identical; out-of-scope untouched |
| P4 | ten commits replayed, in order | `git log` | ten commits `REPLAY(grok-rete #1..#10)`, each with its source hash; #8 a plain cherry-pick |
| P5 | behaviour arrived | for each step, the tests that C adds or changes, by name | all pass at their own step, logged |
| P6 | syntax enforced | `--check` of every `.wat` the pilot produced | clean (or a deliberate `.bad`) |
| P7 | the one real `.wat` 3-way (`wat/cache.wat`, #9 and #10) | the log: converted C^/C, the merge-file result, the conflicts and their resolutions | both sides' intent present; resolution rationale logged |
| P8 | checkpoints green | `scripts/floor.sh` at #5 and #10 · clippy | 0 failed · 0 lines |
| P9 | the cost is measured | PILOT-LOG wall times per step | a per-step time and a per-kind average (docs / .rs / .wat new / .wat 3-way) → an extrapolation to 651 |
| P10 | tricks are catalogued | PILOT-LOG "tricks" column | every non-mechanical move written down, reusable by the next branch |

**Runtime prediction:** 4–8 h, mostly #3 (four rete files main reworked) and #9 (twelve files).

**Trap doors, named in advance:**
- **Taking one side of an `.rs` conflict wholesale.** That is how the first merge lost work.
- **Hand-editing a converted `.wat` to make it pass.** That is STOP-2, not a fix.
- **"The test passes" because the test did not run.** Name each test and read its PASS line.
- **Treating the provisional override as settled.** The orchestrator's composition check decides it.
