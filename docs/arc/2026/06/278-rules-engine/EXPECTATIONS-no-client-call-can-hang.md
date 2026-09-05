# EXPECTATIONS — no client call can hang

Written **before** the strike. Every "before" is a run of mine on `fba0b4777`.

| # | what | expected |
|---|---|---|
| 1 | **★★ a silent peer no longer hangs a caller** | a probe that calls a generated method against a peer that never replies **raises within ~10 s** naming surface, verb and deadline. Before: **forever** |
| 2 | **★★ the arms did not move** | `git diff wat/service.wat` shows **no change** to the `RecvOutcome` match arms; `grep -c "RecvOutcome::Message"` across `.wat` is **643**, unchanged |
| 3 | **the floor** | `5215 passed`, 22 skipped. ⚠ **If it reds with deadline raises, that is row 5, not a failure** |
| 4 | blast radius | `git diff --stat` → `wat/service.wat` only (plus a recorded fix, if row 5 fired) |
| 5 | ⛔ **which surfaces need a longer deadline?** | **report every raise the floor produced, with its elapsed time.** Expected: none. If any, they are the `:deadline-ms` declarations |
| 6 | the census | wat-grep + rete: the true count of generated-method call sites, **with both controls shown** |
| 7 | existing chaos unaffected | check-drop ×3, mark-drop ×3, recv-drop ×3, ack-drop ×3: `distinct=100` |
| 8 | rate-0 | circuit ×5: `total=8000; distinct=8000` |
| 9 | timings | report only. Before: publish `47784–49856` |

### Before-state, recorded verbatim

```
row 2  643 RecvOutcome::Message arms across 282 .wat files
row 3  Summary [ 359.980s] 5215 passed, 22 skipped   .floor/2026-09-05T11-32-03Z/
row 7  check/mark/recv/ack-drop: distinct=100 on every run measured
row 8  total=8000; distinct=8000; dup=0 ×5
row 9  publish 47784 47790 48170 49124 49856
```

## ⛔ ROW 1 NEEDS A PEER THAT IS ALIVE AND SILENT

Not a dead peer — `Lost` already covers that, and it works. **A peer that accepts the connection
and never replies.** `:fanout::Hold` (`circuit.wat:142`, *"Never settles"*) is exactly that, and
`:user::deadline-redial-is-fresh` already drives it.

★ Before this stone, a generated-method call against it hangs forever. **That hang is row 1's
before-state, and it should be demonstrated, not assumed** — the same discipline as provoking the
crash before repairing it.

## ⛔ ROW 3 CAN GO RED HONESTLY

A default of 10 000 ms is generous, but 5215 tests are a lot of round trips. **A red here that
names a surface is row 5 doing its job, not a failed strike.** Report it; do not tune the default
to hide it. STOP-3.

## ⛔ ROW 2 IS THE PROOF THE MIGRATION STAYED DISSOLVED

The entire argument for this shape is that **643 call sites do not move.** If that count changes,
or the `RecvOutcome` arms change shape, the stone became the season it was drawn to avoid.

## RUNTIME PREDICTION

45–75 min. The edit is small; the uncertainty is STOP-1 (peer-kind from a generated method's
locus) and the floor's verdict on the default.

## TRAP DOORS, NAMED

1. **Peer-kind at the caller's locus.** STOP-1 — the one thing not proven anywhere.
2. **A raise that prints nothing useful.** The message must name the **surface, the verb, and the
   deadline**. "call timed out" would repeat the empty-ARM failure this arc has paid for twice.
3. **Tuning the default to green the floor.** STOP-3.
4. **A green floor proving row 1.** It does not — nothing in the floor holds a silent peer open.
   Row 1 needs its own probe.
