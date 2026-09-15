# SCORE — a fired deadline hands back a live peer; the publisher was not touched

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `c54a67560` (DRAWN). Did not commit.

⛔ The headline is that **`call-by-deadline` re-establishes the handle when the timer
wins.** `DeadlineFired` now means: timed out, AND the handle is good. If it could
not be, the answer is `Lost` (or `Closed` when the live connection already EOFd).
The publisher is stone 3. `recv-by-deadline` is **unmeasured**.

```
     Summary [ 562.987s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T04-25-59Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5249 → 5251** (+2 tests). Not a shrink. Clippy **0**. NORUN **0**.
Happy path `distinct=8000;dup=0`.

---

## ⭑⭑ THE HEADLINE — both tiers, every call gets its own tag

Process (`probe-a-slow-peer-desyncs-the-next-call.wat`):

```
call-1(tag=11,dl=100)=DeadlineFired
call-2(tag=22,dl=5000)=Answered(tag=22) sent=22
call-3(tag=33,dl=5000)=Answered(tag=33) sent=33
call-4-GENERATED=Ok(tag=44) sent=44
```

Thread (`probe-a-slow-peer-desyncs-thread-tier.wat`) — **byte-identical**:

```
call-1(tag=11,dl=100)=DeadlineFired
call-2(tag=22,dl=5000)=Answered(tag=22) sent=22
call-3(tag=33,dl=5000)=Answered(tag=33) sent=33
call-4-GENERATED=Ok(tag=44) sent=44
```

Today those were `Answered(tag=11) sent=22` / `Answered(tag=22) sent=33` /
`Ok(tag=33) sent=44`. The shift is gone on **both** tiers, including the
generated client method (which still maps `DeadlineFired → TimedOut` and was
not rewritten — the cell swap is why it works).

The original binding serves after the swap: call-2 uses the same `c`.

---

## STOP-1 — the swap is observable

```
before=0;first=DeadlineFired;after=1;second=Answered:22
```

`(:wat::kernel::redials peer)` is 0 at construction and 1 after a successful
re-establish. A silent reconnect would have left that counter at 0.

---

## Row 3 — a dead peer is not a repair

Stop-then-call on the live connection is **`Closed`** (select idx 0 saw EOF
before the timer). Not `DeadlineFired`. Measured, not assumed.

A failed **redial** (`connect` Refused/Rejected/Failed, or `dialed-from` is
`None`) answers **`Lost Disconnected`**. That arm is the contract for "could
not re-establish"; the stop-then-call path never reaches it because the
existing peer already closed.

STOP-3: `dialed-from == None` is `Lost`, not `DeadlineFired`. After the
thread-dialer fix a client always has `Some`; None is accepted/self/timer.

---

## Stone 1, finished

`ThreadAddress::connect` now stores `Some(self.clone())`. `from_thread` takes
`Option<ThreadAddress>` as a required argument (same latch as `from_socket`).
`dialed-from` returns `Some` for a dialed thread peer. The stone-1 probe's
`client-thread` expected `None` → `Some` (accepted/timer still `None`).

Shipping the redial without this would have repaired only the process half.

---

## WHAT LANDED

- `DialedFrom::{Socket, Thread}` on `Peer`. `redials: u64`.
- `(:wat::kernel::replace-peer dest src) -> i64` — `with_mut`, drops the old
  peer (the stale fd), increments `redials`. Same owning thread as
  `call-by-deadline`.
- `(:wat::kernel::redials peer) -> i64` — peek.
- `:wat::service::deadline-reestablish` — the three timer-won sites in
  `call-by-deadline` call it. Generated-client match at `:2733` **untouched**.
- Fixtures in scratch-pad **not rewritten**. New tests:
  `tests/comms/probe_a_fired_deadline_hands_back_a_live_peer.{rs,wat}`.

Did **not** touch the publisher. Did **not** touch `recv-by-deadline`.

---

## Floor delta / redial cost (rows 6, 11)

**562.987 s** vs baseline **535.833 s**. Delta **+27.2 s** on this box.
The orchestrator's quiet-box run grades.

Redial cost: `redials-and-own-tag` isolated **2.154 s**, of which the
handler delays are 100 ms (deadline) + remainder of the first 600 ms + 600 ms
on the fresh connection. `connect` + `replace-peer` are not isolated below
process spawn; they sit in the gap after the timer wins. A round trip on a
path that just lost one — expected, not a gate.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ process desync gone | every call its own tag, including generated |
| 2 | ⭑⭑ thread desync gone | byte-identical to row 1 |
| 3 | ⛔ dead peer not DeadlineFired | **Closed** (EOF on the live conn). Lost is the failed-redial arm |
| 4 | ⛔ original handle works | `second=Answered:22` on the same binding |
| 5 | ⛔ swap observable | `redials` 0→1 |
| 6 | floor | **5251 passed**, 0 FAIL |
| 7 | clippy | 0 |
| 8 | tests compile | NORUN=0 |
| 9 | 13 match arms untouched | generated client `:2733` unchanged; shape of DeadlineFired unchanged |
| 10 | happy path | `distinct=8000;dup=0` |
| 11 | ⚠ redial cost | 2.154 s isolated cell, delays dominate; connect+swap not isolated |
| 12 | ⚠ recv-by-deadline | **unmeasured**. Not fixed, not vouched |
| 13 | scope | this SCORE's headline. Publisher untouched |
