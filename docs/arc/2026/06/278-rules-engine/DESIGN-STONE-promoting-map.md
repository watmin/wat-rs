# DESIGN-STONE — the promoting map: array to 8, then trie

> **Origin — the builder's, and repeatedly.** *"array up to 8, trie after is unquestionably a
> superior solution for correctness."* Named at least three times across sessions, carried in the
> seam's OWED list, and recorded in `UNADOPTED.md:32` as **OPEN — "not built, third door named
> 2026-08-01, shut."** It sat unbuilt in the very file written to catch things that get agreed and
> not done. Ruled to build 2026-08-01.

## Why it is a CORRECTNESS stone, not a perf one — and why R60's cut does not apply

`Value::wat__core__PersistentMap` (`src/value/value.rs:105`) is an **unconditional**
`rpds::HashTrieMapSync<Value, Value>`. Every `{:a 1}` any wat program writes allocates a HAMT.

Choosing ONE representation globally is a **corpus bet** — it asserts something about the size of
maps users will write, and we have no standing to make that claim
(`[[feedback_a_benchmarks_shape_manufactures_its_result]]`; R60's discarded census). Promoting **per
instance** is *not a bet at all*: each map picks its own representation from its own size, at
runtime, with no claim about anyone's corpus. That is exactly why R60's "don't optimize for a
measured distribution" cut does **not** reach this stone — the builder's point, and it is the whole
justification. The stone is not "small maps are common"; it is "the engine should not assume."

## ★ THE CRAWL CHANGED THE STONE — two findings, both grounded this session

**1. The `Hash` constraint the seam called hard is ALREADY SATISFIED.** The seam warned that
cross-representation `Hash`/`Eq` must be structural "because at `Value` level a map CAN be a hash
key" — and treated that as the blocker. Reading `impl Hash for Value`'s map arm
(`src/value/value.rs:787`), it collects `(key_hash, val_hash)` pairs, **sorts them**, and hashes the
sorted vector — deliberately order-independent (arc-278-0a). *That algorithm does not touch the
container.* An array-backed map with the same entries produces the **identical** hash by
construction, provided both arms run the same routine. The hard half of the constraint is a
non-event; it was designed correctly before anyone needed it.

**2. `PartialEq` is the half that actually needs work.** `src/value/value.rs:613` is
`(PersistentMap(a), PersistentMap(b)) => a == b` — delegating to rpds' own `PartialEq`, which **is**
representation-dependent. This is the real constraint and it is narrow: one arm, replaced by an
entry-set comparison (same length, and every `(k,v)` in `a` present in `b`) that is correct across
any pairing of representations.

**3. TWO POPULATIONS, and only one is the target.** 62 `Value::wat__core__PersistentMap(`
construction sites are the user-facing map. Separately, ~55 raw `HashTrieMapSync` uses live in
`rete/kernel.rs` (41), `rete/matcher.rs` (10) and `rete/compiled_rhs.rs` (4) — that is
**`Token.bindings`**, which `DESIGN-STONE-element-bindings-array.md` *deliberately kept as a trie*
after measuring build/lookup/clone/drop at every width ("Tokens are the thing that extends, and the
trie wins"). **`Token.bindings` is OUT OF SCOPE.** Conflating the two would re-open a ruling that was
already measured and settled, in the opposite direction.

## The change

```rust
pub enum PMap {
    /// ≤ THRESHOLD entries, insertion-ordered, linear scan. No HAMT allocation.
    Array(Arc<Vec<(Value, Value)>>),
    /// Above THRESHOLD — today's representation, unchanged.
    Trie(rpds::HashTrieMapSync<Value, Value>),
}
const THRESHOLD: usize = 8;   // Clojure's PersistentArrayMap boundary
```

`Value::wat__core__PersistentMap(PMap)`. Every op (`get` / `assoc` / `dissoc` / `length` /
`empty?` / `contains-key?` / `keys` / `values`) dispatches on the arm.

**★ THE ONE CONTRACT DECISION — promotion is ONE-WAY.** `assoc` past the threshold promotes
Array→Trie; `dissoc` below it does **not** demote. Two reasons, and the second is the load-bearing
one: demotion invites thrash at the boundary (assoc/dissoc/assoc rebuilds the representation each
time); and one-way promotion means **a map's representation is a function of its high-water mark**,
which is far easier to reason about than one that depends on its whole history. Clojure does the
same. The corollary is that `Eq`/`Hash` MUST be cross-representation regardless — a 3-entry Array
and a 3-entry Trie (demoted-from-9 in a hypothetical, or built by different paths today) are the
same value and must behave identically. That is the wall, not an optimization.

## The gate

1. **★ THE CROSS-REPRESENTATION LAW, both directions.** For the same entry set built as Array and
   as Trie: `a == b`, `b == a`, and `hash(a) == hash(b)`. Property-tested over generated entry sets
   spanning the threshold (0,1,7,8,9,64 entries), with keys of mixed `Value` kinds — **including a
   map as a key**, which is the case the seam flagged and the only one where a wrong answer is
   silent.
2. **Per-op differential.** Every op, both arms, same inputs → same outputs, same errors. Not a
   spot-check: the op list comes from the dispatch table's `PersistentMap/` arms, so a new op cannot
   be added without a row.
3. **The promotion boundary is exercised, not assumed** — a map built by 9 successive `assoc`s must
   equal one built by a single 9-entry construction, and a counter proves the promotion actually
   fired (a test where nothing promotes is vacuous).
4. **EDN round-trip byte-identical** across both arms — the wire form must not leak representation.
5. **`:accuracy :match` on all nine grid axes; `:derived` byte-identical.**
6. **The release floor and clippy 0**, by my own re-run.
7. **No regression at large N.** Interleaved, medians. The claim is "no worse above the threshold",
   not a speedup — and per today, no timing row gets stated beyond what is measured.

## Out of scope = REJECTED

- **`Token.bindings`** and every other rete-internal `HashTrieMapSync` — measured and ruled the
  other way by `DESIGN-STONE-element-bindings-array.md`.
- **`HashMap` / `PersistentVector`** — the same question may apply; it is not this stone, and
  answering it here would make the attribution unreadable.
- **Demotion on `dissoc`** — pinned above as the contract's corollary.
- **Tuning THRESHOLD from our corpus.** 8 is Clojure's, chosen because it is *someone else's*
  well-worn boundary rather than a number fitted to code we wrote. Changing it wants a measurement
  over generated maps, not over our test suite.

## Honest state

The two crawl findings above are grounded (`value.rs:613`, `:787`, and the kernel/matcher split).
**Nothing is built.** The cost is not yet measured — and per this stone's own framing it does not
need to be to justify the change, because the argument is that an unconditional representation is a
bet we have no standing to make. But the gate's row 7 still has to hold, and that is a measurement.
