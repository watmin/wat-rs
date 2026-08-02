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

## ★ THE MEASUREMENT THIS STONE OWES — and it is a real risk, not a formality

**The 3.4× extend win is real above the threshold, and repeated boundary crossing is the case that
could make this a loss.** A Token that grows past 8 pays an array→trie rebuild; if tokens routinely
cross 8, promotion cost is paid per token per fire.

Step 0 of this stone, before any code:

1. **A token-binding-width census across all nine grid axes** — the distribution of
   `Token.bindings.size()` at every join. Not to fit a threshold to our corpus (the promotion
   decision is per-instance and needs no distribution), but to answer **how often the boundary is
   crossed**, which is the risk.
2. **An extend-heavy A/B at the boundary** — build a token to 7, 8, 9, 12 bindings via successive
   `assoc`, both representations, interleaved, medians. The array must not lose materially below 8,
   and the trie must still win above it.

**STOP-0:** if tokens routinely cross the threshold on the live axes, the shape is wrong as
specified and wants either a different threshold or the trie kept for Tokens specifically — report
it, do not tune the number to make the result look good.

## The change

`Token.bindings: rpds::HashTrieMapSync<Value, Value>` → `Token.bindings: PMap` (`kernel.rs:49`).
`PMap` already implements `Bindings`, so `resolve_operand` / `eval_test_core` / `key_of` read it
unchanged. `extend_token` uses `assoc`. The `Value`↔`Token` seam
(`value_token_to_native` / `native_token_to_value`) **stops converting** — both sides become `PMap`,
so `to_trie`/`from_trie` disappear from that path rather than multiplying.

## The gate

1. **The differential: native == the wat oracle, bit-for-bit**, on every input. Token bindings feed
   joins and productions; a wrong binding is a wrong derivation.
2. **`:accuracy :match` on all nine axes; `:derived` byte-identical.**
3. **The census + boundary A/B from Step 0**, reported per axis — including the crossing frequency,
   which is what STOP-0 turns on.
4. **`filter` and `hash-join` do not regress** in `node_share_fire_phase_census` — Token bindings
   are read in both.
5. **Release floor and clippy 0**, by my own re-run.
6. **`PMap`'s six laws still green** — this stone must not loosen the wall the last one built.

## Out of scope = REJECTED

- **Re-tuning `PROMOTION_THRESHOLD`.** 8 is Clojure's. If the census says the boundary is crossed
  constantly, that is STOP-0 and a conversation, not a knob to turn quietly.
- **`Element.bindings`.** Already an `Arc<[(Value, Value)]>` and correct — it never extends, so it
  has no promotion question. Untouched.
- **Demotion.** Same one-way contract as `PMap` itself.

## Honest state

Nothing built. The correction is grounded (`element-bindings-array.md:36/:94/:116`), the mechanism
exists and is proven, and the risk is named with a STOP on it. The one number that decides the shape
— how often a Token crosses 8 — **is not yet measured**, and this stone does not proceed past Step 0
without it.
