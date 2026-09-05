# EXPECTATIONS — the death notice is not a malformed frame

Written before the strike. Every "before" is a run of mine on `3edbd87ed`.

| # | what | expected |
|---|---|---|
| 1 | ★★ rst green | `client_sees_peer_crashed_not_bare_disconnect` **passes** — a crashed peer reaches the client as `Lost`, not `CLOSED` |
| 2 | ★★ severed green | `an_owner_drop_reaches_the_client_as_severed` **passes** — `SEVERED`, not `CLOSED:MUTE` |
| 3 | ★★ **the floor** | **`5215 passed, 0 failed`, 22 skipped** |
| 4 | ⛔ the scrub holds | no newly returned cause carries text beyond `message_only_failure` |
| 5 | blast radius | `src/runtime.rs`, `wat/service.wat`, `circuit.wat` — **no codemod, no `.wat` corpus edits** |
| 6 | `PeerGone` is gone | `grep -c PeerGone` across `.wat` is **0**; the four call sites match `Lost`/`Closed` |
| 7 | chaos unaffected | check/mark/recv/ack-drop ×3 each: `distinct=100` |
| 8 | rate-0 | circuit ×5: `total=8000; distinct=8000; dup=0` |
| 9 | timings | report only. Before: publish `52240–64111` (ran concurrent with chaos; re-measure quiet) |

### Before-state, recorded verbatim

```
row 1/3  Summary [362.856s] 5215 run: 5213 passed, 2 failed, 22 skipped
         .floor/2026-09-05T21-32-16Z/
         rst      got: Ok(String("CLOSED"))
         severed  left "CLOSED:MUTE" right "SEVERED"
row 7/8  distinct=100 on every chaos cell; total=8000; distinct=8000 x5
```

## ⛔ ROW 3 IS GREEN THIS TIME, AND THAT IS THE POINT

The last two stones each expected a red count and got a different one, because I gated a
**consequence** instead of what the stone **controlled**. This stone controls both reds
directly: change 1 gives rst its `Lost`, change 2 gives severed its cause, change 3 stops both
being flattened. **There is no third defect underneath that I know of — and if there is, that is
the finding, reported, not worked around.**

## ⛔ ROW 4 IS THE ONE THAT PASSES QUIETLY WHILE LEAKING

Rows 1–3 can all go green while a `Lost` carries a raw panic message to a client. That would be
worse than the reds. **Check the value, not just the variant.**

## RUNTIME PREDICTION

60–90 min. Change 1 is one arm. Change 2 is two predicates. Change 3 touches a stdlib enum and
four call sites, and needs a rebuild.

## TRAP DOORS

1. **Returning the variant, forgetting the scrub.** Row 4 / STOP-2.
2. **Assuming the process tier can see a sever.** STOP-1 — report it, do not synthesize one.
3. **Changing `poll`'s admin arm** because it looks like the others. Different channel.
4. **A remaining red.** Report which and why; do not tune a test.
