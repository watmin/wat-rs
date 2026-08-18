# BRIEF — compile the `where` predicate (`src/rete/compiled_where.rs`)

> ⛔ **STALE as of 2026-08-17. Do not execute this brief.**
>
> The builder cut `Op::Interp` on sight. A third sibling compiler was refused in
> favor of **one core** (`src/rete/expr_ir.rs`) and three adjacent flips.
> Live breadcrumb: **`CURRENT-STATE-annihilate-interpretation.md`**.
> Design: `DESIGN-STONE-the-one-expression-core.md` + `DESIGN-STONE-compiled-where.md`
> (the three-step plan and Step 0 numbers are still load-bearing; the `Interp` arm
> and the `compiled_where.rs` sketch in *this* file are not).
>
> Kept on disk so the hatch's refusal has a document to point at.

Design: `DESIGN-STONE-compiled-where.md` (read its Step-0 table and the probe results first).
Prior art to mirror: `src/rete/compiled_cond.rs` — same shape, same contract style, one arm different.

## The work, in one paragraph

`where` is the one condition family never compiled. Every token × every TestNode, `eval_test_core`
(`src/rete/matcher.rs:1102`) builds a child `Environment` and walks a `WatAST` through the general
interpreter — **540 ns/eval, measured, against a 21 ns floor.** Build `src/rete/compiled_where.rs`:
compile each TestNode's predicate **once** at the fire's setup site into a slot-resolved op vector,
and execute it against a reused scratch buffer with **no `Environment` and no head dispatch**. Any
shape the compiler does not model compiles to `Op::Interp`, which calls today's `eval_test_core`
verbatim — **fall back, never fail.**

## Read in order, and why you are being sent there

1. **`src/rete/compiled_cond.rs`** (445 lines, whole file) — the exemplar. Copy its structure: the
   `Op` enum, `compile_*` walking a classified shape, `exec_*` over a caller-owned
   `&mut Vec<Option<Value>>` scratch, the `n_slots()` accessor, the module doc that states the
   contract. Your module is its sibling.
2. **`src/rete/matcher.rs:1067-1140`** — `build_test_env` + `eval_test_core`. This is the exact
   behaviour you must reproduce: the child env, the `Value::String` binding keys, `eval_inner`, and
   the **`TypeMismatch` on a non-bool result** (`:1123`). `eval_test_core` stays; it is your
   `Op::Interp` body and the other half of the differential.
3. **`src/rete/kernel.rs:2132-2154`** — where `compiled_cond` forms are built at setup, keyed by
   alpha id, with **one** scratch buffer sized to the max `n_slots`. Your build loop goes beside it,
   keyed by TestNode id.
4. **`src/rete/kernel.rs:2189-2205`** (`beta_readers`) — proof that walking `node_ids` and reading
   node records is already done at setup. Your TestNode walk is the same shape.
5. **`src/rete/kernel.rs:2705-2745`** — the filter loop and the call you re-point (`:2727`), plus the
   `filter:test-evals` / `filter:test-pass` counters already in place.
6. **`tests/rete/probe_arc278_compiled_where_ops.rs`** — the committed probe. Its first test is the
   worked reference for reading a record field without the head dispatch; its second prints the
   corpus's predicate shapes, which is your op-coverage worklist.
7. **`src/rete/kernel.rs` tests: `node_share_where_cost_decomposition`** — the before/after
   instrument. Same test, re-run after your change; arm B is your score.

## Implementation sketch — fill this in, do not invent the shape

```rust
// src/rete/compiled_where.rs
pub(crate) enum WOp {
    Slot(usize),                                  // a ?var, pre-resolved
    Lit(Value),                                   // a literal, built once
    Field { recv: Box<WOp>, name: Arc<str> },     // (:Class/field ?r) — no head dispatch
    Cmp   { op: CmpOp, lhs: Box<WOp>, rhs: Box<WOp> },
    Arith { op: ArithOp, lhs: Box<WOp>, rhs: Box<WOp> },
    Not(Box<WOp>),
    Or(Vec<WOp>),
    And(Vec<WOp>),
    Interp(WatAST),                               // anything above unmodelled — see below
}

pub(crate) struct CompiledWhere {
    root:    WOp,
    /// (binding key, slot) for the ?vars the predicate READS — pre-resolved at setup.
    reads:   Arc<[(Value, usize)]>,
    n_slots: usize,
}

pub(crate) fn compile_where(expr: &WatAST) -> CompiledWhere;

pub(crate) fn exec_compiled_where<B: Bindings>(
    c: &CompiledWhere, bindings: &B, scratch: &mut Vec<Option<Value>>,
    env: &Environment, sym: &SymbolTable,
) -> Result<bool, EvalBreak>;
```

- **Load the slots once per call**, from `reads`, before walking `root`. That is the env build,
  replaced.
- **`Op::Interp` builds an env for the fallback subtree ONLY** — bind just the `?var`s that subtree
  reads, then call `eval_test_core`. Count every entry with a new
  `census_count("filter:test-interp-fallback")`.
- **`Field` resolves at runtime, not compile time.** The receiver's class is not known at setup (the
  probe proves this). Read the class off the `Value::Aggregate`, look the field name up in
  `sym.types()`, take the index. That still skips the entire head-dispatch chain.
- **Reuse, never reimplement, `compare_values`** from `matcher.rs` — the same reason
  `compiled_cond.rs:432` gives: an ordering definition must not be able to drift between the
  interpreter and the executor.

## Blast radius

`src/rete/compiled_where.rs` (new), `src/rete/mod.rs` (declare it), `src/rete/matcher.rs` (nothing
removed — `eval_test_core` and `build_test_env` stay), `src/rete/kernel.rs` (build the forms at
setup, one scratch buffer, re-point `:2727`). **Nothing under `wat/`** — the oracle does not move.
No new public wat verb. No change to any `.wat` file.

## STOP triggers — each is a rejection criterion. Ship nothing; report the gap.

**STOP-1.** If reproducing `eval_test_core`'s **error** behaviour is not possible for some op — a
non-bool result must still raise the located `TypeMismatch` at `matcher.rs:1123`, and an arithmetic
raise (div-by-zero) must still propagate — STOP. Returning `false` where the interpreter raises
converts a located error into a silent non-match, and that is the class this arc exists to kill.

**STOP-2.** If a shape you cannot model cannot be routed to `Op::Interp` — if the fallback needs
something the executor cannot hand it — STOP. Do not compile an unmodelled shape to a constant, to
`false`, or to an unconditional fail. `compiled_cond` may use `Op::Fail` because `eval_clause`
returns `None` for those same shapes; **`eval_test_core` does not**, so the same move here is a
behaviour change wearing a specialization's clothes.

**STOP-3.** If the differential (below) disagrees on ANY (predicate, bindings) pair from the corpus,
STOP and report the pair. Do not adjust the differential to accommodate the executor.

**STOP-4.** If building the compiled forms at setup requires the round loop's state — if the TestNode
records are not readable at `kernel.rs:2189`'s point the way `beta_readers` reads them — STOP and
report what setup cannot see.

## The RED gate — write it first, watch it fail, then make it pass

A differential over every `where` predicate in the corpus (the probe's second test enumerates them)
crossed with bindings drawn from a real fire:

```
for (predicate, bindings) in corpus_pairs:
    assert compiled(predicate, bindings) == eval_test_core(predicate, bindings)   // Ok(bool)
    assert compiled_err_kind == interp_err_kind                                   // and Err
```

Plus, on `node_share_filter_eval_census` at `[50 200]`:
`filter:test-env-builds == filter:test-interp-fallback` (both counters reported, not just the first),
with `filter:test-evals > 0` held as the non-vacuity guard.

## Out of scope = REJECTED (do not widen)

- **(b), the discrimination tree.** Its own stone, and it lands second by ruling.
- **Hoisting the per-TestNode `new_tokens` clone** (`kernel.rs:2701`) — task #50, a separate stone.
  Bundling it destroys the attribution.
- **A monomorphic inline cache on `Field`'s class→index lookup.** Land the honest lookup first; the
  cache is a follow-on with its own measurement.
- **Deleting `eval_test_core` or `build_test_env`.** Both stay: the reference half of the
  differential, and the `Op::Interp` body.
- **Any change under `wat/`, or to any `.wat` file.**
- **Keyword/string interning.** A language-level change with its own arc.
