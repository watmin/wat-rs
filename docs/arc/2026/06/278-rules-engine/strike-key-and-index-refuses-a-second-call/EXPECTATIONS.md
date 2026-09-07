# EXPECTATIONS — key_and_index refuses a second keying

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the guard is wired | `#[cfg(debug_assertions)] #[should_panic]` test panics under a **debug** run; quote it |
| 2 ★ | the rune carries its reason | `rune:excusare(no-falsifier)` on the test, its reason naming what was tried on the release floor and why it cannot work |
| 3 ★ | severity honest | the SCORE says hygiene on an **unreachable** branch — both callers gate on `is_keyed` AND reuse the stored list |
| 4 ★ | no release behaviour | floor **5470 run / 22 skipped, unchanged** |
| 5 | the doc matches the code | *"a later call does not replace keys"* replaced by *a later call is a caller bug; `writer()` is the route* |
| 6 | still two callers | `grep key_and_index(` → the same two |
| 7 | clippy | rc=0 |

★ load-bearing. **Row 1 is the only proof available**, and row 2 is what makes its absence from the
floor a stated limit rather than a hole.

## Trap doors, named in advance

- **`assert!` to win a floor-provable mutation.** Release cost for an unreachable branch, so a test
  can pass. Rejected in DESIGN; reaching for it is the strike failing.
- **Claiming the floor proves the guard.** It cannot — `debug_assert` is compiled out. That is the
  `no-falsifier` reason, and writing it as anything else is the convenience-plea `excusare` strikes.
- **Inflating the severity.** Both callers are structurally incapable of a second call; say so.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- `assert!` in place of `debug_assert!`.
- A floor number that moved.
- A rune whose reason does not answer `no-falsifier`'s decisive test.
- Severity written up as a live defect.
