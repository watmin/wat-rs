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

## ★★ RULED 2026-08-06 — ONE CORE, THREE ADJACENT FLIPS. This is the shape of the build.

> **Builder:** *"it strongly appears like we need to have a single compilation solution for the rete
> forms … or is it best to build where now, and then use the existing ones as an oracle for a
> unified replacement? know it works and then make it better, adjacent and then flip to the
> adjacent one — we've done this pattern /many/ times."*

Both halves of that are right, and they are **not in tension**: the first is the MODEL, the second
is the EXECUTION.

> **▶ STEP 1 IS DRAWN (2026-08-06):** the builder ruled the layout — **nesting, "matches the
> precedent"** — and the Op set is drawn in **`DESIGN-STONE-the-one-expression-core.md`**,
> derived from the 75-row `RETE_OPS` table, `dispatch_rete_op`, `eval_test_core`, and a census
> of all 173 corpus `where` predicates. **There is NO interpreter escape hatch** — `lower()` is
> total or it refuses (the builder cut the `Interp(WatAST)` arm on sight; the stone records why).
>
> **2026-08-17.** The endeavor is now: annihilate all interpretation in wat-rete.
> Live breadcrumb: **`CURRENT-STATE-annihilate-interpretation.md`**.
> STOP-1 (match grammar) and STOP-3 (`CallUser` lowers) closed the same day as the draw.
> STOP-2 (frame) ruled 2026-08-17: copied captures. STOP-4 (eval returns `Result`) stands.
> Do **not** follow `BRIEF-compiled-where.md` — it still specifies `Op::Interp` and a
> third sibling `compiled_where.rs`. The module is `src/rete/expr_ir.rs`.
>
> **Step 2 is done** (`30725034`, local, not pushed): `where` is wired;
> native TestNode stashes `Program` and calls `exec_where`. The
> "compiled_where | to write" row below is stale. **Step 3
> (`cond`, then `rhs`) is PARKED** until the oracle exists/not path
> is rete-sane. Do not flip them because this stone lists them next.

### THE THREE STEPS — do not collapse them

1. **DESIGN THE CORE AS ONE.** One expression model over the closed rete vocabulary. Ruled by the
   four questions (2026-08-06): a per-surface Op model fails Obvious (three models, one alphabet),
   Simple (a new rete op = three edits) and Honest (the stone's own implementation law —
   *"a rete op is NEVER a second implementation"*). One core scores 4×YES, and a new op becomes ONE
   opcode in ONE table.
2. **WIRE ONLY `where` TO IT.** Differential against `eval_test_core`, exactly as its two siblings
   were built.
3. **THEN FLIP `cond`, THEN `rhs` — ONE AT A TIME.** Never together. If a flip diverges, it
   diverges alone.

### WHY THIS IS CHEAP, and the fact that makes it so

**The oracle is the INTERPRETER, not the existing compilers** — and each flip's differential is
therefore **already written and already green on disk**:

| compiler | its standing gate | holds it to |
|---|---|---|
| `compiled_cond` | `compiled_cond_bindings_identical_to_interpreter_at_50_100` (`kernel.rs:6866`) | `alpha_match_inner` |
| `compiled_rhs` | its own differential | `build_insert_fact` |
| `compiled_where` | **to write** — this stone | `eval_test_core` |

So a flip onto the unified core is not a leap of faith; it is a change guarded by a gate that
already exists and already passes. `PARI GRADV` (R1/R9) applied **inside** the compiler: one design,
three adjacent flips, never a big-bang.

### THE DEPENDENCY IS ALREADY DECLARED IN CODE — this is not a speculative unification

```rust
pub(crate) enum RhsOp {
    Bind(Value, String),
    Lit(Value),
    Expr(WatAST),   // ← NOT compiled. Still calls eval_test_core, per fact.
}
```

`compiled_rhs.rs:85` names the owner outright: *"expression-tree compiler (that is `#49a`'s
`DESIGN-STONE-compiled-where`'s to build)"*. We do **not** have two compilers and a third to write.
We have **two consumers with an expression-shaped hole, and one core owed**:

| surface | consumes → produces | expression support today |
|---|---|---|
| `compiled_cond` | fact fields → bindings | `Cmp` only — cannot express `(i64::+ ?a 1)` |
| `compiled_rhs` | bindings → a fact | `Expr(WatAST)` — explicitly uncompiled |
| `where` | token bindings → bool | **does not exist** |
| accumulator fold | gathered values → a value | does not exist |

Four surfaces, one fenced vocabulary (law A armed at all four as of #83/#84), one evaluator owed.

### ⚠ THE TREE-vs-FLAT-STACK FORK IS NARROWER THAN IT LOOKS — grounded, not assumed

`compiled_cond` is **already both**: `exec_ops` is `for op in ops` over a flat `Vec<Op>` — *except*
at control flow, where it NESTS (`Or(Vec<Vec<Op>>)`, `Not(Vec<Op>)`). So the real fork is **nested
sub-programs vs jump offsets**, and it matters more for `where` than it did for `cond` because the
control forms are richer (`if`/`cond`/`match`/`let`/`fn` vs `or`/`not`).

Nesting matches the precedent and is the smaller step; offsets are what the indexing phase wants.
**Which is FASTER is UNMEASURED and may not be claimed** — no benchmark distinguishes the two
layouts; measure it or leave it open (`[[feedback_measure_the_decomposition_never_read_it]]`).

> **✅ CORRECTED 2026-08-06 (far side).** This sentence originally ended *"— that is **Step 0's
> number**, under this stone's own STOP."* **It is not.** Step 0 is the A/B/C/D/E cost decomposition
> and it **RAN 2026-08-01** — see `⚠ STEP 0 HAS RUN` below, ninety lines down in this same file.
> Attaching a measured section's name to an unmeasured question makes the measured one look open;
> the layout fork is its own, un-run, and unnamed. Kept visible, not rewritten away.

### ⛔ WHAT IS NOT YET PROVEN — the probe that gates step 1

That one Op model actually covers all four surfaces is a DESIGN CLAIM with no evidence behind it
yet. The disconfirming probe, before a line of the real thing: draft `where`'s Op set, then check
whether `compiled_cond::Op`'s six variants fall out of it as **driver-level** concerns
(fact-fields-in vs bindings-in) or genuinely do not fit. **If `Bind`/`BindCheck` do not fall out
cleanly, the one-core claim is WRONG** and the three-step plan above collapses to "build `where`
alone". Better to learn that from a probe than from a migration.

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

### The node-share arithmetic — TRUE OF NODE-SHARE, and NOT a statement about the design

| | evaluations | ns/eval | predicate cost | saves |
|---|---|---|---|---|
| today | 10,000 | 540 | 5.40 ms | — |
| (a) alone — compile | 10,000 | ~21 | 0.21 ms | 5.19 ms |
| (b) alone — index | 200 | 540 | 0.11 ms | 5.29 ms |
| both | 200 | ~21 | 0.004 ms | 5.40 ms |

On **this axis** either attack alone drives the product near zero. That is an arithmetic fact about
node-share (50 rules, ONE shared key expression), not about the two designs. It was reported once as
"(a) and (b) are substitutes"; see the retraction below. `[[feedback_a_measurements_boundary_is_its_claims_boundary]]`.

## ⚠ AMENDED — the DDoS lab is the second oracle, and it kills a load-bearing claim of mine (R61, again)

The builder: *"is this not… something from the alpha tree we did in the ddos lab we can reuse?"*
It is. `~/work/holon/holon-lab-ddos/veth-lab/filter/src/tree.rs:75` — read this session:

```rust
pub(crate) struct ShadowNode {
    dim_index:      usize,
    children:       StdHashMap<u32, Rc<ShadowNode>>,       // equality fan-out, O(1)
    wildcard:       Option<Rc<ShadowNode>>,                // rules not constraining this dim
    range_children: Vec<(RangeEdge, Rc<ShadowNode>)>,      // (op, threshold) GUARD EDGES
}
```

**RETRACTED: "a hash cannot dispatch a range."** That was the pivot of a `Honest? NO` verdict on (b)
and of the "(b) applies to 1 of 24 predicate families" scope claim. Both are dead.
`series-003-004-the-rule-engine.md` states the mechanism outright: *"These evaluate as guard edges in
the tree — the node checks the inequality at traversal time without expanding the rule into multiple
equality branches. No tree bloat."* Shipped, at 1.3M pps, in this workspace, since February.

Also uncredited: **`any_constrains`** (`tree.rs:478`) — a dimension no active rule constrains
generates zero nodes and zero runtime cost. That answers the other objection ("~8 key expressions
across 7 arena rules, nearly one index each"): each key expression is a level, non-constraining rules
take the wildcard edge, and the tree prunes regardless.

**⇒ (b) is a general engine capability, not a node-share special case**, and "conditional on a
workload that has not appeared" is **RETRACTED**.

### What survives the retraction — two things, and the second is sharper than what it replaced

1. **Raise suppression, and it is OURS, not the lab's.** The lab's dimensions are field extractions
   from a packet struct; they cannot raise. Our key expressions are arbitrary wat — div-by-zero,
   unbound var, a user fn that panics. A token routed away from rule 37 never evaluates rule 37's
   `where`, so a raise that fires today vanishes. Independent of hash-vs-tree; a hidden failure in
   the arc whose law forbids them (R55).
2. **The lab's tractability is that `FieldDim` is a DECLARED enum of 15 fixed dimensions.** Ours
   would have to **derive** dimensions by canonicalizing arbitrary expressions and proving two
   sub-ASTs are the same dimension. That is categorically harder than anything in the lab, and
   getting it wrong routes tokens to the wrong rule, silently, with the right type. It is also the
   open door: *the lab did not derive its dimensions — it declared them.* See the probe below.

### ★ THE RULING — (a) then (b), on three INDEPENDENT reasons (builder-ratified 2026-08-01)

1. **(a) is semantically inert; (b) is not.** Land the inert stone with its differential, then make
   the semantic change on a proven base. (The builder's original ruling; untouched by any of today.)
2. **(b) needs (a)'s structured Ops.** Recognizing dimension shapes and proving sub-expression
   equality is an analysis written against *something*; against raw `WatAST` you write it twice.
3. **★ (b) WITHOUT (a) CANNOT HIT THE TARGET RATE.** The tree cuts the *number* of key-expression
   evaluations to ~one per level; it does not make one cheap. At 540 ns interpreted × ~5 levels that
   is ~2.7 µs/token against the lab's shipped budget of ~770 ns/packet at 1.3M pps. Compiled at
   21 ns/eval it is ~105 ns and fits. **(a) is a PREREQUISITE for (b) to be worth having at line
   rate**, which is R25's chaos engine, which is what this arc is for.
   *(Arithmetic on a projection — both inputs measured, the composition is not.)*

The dead reason — the "1 of 24" scope argument — is **not replaced**. The three above are independent
of it and of each other.

### ✅ THE DISCONFIRMING PROBE HAS RUN — `tests/rete/probe_arc278_compiled_where_ops.rs`, both GREEN

**1. `Op::Field` is sound — with one caveat the brief must carry.** The interpreted accessor
`(:p::Route/status r)` and a direct `a.fields[0]` read agree, and the field index IS resolvable from
the `TypeEnv` **given the class**. ⚠ But at TestNode-compile time we do **not** know `?route`'s class
— it is bound dynamically by an earlier condition. So `Op::Field` cannot be a compile-time index for
a `?var` receiver. It is either a runtime class→index lookup (cacheable) or it carries the field
NAME and does `keyword_accessor_record`'s work **without** the head dispatch. Both beat the status
quo (today the accessor is the LAST arm after user-fn lookup, def-bound and sandbox all miss); the
probe proves the value is reachable and the registry lookup works, and proves nothing more.

**2. THE (b) SCOUT — the corpus ratio is 1.6, the lab's is ~66,000.**

```
  preds  distinct-keys  no-key  ratio  file
     11              7       0    1.6  tests/services/probe_arc278_sift_rules_arena.wat
     11              7       0    1.6  wat-scripts/scratch-pad/probe-arena-rich-graph.wat
      5              0       5    0.0  wat-scripts/fixes/to-faithful-clojure-net.wat
      5              0       5    0.0  wat-scripts/fixes/to-faithful-clojure-rete.wat
      …
     55             21      24         TOTAL
```

A tree level pays off at MANY rules : FEW dimensions. The lab runs **1M rules over 15 declared
`FieldDim`s ≈ 66,000:1**. Our corpus runs **~1.6:1** — and **24 of 55 predicates have no key
expression at all** (user fns, `not`, `or`, var-to-var), so under a tree they sit on the wildcard
edge and are evaluated for every token regardless. *A tree cannot help those; only (a) can.*

⚠ **Two honest caveats.** The scanner is line-based, so the multi-line predicates (node-share,
strat-neg, three probe copies) report 0 keys — the 1.6 is a **floor**, and node-share's real ratio is
50:1. And the corpus is not the users (R60): R25's chaos engine — packets against thousands of rules
— is precisely the lab's regime, which is why (b) is worth building at all.

**⇒ This SHARPENS the ruling rather than changing it.** (a) helps every ruleset we have today,
including the 24 predicates a tree can never touch. (b) helps the regime we are building toward. And
reason 3 becomes the load-bearing one: in that regime, an interpreted key expression blows the
per-packet budget, so **(a) is what makes (b) worth having.**

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
