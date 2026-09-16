# BRIEF — one `Option`, two facts

Split `operand_slot`'s conflated `None` into named outcomes so "the bucket was empty" and "the var
names nothing" stop sharing an answer. Today `Sum` reads the second as the first and returns
`i64(0)`; `Min`/`Max`/`Mean` read it as absence and drop the fact. Read `DESIGN.md` beside this
file first — its ★ ONE CONTRACT DECISION and its arm table govern.

## Read in order, and why

1. **`src/rete/kernel/fire/acc.rs`, `operand_slot`** — twelve lines, and both facts are visible in
   them: `bucket.first()?` is the empty bucket, `.position(…)` is the missing var. This is the
   function that changes.
2. **`acc.rs:321-323`** — `fold_bucket`'s `Sum` arm. `else { return Ok(Some(Value::i64(0))) }`.
   The probe reaches this one.
3. **`acc.rs:345-347`** — the `Min`/`Max`/`Mean` arm. `else { return Ok(None) }`. Same `None`,
   different wrong answer. The probe does NOT reach this one; see the mutation section.
4. **`acc_refusal`** in the same file — built by `c449cd24d` for exactly this class. Reuse it; do
   not invent a second refusal shape.
5. **`packed_operand_field` and its doc** — the sibling `Option` that is CORRECT. Read the doc so
   you can tell them apart; DESIGN cuts it explicitly.
6. **`strike-silent-zero/probe.rs.txt`** — append to the existing
   `tests/rete/probe_arc278_import_fold_key.rs`. It reuses that file's helpers (`call_import`,
   `rewrite_sum_keys`, `unbound_key`, `poke`, `field_of`, `seq_values`) and needs no new fixture.

## Implementation sketch

```rust
pub(super) enum OperandSlot {
    EmptyBucket,        // sum's identity applies; min/max/mean drop
    Slot(usize),
    Unbound,            // the fold names a var no condition binds — REFUSE
}
```

Every caller matches all three. No `_ =>`. The point of the enum is that a future arm cannot
silently inherit one meaning while intending the other, so a catch-all would give the defect its
representation straight back.

## Blast radius

`src/rete/kernel/fire/acc.rs` and the probe. **`operand_slot` has exactly two callers**, verified
on the disk at HEAD `2a7051c67`:

```
src/rete/kernel/fire/acc.rs:136   pub(super) fn operand_slot(
src/rete/kernel/fire/acc.rs:321       Sum          — the arm the probe reaches
src/rete/kernel/fire/acc.rs:345       Min/Max/Mean — the arm it does not
```

If you find more than these two, STOP rather than widening.

## STOP triggers

1. **If `operand_slot` has a caller outside `acc.rs`, STOP** and surface it before changing a
   `pub(super)` signature.
2. **If you find yourself writing `_ =>` on the new enum, STOP.** That re-mints the conflation in
   its own cure, which this arc has done before and recorded.
3. **If the `Min`/`Max`/`Mean` arm's `Unbound` case is hard to reach, that is a reporting
   obligation, not a licence to leave it `Ok(None)`.** Convert it anyway and say it is unproven.
4. **If converting touches `packed_operand_field`, STOP** — DESIGN cuts it, and its `None` is a
   legitimate dispatch.

## The mutation proof — TWO arms, and the probe reaches one

- **`Sum` (`:321`)** — the banked probe is RED today and must go GREEN via `Ok(Err(_))`. Free.
- **`Min`/`Max`/`Mean` (`:345`)** — **prescribed, not merely warned about.** The fixture's slot
  rule uses `acc::sum`. Add a second rule over the same `:from` using `acc::min`, export it, tamper
  its fold key the same way, and assert the refusal. If the pass will not route a `min` fold down
  the slot path on that shape, say so, name it unproven, and convert it regardless.
- Then break each `Unbound` arm deliberately — return the old answer — and confirm the matching
  probe reddens for **that** arm. Restore.

Report each arm as **proven**, **converted but unproven**, or **not reachable, and why**.

## A prior comparable result

`strike-acc-panics/` (`c449cd24d`) — same file, same refusal helper, and its rider's report is the
standard: it rejected a prescription of mine that would have cost five allocations on a path
measured at ~27% of fire, and said so plainly. Do that.
