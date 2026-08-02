# BRIEF — `PMap::extend` (the batch door) + `Token.bindings` becomes a `PMap`

Design: `DESIGN-STONE-token-bindings-promoting.md` — read its **STEP 0 RESULT** and **The change**
sections first; the measurement that decided this stone is there and does not need re-deriving.

Two parts, in order. Part A is a new method with a law. Part B is the type flip that consumes it.

## Part A — `PMap::extend`

Add one inherent method to `src/value/pmap.rs`, beside `assoc` (`:103`):

```rust
/// Apply many entries in ONE clone of the backing storage. `assoc` in a loop copies the whole
/// Vec per key; this copies it once. Exists because the production caller (`extend_token`)
/// folds an element's entire binding array into a token in a single act — the shape the trie
/// arm already had (clone once, then `insert_mut` per key) and the array arm did not.
pub fn extend<I: IntoIterator<Item = (Value, Value)>>(&self, pairs: I) -> Self
```

**THE LAW — this is the wall, and it is what the RED gate proves:**

> `m.extend(pairs)` is observationally identical to
> `pairs.into_iter().fold(m.clone(), |acc, (k, v)| acc.assoc(k, v))`
> — same entries, same later-key-wins, **and the same arm** (`is_trie()` agrees).

Arm behaviour:
- **`Array`** — clone the Vec once into a local; per pair, overwrite in place if the key is present
  else push. Then pick the arm from the **final** length, so a batch that crosses
  `PROMOTION_THRESHOLD` promotes exactly as successive `assoc` would (mirror `from_pairs` `:53-70`,
  which already does the final-size choice).
- **`Trie`** — clone once, `insert_mut` per pair.
- **Zero pairs must not clone the backing Vec.** Materialise the working copy lazily, on the first
  item.

`extend` stays an inherent method. It does **not** join the `Bindings` trait — that trait's comment
(`src/rete/matcher.rs:84`) states it must never grow an `insert`, and this stone leaves it as it is.

### Part A's test — in `pmap.rs`'s existing test module, beside the six laws

The law above, exercised over sequences that (i) stay under the threshold, (ii) land exactly on 8,
(iii) cross it — plus a batch applied to a map that is **already** a `Trie`, plus duplicate keys
within one batch, plus the empty batch. Assert entries **and** `is_trie()` in every case. A law that
checks entries but not the arm is half-tested and would let a silent representation drift through.

## Part B — `Token.bindings: PMap`

`src/rete/kernel.rs:49` — flip the field. Then follow the compiler. The rooms, and why each:

| site | why you are here |
|---|---|
| `:49` `Token.bindings` | the flip itself; update the field's doc comment (it currently says "Stays rpds") |
| `:57` `impl Clone for Token` | it exists only to hook a census counter that is now gone — see **Part C** |
| `:183` `value_token_to_native` / `:226` | today decodes `PersistentMap` → trie. Now it is already a `PMap`: take it directly |
| `:242` `native_token_to_value` / `:254` | today wraps via `PMap::from_trie(tok.bindings)`. Now the value IS the field |
| `:686` `token_matches_bindings` | returns a raw trie today; it should return the `PMap` |
| `:836`, `:2373` `seed_token_bindings(bindings)` | the seed call sites |
| `:877` `token_element_compatible` | takes `&rpds::HashTrieMapSync`; retype to `&PMap` |
| `:893-916` `extend_token` | **the point of the stone** — see the sketch below |
| `:929` `seed_token_bindings` | becomes `PMap::from_pairs(el_bindings.iter().cloned())` |
| `:1391` `sample_bindings` | retype |
| `:2713` `tok.bindings.insert(k, v)` | rpds' non-mut insert → `assoc` |
| `:988`, `:1140`, `:2440`, `:2498-2605`, `:2697-2901` | reads — `key_of`, `eval_test_core`, `build_insert_fact`, `exec_compiled_rhs`. `PMap` already implements `Bindings`, so these should need nothing beyond the type. If one does, that is worth reporting |

### The sketch — `extend_token` (`:893-916`)

```rust
let new_bindings = tok.bindings.extend(
    el_bindings.iter()
        .filter(|(k, v)| tok.bindings.get(k) != Some(v))
        .map(|(k, v)| (k.clone(), v.clone())),
);
```

One clone, K applications. The filter preserves the existing same-value skip, and also avoids
cloning a key/value that is already present.

**★ One semantic difference to VERIFY rather than assume.** The old loop tested each key against the
*accumulating* map; this filter tests against the *original*. My grounding says they agree — a
duplicate key later in `el_bindings` either repeats the same value (skip, versus overwrite with an
identical value) or carries a different one (both land on the last value). Convince yourself against
the code, and if you find a case where they diverge, that is STOP-2.

## Part C — the census instrument stays gone

The Step-0 counters were reverted before this brief; the tree is at HEAD with zero
`census_count("token:…")` sites. **Do not re-introduce them**, and do not add new ones for this
stone. Two consequences to handle:

- `impl Clone for Token` (`:57`) exists *only* to hook `token:clones`, and its own doc comment says
  so. With the counter gone it is a hand-written copy of what `#[derive(Clone)]` produces — collapse
  it to the derive and delete the comment. (`PMap` is `Clone`, so the derive works.)
- If the compiler names anything else left orphaned by the revert, say so; do not paper over it.

## Blast radius

`src/value/pmap.rs`, `src/rete/kernel.rs`, and whatever those two force in `src/rete/matcher.rs` /
`src/rete/compiled_rhs.rs`. **No `.wat` file. No new wat verb. No surface change.** This is invisible
to every wat program — a wat program's `{:a 1}` already went through `PMap` in `f794b637`.

## STOP triggers — rejection criteria. Ship nothing; report the gap.

**STOP-1.** If `extend` cannot satisfy the law for some sequence — most likely at the promotion
boundary — STOP and report the exact sequence. Do not relax the law, and do not special-case the
boundary to make a test pass.

**STOP-2.** If the `extend_token` filter is NOT equivalent to the old accumulating-map check for some
`el_bindings` shape, STOP and report the shape. Do not "fix" it by restoring the loop; the divergence
is the finding.

**STOP-3.** If any read site needs `to_trie()` to keep compiling, STOP and name it. A conversion
surviving in the token path means the flip is incomplete, and the honest answer is which reader still
demands rpds — not a conversion quietly left in the hot path.

**STOP-4.** If `binding_cardinality_distribution` (`src/rete/kernel.rs:5666`) changes at all, STOP.
That gate is byte-identical-or-nothing: it is the one check a *fast wrong answer* would otherwise
pass.

## The gate — run these yourself, in the FOREGROUND, and report what each said

1. `cargo build --release --all-targets` — exit 0, **zero warnings**. This is the arbiter; a red
   squiggle from the editor on a file mid-edit is not.
2. `cargo test --release --lib -- value::pmap --nocapture` — the six existing laws plus Part A's.
3. `cargo test --release --lib -- rete::kernel::tests::binding_cardinality_distribution --nocapture`
   — STOP-4's gate.

Report the numbers you saw. The full `cargo nextest run --release` floor and the nine-axis grid are
the orchestrator's to run; leave those alone so the build lock stays free for them.

## You are a rider, not the orchestrator

**Ending your turn ENDS you** — it does not suspend you, and nothing will wake you. There is no
notification coming. Run every command in the FOREGROUND and block on it; your turn ends when the
numbers are in your hands, not when a command is launched. Do not commit, do not push, do not stash.
Report what you ran and what it said.
