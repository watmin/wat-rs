# BRIEF — key the Accumulate / Negation / Exists gathers

## The work

Three node kinds in the native fire kernel gather their elements by **scanning the whole cumulative
alpha memory once per token**. The join nodes solved this in P6 with a key index. Give the gathers
the same index: build a `HashMap<Vec<Value>, Vec<usize>>` over the node's elements **once per node
per round**, keyed by the shared-variable tuple, then let each token probe it in O(1) and work only
its own bucket. This is `O(tokens × elements)` → `O(elements + tokens)`. A RED gate is already in
the tree and measures exactly this; your job is to turn it green without moving any other number.

## Read in order (the rooms, and why you are being sent to each)

1. **`src/rete/kernel.rs:779-831` — `keyed_join`.** This is the EXEMPLAR. It already does every
   piece you need, in this file, in this style: derive `join_keys` as the sorted intersection of
   token/element binding keys from a sample of each side (`:785-801`); index the elements by the
   key tuple into `HashMap<Vec<Value>, Vec<usize>>` (`:804-814`); probe with each token (`:816-825`).
   Copy this shape. Your gather differs only in what it does with the bucket.

2. **`src/rete/kernel.rs:1203` — `key_of`.** The existing helper that extracts a key tuple from a
   bindings map given a `join_keys` list. Reuse it for the probe side rather than re-deriving it.

3. **`src/rete/kernel.rs:1944-2001` — the Accumulate-pass.** Site one. `:1979` snapshots the full
   `wm.alpha[from_alpha_id]`; the `.filter(...)` closure at `:1988-1993` re-walks that whole vector
   for every token in `new_tokens`. The `census_gather_visit()` call at `:1989` is the instrument —
   keep it inside whatever closure ends up examining an element, so the gate keeps measuring the
   real work.

4. **`src/rete/kernel.rs:2004-2072` — the Filter-pass.** Site two. The `TestNode` branch is not
   yours — leave it alone. The `else` branch (`NegationNode` / `ExistsNode`) snapshots
   `wm.alpha[alpha_id]` at `:2054` and runs a per-token `.any(...)` at `:2060-2064`. Same defect,
   same fix; the verdict still inverts on `is_exists`. The `census_gather_visit()` at `:2061` is the
   instrument — same rule as above.

5. **`src/rete/kernel.rs:730-743` — `token_element_compatible`.** Read it to see WHY the key is
   exactly right rather than approximately right: it requires agreement on the INTERSECTION of the
   two binding key-sets, which is precisely what `join_keys` + tuple equality checks. The predicate
   is correct; only how often it runs is wrong.

6. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-keyed-gather.md`** — the contract, the
   measurement that grounds it, and the affirmative scope cuts.

## Implementation sketch (the shape; fill it in)

One shared helper, used by both sites:

```rust
/// Index `elements` by the join-key tuple shared with `sample_bindings`.
/// Returns (join_keys, index) — buckets hold element INDICES in iteration order.
fn gather_index(
    sample_bindings: &rpds::HashTrieMapSync<Value, Value>,
    elements: &[Value],
) -> (Vec<Value>, HashMap<Vec<Value>, Vec<usize>>) {
    // join_keys: sorted intersection of sample_bindings' keys and elements[0]'s keys
    //            (mirror keyed_join:785-801 exactly, including the stable string sort)
    // index:     for each (i, el) — key_of(el_bindings, &join_keys) -> push i
}
```

Accumulate site, per node per round:

```rust
let (join_keys, index) = gather_index(&new_tokens[0].bindings, &from_elements);
for tok in new_tokens {
    let bucket: &[usize] = index.get(&key_of(&tok.bindings, &join_keys))
        .map_or(&[][..], |v| v.as_slice());
    let gathered: Vec<&Value> = bucket.iter()
        .map(|&i| &from_elements[i])
        .filter(|el| { census_gather_visit(); /* still verify compatibility */ })
        .collect();
    if let Some(aggregate) = accumulate_value(&acc_form, &gathered, sym)? { /* unchanged */ }
}
```

Negation/Exists site: same index, and `any_compat` becomes a probe of the bucket instead of a walk
of the whole memory. The `pass` computation and both pushes stay exactly as they are.

## The three contract clauses (each one is a trap; the DESIGN names them)

1. **Order is preserved.** Buckets hold indices pushed in element-iteration order and are read in
   that order. The existing gather is documented as "in alpha-memory insertion order (matches the
   wat foldl over from-els)" and the kernel is differential-tested bit-for-bit against the wat
   oracle. A reordered gather is a silent oracle divergence.

2. **A missing bucket yields an EMPTY gather — never a skipped token.** `accumulate_value` on an
   empty gather is meaningful: `count`/`sum` emit their identity, `min`/`max` return `None` and the
   token is dropped by the existing `if let Some(...)`. Writing `if let Some(bucket) = index.get(..)`
   and `continue`-ing on `None` silently drops every `count`/`sum` token whose group has no
   elements. Use the `map_or(&[][..], ...)` form in the sketch.

3. **Empty `join_keys` must still work.** When token and element share no variables the tuple is
   `[]` for everything: one bucket, every element, every token probes it. That is the cartesian case
   and it is correct — same answer, same cost as today. No special-casing needed; just do not panic.

## Blast radius

`src/rete/kernel.rs` only. No other file. No `.wat` changes, no corpus migration, no codemod — the
wat oracle does not move; it is the reference this is checked against. Do not change
`token_element_compatible`, `accumulate_value`, `extend_token`, or the `TestNode` branch.

## STOP triggers (each is a rejection: ship nothing, report the gap)

1. **STOP-1** — if the key derived for an element and the key derived for a token cannot be made to
   agree for a workload the differentials already cover (i.e. `key_of` panics on a missing key for
   either side), STOP and report which node kind and which side. Do not add a fallback scan path.

2. **STOP-2** — if any existing rete differential (native vs the wat oracle) goes red, STOP and
   report the test name and the diff. A green gate with a red differential is not the work; the
   oracle is the anchor.

3. **STOP-3** — if turning the gate green appears to require changing what the gather RETURNS
   (different elements, a different order, or a different empty-case), STOP and report. The contract
   above is fixed; the only thing changing is how the elements are found.

## Definition of done

- `cargo nextest run --release -E 'test(keyed_gather_visits_do_not_scale_with_group_count)'` passes.
- `cargo nextest run --release -E 'binary_id(wat::rete)'` passes (the differentials).
- `cargo clippy --all-targets --release` emits nothing.
- Report the gate's printed visit numbers (both runs and the ratio) in your write-up.

Leave the tree dirty and uncommitted — the orchestrator weighs by its own re-run and commits.

## A prior result to copy for shape

`keyed_join` (`:779-831`) is the pattern, and P6's `left_idx`/`right_idx` at `:1542-1544` is the
same idea persisted per node. Read both before writing; this stone is the third application of a
technique this file already contains twice.
