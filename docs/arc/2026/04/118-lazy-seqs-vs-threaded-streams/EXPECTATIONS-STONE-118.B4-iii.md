# EXPECTATIONS — STONE 118.B4-iii · the wall

Written before the strike. The scorecard cannot move.
**Rows 6–8 are the ORCHESTRATOR's to run** — the rider does not touch the floor.

| # | what | who | expected |
|---|---|---|---|
| 1 | census over all **16** files after phase 1 | rider | **0** |
| 2 | ★ walk C (`empty?`+`first`+`next`) refused, message names `next` | rider, `--check` | refused |
| 3 | ★ `(nth s i)` refused, message names `drop`+`next` | rider, `--check` | refused |
| 4 | walk A still yields **6 FORCED for 5 elements** | rider, the probe | 6 |
| 5 | four dead arms converted to `unreachable!()` | rider, file:line each | 4 |
| 6 | floor | **orchestrator** | ≥4765 run, 0 FAIL, 19 skipped |
| 7 | clippy | **orchestrator** | 0 |
| 8 | ignores | **orchestrator** | 13 |
| 9 | blast radius held | rider, `git status --short` | 5 `src`/`wat` + 3 phase-1 + 4 phase-3 test files |
| 10 | arc 118's tests **rewritten, not deleted** | orchestrator, read the diff | files present, eager rows intact |

**Rows 2 and 3 together are the wall.** Row 2 alone leaves the quadratic shape open; row 3 alone
leaves the 3× walk open. The ruling was to close both.

## Independent prediction

**60–90 minutes.** Phase 1 is minutes. Phase 2 is four flips plus four `unreachable!()` conversions
plus one new `infer_list` arm. Phase 3 is the judgement work and will take the longest — three
distinct tests whose *intent* has to survive a spelling change.

## Trap-doors named in advance

- **The compiler does not catch the dead arms.** Measured twice. A rider that flips the bits, sees a
  clean build, and moves on has left four arms that read as live and can never execute.
- **Phase 3 is where the honesty risk lives.** The easy way to make `probe_arc118_lazy_seq` pass is to
  assert less. Row 10 and STOP-1 exist for that.
- **`nth-spec` drops to three arms.** If a rider leaves the `Seqable` arm in place, the oracle and the
  native disagree about the receiver set and the differential silently stops covering what it claims.
- **Do not re-run the codemod over `wat/`** — it is already a fixed point there; phase 1 is `tests/`
  only. A second full-corpus run should be a no-op, and if it is not, that is STOP-2.
