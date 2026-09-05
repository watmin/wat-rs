# BRIEF — select returns what it sees

Make `select`'s process tier return a matchable `ServiceEvent` everywhere it currently raises.
`src/runtime.rs` only.

Read `DESIGN-select-returns-what-it-sees.md` first. **Every fix has an exemplar in this same
file — copy them, do not design.**

## READ IN ORDER

| room | why you are there |
|---|---|
| `src/runtime.rs:27185-27200` | **`poll`'s client-message decode.** `Ok(msg) => Message[idx,msg]`, `Err(e) => Malformed[idx,cause]`. **This is the shape.** |
| `src/runtime.rs:25040-25050` | **`recv`'s decode.** `Err(e) => recv_outcome_lost(...)`, and the comment naming the `#wat.kernel/ProcessPanics` envelope |
| `src/runtime.rs:26067` | **`select` returning `ServiceEvent::Shutdown`** — its own precedent for the three shutdown raises |
| `src/runtime.rs:6541-6560` | **the scrub** and arc-294's ruling. Every value you newly return must be reason-free |
| `:26095`, `:26320`, `:26345` | the decode / io_uring / UTF-8 raises |
| `:25919`, `:26109`, `:26251` | the three `SelectOutcome::Shutdown => Err(...)` raises |

## THE MAPPING

```
EDN decode failed        -> ServiceEvent::Malformed [idx, message_only_failure(...)]
peer msg not valid UTF-8 -> ServiceEvent::Malformed [idx, message_only_failure(...)]
io_uring error           -> ServiceEvent::Lost      [message_only_failure(...)]
interrupted by shutdown  -> ServiceEvent::Shutdown          (×3 sites)
```

## BLAST RADIUS

`src/runtime.rs` only. **No `wat/`, no `.wat`, no codemod, no test edits** — the tests that
should change behaviour are the ones already asserting the correct contract.

## STOP TRIGGERS

- **STOP-1** — if any newly returned value carries an **unscrubbed** cause, STOP. Arc 294:
  a client learns no server internals. A panic message reaching a client through `Malformed` is
  a worse defect than the raise you are removing.
- **STOP-2** — if a raise site turns out **not** to have a matching `ServiceEvent` variant, STOP
  and report which. The DESIGN's claim is that all four already do; if that is wrong, the stone
  changes.
- **STOP-3** — if returning `Shutdown` at `:25919`/`:26109`/`:26251` changes behaviour for a
  caller that today relies on the raise, STOP and report the caller. `select:26067` already
  returns it, so this should be making one function agree with itself.
- **STOP-4** — do not touch `CallOutcome::PeerGone`, `poll:27124` (admin), or any `.wat` file.

## THE EXPECTED REMAINDER

⛔ **This stone takes the floor from 2 reds to 1, not to green.**
`an_owner_drop_reaches_the_client_as_severed` is a different stone. **Report 1 red as the
expected remainder, not as a failure.**

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-no-client-call-can-hang.md` and its grading — the strike that uncovered this, including
the checkpoint commit `276f989dc` you can revert to freely.
