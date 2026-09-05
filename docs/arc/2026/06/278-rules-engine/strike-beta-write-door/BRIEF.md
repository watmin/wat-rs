# BRIEF — D3: make the beta-write bypass unwritable

Cure **and** prove it in one strike. **Floor GREEN when you are done.**

## Read in order

1. **`DESIGN.md` beside this file** — the contract is the TOP rung (compile error), not four call-site
   replacements.
2. **`src/rete/kernel/fire/pass/mod.rs:20-32`** — the claim. **`:38-57`** `record_token`, **`:59-75`**
   `record_tokens` — the doors, and the exact body the bypasses duplicate.
3. **`src/rete/kernel/fire/pass/mod.rs:151-158`** — bypass 1, in the file making the claim.
4. **`src/rete/kernel/fire/mod.rs:2089-2093`, `:2099-2103`, `:2123-2127`** — bypasses 2–4, each
   byte-for-byte `record_token`'s body.
5. **`src/rete/kernel/session.rs:440`** (`pub(crate) beta: BetaMemory`) and **`:242-292`**
   (`JoinRightIndex` + `RightIndexWriter`) — **the shape to copy**, landed twice in this arc.
6. **`src/rete/kernel/fire/delta.rs`** — the two `.beta.clear()` round resets that need their own door.

## Implementation sketch

```rust
// session.rs — beta becomes private; the doors are the only mutation
pub(crate) struct WorkingMemory {
    beta: BetaMemory,          // PRIVATE
    …
}
impl WorkingMemory {
    pub(crate) fn beta_get(&self, node_id: &i64) -> Option<&Vec<Token>> { … }
    pub(crate) fn beta_clear_round(&mut self) { … }      // delta.rs's reset, named
    // no accessor hands out `&mut BetaMemory`
}
```

Then `record_token` / `record_tokens` take whatever handle the doors need, and
`fire/mod.rs`'s three sites and `left_activate_join` call them.

**`left_activate_join` is `record_tokens` with `extend` instead of `reserve`+`extend_from_slice`** —
the doors' own doc says reserve is *"the cost that reserve exists to avoid"*, so calling the door is
also the faster shape. `joined` is owned; pass it as a slice.

**Borrow shapes to expect:** in `dispatch_where_tests`, `sink.wm.beta`, `sink.d_beta` and
`sink.beta_readers` are disjoint fields of one `&mut WhereSink` — `struere` predicted these
borrow-check as one-liners but **did not compile it**, so treat that as a prediction, not a fact.

## The proof

**Re-introduce a bypass and quote the compiler.** Aim for the shape the last two cures produced:

```
error[E0616]: field `beta` of struct `session::WorkingMemory` is private
```

A test failure where a compile error was required is not the proof.

## Blast radius

`src/rete/kernel/` only. No wat corpus change, no codemod, no new dependency.

## STOP triggers

1. **If making `beta` private forces changes outside `src/rete/kernel/`, STOP and report** the sites.
   That is a bigger decision than this strike.
2. **If a test needs `&mut` beta** (`pass_semantics.rs` has `.beta.clear()`), give it a NAMED door
   rather than re-opening the field — and say in the SCORE which doors exist and why each is safe.
3. **If any `*_cost` or census gate reddens, STOP.** These sites are exactly where the census counts;
   a moved number is a finding, not a nuisance.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-left-idx-latch/` — private fields, one door, `writer()` returning `None` until the
invariant is established, and the bypass proven unrepresentable by `error[E0616]` in both directions.
