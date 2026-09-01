# DESIGN-STONE — the fence and the executor must share ONE head-space

> **Origin (2026-08-31).** Class A3 of `VIGILIA-2026-08-30-WORK-LIST.md`, found by `experiri`
> (driven). Re-driven here at HEAD `72d8d2c42`, both halves.

## Why

The acc-form fence (`wat/rete/compile.wat:597`) admits a head on **pure ∧ deterministic ∧ total ∧
`primitive?`** — and `primitive?` IS *"has a `RETE_OPS` row"*. The executor then resolves that head
through `lower_named_rete_fn` (`expr_ir/mod.rs:942-960`), whose **only** lookup is
`sym.get(head)` — the USER function table. No `rete_op_for` branch. Its sibling `lower_list`
(`:590-603`) has exactly that ladder and takes it.

**Admitted by one registry, dispatched by another.**

### Driven — the pair, at HEAD

```
experiri-acc-head.wat     :  #wat.runtime/MalformedForm
                             "unknown rete-defn :wat::rete::core::PersistentVector/length"
experiri-acc-wrapped.wat  :  "fired"
```

The second is the **same op**, in the **same position**, behind a one-line
`(:wat::rete::core::defn :probe::len [xs] -> i64 (PersistentVector/length xs))`. So the capability
is real and the operand is good; only *naming the row directly* fails. The raise reads
`unknown rete-defn` about a row of the very table the fence consulted to admit it.

## ★ THE ONE CONTRACT DECISION

**A head admitted by `primitive?` must be resolvable by the lowering.** `lower_named_rete_fn` gains
the `rete_op_for` ladder its sibling already has: a minted `RETE_OPS` row lowers to a synthesized
`Program` whose root is `Expr::Call { op, args: slots }`.

**The other direction — tightening the fence to refuse what `sym.get` cannot find — is REJECTED,
and the wrapped control is why.** The op runs correctly in this position; refusing it would delete
a working capability in order to make two registries agree, and would leave the split intact for
every future row. The fence is right; the ladder is missing.

## The algorithm

`rete_op_index(head)` (`vocabulary.rs:1545`) already returns the `u16` the `Expr::Call` op field
wants, and `rete_op_for(head).params.len()` gives the row's declared arity. Synthesize:

```rust
Program {
    frame_len: n,
    root: Expr::Call { op, args: (0..n).map(|k| Expr::Slot(k)).collect() },
    params: (0..n).collect(),
    reads: Arc::from([]),
    names: …,
    span,
}
```

**Arity is already walled, by this arc, hours ago.** The acc-form supplies exactly one argument
(`(head ?v)`), so a synthesized program for a row of arity ≠ 1 is refused at `exec_program_on` by
**D3**'s check (`057f9d494`) with both counts named — strictly better than today's
`unknown rete-defn`. **Do not add a second refusal for it.**

## The gate — this is what makes it a class cure

The class is *"any site that admits by `RETE_OPS` and dispatches by a different registry."*
`holon_rete_ops_have_opexec` (`expr_ir/eval.rs:1385`) gates one such pair and its own doc says
**"DO NOT WIDEN IT HERE"**, pointing at `reachability.rs` as strictly better because it *drives*
every row and requires a verdict.

So the gate belongs in `reachability.rs`, which already owns "does every `RETE_OPS` row work in
position X" and carries `synth` / `drive` / `Verdict` to do it. **Compute the eligible rows from
`RETE_OPS` at run time** — the rows whose declared signature fits the acc-form's `(head ?v)`
convention — and drive each as an acc-form head, asserting it FIRES. Today that set is one row;
it stays correct as rows are added, which a hard-coded name would not.

⚠ **`reachability.rs:65-68` currently states that only two positions are modelled and that the
accumulator position is deliberately absent.** Modelling it makes that paragraph false. **Move the
doc with the code** — a true sentence describing an absent guard is how A6's `unpack_driver` hid
for months.

## Blast radius

`src/rete/expr_ir/mod.rs` (the ladder) and `src/rete/reachability.rs` (the gate + its doc).
`lower_named_rete_fn` has **exactly one caller** — `src/rete/kernel/arm.rs:430`, the acc-form path
— enumerated, not assumed.

## Out of scope — AFFIRMATIVELY CUT

- **⛔ APPENDING THE BANKED HARNESS.** `harness-experiri/positions-3-4.rs.txt` is
  **RECONNAISSANCE, NOT A GATE** — verified here by count: **ONE** real Rust assertion across
  **EIGHT** `#[test]`s. Its README says so in a ⛔ CORRECTION. Appending it would put seven hollow
  tests on the floor in an arc that removed 26 of exactly that kind. **Its value is the fixture
  shapes and the two `.wat` repros; take those, write the assertion yourself.**
- **The `:then` value-operand position (D5).** The other half of the banked recon, its own row in
  the work list, with its own defect (`walk_nested_constructors` cannot tell a match ARM from a
  CALL). Not this strike.
- **Tightening the fence.** Rejected above, by the ★ decision, on driven evidence.
- **A rune for `experiri-acc-head.wat`.** It does not need one and must not get one: the
  `docs_wat_loads_or_declares_why_not` gate calls `startup_from_source`, which loads and
  type-checks but does not run `:user::main` — so that file already loads today, and the refusal
  it demonstrates is a run-time one. Verified, not assumed.
