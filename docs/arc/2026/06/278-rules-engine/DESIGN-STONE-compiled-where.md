# DESIGN-STONE — the `where` predicate compiles once; the filter stops building an Environment per token

> **Origin (2026-08-01).** `where` is the ONE condition family never compiled. Alpha conditions →
> `compiled_cond.rs`; the RHS → `compiled_rhs.rs`; `where` still walks a `WatAST` through
> `eval_test_core` (`matcher.rs:1073`), **per token × per TestNode**. On node-share — the grid's
> weakest engine cell (`:ratio` 1.56) — the `filter` phase is **89.5%** of the fire and grows
> linearly with rule count while alpha stays flat.
>
> Ratified order (builder, 2026-08-01): **(a) compile the predicate, THEN (b) index it.** (b) changes
> *what is evaluated* and can therefore suppress a raise that surfaces today — a semantic change, in
> the arc whose law forbids hidden failures. (a) is semantically inert. Land the inert one with its
> differential first; make the semantic change on a proven base. This stone is (a).

## What is MEASURED, and what is not

Measured this session (`34c3c5db`, `node_share_filter_eval_census`, `[10|25|50] × 200`):

```
 rules  items |    evals   passes  wasted  waste%  | envs    keyallocs
    10    200 |     2000      200    1800   90.0%  |  2000       2000
    25    200 |     5000      200    4800   96.0%  |  5000       5000
    50    200 |    10000      200    9800   98.0%  | 10000      10000
```

`evals/rule` is pinned at exactly 200 — every token is tested by every rule, **no pruning**, and at
most one rule can match. Waste GROWS with rule count. One Environment and N Strings are built per
evaluation, **98% of them for a predicate about to fail**.

**⚠ What that does NOT establish.** The counters prove the *mechanism* exactly. They say nothing
about the *share* — and this stone must not repeat
`[[feedback_measure_the_decomposition_never_read_it]]`, which cost four wrong attributions in one
session. Two specific gaps, both grounded:

1. **`filter` is the whole filter-pass loop, not the predicate.** The phase mark spans
   `kernel.rs:2680–2793` — which includes the Negation/Exists branches AND a per-TestNode
   `new_tokens: Vec<Token> = ts.clone()` (`:2701-2704`). With a shared join prefix every TestNode has
   the *same* parent, so at `[50 200]` that is **50 clones of a 200-Token vector = 10,000 Token
   clones per round**, the same order of magnitude as the 10,000 env builds, in the same loop, and
   **not the predicate at all**. "filter is 89.5%" ≠ "the predicate is 89.5%".
2. **Env-build vs expression-walk is unsplit.** Killing the env build wins nothing if `eval_inner`'s
   walk dominates.

**⇒ STEP 0 IS A MEASUREMENT, and it is a STOP gate.** See § Step 0 below.

## What one `eval_test_core` call actually pays — grounded

For node-share's predicate `(= i (- ?k (* (/ ?k n) n)))` (`wat-scripts/perf/grid/node-share.wat:68`),
which references `?k` **three** times:

| site | cost, per token × per TestNode | why it is waste |
|---|---|---|
| `matcher.rs:1090` | `env.child()` → a fresh `HashMap<String, BoundEntry>` | the binding *names* are fixed at rule-compile time |
| `matcher.rs:1093` | `s.as_str().to_string()` | **heap allocation** per binding, rebuilding a constant key from an `Arc<String>` that already holds it |
| `matcher.rs:1097` | `TrackedValue::from(v.clone())` + `rust_caller_span!()` | a Value clone and a **Span construction** per binding |
| `matcher.rs:1099` | `b.build()` → `Arc::new(EnvCell)` | **second heap allocation**, per evaluation |
| `environment.rs:162` | `lookup(name, span)` — `HashMap<String,_>` get | a **string hash + compare** per `?var` *reference* (×3 here) |
| `environment.rs:175` | `Provenance::SymbolBound { binding_span.clone(), head_span.clone() }` | **two Span clones per variable read** — diagnostic provenance built for a predicate nobody will diagnose |

That is *at minimum* 2 heap allocations + N Spans + N Strings per evaluation, plus a hash and two
Span clones per variable reference — 10,000 times, 98% of it discarded. It is verbatim the defect
`compiled_cond`'s own doc names on the alpha path (*"two heap allocations rebuilding the constant
binding key on every call, including every call that is about to FAIL, which is most of them"*),
except heavier: alpha paid 2 allocations, this pays ~6 plus per-read hashing.

## ★ Step 0 — the decomposition, BEFORE the build (a hard STOP)

An out-of-fire micro-benchmark with an **identity control**, the shape that cracked the quadratic
(`probe-into-is-quadratic.wat`): build the real bindings trie and the real `WatAST` from node-share's
`[50 200]` network, then time, at the same n, interleaved:

| arm | what it runs | isolates |
|---|---|---|
| **A** | the env-build block alone (`matcher.rs:1090-1099`), result discarded | the env cost |
| **B** | env-build + `eval_inner` (the full `eval_test_core`) | env + walk |
| **C** | `ts.clone()` of a 200-Token vector, ×50 | the per-node token clone (finding 1) |

`B − A` is the walk. Assert both arms produce the same count of evaluations (non-vacuity) and that
A's result is actually constructed (not optimized away).

**STOP-0.** If `A` is a small fraction of `B` — the walk dominates, not the env build — then the
counters in the seam's gate (`env-builds → 0`, `key-alloc → 0`) would be a mechanism win with no
timing behind it, and **the stone's shape is wrong**: the answer is a full expression IR, or (b)
first. Surface it; do not build the env fix and call it (a).

**STOP-0b.** If `C` is comparable to `A + B`, the token clone is a peer cost, and it is a *different,
cheaper* stone (hoist the clone out of the per-node loop — every TestNode with the same parent reads
the same vector). Surface it and let the builder order the two.

## ⚠ STEP 0 HAS RUN (2026-08-01) — **STOP-0 FIRED.** The gate was aimed at 23%; the walk is 77%

`node_share_where_cost_decomposition`, node-share `[50 200]`, one round's worth per arm, 15
**interleaved** reps, medians, inputs captured out of a **real fire** (1 predicate × 200 tokens ×
1 binding; 4/200 pass). Load-gated, run under the memory guard, my own re-run:

```
  A  env build alone         ( 10000 x)     1.225 ms
  B  env build + walk        ( 10000 x)     5.401 ms
  C  token clone             (    50 x)     0.773 ms
  D  env + walk, VAR-FREE    ( 10000 x)     4.339 ms      <- the identity control
  E  hand-written Rust       ( 10000 x)     0.210 ms      <- THE FLOOR
  ------------------------------------------------------------------------
  the walk        B-A      4.177 ms   77.3% of B    417.7 ns/eval
      ?var lookup    (B-A)-(D-A)   1.062 ms   25.4% of the walk
      node dispatch  D-A           3.114 ms   74.6% of the walk
  the env build   A        1.225 ms   22.7% of B    122.5 ns/eval
  the token clone C        0.773 ms
  ------------------------------------------------------------------------
  RECONSTRUCTION  B+C = 6.175 ms  vs a measured `filter` of 6.83 ms  (90% accounted)
  HEADROOM        B-E = 5.192 ms is what a PERFECT compile could remove
```

**The reconstruction is the load-bearing row.** Two arms measured out-of-fire account for 90% of the
in-fire phase reading, so this is a decomposition of the real thing, not of a synthetic that
resembles it.

### What this CHANGES in this document

- **The seam's gate was aimed at the smaller half.** `filter:test-env-builds → 0` removes arm A —
  **1.225 ms, 22.7% of the predicate, 18% of the `filter` phase.** Real, but not the story.
- **The story is the interpreter's per-node dispatch.** 540 ns/eval today against a **21 ns/eval
  floor**; and the split inside the walk says the win is *not* mostly name lookup — dispatch is
  74.6% of the walk, `?var` lookup 25.4%. So the half-measures are bounded: interning the keys plus
  pre-resolving the vars captures at most (122 + 106) / 540 = **42%**. Only the full IR reaches the
  floor.
- **⇒ The stone is NOT the env fix. It is the full expression IR**, and the IR subsumes the env
  build by construction (slots, no `Environment` at all). Everything below stands; what changes is
  that `Op::Interp` is the *exception* arm, not a comfortable majority — and the gate gets a timing
  row it can now state honestly (below).
- **Task #50 (the token clone) is real and third-order** — 0.773 ms, 11% of the phase. Confirmed
  NOT a peer cost (STOP-0b did not fire); it stays its own cheaper stone.

### ★ AND IT INVERTS ONE CLAIM IN TASK #49 — (a) and (b) are SUBSTITUTES here, not complements

`cost = evaluations × cost-per-evaluation` is true as a formula, and the task called the two attacks
"independent and MULTIPLICATIVE." Measured, on this workload, they overlap almost entirely — because
**either one alone drives the product to near-zero**:

| | evaluations | ns/eval | filter's predicate cost | saves |
|---|---|---|---|---|
| today | 10,000 | 540 | 5.40 ms | — |
| **(a) alone** — compile | 10,000 | ~21 | 0.21 ms | 5.19 ms |
| **(b) alone** — index | 200 | 540 | 0.11 ms | 5.29 ms |
| both | 200 | ~21 | 0.004 ms | 5.40 ms |

**After (a), (b) is worth ~0.2 ms.** The ratified order still holds — it was ruled on *correctness*
(land the semantically-inert stone first; (b) can suppress a raise that fires today), and that
reasoning is untouched. But the *value* argument for doing both must be retired: the second one
lands on almost nothing.

**Bounded honestly (R60):** this is ONE axis, ONE predicate shape, and both sides of it are ours.
A workload with an expensive predicate *and* a well-pruning join would make (a) matter and (b) not;
node-share happens to be one where both attack the same mass. Do not generalize the substitution.

## The change — a compiled predicate, built where the TestNode is built

Mirrors `compiled_cond.rs` exactly: a pre-resolved instruction sequence produced **once**, at the
same setup site, from the immutable network; executed against a caller-owned reused scratch buffer.

```rust
struct CompiledWhere {
    ops:     Vec<Op>,        // slot-resolved; ?var -> usize, literals built once
    n_slots: usize,
    /// The ?var names this predicate actually READS, resolved to slot indices once.
    /// The interpreter fallback binds ONLY these — not every binding the token carries.
    reads:   Arc<[(Value, usize)]>,
}
```

**The one place this differs from `compiled_cond`, and it is load-bearing.** `compile_condition`
maps an unrecognized clause to `Op::Fail`, because `eval_clause` *also* returns `None` for those —
compiling to failure is semantics-preserving there. **Here it is not.** A `where` is a general
expression, and the live corpus contains user-fn calls (`(:fix::has-ns? ?name)`,
`wat-scripts/fixes/`) and record accessors (`(:arena::Route/status ?route)`) that the interpreter
handles correctly. So an unrecognized shape must compile to:

```rust
Op::Interp(WatAST)   // build the env — binding only `reads` — and call eval_inner
```

**Fall back, never fail.** A compiled `where` that silently stopped matching a predicate it did not
understand would be a hidden failure in the arc whose law forbids them.

## ★ THE ONE CONTRACT DECISION

**The compiled path must be a total specialization of `eval_test_core`: for every (predicate,
bindings) pair, `compiled(...) == eval_test_core(...)` — same `bool`, and the same `Err` for the same
input.** The error arm is explicitly in the contract: a `where` returning a non-bool raises a located
`TypeMismatch` today (`matcher.rs:1104`), and a compiled form that returned `false` instead would
convert a located error into a silent non-match. Not narrower, not louder — identical.

The corollary that keeps it honest: **`eval_test_core` is NOT deleted.** It stays as the reference
implementation and the other half of the differential, and it *is* the `Op::Interp` fallback's
body — so the two cannot drift, because the compiled path calls the interpreter for everything it
does not model. Whether it survives having no direct production caller is a separate ruling
(`[[feedback_no_consumers_does_not_mean_dead]]`).

## Blast radius

`src/rete/matcher.rs` (the compiler + executor, beside `eval_test_core`) and `src/rete/kernel.rs`
(compile at the setup site alongside the alpha compiles; call the executor at `:2727`). **Nothing
under `wat/`** — the oracle does not move, and stays naive by ruling (`OCVLI NOVI, ORACVLVM
IMMOTVM`, R22).

## The gate — counters and a differential, not a stopwatch

1. **The differential, on the verdict AND the error.** Over the (predicate, bindings) pairs drawn
   from every live grid axis and the `wat-scripts/fixes/` rule corpus, `compiled` must equal
   `eval_test_core` — same `bool`, same `Err` variant on the non-bool case. A boolean-only comparison
   would pass while converting a located error into a silent drop.
2. **`filter:test-env-builds` → 0 and `filter:test-key-alloc` → 0**, with **`filter:test-evals` held
   as the non-vacuity guard** (a fire that never reached a TestNode also reports zero allocations).
   Counters, not a timer: at ~78 ns per mark pair against a sub-microsecond body a stopwatch would
   measure mostly itself — the failure `e563d708` caught in three of alpha's five children.
   **⚠ Both counters go to zero only on the compiled path; the `Op::Interp` fallback still builds an
   env.** So the gate is honest only if a *third* counter — `filter:test-interp-fallback` — is
   reported alongside, and the gate reads: env-builds == interp-fallbacks. Zero-with-a-nonzero-
   fallback-count is the real, un-gilded result.
3. **`:accuracy :match` on every grid axis**, and the release floor unchanged
   (`cargo nextest run --release`, the Summary line, my own re-run).
4. **Setup cost stays bounded** — compiling N predicates does not push `SETUP: indexes` past the
   budget the alpha-tree stone set.
5. **The timing row, now that Step 0 has run and licenses one.** Unlike `compiled_cond` — whose
   target was 1.2 ms of a 115 ms fire, where any scorecard row demanding an improvement would have
   been unfalsifiable — the target here is **5.19 ms of a 6.83 ms phase**, measured, with a
   **21 ns/eval floor** to score against. So the row is stated and it is falsifiable:
   **`node_share_where_cost_decomposition`'s arm B must fall from ~540 ns/eval toward E's ~21, and
   `filter` in `node_share_fire_phase_census` at `[50 200]` must fall materially from 6.83 ms.**
   Re-run the decomposition after the strike — it is the same instrument, so before/after are
   directly comparable. Interleave, gate on load, and never compare across batches.

## Out of scope = REJECTED (affirmative cuts)

- **(b) indexing the predicates.** Its own stone, and it lands second by ruling — it carries the
  correctness angle (a token that tests only its matching rule never raises the unbound-var or
  div-by-zero a non-matching rule's `where` raises today) and it needs *this* stone's structured Ops
  to recognize `(= const expr)` and prove two sub-expressions equal. Against raw `WatAST` that
  analysis gets written twice.
- **Hoisting the per-TestNode `new_tokens` clone** (`kernel.rs:2701`). A real, separately-attributable
  cost inside the same phase — and a *different* stone. Bundling it would destroy the attribution,
  which is exactly how the seam ended up unable to say what "filter is 89.5%" meant.
- **Keyword/string interning.** `109-kill-std/NOTE-keyword-storage-must-intern.md` would kill the
  per-binding `.to_string()` at the language level. It is a language-level change with its own arc;
  smuggling it into this diff would destroy the attribution the same way.
- **Any JIT or native codegen.** A resolved instruction vector is the whole idea.
- **`wat/rete.wat`.** The oracle is never optimized.
- **Deleting `eval_test_core`.** Pinned above as the contract's corollary.
