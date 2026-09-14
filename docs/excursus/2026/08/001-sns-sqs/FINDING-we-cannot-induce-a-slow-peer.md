# FINDING — we can induce a peer that is DEAD or SILENT, never one that is SLOW

**Found 2026-09-14**, answering the builder's standing direction:
*"all of the chaos engineering work is meant to simulate networks doing networking things … let's
continue to find ways to inject chaotic networking into our circuit and see where else we need to
build resiliency."*

## 1. The injector inventory reduces to THREE primitives

Every chaos knob in the circuit's world — `drop-recv-bp`, `drop-ack-bp`, `drop-check-bp`,
`drop-mark-bp`, `store-drop-reply-bp`, `store-die-bp`, `disrupt-bp`, and `chaos-bp` which arms the
rest — is one of:

| primitive | what the caller observes |
|---|---|
| **suppress a reply** | `TimedOut` after the client's deadline |
| **kill the process** | `Lost` |
| **oversize a frame** | `Malformed` |

**All three are instantaneous.** ⛔ **There is no latency injector.** Measured: zero matches for
`delay-bp` / `latency` / `slow-bp` / `late-bp` / `stall-ms` / `jitter-bp` as a knob anywhere in
`wat/`, `wat-scripts/{queue,topic,fanout,query}` — the only `latency` hits are prose about *measured*
latency in comments.

So a peer can be **dead** or **silent**. It cannot be **slow**.

## 2. ⭑⭑ Why slow is the one that matters, and it is already written down as a hazard

Dead and silent are the easy cases: both sides agree something failed. **Slow is the case where the
two sides believe different things** — the reply arrives *after* the caller gave up. Both halves then
act on incompatible views, and nothing in the transport says so.

That exact hazard is already reasoned about in this campaign, and never induced.
`the-gate-methods-face-an-outcome/DESIGN.md` ruled **re-recv over re-ask** for precisely it:

> ⛔ **But a re-ask can leave a surplus ack queued on the lineage peer.** If the first ack was merely
> *slow* rather than lost, re-sending produces two, the loop reads one, and the second stays on the
> wire. A later `<S>/stop` then does `send Admin::Stop; recv` and picks up the stale `PeersAllowed` →
> `Message(other)` → `"defservice stop: expected Status::Stopped"`. **That is character-for-character
> the frame-desync class that produced today's crash**, and it is silent until it detonates somewhere
> else.

★ A contract decision was made to avoid a failure mode **no instrument in this tree can produce.** The
ruling may well be right; it has never been tested, and "merely slow rather than lost" is exactly the
state we cannot construct. Every deadline in the system — `call-by-deadline`, `recv-by-deadline`, the
10 s generated client deadline, the inbox and sub visibility timeouts — has only ever been exercised
against peers that answered promptly or not at all.

⚠ And the one time a deadline *did* fire in anger, it killed a process: the publisher at 1 ms × p=3
(`the-benchmark-has-more-than-one-publisher/SCORE.md`, the dagger cells) — *"the topic goes silent past
that deadline, `-run` asserts TimedOut, the process dies."* That was reached by **accident, under
load**, not by an injector.

## 3. ⛔ The chaos harness is 100 % PROCESS tier — the thread tier has never been chaos-tested

Measured inside `:fanout::run-with` (`wat-scripts/fanout/circuit.wat:2876`–`:3300`), the harness every
chaos run goes through:

```
spawn::process = 8      spawn::thread = 0
```

The 12 `spawn::thread` sites in that file are all in separate small entry points (`:3882`, `:3931`,
`:3988`) — demo/regression mains, not the chaos circuit.

The builder's framing of the two tiers is what makes this load-bearing:

> *"services are meant to be network addressable … we expose networking via unix domains (not file
> system access, yet) and 'internal services' using threads that 'look like IPC' but aren't."*

**A thread peer cannot have latency, a full socket buffer, or a partition.** So the two tiers have
genuinely different fault surfaces, and the chaos suite exercises exactly one of them:

- **process tier** — the faults we inject; the ones a real network will also produce.
- **thread tier** — *"looks like IPC but isn't."* Never chaos-tested. Whatever assumptions it holds
  because the transport is instant and infallible are untested assumptions, and they will be the ones
  that break when a thread service is later moved onto a socket.

⭑ The asymmetry runs both ways: the thread tier is a shared address space with no serialization
boundary, which is where **races** live — and this campaign has already recorded that a quiet box
*"measures timing while hiding races"* and that a real race (`<S>/stop` returning on the ACK, not the
reap) survived nine green floors.

## 4. What a UDS transport can do that we never ask it to

Beyond latency, and named rather than swept: **backpressure** (`TrySendOutcome::WouldBlock` FIRES in a
scratch probe and has never fired in the circuit — the peer accepts but does not read, the socket
buffer fills); **a partition that heals** (unreachable for a window, then back — our only terminal
knob, `store-die-bp`, is permanent); **half-close**; and **byte corruption distinct from oversize**.

⚠ Reordering is NOT on this list: `SOCK_STREAM` cannot reorder. That cell is empty by physics, which
is a wall holding, not a gap — the distinction `the-crash-surface-is-enumerated` §5 was corrected to
make.

## The shape of the next stone (the builder's ruling, not a proposal to act on)

**A latency injector — `delay-bp` + `delay-ms`, on the shape `faulting-store.wat` already proved.**
A rate, a fire counter at the delay site (never at the dice roll), and a wait that arrives by timer
channel, not a sleep (`mora`). It buys, at minimum:

1. the first real exercise of every deadline in the system,
2. the **surplus-ack desync** above becoming constructible instead of reasoned-about,
3. the publisher-death cell reachable **on purpose** rather than by load accident.

And separately, **whether the chaos harness should run at both tiers** is a bigger question than a
knob: it is the difference between testing our transport and testing our services.
