# SCORE — the death notice is not a malformed frame

**FLOOR GREEN.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`src/runtime.rs` + `src/kernel/peer.rs` (predicates `pub(crate)`) + `wat/service.wat` + `circuit.wat`.
A peer's death is a death at every tier; the client learns no cause it should not have.

```
Summary [ 373.919s] 5215 tests run: 5215 passed (4 slow), 22 skipped
```

`.floor/2026-09-05T22-09-10Z/`

Both remaining reds are green. `CallOutcome::PeerGone` is gone.

## THE THREE CHANGES

1. **Process tier.** Select's EDN-decode and UTF-8 arms return `ServiceEvent::Lost` (was `Malformed`), matching `recv:25047`. The reserved sentinel is recognized on the raw wire *before* decode — the same door as `Peer::recv_wire` — so a crashed frame and a severed frame are not collapsed into "undecodable."
2. **Thread tier.** Select checks `is_peer_crashed_sentinel` / `is_peer_severed_sentinel` before treating a `Value` as a `Message`. Severed is the only path that can produce the variant the severed test names.
3. **`CallOutcome`.** `PeerGone` splits into `Lost [cause <- LociDiedError]` and `Closed`. Path A (`send-recv-form`) and Path B (`send_recv_ast`) map `Lost` → `RecvOutcome::Lost(cause)` and `Closed` → `Closed`. The four circuit sites redial both, identically.

`ServiceEvent::Lost` still carries `Failure`. `lost-cause-from-select` maps that Failure to `LociDiedError`: the canonical Severed class string (`LociDiedError/message` of Severed, the same string `eval_died_error_to_failure` mints) survives as the payload-free variant; everything else is `Disconnected`. Arc 294's inner Path B scrub is then identity on both.

## STOP-1 — process tier CAN distinguish; I did not invent a cause

The brief said: if process tier cannot tell a severed peer from any other undecodable frame, STOP. Decode-fail alone cannot. The raw wire string can — `recv_wire` already does an exact compare against `PEER_CRASHED_SENTINEL` / `PEER_SEVERED_SENTINEL` before any EDN parse. Select now uses that same compare. A generic undecodable frame is still `Lost` with `"select EDN decode failed"` → client `Disconnected`, not a fabricated `Severed`.

## FINDING — send-Lost used to eat the death notice

The first severed run after change 1–3 was `LOST:Disconnected`, not `CLOSED:MUTE`. Progress: flatten-to-Closed was gone. Remaining: `call-by-deadline` returned on `SendOutcome::Lost` without selecting. The sentinel rides the **recv** channel; send sees only EPIPE to a dropped receiver. Owner-drop therefore never reached the intercept.

Fix (in `call-by-deadline`, not a new surface): `SendOutcome::Stopped` still aborts; `Sent` / `Closed` / `Lost` all select. The death notice is read. RST stayed green (process-tier send still delivers, then select sees the crash sentinel).

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ rst green | ✅ `client_sees_peer_crashed_not_bare_disconnect` passes — `Lost`, not `CLOSED` |
| 2 | ★★ severed green | ✅ `an_owner_drop_reaches_the_client_as_severed` passes — `SEVERED`, not `CLOSED:MUTE`. Control `a_held_handle_still_replies` still `REPLIED` |
| 3 | ★★ floor `5215 passed, 0 failed`, 22 skipped | ✅ `Summary [ 373.919s] 5215 tests run: 5215 passed (4 slow), 22 skipped` |
| 4 | ⛔ the scrub holds | ✅ newly returned select Failures are `message_only_failure` of a class string. Client-facing Lost is `Severed` (payload-free) or `Disconnected` (payload-free). RST sentinel `RST-BASELINE-SENTINEL-7731` does not leak |
| 5 | blast radius | ⚠ three named files plus `src/kernel/peer.rs` (`is_peer_*_sentinel` `pub(crate)` so select calls the ONE recognizer). No codemod, no corpus edits. `189+/81-` |
| 6 | `PeerGone` is gone | ✅ `grep -c PeerGone` across `.wat` is **0**; rust sources **0**. Four circuit sites match `Lost`/`Closed` |
| 7 | chaos unaffected | ✅ check/mark/recv/ack-drop ×3 each: `distinct=100` |
| 8 | rate-0 | ✅ circuit ×5 `total=8000; distinct=8000; dup=0` |
| 9 | timings | report only. Quiet publish: `51301 50822 51022 51460 50290` |

Chaos (verbatim `distinct=100`):

```
DROP-CHECK-TINY … distinct=100  ×3
R2 AFTER-TINY   … distinct=100  ×3
DROP-RECV-TINY  … distinct=100  ×3
DROP-ACK-TINY   … distinct=100  ×3
```

Circuit ×5 (quiet, after floor): `total=8000; distinct=8000; dup=0` every run.

## ROW 4 — the value, not just the variant

Select returns `message_only_failure` of:

| class string | becomes (client) |
|---|---|
| `"service severed: its owner released the service handle"` | `LociDiedError::Severed` (payload-free; canonical `/message`) |
| `"select peer crashed"` | `Disconnected` |
| `"select EDN decode failed"` | `Disconnected` |
| `"select (process tier): peer message is not valid UTF-8"` | `Disconnected` |

No decode `e`, no `io_err`, no panic envelope is interpolated. Path B's arc-294 scrub is unchanged and still passes `Stopped`/`Severed`.

## NOT TOUCHED

`poll:27124` (admin). ServiceEvent / LociDiedError variant sets. Rung-3 migration. Tests. Perf.

## KEEP FROM LAST STONE

The other eight select conversions (Shutdown sites, io_uring Lost, setup `MalformedForm`s) stand. Only the mis-cited decode/UTF-8 arms flipped from `Malformed` to `Lost`.

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK. FLOOR GREEN.** Every row re-run by me.

| # | my result | |
|---|---|---|
| 1/2 | both reds gone — no `FAIL` lines in my floor | ✅ |
| 3 | **`Summary [ 366.205s] 5215 tests run: 5215 passed, 22 skipped`** — `.floor/2026-09-05T22-25-51Z/` | ✅ |
| 4 | every new cause is `message_only_failure(class)`; **no `format!` interpolation on any added line** | ✅ |
| 5 | 4 files, `189+/81-` — the fourth is `src/kernel/peer.rs`, flagged by the executor | ⚠→✅ |
| 6 | `PeerGone` → **0** in `.wat`, **0** in `src/` | ✅ |
| 7 | check/mark/recv/ack-drop ×3 each — `total=100; distinct=100; dup=0`, **all twelve** | ✅ |
| 8 | rate-0 ×5 — `total=8000; distinct=8000; dup=0` | ✅ |
| 9 | publish `49739 50975 51300 52904 53458` — reported | ✅ |

## ★★ ROW 4 IS PROVEN BY THE FLOOR, NOT BY MY READING

`probe_arc278_rst_peer_notify_baseline.rs:51` asserts

```rust
!text.contains("RST-BASELINE-SENTINEL-7731")
```

**That assertion is in the green floor.** The panic text provably does not reach the client — arc
294 holds by test, not by inspection of a diff. That is the strongest form this row could take.

## ★ STOP-1 WAS ANSWERED BETTER THAN THE BRIEF ASKED

I asked: if the process tier cannot distinguish a severed peer, **stop and report** — inventing a
process-tier sever would be fabricating a cause.

Grok neither invented one nor stopped. **`recv_wire` already compares the raw wire string against
`PEER_CRASHED_SENTINEL` / `PEER_SEVERED_SENTINEL` before any EDN parse**, and select now uses that
same door. A generic undecodable frame is still `Lost`/`Disconnected` — **not a fabricated
`Severed`.** The recognizer was found, not built.

## ★★ THE FINDING I WOULD HAVE SPENT A DAY ON

After all three changes, severed still failed — as `LOST:Disconnected`, not `CLOSED:MUTE`. The
flatten was gone; something else remained:

> **`call-by-deadline` returned on `SendOutcome::Lost` without selecting. The sentinel rides the
> RECV channel; send only sees EPIPE to a dropped receiver — so an owner-drop never reached the
> intercept.**

Fix: `Sent` / `Closed` / `Lost` all select; only `Stopped` aborts. ★ **A death notice you never
read because you gave up on the send path** is exactly the class this arc keeps finding, and it
was two layers below where I was looking.

## ROW 5 — THE DEVIATION IS THE RIGHT CALL

Four files, not three: `is_peer_*_sentinel` became `pub(crate)` so select calls **the one
recognizer** rather than duplicating the compare. `peer.rs:414` calls itself *"the ONE place to
recognize it"* — keeping that true is better than honouring my file list. Flagged by the
executor rather than slipped in.

## ★★★ WHAT IS NOW TRUE

**Rung 3 is complete and the floor is green.** No generated client call can hang; `select`
returns values instead of raising at every peer-wire site; a peer's death is a death at every
tier, carrying `Severed` where it is real and reason-free everywhere else; and `TimedOut` is a
matchable arm across 526 sites.

★ The two reds were not damage from the migration. One was a latent `select` defect the migration
**exposed** — unreachable until something selected on a panicking peer. The other was **my own**,
introduced in the CallOutcome stone and graded there as *"not a defect today, but the shape
permits one."* Both are closed.
