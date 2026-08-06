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
    Match  { scrutinee: Box<Expr>, arms: Box<[(Pat, Expr)]> },   // STOP-1
    /// A compiled lambda — required by the higher-order rows
    /// (`foldl`/`foldr`/`map`/`filter`/`reduce`). STOP-2.
    Lambda { params: Box<[u16]>, body: Rc<Expr> },

    // ── the honest escape hatch ──────────────────────────────────────────────
    /// A shape not yet lowered, executed by the SAME routine the interpreted path
    /// uses, so the two surfaces cannot independently drift — exactly the role
    /// `RhsOp::Expr(WatAST)` plays today (`compiled_rhs.rs:85`).
    /// ★ ACCEPTANCE: this arm must be UNREACHABLE for every row of the
    /// where-expressivity corpus. It is the exception, never a comfortable majority.
    Interp(WatAST),
}

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

## ⛔ FOUR STOPS — the parts I did NOT resolve, named so they are not hand-waved

- **STOP-1 — `Match`'s `Pat` is unenumerated.** I did not read core `match`'s pattern grammar this
  session. Before `Match` is lowered, enumerate the pattern forms the checker admits; until then
  `match` lowers to `Interp`. The corpus uses it **twice**, so this is cheap to defer and dishonest
  to guess.
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

## ⚠ Un-blocking note — the perf half of #49 cannot be measured end-to-end today

`node-share` is *the* axis this stone's numbers come from, and
`wat-scripts/perf/grid/node-share.wat` **dies at rule-compile** under law A. See
`FINDING-the-grid-axes-are-dead-on-run.md`. Step 0's numbers are unaffected (they come from a
Rust-side test that builds the network natively), but no end-to-end grid claim can be made until
those four axes are migrated.
