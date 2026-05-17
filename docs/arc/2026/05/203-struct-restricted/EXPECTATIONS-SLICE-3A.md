# EXPECTATIONS — Arc 203 Slice 3a: server dispatch loop (thread, N=1)

**BRIEF:** `BRIEF-SLICE-3A.md`
**Drafted:** 2026-05-17 post-Stone-C3 commit `cfdf3b9`.

## Independent prediction

**Runtime band:** 45-75 min sonnet.

Reasoning:
- One new wat-tests file, ~100-150 lines
- Server dispatch is a single recursive defn with select + match
- Channel pair creation + spawn-thread is established pattern (Counter actor proofs)
- Main novelty: select-based admin/user routing (vs single-channel dispatch in prior proofs)
- High likelihood STOP 1 fires (heterogeneous select rejected → unified Wire enum pivot)

**Time-box:** 90 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — form parses + compiles | YES | high |
| B — user round-trip succeeds | YES | high |
| C — admin Stop succeeds with Final | YES | medium-high |
| D — workspace baseline preserved | YES | high |

**4/4 PASS predicted; ~75% confidence overall.** Lower than struct-restricted slices because select with mixed-type receivers may need the Wire-enum pivot mid-implementation; if so, sonnet adapts cleanly per the BRIEF guidance.

## Honest deltas predicted

### Highly likely

1. **STOP 1 fires** — heterogeneous select rejected; sonnet pivots to unified `:counter::Wire` enum with Admin/User tagged variants. This is actually a CLEANER architecture for the process variant later (stone 3d uses the same Wire enum on the stdio stream). Surface as honest delta + suggested DESIGN update for stones 3b-3d.

2. **`spawn-thread` fn signature requires `Fn[Receiver<I>, Sender<O>]`** — for our N=2-channel server, we'd need a wrapping pattern (server fn closes over the 4 channels via let-binding, or spawn-thread is called with channels-as-args). Verify the established pattern in counter-actor-proof-thread.wat — it wraps server-rx/server-tx into a ThreadPeer. Same pattern applies.

### Less likely

3. **Tuple return shape for spawn** — returning a 4-tuple of channel ends may need specific wat::core::Tuple/N indexing; sonnet uses what works
4. **recv result handling** — arc 110/111 returns Result<Option<T>>; sonnet uses option::expect or match per established patterns

## Workspace baseline (post-Stone-C3 commit `cfdf3b9`)

3 pre-existing failures (deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips).

Post-3a target: +1 passing deftest; 3 failures preserved.

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 45-75 min | TBD | TBD |
| Scorecard rows | 4/4 PASS | TBD | TBD |
| Workspace fail count | 3 | TBD | TBD |
| New deftest count | 1 | TBD | TBD |
| STOP-1-fires (Wire enum pivot) | YES (likely) | TBD | TBD |
| Other substrate↔assumption gaps surfaced | 1-2 | TBD | TBD |
| BRIEF corrections suggested for stones 3b-3d | 1-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
