# Arc 278 — Realizations

## R1 — wat fell back into being a spec language: the correct-but-slow oracle that guides the Rust impl

We did not plan this. We built the rete engine in wat, stone by stone, and to make the cascade (4b) work we
chose **re-run-from-scratch**: every `fire-rules` recomputes all memories from `facts`, the fixpoint loops the
whole thing. Correctness-first; the pure `WM = fn(facts × rules)` thesis made it the obvious move. I even
defended it on those grounds and filed "incremental delta" as a deferred perf nicety.

Then the builder pulled the parity docs back into view — *"we wrote a bunch on what we're going to be in parity
with and what we won't be"* — and the DESIGN was unambiguous: delta propagation is a **hard v1 requirement**,
*"the wasteful tree is forbidden outright… N rules over M facts is therefore NOT N×M"* (`DESIGN.md:276-278`).
Re-run-from-scratch *is* the wasteful tree. So I'd shipped a perf compromise and mislabeled it a deferral.

He didn't want it asserted, he wanted it measured — *"i want this tooling for line processing of HTTPS
requests and sampled packets — it's gotta be good."* So we benched the wat engine for the first time
(`tests/perf_arc278_fire_baseline.rs`):

```
N= 25  (  50 facts)   ~61 ms     ~820 facts/s
N= 50  ( 100 facts)  ~201 ms     ~500 facts/s
N=100  ( 200 facts)  ~762 ms     ~260 facts/s
N=200  ( 400 facts) ~1799 ms     ~220 facts/s
N=400  ( 800 facts) ~6134 ms     ~130 facts/s
```

Per-fact cost climbs 1.2 ms → 7.7 ms — textbook O(N²) (re-run-from-scratch × the deferred-index cross-join).
130–820 facts/s is 4–7 orders of magnitude under line rate. The wat-interpreted fire loop is, measured,
hopeless for the bar.

The instinct was to call that a failure and feel bad about the wat stones. The builder saw the opposite:

> *"this is insane to me — we have a known correct-but-slow impl that we can directly measure against — we
> didn't plan for this… it just fell out… wat guides the rust impl … this is how wat started as a spec
> language, not an interpreted one — this is a realization."*

That's the realization. The "slow" wat engine is not waste to delete — it is a **known-correct executable
specification**. The Rust fire kernel we now build (delta propagation + `join-bindings`-keyed joins + native
mutable memories, frozen `Session` out) is the *optimization*, and the wat engine is the **differential
oracle** it must match bit-for-bit on every input. The hardest, most error-prone code in any Rete engine —
incremental delta + truth-maintenance cascade (Clara's hazard #1) — gets validated against a reference so
simple it's obviously correct. We get to write the dangerous fast thing with a net under it, and the net cost
us nothing extra: it fell out of building correctness-first in wat.

And it reconnects wat to what it was *for*. The builder's framing the same session:

> *"i view wat as an orchestration of rust — wat exists because i want rust without rust's syntax."*

wat began as a **spec language**, not an interpreted one. The interpreter is a convenience that grew on top;
the original job was to *say what the system does*, cleanly, so Rust could do it fast underneath. The rete
engine makes that literal: wat says the semantics (the oracle), Rust executes them at speed (the kernel), and
wat keeps Rust honest (the differential test). The thing we reached for as "the interpreter is too slow, move
it to Rust" turned out to be wat resuming its first role — **the spec that guides and validates the
implementation.**

**Why it actually works as an oracle (named honestly):** the wat engine is pure value-semantics — `Session`
in, `Session` out, no hidden state — so the differential test is a total function comparison: same input → the
oracle's frozen `Session` must equal the kernel's frozen `Session`, structurally. The kernel's internal
mutation (transient-during-fire) is sealed behind the freeze boundary and never observable; the only thing the
test compares is the immutable result. Two engines, one contract, byte-for-byte.

> We set out to interpret a language and accidentally rediscovered why we wrote it: not to run the program,
> but to *specify* it — and to hold the fast implementation to account.

### The bar this sets

> *"we raise the bar through the fucking roof, relentlessly — i want the perf i had with Clara (if not
> superior since we're backed by Rust, not Java)."*

Clara-parity-or-superior, Rust-backed, validated against a wat oracle. The full plan: `PERF-ARC-rust-fire-kernel.md`.
