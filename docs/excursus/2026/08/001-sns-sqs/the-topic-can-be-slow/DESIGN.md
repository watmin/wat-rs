# DESIGN — the topic can be slow

Builder: *"all of the chaos engineering work is meant to simulate networks doing networking things …
let's continue to find ways to inject chaotic networking into our circuit and see where else we need
to build resiliency"* — and, on the publisher: *"we are seeking out and destroying these disgraceful
crashes."*

**Drawn 2026-09-14. NOT STRUCK.**

## WHY — we can induce DEAD and SILENT peers, never a SLOW one

`FINDING-we-cannot-induce-a-slow-peer.md`, measured: every chaos knob in the circuit reduces to three
**instantaneous** primitives — suppress a reply (`TimedOut`), kill the process (`Lost`), oversize a
frame (`Malformed`). Zero latency knobs anywhere.

Slow is the fault that matters most, because dead and silent are the cases where **both sides agree**
something failed. Slow is where they believe different things. And it is not hypothetical:

- ⛔ `FINDING-a-fired-deadline-desyncs-the-connection-forever.md` — measured this session: a fired
  deadline against a merely-slow peer **permanently shifts every later reply on that connection**, and
  the generated client method inherits it and reports `Ok`.
- ⛔ The publisher's death at 1 ms × p=3 (`the-benchmark-has-more-than-one-publisher/SCORE.md`, the
  dagger cells) — *"the topic goes silent past that deadline, `-run` asserts TimedOut, the process
  dies."* Reached **by load accident**, never on purpose. Its arm is `circuit.wat:2270`, whose own
  message reads *"recv: timed out — **the peer is alive and silent**"* — the slow case, named in the
  code, unreachable by any instrument.

⭑ **A crash we have observed and cannot reproduce is not yet a bug we can fix.** This stone makes it
reproducible.

## The mechanism is already proven

`wat-scripts/scratch-pad/probe-a-slow-peer-desyncs-the-next-call.wat` (committed, `9d887547b`) parks a
**forked process-tier service handler** on a timer channel and measures the consequence. That settles
the two things a delay injector could have failed on:

1. **A handler can wait without a sleep** — `(recv (after PeerKind::thread (Milliseconds n) :done))`,
   which is what `mora` permits. Not a sleep, not a guess.
2. **It must be INLINED.** A process child is assembled from service-forms only and cannot see parent
   helpers (`the-benchmark-has-more-than-one-publisher/SCORE.md`). The probe inlines it; copy that.

## ⛔ THE ONE CONTRACT DECISION

**`delay-bp` + `delay-ms` on `:demo::topic`, defaulting to 0, and the fire counter increments AT THE
DELAY SITE — never at the dice roll.**

Zero means today's behaviour **byte-identical**, so every existing invocation and every floor
expectation is untouched. The counter placement is not style: `disrupt-hits` could never increment
because it counted at the wrong place, a zero that took **three independent causes found one at a
time** to explain. `faulting-store.wat` is the shape that got this right — a rate, and a counter where
the fault is actually produced.

## Out of scope = REJECTED

- **Delaying the queue or the store.** Same shape, and it should follow — but the publisher's death is
  the named target and the topic is its peer. One injector, measured, before a family.
- **Exposing `p` (publishers) or the backoff policy on the CLI.** Both would be needed to reproduce
  the *historical* dagger cell (p=3, fixed 1 ms). ⭑ This stone does not need them: a directly-slowed
  topic reaches the same arm at p=1 on purpose, which is strictly better than recreating the accident.
  ⚠ `:user::run-p*` is still called from nowhere — `FINDING-we-cannot-induce-a-slow-peer.md` §3 — and
  that remains true and unaddressed.
- **Fixing the desync.** `DeadlineFired` abandoning a frame is a separate, larger stone
  (`FINDING-a-fired-deadline-desyncs-the-connection-forever.md` names the ladder). This stone builds
  the instrument that would let it be *tested*; it does not repair it.
- **Wrapping internal arms so the publisher survives.** ⛔ **Deliberately NOT bundled, and the order
  matters:** wrapping `-run` without a redial would turn the publisher's loud death into a silent
  continue **holding a desynced topic peer**. This stone produces the failing fixture; the wall comes
  after, gated on it.

## Blast radius

```
wat-scripts/topic/sns-fanout.wat   :demo::topic Record +2 fields, the delay site, the counter
wat-scripts/fanout/circuit.wat     the run-with wiring + one argv slot (17, append; 16 in use today)
.rs / goldens                      0 expected — delay-bp 0 is byte-identical. VERIFY, do not assume.
```

## Trap-doors named up front

1. ⛔ **A service is a serializing actor, so parking its handler makes the WHOLE topic slow**, not one
   caller. That is the correct simulation of a slow peer and it is also a blunt instrument: at a high
   rate the circuit will not fill. `disrupt-bp` at 100 % already stalls fill — same failure mode, and
   the reason a *rate* exists rather than a switch.
2. ⚠ **The publisher's effective deadline is the generated 10 s default** — `:deadline-ms 120000` was
   deleted as inert by D4. So a delay that actually fires its `TimedOut` arm must exceed **10 s**,
   which makes that gate a ~12 s run. ⛔ **The floor's terminate wall is 30 s and a test already timed
   out at 40 s under contention this week.** Put the slow proof in a scratch-pad run reported in the
   SCORE; keep the floor assertion cheap (a relation, per below).
3. ⭑ **Gate on a RELATION, not a threshold.** `delays-fired == delay-draws` at rate 10000 holds at
   every n; `fires > 0` at a low rate is a coin flip that couples the gate to a parameter another
   stone may move. That mistake cost a floor red this week.
4. **Seed it per-tier.** Sharing one seed across tiers makes them fail on the same calls — measured:
   all four subs reported identical drop counts to the digit, which is correlated failure wearing
   independence's clothes (`circuit.wat:2950`).
5. **`.wat` corpus edits go through the codemod, never sed** — though at this size a hand edit of two
   named files is the expected shape; say which you did.
