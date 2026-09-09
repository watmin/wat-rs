# DESIGN — a client has a deadline

**Stone T1.** Every client call in this system waits forever for a reply that may never come. This
gives it a deadline, and on expiry it discards the connection and retries.

> Builder, 2026-09-04: *"clients must have a deadline on when they expect an answer by … they
> discard the connection and try again if they don't get it."*

## WHY

We built `Closed`-recovery and reconnection on the client side (3c-pre, 17 arms) and then **never
gave the client a way to notice it needed them.** A server that goes silent — deferring a reply it
never settles — hangs its caller permanently. Measured today:
`probe-reply-drop-is-userland.wat` exits 124.

That is the same unfalsifiable-hang class this campaign spent six stones removing, sitting in the
one place we had not looked: the client's own `recv`.

## ★ THE MECHANISM IS ALREADY IN THE TREE — probed, not proposed

`wat-scripts/scratch-pad/probe-client-deadline-via-select.wat`, **3/3**:

```
fast=reply ; never=TIMED-OUT ; verdict=CLIENT-DEADLINES-ARE-EXPRESSIBLE
```

**No substrate change.** A client does what the serve loop does — races its reply peer against a
timer in one `select` set — by laundering the timer through an annotated vector
(`service.wat:1618-1623`, *"a check-time-only detour"*; type params are erased at runtime), with the
orientation inverted: the loop is `[Reply Op]`, a client is `[Op Reply]`.

⚠ **An earlier draft of this design proposed giving `recv` a deadline.** That was a *second* timeout
implementation beside a working one — the crossbeam and io_uring select paths and their timerfds are
already correct. Rejected, and recorded so it is not re-proposed.

## ⛔ THE ONE CONTRACT DECISION

**A client's deadline must exceed any server-side wait on the same call.**

`Queue/receive` carries `:wait (Wait::UpTo …)` — the server legitimately holds the request. A client
deadline shorter than that **manufactures the timeout it exists to detect**, and every long poll
becomes a spurious reconnect.

So: deadline = *server wait + slack*, and on a call with no server-side wait, any positive deadline
is honest. **If a deadline can fire while the server is behaving correctly, the stone is a fault
injector wearing a fix's name.**

## ★ AND THIS IS WHAT PRODUCES THE DUPLICATE

Follow it: a client times out, discards, redials, and **retries a request the server may already
have processed.** That retry is a duplicate by construction.

`seen-dups` has never moved outside a deterministic gate. **This stone is the first thing that can
move it under ordinary operation** — before R2, and without any fault injected at all. R2 then makes
it *deterministic* rather than timing-dependent.

## WHAT IT DELIVERS

A client-side call shape:

1. `send` the op
2. `select [peer, timer]` — **timer at the peer's tier**
3. **reply** → done · **timeout** → discard the peer, redial the address, retry, bounded
4. every `ServiceEvent` arm named

### ⛔ TIER IS NOT THE CALLER'S — it is the RACED PEER'S

```
malformed :wat::kernel::select form: peers[1]: mixed-tier select set
(a non-socket peer among socket peers) is not a representable-good state
```

A process-locus service's connection is a **socket** peer; a `PeerKind::thread` timer cannot join
that set. **The serve loop never has to choose** — it uses `(Env/peer-kind (env))` and matches by
construction. A client must pick the tier of the peer it is *racing*. This cost the probe two
attempts.

### `ServiceEvent` has more arms than the obvious three

`Message` · `Closed` · `Lost` · **`Shutdown` · `Admin` · `Connection` · `Malformed` · `Rejected`**.
The probe took a wildcard and said so. **A shipped client face names each** — that is the wall that
turned 17 fatal `Closed` arms into reconnections.

## SCOPE

`wat-scripts/fanout/circuit.wat` — the helper, and adoption at **`Seen/claim`** only.

That call has **no server-side wait**, so its deadline is unambiguous; and it is the exact call R2
will drop. Adopting everywhere at once is a blast radius with no evidence behind it yet.

## OUT OF SCOPE = REJECTED

- **`Queue/receive`.** It carries `:UpTo`; its deadline must be `UpTo + slack`, which is a second
  decision needing its own measurement. **S35.**
- **`Queue/ack` / `Queue/send`.** Same shape as `claim`, no server wait — adopt after `claim` proves
  the pattern. **S35.**
- **`recv` gaining a deadline.** Rejected above.
- **`Never` as the honest timer input.** It would *delete* the laundering rather than use it — a
  substrate stone, and a better one, but not this one. **S36.**
- **R2, the server drop.** After this.

## THE PROOF

1. **★★ A silent server times out instead of hanging.** The committed probe, still green.
2. **★★ The retry reaches a FRESH connection.** After the discard, show the redial produced a new
   peer and the retry landed on it — not the dead one. ⛔ Reusing the discarded peer is an infinite
   loop that looks like a hang, which is the failure this arc removed six times.
3. **★ `seen-dups`** — report it, five runs. ⛔ **Any value is the result.** Non-zero means the retry
   produced a duplicate and the consumer absorbed it, which is the thing we have been chasing since
   R69. Zero means the timeout never fired in normal operation, which is also correct.
4. **The invariant.** `distinct=8000; dup=0`, five runs.
5. **The deadline does not fire in normal operation** — if it does, it is too short, and the fix is
   a longer deadline, never a faster server.
6. **The floor**, `5214/5214`.
