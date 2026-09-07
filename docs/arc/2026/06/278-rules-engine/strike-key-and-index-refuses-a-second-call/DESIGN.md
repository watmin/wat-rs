# DESIGN — the first-keying door does not refuse a second keying

Board row carried from A1 (*"deliberately cut there"*), re-grounded at HEAD 2026-09-07. **The last
A1 remnant.**

## ⚠ Severity — smaller than the row implies, and the DESIGN says so first

The board reads: *"a third caller computing a different key list would be hidden rather than
surfaced."* True — and grounding shows the hazard is **triply** guarded today:

1. **Caller 1** (`fire/mod.rs:883`) is inside `if !idx.left_idx.is_keyed(join_id)`, with the
   `else` arm going to `writer()` instead.
2. **Caller 2** (`hash_join.rs:282`) is inside the `first_keying` branch, where
   `first_keying = !left_idx.is_keyed(*child_id)`.
3. **Both reuse the stored list when present** — caller 1 via
   `match idx.left_idx.keys(join_id) { Some(k) => k.to_vec(), None => gather_join_keys(…) }`,
   caller 2 via `else { left_idx.keys(*child_id).expect("keyed join has keys").to_vec() }`.

So a second call **cannot happen**, and even if it did the lists **could not disagree**. This is
hygiene on an unreachable branch — not a live defect, and the SCORE must not dress it as one.

## The finding

`src/rete/kernel/session.rs:258-272`:

```rust
/// The ONE first-keying door. Sets the key list and indexes `toks` in a
/// single act. A later call does not replace keys; it still indexes `toks`.
pub(crate) fn key_and_index(&mut self, join_id: i64, keys: Vec<Value>, toks: &[Token], mut key_of_tok: …) {
    self.keys.entry(join_id).or_insert(keys);
    …
}
```

**The door is named for a contract it does not enforce.** `or_insert` silently keeps the first list
while indexing with the *second* caller's `key_of_tok` — so `keys(join_id)` would describe one key
set while the buckets were built from another. A later probe using `keys(join_id)` would miss.

And the doc sentence — *"A later call does not replace keys; it still indexes `toks`"* — reads as
though later calls are an expected mode. They are not: both callers are structured to prevent one,
and `writer()` (A1's cure) is the sanctioned way to add tokens after keying.

**The invariant lives at two call sites, independently. Nothing at the door holds it.** Same shape
as A4, where `from_parts` being the sole stamping site was the whole protection.

## THE ONE CONTRACT DECISION

**Make the door refuse a second keying, and correct the doc to match the API that already exists.**

```rust
debug_assert!(
    !self.keys.contains_key(&join_id),
    "key_and_index is the FIRST-keying door and join {join_id} is already keyed — \
     add tokens through `writer()`, which A1 made the sanctioned post-keying route"
);
```

`debug_assert`, not `assert`: the branch is unreachable from both callers, so a release-cost guard
for a hazard nobody has is the C10 shape — *"an engine edit for an instrument's benefit"* — and this
tree refuses that. Debug surfaces it for the third caller that would introduce the bug while
developing.

The `or_insert` stays (it is now provably a no-op on the assert's success path) and the doc's
"a later call" sentence becomes what the code means: **a later call is a caller bug**, and here is
the route it should have taken.

## ⛔ The proof limitation, named with the vocabulary this arc just minted

The floor runs `--release`, where `debug_assert` compiles out. **A `#[should_panic]` test therefore
cannot run on the floor** — it must be `#[cfg(debug_assertions)]`-gated, and the floor will neither
run it nor be able to fail on this guard.

That is `rune:excusare(no-falsifier)` exactly: **nothing achievable on the release floor makes this
check fail.** The test must carry that rune, with its reason naming what was tried (a
`#[should_panic]` on the release floor) and why it cannot work (`debug_assert` is compiled out).

Fittingly, the vocabulary minted one strike ago is what lets this be stated rather than hand-waved.

## Out of scope = REJECTED

- **`assert!` instead of `debug_assert!`** to get a release-floor proof. It buys a mutation proof at
  the cost of a per-first-keying lookup for an unreachable branch — paying release cost to make a
  test provable is the tail wagging the dog, and C10 already refuses the shape.
- Changing `or_insert` to `insert`, or altering what is stored.
- Touching `writer()` or `is_keyed`.

## STOP triggers

1. The debug-gated `#[should_panic]` test does not panic under a **debug** run → STOP; the guard is
   not wired.
2. Any release behaviour changes → STOP. `debug_assert` is release-free by construction; if a floor
   number moves, something else did.
3. A third caller turns up in the grep → STOP and report; the severity assessment is then wrong and
   this becomes a live defect rather than hygiene.
