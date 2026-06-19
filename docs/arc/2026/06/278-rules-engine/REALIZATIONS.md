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

## R2 — a complete Rete fell out in a day, because it was assembly, not invention

The north star went green and the builder said it plain:

> *"we built a complete rete in under a day — that's…. insane."*

It is worth being precise about *what*, so "insane" reads as method and not luck. What shipped, end to end,
in a day: alpha matching; **equality joins with real cross-condition unification** (the HashJoinNode — the
part every toy evaluator hand-waves); production firing; **cascade to a monotone fixpoint**; **truth maintenance
with transitive retraction** (retract a supporting fact, its whole derived chain vanishes); and a homoiconic
`defrule` / `query` surface — in a language that did not exist as a general substrate a year ago. Not a naive
stand-in. The thing Forgy's 1974 thesis is about, with the parts CLIPS / Drools / Clara add bolted on,
green against an acceptance test (`cold-and-windy`) that was written on day one and never moved.

**Why it was possible — the grounded version:**

- **It was assembly, not invention.** The hard parts were already on the shelf: persistent collections
  (0a–0d), the total-pure macro-eval engine (arc 249), types-as-forms (arc 251), records, the WatAST bridge,
  symbol-table reflection. Rete was *orchestration of Rust with clean syntax* — exactly what wat is for
  ([[project_wat_is_spec_rust_is_impl]]). The day was wiring capability that already existed; every prior arc
  was a stone in this foundation without knowing it.
- **The north star was the contract from minute one.** One green test fixed the target; every stone aimed at
  it. No drift, no scope-debate mid-build.
- **The strike discipline did the compounding.** Each stone: draw (DESIGN + a RED probe that fails on
  *exactly* the gap) → fire one sonnet → weigh against an independent re-run + the diff → ship green. Slow is
  smooth; we never fought the same boss twice.
- **The inserts-only thesis paid triple** ([[project_rete_inserts_only_replay]]): it kept the engine simple
  (pure value-semantics, no mutation), it made TM *fall out of replay* instead of needing a justification
  graph, and it handed us a known-correct **oracle for free** (R1).
- **Grounding caught the rabbit holes before they cost a day each:** the `defrule` macro loop, the `query`
  Bundle-archaeology smell (→ the `return-type-of` intrinsic), the 4b input/derived TM bug, the
  keyword-resolves-to-its-constructor "is-it-a-defect" question (it isn't — names resolve to bindings). Each a
  fifteen-minute probe, not a five-hour wrong turn.

**The honest asterisk:** this is the *correct-but-slow* Rete (`~130–820 facts/s`, O(N²) — the
re-run-from-scratch oracle). The day did not produce the fast engine. It produced the **spec the fast engine
will be held to** — which is the more valuable artifact, and the reason the speed is *repeatable* rather than a
one-off.

**This is the bar, and it is the close.** The builder set both the target and the scope:

> *"we exceed clara/java — at minimum not having a gc means we are theoretically faster already?"*
> *"i don't think this is a new arc — i think this is the closing condition for the rete arc as a whole."*

So arc 278 does not close at the green north star. It closes when the **Rust fire kernel** — delta propagation,
`join-bindings`-keyed joins, native mutable memories behind the transient/freeze boundary — is **differential-
tested bit-for-bit against this oracle** and **benched at or past Clara**. And the GC point is real, not a
boast: Clara runs on the JVM, where a stop-the-world pause is a tail-latency spike at exactly the wrong moment
for line-rate packet processing. Rust has no GC — ownership + `Arc` refcounting, no pauses, cache-dense native
structures. At the line, *predictable* latency (no GC jitter) may matter as much as raw throughput, and we get
it by construction. Theoretically ahead before we optimize a thing; the arc closes when we prove it on the
bench.
