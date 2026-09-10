# DESIGN — widen the clause, then refuse the fence

**Status:** proposed 2026-09-09, not drawn. Rests on
`FINDING-the-fence-says-what-the-clause-cannot.md`.

## The two obvious cures, and why both are rejected

### ⛔ Rejected — hoist in the compiler

Move a fence's predicate into its condition's alpha test automatically. Attractive: the mistake stops
existing rather than being refused, no migration, and it reaches predicates the surface syntax cannot
express.

**Rejected on the builder's principle, and the evidence backs the principle:**

> *"i argued against hoisting… because that allows the users to express things that aren't what they
> express."*

**And hoisting is not semantically free here.** One of the 24 failures was *a termination-bounding
fence is not the same input to the round-cap verifier once inlined.* That is not evaluation order —
it is a different program submitted to the verifier. An engine that silently relocates predicates
would sometimes analyse something other than what was written.

### ⛔ Rejected — refuse the fence first, migrate the corpus

This was tried. The check flagged 221 sites; the codemod migrated 63 of them cleanly; **the floor
found 24 failures, nine of which are predicates the inline position cannot express.** Refusing a form
while its alternative is unsayable does not improve anyone's code — it tells the author they typed it
wrong when the language left them nowhere else to go.

## The proposal — in this order, and the order is the point

### 1. Measure the admissibility gap

Extend the three-way differential with an axis that is **not** predicate *shape* but predicate
*admissibility*: for each shape, does the **fence** accept it, does the **inline clause** accept it,
does **Clara** accept it?

⭐ **This is the half `where-inline-computed.wat` left open.** It pairs predicates that work in both
positions and requires agreement; **the ones that work in only one never enter the corpus.** The new
axis must be built from shapes chosen to *probe the boundary*, not from shapes already in use — the
empty cells are the deliverable.

Known starting points, all accepted by Clara and refused by wat: a bare user-fn call; a user fn
nested inside a comparison; a closure/HOF.

### 2. Widen the inline clause grammar

`expr_is_provably_boolean` is the gate that refuses a foreign call in clause position. Widen it to
admit what the fence admits — bounded by whatever alpha-position evaluation genuinely requires
(purity, totality, no cross-condition reads), **not by what the current grammar happens to parse.**

⛔ **PINNED: every cell closed here must be justified by what an ALPHA TEST can soundly do, never by
"Clara allows it."** Clara is evidence that the restriction is not inherent to rete. It is not
evidence that any particular predicate is safe at fact entry in *this* engine.

### 3. Then refuse the hoistable fence

With the grammar widened, the check at `f47a9fccc` becomes fair: **every refused site has a legal,
expressible alternative**, and the codemod at `wat-scripts/fixes/hoist-where-into-condition.wat` can
migrate the corpus. `where` then means one thing only — a genuine cross-condition join, which is what
it is for.

⚠ The 2 diagnostic-identity failures and the 1 termination-verifier failure are **not** grammar gaps
and will still need deciding. They may be the legitimate residue where the fence is load-bearing.

## What this costs, honestly

Step 2 is a change to the language's accepted surface, and step 3 rewrites ~221 corpus sites — **143
of them in `wat-scripts/perf/grid/`, which invalidates all 29 recorded performance grids.** That is
the finding landing, not damage; the `#grid/Capture` header added in `35f1f4e8b` means the
replacement baseline will be the first provenance-bearing one.

**None of it should start while a merge into `main` is in flight.**
