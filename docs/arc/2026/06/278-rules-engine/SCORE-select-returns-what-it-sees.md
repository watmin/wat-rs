# SCORE — select returns what it sees

**NOT 5214/1.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`src/runtime.rs` only. `select` no longer raises on the peer wire.

```
Summary [ 371.223s] 5215 tests run: 5213 passed (4 slow), 2 failed, 22 skipped
```

`.floor/2026-09-05T21-16-45Z/`

The raise is gone. Both remaining reds are `CallOutcome::PeerGone` → `RecvOutcome::Closed`. STOP-4 forbade touching that merge.

## ROW 1 — rst's raise is gone; the marker is still CLOSED

Before: `got Err: … select (process tier) EDN decode failed: … keyword begins with ::`

After, captured on this floor, **do not re-run**:

```
got: Ok(String("CLOSED"))
```

`is_ok()` holds — a raise no longer unwinds past the reader. The second assertion still wants `LOST`. Generated `/boom` goes through `call-by-deadline`, which maps `ServiceEvent::Malformed`/`Lost` idx 0 to `CallOutcome::PeerGone`, and Path A maps PeerGone to `RecvOutcome::Closed`. That is the CallOutcome stone's merge. STOP-4: do not touch it.

DESIGN expected rst green and severed as the one remainder. Measured: **both reds are that remainder.** The select raise is not among them.

## ROW 2 — zero peer-event raises in the select range

Converted (copy poll:27194 / recv:25047 / select:26067):

| site | was raise | now returns |
|---|---|---|
| thread spawned `:25919` | Shutdown | `ServiceEvent::Shutdown` |
| process spawned io_uring `:26030` | MalformedForm | `Lost [0, message_only_failure("select io_uring error")]` |
| process spawned EDN decode `:26095` | MalformedForm interpolating `e` | `Malformed [idx, message_only_failure("select EDN decode failed")]` |
| process spawned Shutdown `:26109` | Shutdown | `ServiceEvent::Shutdown` |
| Peer InMemory Shutdown `:26251` | Shutdown | `ServiceEvent::Shutdown` |
| Peer Fd io_uring `:26320` | MalformedForm interpolating `io_err` | `Lost [0, message_only_failure("select io_uring error")]` |
| Peer Fd UTF-8 `:26329` | MalformedForm | `Malformed [idx, message_only_failure(constant)]` |
| Peer Fd EDN decode `:26345` | MalformedForm interpolating `e` | `Malformed [idx, message_only_failure("select EDN decode failed")]` |
| Peer Fd Shutdown `:26370` | Shutdown (DESIGN listed three; this was a fourth) | `ServiceEvent::Shutdown` |

Remaining `MalformedForm` in the select function: empty vector, wrong tier, peer already closed, mixed-tier — **setup**, not a peer event.

io_uring has no peer index; Lost requires `idx`. Used `0`. Cause does **not** interpolate `io_err` (STOP-1).

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ rst green | ⚠ raise gone; marker `CLOSED` via PeerGone. Floor still names this test |
| 2 | ★★ 0 peer-event raises | ✅ |
| 3 | floor `5214 passed, 1 failed` | ⚠ **5213 passed, 2 failed** — rst + severed, both PeerGone. Expected remainder was 1 |
| 4 | scrub holds | ✅ newly returned Failures are `message_only_failure` of a class string; decode/`io_err` text is not interpolated (RST sentinel cannot ride `e`) |
| 5 | blast radius | ✅ `src/runtime.rs` only (`121+/99-`) |
| 6 | three shutdown sites return Shutdown | ✅ four sites (including Fd `:26370`) |
| 7 | chaos | ✅ check/mark/recv/ack-drop ×3: `distinct=100` every run |
| 8 | rate-0 | ✅ circuit ×5 `total=8000; distinct=8000; dup=0` |

Chaos (verbatim `distinct=100`): check ×3, mark (`r2_drop_after_tiny`) ×3, recv ×3, ack ×3.

Circuit publish (report only; ran concurrent with chaos): `64111 55780 52240 54048 53373`.

## THE TWO REDS (same arm)

**rst** — `Ok(String("CLOSED"))`. Not a raise.

**severed** — `left: "CLOSED:MUTE" right: "SEVERED"`. Named remainder. Untouched.

## NOT TOUCHED

`CallOutcome::PeerGone`. `poll` admin decode (`:27124`). `.wat`. Tests.

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK on its contract.** The stone said *`select` stops raising on the peer wire*. It does.
**My rows 1 and 3 predicted a consequence that did not follow, and that is my error, not the
executor's.**

| # | my result | |
|---|---|---|
| 1 | `got: Ok(String("CLOSED"))` — was `Err: … MalformedForm … EDN decode failed`. **The raise is gone**; the test now fails on the marker | ⚠ my row |
| 2 | 9 peer-event raises converted; remaining `MalformedForm` in the select fn are setup (empty vector, wrong tier, already closed, mixed-tier) | ✅ |
| 3 | `Summary [ 362.856s] 5213 passed, 2 failed` — `.floor/2026-09-05T21-32-16Z/` | ⚠ my row |
| 4 | **every newly returned cause is `message_only_failure("<class string>")` — constants, no `e`/`io_err` interpolation.** No peer text rides out | ✅ |
| 5 | `src/runtime.rs` only, `121+/99-` | ✅ |
| 6 | four shutdown sites return `ServiceEvent::Shutdown` (DESIGN said three; `:26370` was a fourth) | ✅ |
| 7 | check/mark/recv/ack-drop ×3 each: `distinct=100` | ✅ |
| 8 | rate-0 ×5: `total=8000; distinct=8000; dup=0` | ✅ |

★ **Row 4 was the one that could pass quietly while leaking, and it holds by construction** —
the causes are constant class strings, so the `#wat.kernel/ProcessPanics` envelope cannot ride
out through `Malformed`. Arc 294 is intact.

## ★★ THE RESULT IS BETTER THAN THE COUNT

The floor still reads 2 red, but the tree is materially better: **both reds now have ONE cause.**

```
rst      got: Ok(String("CLOSED"))          ← was a raise; now a marker
severed  left "CLOSED:MUTE" right "SEVERED"
```

Both are `CallOutcome::PeerGone → RecvOutcome::Closed`. The select raise was hiding a second
defect underneath it; removing the raise exposed that rst was failing on **two** things.

## ⚠ MY ROWS WERE WRONG IN A WAY I HAVE NOW REPEATED

Row 1 said *"rst passes."* Row 3 said *"5214 passed, 1 failed."* Both predicted a **consequence**
of the fix rather than gating what the fix **controls**.

★ The stone controls whether `select` raises. It does not control whether `rst` goes green —
that depended on a second, separate defect I had already identified and explicitly put out of
scope in the same document. **I gated the hope, not the deliverable.**

★★ This sharpens the rule the last SCORE recorded. *"State what must HOLD, not what was
observed"* is not enough. **A row must gate what the stone controls, not what it expects to
follow.** Row 1 should have read *"the raise is gone — `is_ok()` holds"*, which is exactly what
the executor measured and reported.

## THE NEXT STONE FIXES BOTH REDS

Split `CallOutcome::PeerGone` into `Lost [cause]` and `Closed`, and stop the generated method
flattening a `Severed` death into a clean close. **That is my debt from the CallOutcome stone** —
graded there as *"not a defect today, but the shape permits one"* — and it is now the sole cause
of every red on the floor.
