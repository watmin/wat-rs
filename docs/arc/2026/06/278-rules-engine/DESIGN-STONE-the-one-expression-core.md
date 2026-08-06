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
- **STOP-3 — `CallUser` is the biggest open question, and it is not a corner.** The fence proves
  the callee pure ∧ det ∧ total ∧ rete-composed, but its BODY is arbitrary wat. Two honest routes:
  (a) compile it too — the real win, but the composition door **admits recursion by design** (the
  purity walk returns `Ok` on a back-edge), so a compiled call must handle a back-edge; or (b) call
  the interpreter for the body — cheap, honest, and leaves most of the corpus's user-fn sites
  interpreted. **Pick (b) first and MEASURE what it leaves on the table.** Do not assume (a).
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
