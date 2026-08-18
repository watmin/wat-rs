# BRIEF — `d_alpha` is indices into `wm.alpha`

## The work

Step 1 clones every `Element` into `d_alpha` after pushing it
to `wm.alpha`. Readers only need “new this round.” Store
`Vec<usize>`. Move into `wm.alpha`. Probe by index.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — #1 and #2 landed; this is
   the weigh-named next copy, not persist.
2. `DESIGN-STONE-delta-alpha-indices.md`.
3. `kernel.rs` ~3790 (`d_alpha` decl), ~3854 (push),
   ~3878 (root-join), ~4086 (hash-join `dr`).

## Sketch

```rust
let mut d_alpha: HashMap<i64, Vec<usize>> = HashMap::new();
// push
let v = wm.alpha.entry(*aid).or_default();
v.push(el);
d_alpha.entry(*aid).or_default().push(v.len() - 1);
// read
for &i in d_alpha.get(aid).unwrap_or(&[]) {
    let el = &wm.alpha[aid][i];
}
```

Split-borrow `wm.alpha` vs `wm.beta`. Do not clone to silence
the checker.

## STOP

1. **STOP-1** — a reader still needs an owned Element from
   `d_alpha` (other than `right_idx`’s existing clone). Report.
2. **STOP-2** — rete differential red.
3. **STOP-3** — `right_idx` rewritten in this diff.

## Done

- Push has no `el.clone()`.
- `[200 200]` FIRE < 88 ms, or a written “did not move”
  with the before/after table.
- rete + clippy green.

Leave dirty.
