# The Rust `fire` kernel — arc 278's CLOSING CONDITION (not a new arc)

**Status:** committed (2026-06-19). The oracle is complete (north star green, `44ebffca`); the kernel is not
yet built. **This is the closing condition for arc 278 as a whole** (builder: *"i don't think this is a new arc
— i think this is the closing condition for the rete arc"*). Arc 278 does NOT close at the green north star —
it closes when the Rust fire kernel is **differential-tested bit-for-bit against the wat oracle** and **benched
at or past Clara**. The wat engine had to be a complete, correct oracle first; it is.

**The bar:** exceed Clara/Java. The GC point is real, not a boast — Clara runs on the JVM (stop-the-world GC
pauses = tail-latency spikes at the worst moment for line-rate packet processing); Rust has no GC (ownership +
`Arc`, no pauses, cache-dense native structures), so we are *theoretically ahead before optimizing anything*,
and we get *predictable* (jitter-free) latency by construction. The arc closes when the bench proves it.

## The bar (builder, 2026-06-19)

> *"we raise the bar through the fucking roof, relentlessly — i want the perf i had with clara (if not
> superior since we're backed by rust, not java)."*
> *"i view wat as an orchestration of rust — wat exists because i want rust without rust's syntax."*
> *"if the 'unsafe part' is scoped and out of user's hands then we are justified to do the 'unsafe' thing
> because we know we are doing it correctly."*

Target consumers: **line processing of HTTPS requests and sampled packets** (DDoS lab line-rate). The bar is
**Clara-parity or superior** (Rust, not Java).

## The grounded baseline (why this arc exists)

`tests/perf_arc278_fire_baseline.rs` measured the current **wat-eval re-run-from-scratch** `fire-rules`
(2026-06-19):

| N | facts | fire-rules | facts/s |
|---|---|---|---|
| 25 | 50 | ~61 ms | ~820 |
| 50 | 100 | ~201 ms | ~500 |
| 100 | 200 | ~762 ms | ~260 |
| 200 | 400 | ~1799 ms | ~220 |
| 400 | 800 | ~6134 ms | ~130 |

Per-fact cost climbs 1.2 ms → 7.7 ms — **O(N²)** (re-run-from-scratch + the deferred-index cross-join). At
**130–820 facts/s** this is **4–7 orders of magnitude** below line rate. The wat-interpreted fire loop is
measured-hopeless for the bar — not a guess.

## The decision (four-questions, perf premise grounded)

**`wat` orchestrates; Rust does the engine work.** This is not a compromise of "the engine is wat" — it is
what wat is *for* (orchestration of Rust, clean syntax over native speed).

- **Production engine = a Rust `fire` kernel.** Inside `fire`, native mutable structures (O(1) `HashMap`s):
  - **`join-bindings`-keyed joins** (the real hash join — a new element looks up only matching tokens, not a
    full cross). Closes the 3b deferral.
  - **delta propagation** — insert/retract apply as the *change* through affected nodes; the within-fire
    fixpoint propagates deltas, never a full re-scan. Closes the 4b re-run-from-scratch.
  - **transient-during-fire / persistent-at-rest** (CLARA-REF §5) — mutate native memories inside `fire`,
    freeze to a persistent `Session` on return. The mutation is **sealed in Rust = literally out of the
    user's hands**; the user surface stays pure value-semantics. This is the builder's "scoped unsafe is
    justified" — the freeze boundary is the correctness witness.
  - **TM = the support store + the token `matches` chain** (CLARA-REF §3) — cascade-retract via
    `production-memory`, using the provenance chain 3a/3b already build (currently unused).
- **wat = orchestration** — compile, rules-as-data, `defrule`/`defquery`/`query`/`insert`/`retract` surface;
  calls the kernel. Built once; orchestrates either engine.
- **The wat re-run-from-scratch `fire-rules` (stones 4a/4b) = the reference SPEC + differential oracle.** The
  Rust kernel must produce a **bit-identical `Session`** on every input. The oracle is simple and obviously
  correct; the kernel is fast and complex; differential testing (oracle == kernel) is the safety net for the
  hardest code in the engine (Clara's delta logic is hazard #1). The oracle is *kept*, not ripped out.

## Why slice-first is required (not just preferred)

The kernel is validated by differential test against the oracle. The oracle is the wat engine *completed
through query*. So the semantic slice (4c TM-via-replay + 5 defrule/collect-rules/query → north-star green)
must land first — it *is* the spec the kernel matches. North-star green = "semantically complete on the
reference engine; Rust kernel perf arc pending" (label it; do not call the engine done at spec-speed).

## Decomposition (when this arc opens — to be refined)

A likely path (each its own examinare strike, differential-tested against the oracle):
- **P1** — native working-memory representation + the transient/persistent freeze boundary (the stone-0
  `to-transient`/`to-persistent!` pair that was skipped, now in the kernel).
- **P2** — `join-bindings`-keyed alpha/beta memories (hash join; close 3b's deferred index).
- **P3** — delta propagation: incremental insert (and the within-fire fixpoint as delta, not re-run).
- **P4** — delta retract + TM cascade via the support store + `matches` chain.
- **P5** — wire the wat surface to call the kernel; differential-test the whole engine vs the oracle; bench
  vs the baseline + vs Clara's published numbers.

## Measurement

`tests/perf_arc278_fire_baseline.rs` is the kept measuring stick — extend it to print the rust-eval column
beside wat-eval; the ratio is the speedup; the absolute is checked against Clara. (No bench harness/criterion
yet — an `#[ignore]`d perf test, run on demand.)
