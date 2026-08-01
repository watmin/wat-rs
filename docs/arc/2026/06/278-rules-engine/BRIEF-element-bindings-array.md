# BRIEF — `Element.bindings` becomes an array

## The work

An `Element`'s bindings are an `rpds::HashTrieMapSync` — a hash-trie — but an Element is built once
and then only read, cloned and dropped. It is **never extended**, which is the single operation a
trie is good at. Measured, the array beats the trie on every operation an Element actually performs:
build 3–7×, lookup 2–7×, clone 2×, drop 2–14×. There are ~80,200 Elements per fire.

Swap `Element.bindings` to `Arc<[(Value, Value)]>`. Leave `Token.bindings` a trie — Tokens extend,
and the trie wins that 3.4×.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-element-bindings-array.md`** — the measurement
   table, the contract decision, and the RESOLVED section (no spill; a plain array at any width).

2. **`src/rete/kernel.rs` — `struct Element`** (right after `struct Token`). The field you change.
   `Token` is beside it and is **not** in scope; the two diverging is the point of the stone.

3. **`src/rete/kernel.rs` — `native_element_to_value` / `value_to_element` / `alpha_to_pm`.** The
   Value boundary. Elements still encode to a wat `PersistentMap` for `fire-once'`; that conversion
   now walks an array instead of a trie.

4. **`src/rete/matcher.rs:343`** — where an element's bindings are built (`alpha_match_inner`'s
   accumulator), and **`:357`**, the insert. Building an array here instead of folding a trie is most
   of the win.

5. **`src/rete/matcher.rs` — the 7 bindings-taking signatures** (`:186 :343 :429 :472 :615 :750 :967`).
   These are the sites the trait touches. Note `build_insert_fact` (`:472`) and the `token_bindings`
   local (`:750`) are **token**-side and must keep working against the trie.

## ★ THE ONE CONTRACT DECISION

**Introduce a minimal READ-ONLY `Bindings` trait; do NOT convert at the boundary.**

matcher reads both kinds — token-side at `:472`/`:750`, element-side at `:343`, and `eval_test_core`
(`:967`) can be either (a `:test` clause may sit after a join). Building an rpds map from the array
wherever matcher wants one would **reintroduce the exact allocation this stone removes, in the hot
path, while every test stayed green.** That is the failure mode to avoid.

```rust
pub(crate) trait Bindings {
    fn get(&self, k: &Value) -> Option<&Value>;
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)>;
}
```
Implement for both; make matcher's readers take `impl Bindings`. Monomorphised — no vtable.

**The trait must never grow `insert`.** The moment it does, the two representations are being forced
through one interface and the array will be made to pay for the trie's operation.

## Implementation sketch

```rust
pub(crate) struct Element {
    pub(crate) fact:     Value,
    pub(crate) bindings: Arc<[(Value, Value)]>,   // was rpds::HashTrieMapSync<Value, Value>
}
```
`alpha_match_inner` collects into a `Vec<(Value, Value)>` and seals it with `.into()`. Lookup is a
linear scan (`iter().find(|(k, _)| k == key)`). No spill, no threshold — see the DESIGN's RESOLVED
section for why.

## Blast radius

`src/rete/kernel.rs` and `src/rete/matcher.rs`. Two files, not one.

## STOP triggers (each is a rejection: ship nothing, report the gap)

1. **STOP-1** — if you find yourself building an `rpds` map from an array anywhere in the fire path,
   STOP. That is the failure this brief exists to prevent.
2. **STOP-2** — do not change `Token.bindings`. If the trait seems to require it, STOP and report.
3. **STOP-3** — if `probe_arc278_2b_insert_alpha` or `round_trip_fired_session` goes red, STOP and
   report the assertion. They are the only two tests that exercise the element Value boundary.
4. **STOP-4** — if any of the ~24 count differentials moves, STOP. The RESULT must not change.
5. **STOP-5** — if a duplicate key can reach the array (the trie deduped for free; a `Vec` will not),
   STOP and report where. Silent duplicate bindings would be a wrong answer, not a crash.

## Definition of done

- `cargo nextest run --release -E 'test(/2b_insert_alpha|round_trip_fired_session/)'` — green.
- `cargo nextest run --release -E 'binary_id(wat::rete)'` — all pass.
- `cargo nextest run --release` — 4215/4215.
- `cargo clippy --all-targets --release` — silent.
- `... -E 'test(accum_fire_phase_census)' --no-capture` — report `alpha`, `accum:fold` and
  `round:drop-memories` before/after.
- `... -E 'test(binding_cardinality_distribution)' --no-capture` — the ELEMENT histogram must be
  unchanged (same counts, same buckets). A representation that silently drops a binding shows up here.
- Report `git diff --stat`.

Leave the tree dirty and uncommitted. Do not commit, push, or stash.

## A prior result to copy for shape

`32142f8a` (native Element) is the immediate predecessor and touched the same struct. Its diff is the
shape: a type change, a conversion pair, and the compiler naming the rest.
