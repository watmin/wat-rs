# A peer that is merely slow

**The chaos gap, already measured and already ruled.** Excursus `001-sns-sqs`. Read first:

- `../FINDING-we-cannot-induce-a-slow-peer.md` — **the measurement this stone executes.** It
  inventories every chaos knob in the tree and names the shape of the fix. That shape is the
  builder's ruling, not a proposal: do not redesign it.
- `../the-gate-methods-face-an-outcome/DESIGN.md` — the contract decided *because of* a failure
  mode nothing here can produce.

## The sentence

> **Induce a peer that is SLOW. Today every injector makes one that is DEAD or SILENT.**

## Why this and not something else

The FINDING reduces every knob in the codebase — `drop-recv-bp`, `drop-ack-bp`, `drop-check-bp`,
`drop-mark-bp`, `store-drop-reply-bp`, `store-die-bp`, `disrupt-bp`, `chaos-bp` — to **three
primitives**, and all three are instantaneous:

| primitive | observed |
|---|---|
| suppress a reply | `TimedOut` |
| kill the process | `Lost` |
| oversize a frame | `Malformed` |

⭐ **Slow is the case where the two sides believe different things** — the reply arrives *after* the
caller gave up. Dead and silent are the easy cases; both ends agree. And the hazard is not
hypothetical: a contract decision (re-recv over re-ask) was *already made* to avoid a surplus-ack
desync that begins **"if the first ack was merely slow rather than lost"** — a state no instrument
in this tree can construct. The ruling may be right. It has never been tested.

⚠ And every deadline in the system — `call-by-deadline`, `recv-by-deadline`, the 10 s generated
client deadline, the queue's visibility timeouts, `collect-deadline-ms` — **has only ever faced
peers that answered promptly or not at all.**

## The shape — RULED, copy it

> *"A latency injector — `delay-bp` + `delay-ms`, on the shape `faulting-store.wat` already proved.
> A rate, a fire counter at the delay site (never at the dice roll), and a wait that arrives by
> timer channel, not a sleep."*

Both exemplars are on disk — **cite them, do not reinvent**:

- **injector shape**: `wat-scripts/query/faulting-store.wat`. Its header carries the trap in
  capitals: *"⛔ Count at the suppression site (None / Stop), never at the dice roll."* A counter on
  the roll counts intentions; a counter at the site counts events, and only the second can make a
  green non-vacuous.
- **the wait**: `(:wat::kernel::after peer-kind duration msg)` returns a **timer peer**, already
  used at `wat/queue.wat:1643` and `:1699`. ⭐ There is **no sleep intrinsic in the tree** — zero
  `wat_intrinsic(":wat::…sleep")` — so the timer channel is not merely preferred, it is the only
  thing available. A blocking sleep would also stall the serve loop it is supposed to slow, which
  is the bug and not the fault.

## The work

### 1. The knob

`delay-bp` (rate) + `delay-ms` (magnitude) on the faulting store, mirroring its siblings' plumbing
exactly. ⛔ Fire counter **at the delay site**.

### 2. ⭐ Make the reasoned-about hazard constructible

The point is not the knob, it is what it reaches. At minimum, drive **the surplus-ack desync**: an
ack that is *slow, not lost*, so a re-ask leaves a second ack on the wire. `the-gate-methods-face-
an-outcome` chose re-recv over re-ask on exactly that reasoning — ⛔ **report whether the ruling
holds under a fault that can now actually be made.** If it does, that is the first evidence for a
contract decided without any; if it does not, that is a finding worth more than the injector.

### 3. Exercise a deadline on purpose

Pick one deadline and make it fire because a reply was LATE rather than absent, and say which.
⚠ A late reply that arrives after the caller moved on is a different event from no reply at all —
**show the difference**, do not assert it.

### 4. Non-vacuity, and it is the row most likely to be faked

A delay injector's failure mode is a green that never delayed. Report the **fire count from the
delay site** and the observed wall-clock difference. ⛔ A test that passes with `delay-bp=0` and
also with `delay-bp=10000` has measured nothing.

## Scope wall

⛔ **Not in this stone**, each named because they are real and each is its own:

- **Pointing the injectors at the bracket PEER path.** All three primitives are queue/circuit-
  scoped today (`drop-recv-bp`/`drop-ack-bp` live only in `wat/queue.wat`), so a bracket on the peer
  path can be faulted *only* by killing a runner. That is why the `Malformed` re-dispatch fix
  (`c4026f99e`) is guarded by a **source pin rather than an induced fault**. Next stone.
- **The thread tier.** The chaos harness is measured at `spawn::process = 8, spawn::thread = 0` —
  100% process tier. *"A thread peer cannot have latency, a full socket buffer, or a partition"*, so
  the tiers have genuinely different fault surfaces and one has never been chaos-tested. Whether the
  harness should run at both is *"a bigger question than a knob"*.
- **Backpressure, a partition that heals, half-close, byte corruption distinct from oversize** —
  named in the FINDING §4, all still absent.
- ⚠ **Reordering is NOT a gap** — `SOCK_STREAM` cannot reorder. That cell is empty by physics: a
  wall holding, not a hole.

## The four questions

- **Obvious** — ✅ slow is the ordinary network failure and we cannot make one; the gap is already
  measured and the shape already ruled.
- **Simple** — ✅ a rate and a timer peer on plumbing that exists twice over. The honest risk is
  row 2, not row 1.
- **Honest** — ✅ it converts a contract decided from reasoning into one that can be tested, and
  row 2 permits the answer "the ruling does not hold".
- **Good UX** — ✅ a knob on a test double; no surface changes anywhere.
