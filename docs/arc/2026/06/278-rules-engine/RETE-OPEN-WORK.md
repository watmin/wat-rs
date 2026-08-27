# Rete — the open work, indexed

> **What this file is.** One place to find everything still open on rete, written 2026-08-27 when
> `RETE-FIX-LIST.md` reached empty and "is rete done?" became a fair question. It is not.
>
> **It is an INDEX, not a second copy.** Items that already live somewhere keep living there and
> are POINTED at from here; only items with no other home are owned by this file. A second place
> holding the same truth is the drift this arc keeps pulling out — if you find a detail here that
> also exists in `NEXT-STRIKES-theater-hunt.md`, the other file wins and this one is stale.
>
> **The distinction that made this list necessary:** `wat-gen` the LIBRARY is finished for the
> immediate term — this session used it hard and wanted no combinator it did not have. `wat-gen`
> APPLIED TO RETE is not finished. Those are different questions and conflating them reads as
> "we're done" when only half is.

## Closed 2026-08-26/27 — so the list shows motion, not just backlog

- Fuzzer families **A, B, C** — three silent wrong-answer defects. Ratchet 120 → 72 → **0**, now
  an equality gate rather than a ratchet. See `RETE-FIX-LIST.md`.
- **D** — a bind under `:not` consumed nowhere, refused at declaration-check time.
- Fuzzer widened 1260 → 3168 shapes (retraction, `Option`-returning accumulators) plus a second
  file at 936 (five scalar types, per-type join keys). **4104 generated shapes, zero divergences.**
- Grid: 33/33 `:accuracy :match`, 33/33 `:winner :us`. One 4x perf regression found and fixed.

---

## PILE 1 — fuzzing gaps (owned here). More wat-gen use; ranked by yield.
> **1.1 is done** (2026-08-27). 1.2 is now the head of this pile.

### ~~1.1 Interleaved insert/retract — true truth maintenance~~ · **DONE 2026-08-27**
`wat-tests/rete/differential-fuzz-tms.wat`. The design question this entry raised — *what does
interleaving mean for a fixpoint that recomputes* — resolved to: **not "retract mid-fixpoint"**
(there is no such hook, and `retract` is stage-only by contract) but **a PROGRAM of operations**
with fires among them. That reframing bought a property stronger than engine-vs-engine:

> **PATH INDEPENDENCE** — running a program of inserts, retracts and fires and then firing must
> equal firing ONCE over the multiset that program ends with.

Four numbers per case (native/oracle × interleaved/one-shot), all four must agree, and the
coordinate separates the failures: `ni != oi` is engine disagreement, `ni != n1` is native
carrying state across a FIRE, `oi != o1` would mean the REFERENCE is path-dependent and every
other fuzzer's agreement is suspect. `fire-rules` is contracted as a function of `Session/facts`,
so the only way an interleaved run can differ is state surviving a fire — families A and C one
level up, where those were state surviving a ROUND.

**card 1372 (7³ programs × 4 queries), violations 0**, 39.3s isolated / 73.4s loaded. `prog-len`
is a one-line dial and the deeper setting was RUN, not imagined: at 4 it is card 9604, violations
0, 4m47s isolated — real coverage (it reaches insert/fire/retract/fire) but it would nearly triple
the floor for a space that found nothing at 3 either. Committed at 3 with that measurement
recorded in the file.

### 1.2 Generated rules — the whole `:then` side, and multi-rule interaction
Both fuzzers use a FIXED inert chain. Rule shapes ride along only because a query carries the
rule's own LHS. Never generated: multiple rules sharing an alpha (the `node-share` shape), a
generated `:then` (kwargs order, multi-fact, fn-headed), and rule-vs-rule stratification beyond
the chain. Note `:then` corruption is the exact class arc 294's `defrule` wall exists for, which
means the wall's own coverage is hand-written.

### 1.3 Query params
`:params []` in both files, always. `where-query-params` is a hand-written axis; nothing generates
one.

### 1.4 Deeper combinator nesting
The filter families are FLAT — `:not` of a fact, `:or` across conditions, `:not` of a constraint.
`:not` of `:and` of `:or` and friends are hand-written axes only.

---

## PILE 2 — owned by `NEXT-STRIKES-theater-hunt.md` § "WHAT REMAINS OPEN". Read it there.

19 L1 ward findings stand, each with `file:line`: **`conformare` x9** (a real wat span discarded
for `rust_caller_span!()`, so a user's malformed `:then` points at wat-rs's own Rust source rather
than their file — and `arm.rs:316` already does it correctly in the same file), **`vocare` x6**,
**`intueri` x3**, **`exigere` x1**.

**None of these can produce a wrong answer** — that is why they sit below the piles above, and
also why they will never be found by a differential. `conformare` x9 is the one with real user
impact.

---

## PILE 3 — structural, and elevated out of PILE 2 deliberately

These two come from `circumspicere` and are tracked in `NEXT-STRIKES-theater-hunt.md`, but they
are not tidiness and should not be read as part of the ward tail.

### 3.1 The fixpoint has no cap
`fire_fixpoint_delta_armed` ends only when the delta empties — no round counter, no deadline, no
memory ceiling. A rule deriving a structurally-novel fact each round **hangs the calling thread
and grows heap with no diagnostic**. `DESIGN-STONE-4b-cascade-fixpoint` names it as a deliberate
Datalog choice, but that reasoning lives ONLY there — README, USER-GUIDE, CLAUDE.md and
`rete.wat` say nothing. Not hypothetical: the grid harness needed a cgroup blast door after an
analogous run OOM'd the build machine. **Nothing protects an embedder.**

### 3.2 The arc's closing condition is checked by no CI job
`PERF-ARC` states it as "differential-tested bit-for-bit against the wat oracle AND benched at or
past Clara". The parity scripts need a JDK + Clojure the runner lacks, so they never run there — a
Clara-parity or throughput regression **merges fully green**. `run-all.sh` documents this having
already happened once, with four axes dead for days. Every Clara agreement in this session was
established BY HAND.

---

## PILE 4 — generated by this session (owned here)

### 4.1 A reachability ledger over `RETE_OPS` — and it must be PER CALL-SITE KIND
Rows are gated for purity, totality, arity and type — but **never for "can a user actually get
here"**.

**The design constraint was bought expensively, by getting it wrong twice in one hour.** The
motivating case is `keyword::=` (arc 109's
`NOTE-keyword-is-two-disjoint-type-names-...`). Two obvious ledger designs BOTH fail on it:

- a **grep** ledger calls it DEAD — it appears in only two scratch-pad files;
- a **compiles-somewhere** ledger calls it FINE — those files compile and fire.

Both are wrong, because reachability is not a property of the ROW. `keyword::=` is reachable
inside a `(:wat::rete::where …)` fence and NOT reachable as an inline alpha constraint, and the
difference is a real defect a user cannot infer. So the ledger's unit is **(row x call-site kind)**
— at minimum inline-constraint vs where-fence — and its evidence must be a rule that COMPILES AND
FIRES in that position, or a written reason it cannot.

Measured 2026-08-27: 74 rows (Alias 35, Fallback 20, Redispatch 10, Form 9). Only **3** appear
nowhere in the 1569-file `.wat` corpus at all — `:wat::rete::core::Tuple`,
`:wat::rete::core::f64::-`, `:wat::rete::core::keyword::not=` — which is the FLOOR of the problem,
not its size, since the keyword case proves presence is not reachability.

A full 74-row conformance corpus is its own strike: `Fallback` needs the 4-arg `:undefined` marker
shape, `Redispatch` needs collections, `Form` is bespoke per row. Scope it deliberately; do not
start by hand-writing 74 snippets.

---

## The order, and why

1. **4.1 the reachability ledger** — small, converts a proven-live defect class into a standing
   gate, and immediately tells us how much dead surface there is.
2. **1.1 interleaved retract** — the highest-yield fuzzing gap, in the territory that has paid.
3. **3.1 the fixpoint cap** — the worst item here for anyone who is not us.
4. **3.2 CI parity**, then **1.2 generated rules**, then the PILE 2 tail with `conformare` first.

> ⚠ **A green fuzzer is not an empty list.** 4104 shapes at zero divergences means the engine is
> correct IN THE REGION THESE GRAMMARS REACH. Both spaces are hand-authored; they cover what
> someone thought to encode. That is why the vacuity findings of 2026-08-27 matter more than they
> look — a vacuous dimension shrinks the region without shrinking the number.
