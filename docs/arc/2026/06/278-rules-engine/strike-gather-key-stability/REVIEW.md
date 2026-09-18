# REVIEW — the stability gate is real; its PROOF tests a copy of it

> Weighed against my own mutation and my own reading.

## Accepted

- **Stability is measured, not asserted**: max distinct = **1** across accum, exists-of-leaf,
  not-of-leaf and exists-of-and, with call counts and denominators. The DESIGN demanded that before
  any hoist and you produced it.
- **The live gate is NOT vacuous.** `assert_gather_key_stability` asserts `r.distinct == 1` for
  every row on every axis, extracted as a verb — the `assert_applicable` shape.
- **The hoist is Leaf-only**; combinator drivers still take the per-token path, and you said so.
- **★ And the guarantee is stronger than measured — it is STRUCTURAL, which the SCORE undersells.**
  `gather_join_keys` reads only key NAMES: `.map(|(k, _)| k)` discards values entirely, then filters
  by `col_field_of(&intern, k)` — a function of `alpha_id` and the name. So the set depends on the
  token's key names, `alpha_id`, and `elements[0]`, all fixed across one node's token loop. A3's own
  field doc adds that `Or`/`Not` branch-local binds never reach the output slots, so the names are
  the node's compiled shape. **Say this in the SCORE: the hoist is safe by the shape of the data,
  and the gate is the regression net, not the argument.**

## ⛔ THE ONE THING: the mutation proof asserts a COPY of the gate

`gather_key_stability_gate_reddens_when_one_node_derives_two_key_sets` builds the synthetic, then
writes its own assertion inside the `catch_unwind`:

```rust
let distinct = rows[0].distinct;
let red = std::panic::catch_unwind(|| {
    assert_eq!(distinct, 1, "key-set stability: node 7 alpha 3 derived {distinct} sets");
});
```

**It never calls `assert_gather_key_stability`** — the verb the live gate uses. One rule, two
encodings, nothing forcing agreement.

**Driven, not argued.** I blinded the LIVE gate — `assert_eq!(r.distinct, 1)` →
`assert!(r.distinct >= 1)` — and:

```
MUTATION: the LIVE stability gate blinded (== 1  ->  >= 1)
PASS  gather_key_stability_gate_reddens_when_one_node_derives_two_key_sets
PASS  gather_key_sets_are_measured_per_node_and_alpha
Summary  2 tests run: 2 passed
```

**Both green with the gate blinded.** So the proof cannot detect the gate being weakened, which is
the whole thing it exists to prevent. Mutation reverted; file diff-verified.

This is the third instance of this exact shape in this arc — D1's tautological acceptance test, F1's
two-arm `:or` gate, and now this. It is the defect the strike was drawn to avoid, arriving in the
strike's own proof.

## What to change — one line

Have the synthetic drive the shared verb:

```rust
let rows_ref = &rows;
let red = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    assert_gather_key_stability("synthetic", rows_ref, "two key sets at one node");
}));
assert!(red.is_err(), "…");
```

Then blinding `assert_gather_key_stability` reddens the synthetic, and the proof and the gate cannot
drift. **Re-run my mutation and quote it** — blinded gate must now fail the synthetic.

## Not asking for

Anything else. The measurement, the Leaf-only hoist, the live assertions and the floor are accepted.
Do not widen to the combinator path.
