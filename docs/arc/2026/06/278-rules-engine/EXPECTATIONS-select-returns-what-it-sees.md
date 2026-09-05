# EXPECTATIONS — select returns what it sees

Written **before** the strike. Every "before" is a run of mine on `276f989dc`.

| # | what | expected |
|---|---|---|
| 1 | **★★ the rst red goes green** | `client_sees_peer_crashed_not_bare_disconnect` **passes** |
| 2 | **★★ select no longer raises on the peer wire** | `grep` the select range: **0** `MalformedForm` raises reachable from a peer event |
| 3 | **the floor** | **`5214 passed, 1 failed`** — the severed red is the **expected remainder** |
| 4 | ⛔ **the scrub holds** | no newly returned `Failure` carries a message beyond the canonical reason-free one |
| 5 | blast radius | `git diff --stat` → **`src/runtime.rs` only** |
| 6 | the three shutdown sites | return `ServiceEvent::Shutdown`, matching `select:26067` |
| 7 | chaos unaffected | check/mark/recv/ack-drop ×3 each: `distinct=100` |
| 8 | rate-0 | circuit ×5: `total=8000; distinct=8000` |

### Before-state, recorded verbatim

```
row 1/3  Summary [356.911s] 5215 run: 5213 passed, 2 failed, 22 skipped
         .floor/2026-09-05T13-05-30Z/
         FAIL client_sees_peer_crashed_not_bare_disconnect
         FAIL an_owner_drop_reaches_the_client_as_severed  ("CLOSED:MUTE" vs "SEVERED")
row 2    4 distinct raise reasons in the select range, 6 sites
row 7/8  distinct=100 on every chaos cell; total=8000; distinct=8000 ×5
```

## ⛔ ROW 3 IS ONE RED AND THAT IS A PASS

The severed red is `CallOutcome::PeerGone` merging `Lost` and `Closed` — **our own debt from the
CallOutcome stone**, not something this stone touches. **`5214 passed, 1 failed` is the target.**
A green floor would mean something else changed and needs explaining.

## ⛔ ROW 4 IS THE ONE THAT CAN GO WRONG QUIETLY

Rows 1–3 can all pass while a `Malformed` carries a raw panic message to a client. **That would
be a worse defect than the raise being removed** — arc 294 exists for it. Check the value, not
just the variant.

## RUNTIME PREDICTION

40–70 min. Six sites, four mappings, three exemplars in the same file. The care is the scrub.

## TRAP DOORS, NAMED

1. **Returning the variant, forgetting the scrub.** STOP-1 / row 4.
2. **Assuming all four variants exist.** The DESIGN claims it from a read; if a site has no home,
   that is STOP-2 and the stone changes rather than a variant being invented.
3. **Treating the remaining red as failure** and "fixing" the severed case here. Out of scope; it
   would confound both.
4. **A green floor.** Unexpected — investigate before celebrating.
