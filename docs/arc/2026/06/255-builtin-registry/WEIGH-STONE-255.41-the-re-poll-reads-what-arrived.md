# WEIGH — STONE 255.41: the process `poll` re-poll reads what arrived — ACCEPTED, with one guard carried

**Executor: grok via pulsare, commit `c3e637959`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T22-09-19Z` | `6135 tests run: 6135 passed (10 slow), 22 skipped` |
| both index-0 arms | read `src/kernel/message.rs` ~:1816–1829 (first select) and ~:2036–2046 (re-poll) | **both call `process_lineage_event(result/res2, …)`**. The re-poll now reads the frame: `Ok` → `Admin`, `Err` → `Shutdown` |
| the IDE's "couldn't read `../../../src/…`" | read the test | stale: the file says `include_str!("../../src/kernel/message.rs")` |

Taken from the SCORE: clippy 0; census `no STOP-8`; delta NEW 2 / RECOVERY 0; ledger 198; the helper's two
verdicts pinned by a unit test (bytes `42` → `Admin 42`; `Disconnected` → `Shutdown`); the merge map for
`sns-sqs` (`message.rs` 1360–1409, 1816–1829, 2036–2046, 2147–2179).

## ⚠ Carried: the call-site guard is a text count

The re-poll path (a kernel `POLLIN`, then `accept` `EAGAIN`) cannot be driven honestly from a test; no
sleeps, correctly. So the executor pinned that both arms use the helper with
`include_str!("../../src/kernel/message.rs").matches("process_lineage_event(").count() == 4`. **That is a
substring count over source text**, the class the builder rejected the same day (*"all of wat is strongly
structural.... these kind of matches are incredibly shitty"*). A comment naming the helper, or a fifth
honest call, moves the count without any bug, and a moved call site can keep the count while the bug
returns.

**The structural cure (for a later stone, or the vocabulary split, which rewrites this code):** make the
index-0 handling **one place**, with a single lineage branch the first select and the re-poll both reach,
so there is **no second arm to drift**. Then the text guard has nothing left to protect, and it is
deleted.
