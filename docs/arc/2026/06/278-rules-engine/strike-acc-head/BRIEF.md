# BRIEF — give the lowering the ladder the fence already implies

The acc-form fence admits a head because it has a `RETE_OPS` row; the lowering then looks the head
up only in the USER function table and raises `unknown rete-defn` about a row of the table that
admitted it. Driven: `PersistentVector/length` used directly as an acc-form head is refused, and
the **same op** behind a one-line user `defn`, in the **same position**, prints `"fired"`. Give
`lower_named_rete_fn` the `rete_op_for` branch its sibling `lower_list` already has, then gate the
class. Read `DESIGN.md` beside this file first — its ★ pins the direction and says why the opposite
one is rejected, and its "out of scope" section names four cuts, one of which is a trap you would
otherwise walk into.

## Read in order

1. `src/rete/expr_ir/mod.rs:942-960` — `lower_named_rete_fn`. The `sym.get(head)` at `:947` is the
   whole lookup; the `None` arm at `:950` is the raise you saw driven.
2. `src/rete/expr_ir/mod.rs:585-604` — `lower_list`'s rete-row branch: the sibling that HAS the
   ladder, including how it gets `op` and builds `Expr::Call`. Copy this shape.
3. `src/rete/vocabulary.rs:1538-1554` — `rete_op_index` (returns the index the `op: u16` field
   wants) and `rete_op_for` (returns the row, whose `params` gives declared arity).
4. `src/rete/vocabulary.rs:276-300` — the `ReteOp` row: `rete_name`, `core_name`, `class`,
   `params`, `ret`. `params` is what tells you which rows fit the acc-form's one-argument shape.
5. `src/rete/kernel/arm.rs:425-435` — the **only** caller of `lower_named_rete_fn`, the acc-form
   path. Read it to see what it does with the `Program` you return.
6. `src/rete/reachability.rs:60-80` and its `synth` / `drive` / `Verdict` machinery
   (`:215`, `:262`, `:108`) — the gate's home and the tools it is built from. **`:65-68` is the
   paragraph that goes stale the moment you model this position.**
7. `docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-acc-head.wat` and
   `experiri-acc-wrapped.wat` — the driven pair. These are your fixture shapes.

## Sketch

```rust
// lower_named_rete_fn, BEFORE the user-table lookup
if let Some(op) = rete_op_index(head) {
    let n = rete_op_for(head).map(|r| r.params.len()).unwrap_or(0);
    return Ok(Arc::new(Program {
        frame_len: n as u16,
        root: Expr::Call { op: op as u16, args: (0..n).map(|k| Expr::Slot(k as u16)).collect() },
        params: (0..n).map(|k| k as u16).collect(),
        reads: Arc::from([]),
        names: /* what the struct needs */,
        span: span.clone(),
    }));
}
```

## The gate

In `reachability.rs`: compute from `RETE_OPS` the rows whose declared signature fits the acc-form's
`(head ?v)` convention, drive each as an acc-form head, and **assert it fires**. Do not hard-code
`PersistentVector/length` — compute the set, so a future row of that shape joins the sweep by
itself. Then fix `:65-68`, which currently says this position is deliberately unmodelled.

## Blast radius

`src/rete/expr_ir/mod.rs` and `src/rete/reachability.rs`. Nothing else — `lower_named_rete_fn` has
exactly one caller (`arm.rs:430`), and it is the path this fixes.

## Traps named in advance — each with its step

1. **⛔ DO NOT APPEND `positions-3-4.rs.txt`.** It is reconnaissance: **one** real assertion across
   **eight** `#[test]`s, counted. Seven of them `println!` a matrix and compare it to nothing.
   **Step:** take the `.wat` fixture shapes and write the assertion yourself. If you find yourself
   copying a test whose body ends in `println!`, that is the trap.
2. **Arity ≠ 1 rows are already handled.** D3 landed the `exec_program_on` arity wall this session,
   so a synthesized program for a row of the wrong arity is refused with both counts named.
   **Step:** do not add a second refusal, and do not filter rows in the lowering — filter in the
   *gate*, which is asking a different question.
3. **The gate must be able to FAIL.** **Step:** revert the ladder, confirm the gate goes RED, put
   it back. A sweep that cannot demonstrate it discriminates is one that passes when it reaches
   nothing — the exact finding `complectens` made about 10 of 15 file-walking gates in this tree.
4. **`reachability.rs:65-68` goes false when you model this position.** **Step:** rewrite that
   paragraph in the same commit. A true sentence describing an absent guard is how A6's
   `unpack_driver` survived; do not mint a second one.
5. **New test code trips `wat::lint`.** The last strike's floor went red on
   `no_loose_string_assert` for a `contains` in a new probe. **Step:** run
   `cargo nextest run --release -E 'binary_id(wat::lint)'` before you report, and prefer exact
   `assert_eq!` over `contains` on any deterministic value.
6. **`experiri-acc-head.wat` must not gain a rune.** It already loads (the docs-wat gate calls
   `startup_from_source`, which does not run `main`); its refusal is at run time. **Step:** leave
   it alone.

## STOP triggers

- **STOP-1** — if `Program` needs a field you cannot fill honestly for a synthesized row (`names`,
  `reads`), STOP and report which and why. Do not invent a placeholder that reads as real.
- **STOP-2** — if any currently-green test goes red, STOP and report which. In particular a
  reachability cell flipping verdict is a finding, not noise.
- **STOP-3** — if the eligible-row set computed from `RETE_OPS` comes out empty, STOP: the gate
  would then pass by reaching nothing, which is trap 3 wearing a green.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-calluser-arity/` — the strike immediately before this
one: DESIGN with a pinned contract, probes RED first, one drive per arm, and a report stating per
arm **proven / reachable-but-not-driven / not-reachable-and-why**.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twelve riders before you each returned a prescription of
mine that did not survive contact. The last two found that I had named ONE site where there were
three, and then ONE where there were six — and that I cited a function by a name I had never
grepped. Those were worth more than the code. If a step here is wrong, unnecessary, or impossible,
say it plainly.
