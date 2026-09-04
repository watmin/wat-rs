# EXPECTATIONS — the vocabulary stops mumbling

Written **before** the strike. Re-run by me on a quiet box.

⚠ **Re-baseline the circuit on the grading box first** (S29) — my windows drift upward with session
time while the executor's do not, so a band from hours ago is not an instrument.

| # | what | expected |
|---|---|---|
| 1 | ★★ **the rewrite is visible** | no helper transforms its argument without saying so in its name. `accept!` → `publish-stamped-until-accepted!` |
| 2 | ★★ **the bound reports** | force the retry to expire: it names **depth, cap, attempts, elapsed**. ⛔ "gave up" alone fails the row |
| 3 | ★★ **backpressure survives** | circuit, **five runs**, `total=8000; distinct=8000; dup=0`. If the bound trips in normal operation it is **too short** — the fix is a longer bound, never a smaller queue |
| 4 | the `Lost`-is-ok arm has a WHY | `face-start-tw`'s asymmetry documented, **behaviour unchanged** |
| 5 | `nap-ms` renamed, not consolidated | six homes, six renames, still six |
| 6 | `do-` swept | `do-stats`/`do-depth` say which half they return |
| 7 | codemod recorded + idempotent | re-run reports 0 changes |
| 8 | scope | no S33 merge, no S34 outcome change, no `wat/service.wat`, no `src/` |
| 9 | the floor | `5213/5213` |

## ⛔ ROW 3 IS THE ONE THAT CAN GO WRONG QUIETLY

Bounding a correct backpressure loop is the one move here that can **introduce loss while looking
like a fix.** The retry exists because the queue is bounded and the producer must wait.

- Bound **too long**: harmless. It never trips; the mumble is fixed; nothing else changes.
- Bound **too short**: `accept!` gives up, the message is never published, and `distinct` silently
  drops. **The floor may still be green** — the circuit's own assertion is what catches it, and only
  if you run five.

That asymmetry is why the bound is a **LIVENESS** bound in the arc's taxonomy: *only a hang may trip
it.* When in doubt make it longer, and say what you chose and why.

## RUNTIME PREDICTION

**60–90 minutes.** The renames are a codemod; the retry bound and the `face-start` WHY are the real
work, and the bound is where the care goes.

## TRAP-DOOR RISKS

1. **Two `accept!`s, different bodies.** `circuit.wat`'s stamps; `sns-fanout.wat`'s does not. Two
   names, not one.
2. **The stamp is load-bearing.** `circuit.wat`'s trace histograms parse `{body}|{t0}` — removing the
   rewrite breaks telemetry. **Rename it; do not remove it.**
3. **Comments are not rewritten by the codemod.** Prose mentioning `accept!`/`nap-ms`/`face-start` is
   a named manual pass.
4. **Three of the six `nap-ms` are in scratch-pad probes** and the `every_wat_scripts_file_loads`
   gate type-checks them. Missing one turns that gate red at the end.
5. **Do not write `(:wat::core::None <Type>)`** — phantom form, arc-109 NOTE.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 2 satisfied by a bound that only reports failure without what it saw.
- Row 3 from fewer than five runs, or with the bound tripping and called acceptable.
- The stamp removed rather than renamed.
- The six `nap-ms` consolidated into one — that is a promotion, not a rename.
- The codemod's census reported as mine rather than the finder's.
