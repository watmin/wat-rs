# DESIGN-STONE — THE ONE EXPRESSION CORE: the Op set, drawn

> **Status: DRAWN 2026-08-06.** Step 1 of #49's ruled shape (`ONE CORE, THREE ADJACENT FLIPS`,
> `DESIGN-STONE-compiled-where.md`). The builder ruled the layout the same day:
> **nesting — "matches the precedent"**. A sub-expression is a CHILD NODE, never a jump offset.
>
> This stone draws the set. It does **not** build it. Everything below is derived from the disk
> this session — the 75-row `RETE_OPS` table, `dispatch_rete_op`, `eval_test_core`,
> `compiled_cond::Op`, `RhsOp`, and a corpus census of all 173 `where` predicates.

## What the vocabulary actually is — 75 rows, and only SEVEN are lazy

`OpClass` is a **checker** taxonomy. It is not the evaluation taxonomy, and reading it as one is
the first trap. Grounded in `dispatch_rete_op` (`runtime.rs:8238`):

| `OpClass` | rows | runtime |
|---|---|---|
| `Alias` | 35 | `dispatch_keyword_head_value(core_name, args…)` — head substitution |
| `Redispatch` | 11 | **identical** to `Alias`; the class changes CHECKER routing only |
| `Form` | 9 | **identical** to `Alias` — core's own arm decides strictness |
| `Fallback` | 20 | same call **plus** the totality handler (below) |

So the runtime split is **not** the class split. Two of the nine `Form` rows — `enum::=` and
`enum::not=` — are `Form` **purely so the enum-ness gate can live in Rust**, because a
`TypeScheme` type-param has no bounds field and would be *"generic `=` wearing a per-type name"*
(the row's own comment). At runtime they are strict calls into core `=`.

**⇒ The genuinely lazy set is SEVEN:** `and` · `or` · `if` · `let` · `match` · `cond` · `fn`.
Everything else — 68 rows — is *evaluate the args, then call*.

**`not` is NOT one of them.** `:wat::rete::core::not` is class `Alias`: a strict boolean function.

### ⛔ The distinction that makes the one-core claim TRUE

`compiled_cond::Op::Or` / `Op::Not` are **clause** combinators — they combine *conditions that
bind*, over a scratch clone of the slot array, and are documented to discard a successful branch's
binds. The expression `or`/`not` combine **values** and bind nothing. They are different operators
that share a spelling.

That is why `compiled_cond`'s six variants fall out as **driver-level**, exactly as the #49 probe
predicted but could not explain:

| `compiled_cond::Op` | what it really is |
|---|---|
| `Bind`, `BindCheck` | **prologue** — populate slots from fact fields |
| `Or`, `Not` | **clause** combinators — not expressions |
| `Fail` | a compile-time-known constant |
| `Cmp` | the **only** expression, and it is the degenerate one (a 2-ary strict call) |

## The Op set

```rust
// src/rete/expr_ir.rs — THE ONE EXPRESSION CORE.
// NESTED sub-programs (builder's ruling 2026-08-06): a sub-expression is a CHILD NODE.

pub(crate) enum Expr {
    // ── leaves — zero allocation, zero lookup ────────────────────────────────
    /// Built once at compile time.
    Lit(Value),
    /// A `?var`, a `let` binding, or an `fn` parameter — a compile-time index into a
    /// flat frame. This is the arm that removes Step 0's `?var` lookup: a HashMap get
    /// plus TWO Span clones per variable READ, measured at 25.4% of the walk.
    Slot(u16),

    // ── the strict call — 68 of the 75 rows ──────────────────────────────────
    /// `Alias | Redispatch | Form{enum::=, enum::not=}`. `op` is an index into
    /// RETE_OPS resolved at compile time — no string head comparison at fire time.
    Call { op: OpIdx, args: Box<[Expr]> },

    /// The 20 `Fallback` rows — a strict call PLUS the totality handler. The literal
    /// `:undefined` marker is consumed at COMPILE time and never reaches the IR.
    /// `fallback` is evaluated ONLY at the undefined point, and there are THREE of
    /// them (`dispatch_rete_op`, grounded): the i64 family RAISES `IntegerOverflow`;
    /// the f64 family RETURNS a non-finite (NaN/±Inf — raw IEEE 754, no guard); the
    /// holon family RETURNS an outcome ENUM whose happy variant is projected and
    /// whose every other variant is the undefined point.
    CallFallback { op: OpIdx, args: Box<[Expr]>, fallback: Box<Expr> },

    /// The COMPOSITION DOOR — `head_ok` consults `sym.functions` before the vocabulary
    /// test, so a user fn proven pure ∧ det ∧ total ∧ rete-composed is admitted
    /// transitively. NOT a corner: the corpus census found 30+ such call sites inside
    /// `where` predicates, in two flavours — record accessors
    /// (`:arena::Timing/total-ns`, `:wr::Client/l2`) and user predicates
    /// (`:wr::is-risky?`, `:wsh::big?`). See STOP-3.
    CallUser { f: UserFnIdx, args: Box<[Expr]> },

    // ── control flow — the SEVEN lazy rows, NESTED ───────────────────────────
    If     { cond: Box<Expr>, then_: Box<Expr>, else_: Box<Expr> },
    And    (Box<[Expr]>),                                   // short-circuit, left→right
    Or     (Box<[Expr]>),                                   // short-circuit, left→right
    Cond   { arms: Box<[(Expr, Expr)]>, else_: Option<Box<Expr>> },
    /// Slots, not an `Environment`. This is what deletes `build_test_env` by
    /// construction rather than optimising it.
    Let    { binds: Box<[(u16, Expr)]>, body: Box<Expr> },
    /// 12 pattern arms, closed + recursive (`try_match_pattern`). All lower — see STOP-1.
    Match  { scrutinee: Box<Expr>, arms: Box<[(Pat, Expr)]> },
    /// A compiled lambda — required by the higher-order rows
    /// (`foldl`/`foldr`/`map`/`filter`/`reduce`). STOP-2.
    Lambda { params: Box<[u16]>, body: Rc<Expr> },

    // ── THERE IS NO ESCAPE HATCH. See "the hatch is refused" below. ──────────
}

/// Lowering is TOTAL OR IT REFUSES. There is no arm that quietly means
/// "run this the slow way" — a form this compiler cannot lower is a located
/// compile-time refusal naming the form, never a runtime behaviour.
fn lower(expr: &WatAST, …) -> Result<Expr, LowerError>;

/// The four surfaces differ ONLY in PROLOGUE and EPILOGUE. The core is identical.
pub(crate) struct Program {
    pub(crate) frame_len: u16,       // slot count, known at compile time
    pub(crate) root:      Expr,
    pub(crate) spans:     Box<[Span]>, // built ONCE; cloned only on an error path
}
```

### The one core, and the four drivers

| surface | PROLOGUE (fills slots) | EPILOGUE (reads the `Value`) |
|---|---|---|
| `where` | token bindings → slots | require `Value::bool` (`eval_test_core`'s contract) |
| `compiled_cond` | fact fields → slots (`Bind`/`BindCheck`) | bool **and** the slots ARE the bindings produced |
| `compiled_rhs` | token bindings → slots | the `Value` becomes a fact field |
| accumulator fold | gathered values → slots | the `Value` is the accumulated result |

## Why this is where the 540 ns → 21 ns lives — and why the LAYOUT fork is second-order

Step 0 measured 540 ns/eval against a 210 ns-per-10 000 hand-written-Rust floor (21 ns/eval), with
the walk at 77.3% and dispatch 74.6% *of the walk*. What the IR removes is **not** flatness:

| `eval_inner` pays, per node, every time | the IR pays |
|---|---|
| re-decides the shape by matching `WatAST` | an enum discriminant — a jump table |
| string-compares the FQDN keyword head | `OpIdx`, resolved once |
| `HashMap<String,_>` get + 2 `Span` clones per `?var` READ | `Slot(u16)` — an array index |
| builds `TrackedValue` / `Provenance` per node | nothing |
| `build_test_env`: `Arc<EnvCell>` + `HashMap` + a `String`/`Span`/clone per binding | a flat frame |

**Nested vs flat changes none of those.** It is a layout question about the *remaining* interpreter
loop, which is why "nesting matches the precedent" is both the smaller step and the cheap one — and
why its speed remains honestly unmeasured rather than argued.

## ★ THE HATCH IS REFUSED — and the refusal is the design

The first draft of this stone carried an `Interp(WatAST)` arm, *"the honest escape hatch"*, with an
acceptance note that it should be unreachable. **The builder pushed on it and it does not survive.**

It is the same shape as `RhsOp::Expr(WatAST)` — which this arc names as **the defect #49 exists to
close** (`compiled_rhs.rs:85`: *"NOT a full expression-tree compiler — that is `#49a`'s to build"*).
I drew the escape hatch into the stone whose job is to delete it.

Three reasons it is worse than untidy:

1. **It makes the perf claim unfalsifiable per predicate.** With a hatch, "`filter` barely moved" has
   two explanations — the compiler is not the bottleneck, or that predicate silently was not
   compiled — and nothing distinguishes them. R59 `NISI FRANGAS`: a number nothing depends on is a
   claim, not a proof.
2. **It is the mask class.** A compiler that quietly runs the slow path lies about what it did, and
   nobody trips over it. That is precisely what the no-hidden-failures law forbids, and
   `[[feedback_a_lossy_carrier_makes_the_mask_mandatory]]` is the shape.
3. **It throws away the trustworthy half of the instrument.** R62 `NOMINATO INSTRVMENTO`: the
   *rejection* column is an absolute fact about our substrate that no peer can bound. A
   total-or-refuse compiler HAS one — `LowerError` is that column. A falling-back compiler has none.

**And the hatch was never load-bearing.** Law A closes the language: a `where` predicate can contain
literals, `?var` reads, the 75 rete rows, and user fns admitted through the composition door (whose
bodies are themselves fenced). **There is no fourth thing.** So lowering can be total by
construction, and the hatch existed only to cover one file I had not read.

⇒ `lower()` returns `Result`. A form this compiler cannot lower is a **located compile-time
refusal**, never a runtime behaviour. The three grammar arms the interpreter already rejects
(`Vector`, `Set`, keys-destructure `Map`) become refusals *one phase earlier* — R29 `RVINA ERVDIT`,
strictly improved.

**`CallUser` is NOT a hatch, and the distinction is load-bearing.** A call into an interpreted
callee body is a **call boundary** — named, typed, countable, and inlinable later — the same thing
every compiler has at a foreign call. A hatch is an *expression* that silently means "not compiled".
One is a frontier you can measure; the other is a fact you cannot observe.

## ⛔ THREE STOPS — the parts I did NOT resolve, named so they are not hand-waved

- **✅ STOP-1 CLOSED by grounding (2026-08-06).** `match`'s grammar is **12 arms, closed and
  recursive** (`try_match_pattern`, `runtime.rs:14211`) — and the doc comment above `eval_match` is
  **STALE**: it says *"MVP-scoped to `:Option<T>`; user enums graduate in a later slice"*, which has
  not been true since arc 048/055.
  Real grammar: `:None`/`:wat::core::None` · int/float/rational/bigint/bool/string literals ·
  `Keyword` = user-enum UNIT variant · `_` · bare symbol (binds) · `(Some|Ok|Err p)` recursive ·
  `(:enum::Variant p…)` recursive with arity check · a bare list = **tuple destructure** ·
  `NilLit` = `Unit` · `Map` hash-destructure `{var :field …}` · and **`Vector`/`Set`/keys-destructure
  are already hard errors**.
  Every one lowers: literals → constants, keywords → a compile-time composed-path compare, symbols →
  a slot write, lists → structural match at known arity. The corpus's two sites are trivial
  (integer literals; `(Some v)` + `:None`). **`Match` needs no hatch.** The one arm carrying a real
  question is `Map` hash-destructure, which needs the field index resolved at compile time from the
  scrutinee's declared type — grounded as possible, not yet drawn.
  *(This is the file I had not read, and it is the whole reason the hatch got drawn.
  `[[feedback_ground_the_substrate_not_just_the_chronicle]]` — applied to a doc comment.)*
- **STOP-2 — the frame model across a closure is genuinely NEW.** `compiled_cond` has no closures;
  `Lambda` does. A `fn` handed to `foldl` may reference the enclosing frame, so the capture model
  (flat frame + copied captures vs a parent pointer) is an open decision, not a detail. The corpus
  exercises it: `foldl` ×4, `fn` ×4.
- **✅ STOP-3 RESOLVED, and the first answer was wrong (2026-08-06).** I wrote *"call the
  interpreter for the callee body — cheap, honest"* and defended it as a call boundary. The builder
  cut it: *"this `CallUser` screams 'we did not achieve totality, a user can surprise us'."*
  **He is right, and grounding split it into two separate facts.**

  **(i) For COMPILATION there is no gap — the callee LOWERS.** `classify_fn` (`purity.rs`) admits a
  user fn on law A **iff its body transitively contains only rete primitives**; a native is refused
  outright (`Axis::RetePrimitive => false`). So an admitted callee is *in the closed language*, and
  calling the interpreter for it was the `Interp` mistake one level down — treating an in-language
  construct as foreign. `CallUser` calls a **compiled `Program`**.

  **(ii) For TERMINATION the gap is REAL, and it is exactly one door.** See below.
- **STOP-4 — evaluation RETURNS A RESULT, always.** `CallFallback` faces the 20 partial rows, but
  `CallUser` can still raise and a `Slot` can be unbound (`RhsOp::Bind` documents that exact
  reachable arm and pins its message byte-for-byte). Do **not** design a non-failing evaluator; the
  differential against `eval_test_core` includes its errors.

## The gate

`eval_test_core` (`matcher.rs`) is the oracle, per #49's ruled shape — not `compiled_cond`, not
`compiled_rhs`. The differential asserts the compiled program and the interpreter agree on the same
token bindings, **including which inputs raise and with what message**.

**And the second gate, which the hatch's removal creates:** `lower()` must succeed on **every**
`where` predicate in the corpus — all 173 — and on every row of the where-expressivity corpus. A
`LowerError` on any of them is a **RED**, not a fallback. That is the acceptance criterion for the
flip, and it is checkable before a single predicate is wired: run the lowerer over the corpus and
count refusals. If it cannot reach zero, `where` does not flip — the adjacent-flip discipline
already says so.

## ⚠ Un-blocking note — the perf half of #49 cannot be measured end-to-end today

`node-share` is *the* axis this stone's numbers come from, and
`wat-scripts/perf/grid/node-share.wat` **dies at rule-compile** under law A. See
`FINDING-the-grid-axes-are-dead-on-run.md`. Step 0's numbers are unaffected (they come from a
Rust-side test that builds the network natively), but no end-to-end grid claim can be made until
those four axes are migrated.

## ⛔⛔ THE FIFTH AXIS — the predicate language is NOT total, and it is exactly ONE door

> **Builder, 2026-08-06, on the first draft's `CallUser`:** *"we have been grinding on the totality
> of this for days maybe a week…… this `CallUser` screams 'we did not achieve totality.. a user can
> surprise us….'"*

He is right, and the grounding is sharper than the instinct: a user can surprise us in **exactly one
way**, and it is not the one I drew.

### PROVEN, end to end — `wat-scripts/scratch-pad/probe-rete-predicate-termination-routes.wat`

```
:route-a-admitted true      <- `:wat::rete::compile` RETURNED for a rule whose `where` calls an
                               unboundedly-recursive user fn. pure ✓ det ✓ total ✓ rete ✓
:derived-at-n0 1            <- and it fires correctly at the base case
:bounded-fold true
```

A fact carrying `n > 0` hangs the fire **forever, with no diagnostic.** The fence cannot see it,
because `total` is documented to mean *defined on all inputs, never raises* — **not terminates**.

### There are only two candidate routes, and one is disconfirmed

The closed vocabulary has **no unbounded looping construct**: no `loop`, no `recur`, no `while`, no
`apply`, no `eval`. Its five HOFs (`foldl`/`foldr`/`map`/`filter`/`reduce`) are **bounded iteration
over a finite collection**, and nothing in the vocabulary can extend a collection mid-fold.

| route | verdict |
|---|---|
| **A** — a NAMED recursive user fn through `classify_fn`'s back-edge (`if seen.contains(fqdn) { return Ok(()) }`) | **OPEN** — proven above |
| **B** — a `let`-bound lambda referencing itself | **DISCONFIRMED** — `--check` accepts it, the runtime says `unbound symbol: self`. `let` is a SEQUENTIAL scope. Fails loudly; masks nothing. *(That `--check` accepts it at all is a real, benign checker gap — recorded, not chased.)* |

**⇒ Closing the back-edge closes the language.** The rete predicate sub-language becomes
**strongly normalizing**: every predicate provably terminates.

### It must be a FIFTH AXIS, not a fifth reading of law A

A recursive fn composed only of rete ops genuinely **satisfies** `RetePrimitive` — refusing it there
would make the diagnostic lie (*"not a rete primitive"* about something that is). This is #57's own
reasoning, quoted from the `enum::=` row: *"RetePrimitive is its own axis and not a fourth reading
of :Pure."* The same argument mints a fifth:

```
is-pure ∧ is-deterministic ∧ is-total ∧ is-rete ∧ IS-TERMINATING
```

`classify_fn` already detects the back-edge and already threads `seen`. The change is to **report**
it on the new axis instead of silently returning `Ok`. And the wall for adding an axis **already
exists**: `Axis::ALL` plus `axis_variant_names_round_trip_through_one_door` go non-exhaustive and
name the new variant at the compiler.

### ★ The totality cut and the COMPILER cut are the same cut

Without recursion, a `CallUser` callee can be **INLINED** — inlining always terminates — which
removes the call boundary entirely and makes the compiled predicate one flat closed tree. So closing
the back-edge is not a tax paid for safety; it is what makes the IR's best form reachable.

**What it costs, measured:** 26 user fns are called from `where` predicates today. **Zero** are
recursive; **zero** reach a cycle transitively. (94 recursive user fns exist elsewhere in the
corpus, so the idiom is common — it simply has not entered a predicate yet.) **The cut is free
today and gets more expensive every day it is not made.**

Filed as its own task. The RULING is the builder's — this stone records that the door is open,
proven, and that it is the only one.

### ✅ CORRECTED SAME DAY — the builder challenged the premise and TWO of my arguments died

> **Builder:** *"why is termination a concern here?… i get why in principal, but do we need to
> police the user here?… the op code eval will be correct, just unending?"*

**He is right about termination, and my headline argument was a FALSE ANALOGY. Retracted.**

I wrote: *"there is no jump-table opcode for 'does not terminate', by the identical argument that
armed `total?`."* **The argument does not transfer.** A *raise* is a control-flow ESCAPE the
dispatcher must be able to handle — that is a real dispatch problem, and it is why `total?` was
armed. Non-termination is the **absence** of an escape: the executor simply keeps executing,
correctly. There is no opcode needed and nothing to dispatch. I borrowed the *shape* of the totality
argument without checking that it applied. `[[feedback_a_claims_support_does_not_travel_with_the_claim]]`.

**And measuring it split the case in two, only one of which is ours:**

| predicate shape | outcome | a mask? |
|---|---|---|
| **tail**-recursive, unbounded | hangs forever; TCO holds, no stack growth (10M frames, clean) | **NO** — exactly the builder's read: *correct, just unending* |
| **non-tail**-recursive, deep | **SIGSEGV, core dumped, ZERO diagnostic**, exit 139 | **YES** — a hidden failure |

`(:wat::rete::core::i64::+ 1 (recurse …))` — the recursive call as an ARGUMENT — is the most natural
way to write a recursive accumulator, and it dies silently.

**But that mask is NOT a rete defect.** Any deep non-tail recursion anywhere in wat does this. It is
**task #58** (*"Stack exhaustion is a silent SIGSEGV — and our own stopgap is why"*), already ruled
NOT NOW, and refusing recursion inside predicates would be treating one symptom of a substrate-wide
disease at one site. **Wrong rung.**

### ⇒ REVISED DISPOSITION — the fifth axis is an OPTION, not a recommendation

**RETRACTED:** the fifth axis as a default. Policing termination in the predicate language is not
justified by anything measured here, and the seam's original line — *"totality does NOT include
TERMINATION"* — was the honest position I should have left standing rather than escalating.

**WHAT SURVIVES, and it is independent of all of the above:**

1. **`CallUser` calls a compiled `Program`.** The callee is in the closed language. That correction
   stands on its own — it was never a termination argument.
2. **The lowerer must handle a BACK-EDGE.** Recursion is admitted, so lowering must compile a
   recursive callee rather than inline it. Every compiler does this; it is a design note for #49,
   not a language restriction. Inlining becomes a per-callee optimisation gated on *not* recursive.
3. **The rete surface RAISES #58's PRIORITY without changing its ownership.** What is genuinely
   different here is not the defect but the **exposure**: the engine invokes a predicate on facts
   the author never chose — and R25's chaos engine is line-rate, adversarial input by design. A
   hostile fact that drives a predicate deep is a silent core-dump in the thing built to *stop*
   denial-of-service. That is an argument for fixing **#58**, at the substrate, not for a fifth axis.

## ★★ THE VERIFIER MODEL — the builder's frame, and it is OUR OWN PRIOR ART

> **Builder:** *"how do we attack this problem like an ebpf verifier?… we just impose some generous
> strict limit?"*

**Yes — and the frame supplies the justification my retracted argument never had.** It is also not an
analogy: `holon-lab-ddos/veth-lab/filter-ebpf/src/main.rs:840` is our own XDP rete, shipped February
at 1.3M pps, and its header states the discipline outright:

> *"The BPF verifier sees this as a ~100-instruction straight-line program with 2-3 map lookups and
> **no loops**. The kernel enforces a max of **33 tail calls**, giving us up to 32 DFS steps —
> **plenty for** trees with 15 static + 7 custom dimensions."*

Recursion was not banned and then mourned; it was **re-expressed as an explicit stack in per-CPU
scratch, driven by tail calls, under a hard bound chosen generous against the real workload.**
`[[R61 PAR NON ARGVIT, NOSTRA ARGVVNT]]` — consult our own work, not only the peer.

### ⛔ The load-bearing distinction: a STATIC REFUSAL, never a RUNTIME BUDGET

This is the whole answer to *"just impose a generous strict limit?"* — the limit's **phase** matters
more than its value:

| | what happens when it binds | verdict |
|---|---|---|
| **runtime budget** (fuel, step cap, timeout) | the predicate dies mid-evaluation → you must invent an outcome for *"ran out"* → every caller must face it | **A MASK.** The outcome-wall defect again, and the arc's law forbids it |
| **static limit at lower time** | the RULE IS REFUSED, located, naming the form → and once loaded, **every fire is guaranteed to complete** | **R29 `RVINA ERVDIT`.** The ruin teaches, at compile |

eBPF verifies at LOAD and never polices at runtime, and that is exactly why. Our fence already lives
at rule-compile — the correct phase — so this is a strengthening of a wall that exists, not a new one.

### The four limits, mapped

| eBPF | the rete predicate compiler |
|---|---|
| no back-edges in the CFG | no recursive callee in the lowered tree → **refuse at lower** |
| bounded loops (`bpf_loop`) with a provable bound | `foldl`/`foldr`/`map`/`filter`/`reduce` over a finite collection — **already the only repetition the vocabulary offers** |
| 512-byte stack · call depth 8 | **max `Expr` nesting depth**, computed exactly at lower |
| ~1M instruction complexity limit | **total node count** of the lowered tree |

### ★ Why this is STRICTLY BETTER than the fifth axis I proposed and retracted

It does not merely prove *terminates*. It yields a **statically known worst case per predicate** —
≤ N nodes, ≤ D depth — and that is the number a line-rate engine actually needs. R2 claims our edge
over Clara is a **jitter-free tail** (no GC to flinch). A predicate with an unbounded worst case makes
the tail unbounded, which quietly forfeits the claim. The verifier model restores it *by construction*.

And the depth bound **contains #58 for predicates specifically** — bounded nesting ⇒ bounded stack per
evaluation ⇒ the silent SIGSEGV is unreachable *here* — without pretending to fix #58 globally, which
remains a substrate defect with its own task.

### ⚠ WHERE THE NUMBERS COME FROM — and the trap to avoid

eBPF's limits are **not** derived from a corpus of existing programs. They come from what the
verifier can afford, set deliberately generous so no honest program meets them.

**Do NOT set ours from the corpus.** `[[feedback_the_corpus_is_a_record_of_what_happened_to_compile]]`
and R60's cut — *"you have no fucking clue what our users are going to do"* — apply exactly. The
corpus can tell us a limit **is not binding today**; it cannot tell us **where to put it**. The
honest basis is *what keeps the fire loop's worst case predictable*, and the number is the builder's
to set, the way the kernel set 33.

**One asymmetry, stated:** our XDP walker's bound was handed down by the kernel and the design was
shaped to fit inside it. Here there is no external authority — **we are the kernel.** So the bound is
a decision, not a constraint, and it should be recorded as one.

### What this changes in the plan

- `lower()`'s `Result` gains three refusal kinds: `RecursiveCallee`, `DepthExceeded`, `SizeExceeded`.
- Lowering computes `max_depth` and `node_count` for free while it walks; `Program` carries them, so
  the worst case per predicate is **inspectable, censusable, and gate-able**.
- The gate from the hatch's removal sharpens: not merely *"lowering succeeds on all 173 corpus
  predicates"* but *"and here is the measured depth/size distribution, and the limit sits far above
  its maximum"* — a number that can go red if a future predicate creeps toward the bound.
