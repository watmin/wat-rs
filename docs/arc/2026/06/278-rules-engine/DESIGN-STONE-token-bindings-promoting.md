# DESIGN-STONE — `Token.bindings` becomes a `PMap`

> **Origin (builder, 2026-08-01, correcting me):** *"this is actively choosing to not do precisely
> what we just delivered? … the entire array up to 8, trie for rest came from this exact situation?
> … and chose not to use it?"*
>
> He is right, and the disk says so in our own words.

## The ruling I misapplied, and what it actually decided

`BRIEF-promoting-map-migration.md`'s **STOP-1** forbade touching `Token.bindings`, citing
`DESIGN-STONE-element-bindings-array.md` as "measured and settled." That stone says, at `:94`:

> *"The 8-crossover stays interesting for `Token.bindings` — which extends, and where the trie
> wins —"*

**It named `Token.bindings` as the place the 8-crossover was interesting.** That is where the whole
array-to-8 line came from. Its out-of-scope note (`:116`) reads *"The trie wins extend 3.4× at n=8,
and extending is all a Token does"* — and `:36` states the frame plainly: *"the discriminator was
never rule width or a size threshold."*

That ruling was made under a **binary** choice: array **or** trie, chosen once, globally, per field.
Under that choice "a Token extends, so give it the trie" is correct. **A promoting map is the third
option the dichotomy never had.** A Token with ≤8 bindings holds an array and *promotes when it
grows past 8* — it extends *and* stays cheap, because extension no longer forces the representation
to be picked up front.

So the prior ruling did not decide this question; it decided a different one. I turned it into a
hard STOP a rider could not overturn — an inherited ruling killing an option it never considered.

## Why this is EASIER than the map that just landed

The stone that shipped (`f794b637`) carried one genuinely hard requirement: a `Value`-level map can
be a **hash key**, so cross-representation `Hash`/`Eq` had to be exact or a key would silently miss.
The seam said so, and flagged that it *"cannot [be a key] for `Token.bindings`, which is why
yesterday's note dodged it."*

`Token.bindings` is rete-internal and is never used as a hash key. **The silent failure mode does
not exist here.** `PMap`'s laws are already proven and its `Bindings` impl already exists
(`matcher.rs`). This is the smaller stone, landing second.

## What a Token's bindings actually do — the four-of-five case

From the element-bindings measurements: **the array wins build, lookup, clone and drop; the trie
wins exactly one operation — extend — by 3.4× at n=8.** For a Token below the threshold, the array
wins four of five and loses one. That is the case, and it is per-instance, so it makes no claim
about anyone's corpus.

Where the leverage is, measured: **node-share's tokens carry ONE binding** (Step 0,
`node_share_where_cost_decomposition`, captured from a real fire). Every one of those is a HAMT
holding a single entry, ~10,000 of them per round.

## ★ STEP 0 — RUN, AND IT MOVED THE SHAPE (2026-08-01)

### The census — STOP-0 cannot fire

Four axes fired with per-op counters (widths, extends, clones, lookups):

```
  axis                extends  crossed-8   clones   lookups   (clones+lookups)/extends
  node-share              200          0    10400     10600        105.00x
  accum                   800          0     1000    283000        355.00x
  deep-cascade           5000          0    10000     20000          6.00x
  fanout                40000          0     4000    124000          3.20x
  widest binding observed anywhere: 3.   crossed-8: ZERO everywhere.
```

**The boundary risk this stone was gated on does not exist in the live engine.** Zero crossings on
every axis; the widest Token binding map anywhere is 3. STOP-0 is not merely un-fired — with
`insert_mut`-only growth and no `dissoc` in the whole token path, a Token crosses at most once in its
life, so oscillation was never possible either. (I had written a thrash STOP against a mechanism that
cannot occur; the builder cut it. Kept visible.)

**And the census re-weights the decision:** lookups outnumber extends **105×** on node-share and
**355×** on accum. Whichever representation wins *lookup* wins the engine.

### The A/B — first run aimed at nothing, re-specified at real widths

The first A/B built to 7/8/9/12 — **widths nothing reaches**, my spec error, aimed at a crossing that
is structurally impossible. It returned lookup/clone/drop at parity and build losing, and that table
is **superseded**. Re-specified at widths 1–3 with the *real* construction pattern (`from_pairs`
one-shot seed, then extends against a parent that still holds its own bindings — refcount 2, exactly
`extend_token`), 15 reps × 2000-op batches, medians, interleaved, `!is_trie()` asserted at every
generation:

```
  width  op            A trie ns    B PMap ns   ratio A/B
      1  seed             122.64        84.49       1.45x   PMap
      1  extend→w+1       138.65       123.51       1.12x   PMap
      1  extend→w+2       145.76       176.45       0.83x   trie
      1  lookup            22.29         6.42       3.47x   PMap
      2  seed             174.35       109.06       1.60x   PMap
      2  extend→w+1       131.41       100.65       1.31x   PMap
      2  extend→w+2       141.35       201.04       0.70x   trie
      2  lookup            41.92        12.94       3.24x   PMap
      3  seed             405.60       200.44       2.02x   PMap
      3  extend→w+1       263.15       189.53       1.39x   PMap
      3  extend→w+2       274.55       349.54       0.79x   trie
      3  lookup           284.91        80.67       3.53x   PMap
```

Stable across three runs. **Array wins seed, extend→w+1 and lookup; loses extend→w+2; ties
clone/drop.** The tie is *explained, not mysterious*: `PMap::Array` is `Arc<Vec<…>>`, so a clone or
drop on either arm is one refcount operation. (The seam called this "unexplained" — an overstatement;
the explanation was already in the record.)

## ★ THE ONE LOSS IS AN API GAP — and closing it is half this stone

`extend_token` (`kernel.rs:922`), read, not inferred:

```rust
let mut new_bindings = tok.bindings.clone();     // ONE clone
for (k, v) in el_bindings.iter() {
    if new_bindings.get(k) != Some(v) {
        new_bindings.insert_mut(k.clone(), v.clone());   // K in-place mutations
    }
}
```

Clone once, then mutate the private copy K times. **`PMap` has no door for that shape** — `assoc`
copies the whole Vec per key, so K bindings cost K copies. That is exactly and only what
`extend→w+2` measures: not a property of arrays, a missing constructor
(`[[reference_a_costly_shape_change_means_a_missing_constructor]]`).

**This is NOT `Arc::make_mut`**, which the seam rejected and which stays rejected: it skips the copy
only at refcount 1, so it would speed the benchmark loop and do nothing for `extend_token`, where the
parent token still holds its bindings. The batch door is honest at any refcount.

Worth naming: `el_bindings` is already `Arc<[(Value, Value)]>` — R60 made the Element side an array.
`Token.bindings` is the last trie of the pair.

## The change — two parts, and the second is not optional

### Part A — `PMap::extend`, the batch door

```rust
/// Apply many entries in ONE clone of the backing storage.
pub fn extend<I: IntoIterator<Item = (Value, Value)>>(&self, pairs: I) -> Self
```

**The law, and it is the wall:** `extend` is observationally identical to folding `assoc` —
`m.extend(pairs) == pairs.into_iter().fold(m, |acc,(k,v)| acc.assoc(k,v))`, *including which arm the
result lands in*. Later duplicate keys win. Extending by zero pairs must not clone the backing Vec.

Arm behaviour: **Array** — clone the Vec once, then per pair overwrite-in-place or push; choose the
arm from the FINAL length, so a batch that crosses the threshold promotes exactly as successive
`assoc` would. **Trie** — clone once, `insert_mut` per pair (what the trie already did).

`extend` is an inherent `PMap` method. It does **not** go on the `Bindings` trait — that trait's own
comment says it must never grow an `insert`, and this stone does not touch it.

### Part B — `Token.bindings: PMap`

`Token.bindings: rpds::HashTrieMapSync<Value, Value>` → `PMap` (`kernel.rs:49`). `PMap` already
implements `Bindings`, so `resolve_operand` / `eval_test_core` / `key_of` read it unchanged.
`seed_token_bindings` becomes `PMap::from_pairs`. `extend_token` becomes ONE `extend` call carrying
the same-value skip:

```rust
let new_bindings = tok.bindings.extend(
    el_bindings.iter()
        .filter(|(k, v)| tok.bindings.get(k) != Some(v))
        .map(|(k, v)| (k.clone(), v.clone())),
);
```

The filter tests the *original* map where the old loop tested the *accumulating* one. Grounded as
equivalent: a duplicate key later in `el_bindings` either carries the same value (skip vs overwrite
with the identical value — same result) or a different one (both end at the last value). **Verify it,
do not take my word.**

The `Value`↔`Token` seam (`value_token_to_native` / `native_token_to_value`) **stops converting** —
both sides become `PMap`, so `to_trie`/`from_trie` leave that path rather than multiplying.

### Part C — the Step-0 census instrument is REMOVED (builder-ruled)

Its measurement is harvested and recorded above; a standing instrument for a finished measurement is
a scaffold that becomes architecture. All `census_count("token:…")` sites, `token_wbucket`, and the
three Step-0 test fns go. Already done: the uncommitted WIP was reverted, so the tree is at HEAD with
zero `token:` census sites — **this stone builds on clean ground and must not re-introduce them.**

## The gate

1. **The `extend` ≡ folded-`assoc` law**, over sequences that stay under, land exactly on, and cross
   the threshold — asserting the resulting arm too (`is_trie()`), or the law is half-tested.
2. **The differential: native == the wat oracle, bit-for-bit.** Token bindings feed joins and
   productions; a wrong binding is a wrong derivation.
3. **`:accuracy :match` on all nine axes; `:derived` byte-identical.**
4. **`binding_cardinality_distribution` (`kernel.rs:5666`) byte-identical** — R60's gate, the one
   check a *fast wrong answer* passes everything else on.
5. **`filter` and `hash-join` do not regress** in `node_share_fire_phase_census` — Token bindings are
   read in both, and this is where the lookup win should show if it shows anywhere.
6. **Release floor and clippy 0**, by my own re-run. Baseline at HEAD: **4260/4260**.

   > **Corrected 2026-08-01, after the stone was weighed.** This row first said 4262 — a number
   > carried across a compaction and written down as verified without re-running the floor after the
   > census revert. It is wrong, and the arithmetic that caught it: my own verified floor *with* the
   > census WIP was 4263/4263 at 262 skipped; that WIP added 3 test fns; so HEAD is 4260. The stone's
   > diff adds 1 test and removes 0 (`git diff | grep -c '^+\s*#\[test\]'` → 1, `-` → 0, ignores
   > ±0), predicting 4261 — which is exactly what the floor returned. Nothing was lost; the baseline
   > was. `[[feedback_a_claims_support_does_not_travel_with_the_claim]]`.
7. **`PMap`'s six laws still green** — this stone must not loosen the wall the last one built.

## Out of scope = REJECTED

- **Re-tuning `PROMOTION_THRESHOLD`.** 8 is Clojure's. If the census says the boundary is crossed
  constantly, that is STOP-0 and a conversation, not a knob to turn quietly.
- **`Element.bindings`.** Already an `Arc<[(Value, Value)]>` and correct — it never extends, so it
  has no promotion question. Untouched.
- **Demotion.** Same one-way contract as `PMap` itself.

- **`Arc::make_mut` in `assoc` or `extend`.** Rejected above and staying rejected.
- **Re-measuring the A/B after the change.** The microbenchmark's job is done; the engine-level gate
  (rows 3–5) is what decides whether this earned its place.

## Honest state

**Step 0 is complete and the numbers are above.** What is built: nothing of the stone itself — `PMap`
and its laws exist (`1236f6ad`), the `Bindings` impl exists, the census is reverted, the tree is clean
at HEAD.

**What the measurement does and does not say.** It is a microbenchmark of five operations at three
widths. Weighted by the census op-mix it points clearly at the array, and the one loss has a named
mechanical cause with a named fix. It is **not** an engine result: nothing here has moved a grid
number yet, and the honest verdict is rows 3–5 of the gate. If `:accuracy` moves, the stone is wrong
and reverts; if the grid is flat, the stone is a wash and that gets said plainly rather than dressed
up as the microbenchmark's win.
