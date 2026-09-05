# DESIGN-STONE — `Element.bindings` becomes an array; `Token.bindings` stays a trie

> **Origin (2026-07-31).** Ruled after two microbenchmarks (`e4c73043`) killed the framing this stone
> started with. It was going to be "swap rpds for a small-vec, sized to our corpus." Both halves of
> that were wrong: the corpus census was never a legitimate design input (the builder rejected it —
> *"you have no fucking clue what our users are going to do"*), and the key type turned out not to be
> the cost at all. What survived measurement is narrower, better grounded, and does not depend on
> guessing what anyone writes.

## The finding — two objects, two operation profiles

Grounded, by reading every insert site:

- **`Element.bindings`** is built once by `alpha_match_inner` (`matcher.rs:357`) and is **read-only
  forever after.** No site extends it.
- **`Token.bindings`** is the only one extended — `extend_token` (`kernel.rs:876`) and the accumulate
  result var (`kernel.rs:2364`).

Now the measurement (`binding_repr_microbench`, release, 20k iters, real `Value::String` keys;
A = `rpds::HashTrieMapSync`, B = `Arc<Vec<(Value,Value)>>` — the honest `PersistentArrayMap` analogue,
where clone is a refcount bump in **both** so only lookup differs):

```
  n      build          lookup        clone        extend          drop
  1   298.6/109.9    16.9/  2.5   24.9/10.3   264.0/ 264.3    60.7/ 31.9
  2   345.9/ 92.8    33.6/  9.7   25.6/10.3   252.1/ 230.9   188.6/ 55.5
  3   377.0/ 86.5    18.5/  7.9   13.7/ 7.7   220.2/ 310.8   279.9/ 63.1
  8  1141.7/257.6    16.6/ 16.8   18.8/ 7.9   212.9/ 727.4  1212.5/117.2
 16  4228.0/583.1    41.1/ 54.9   28.9/ 9.1   330.0/1465.4  2831.6/198.1
```

**The array wins build, lookup, clone and drop. The trie wins exactly one operation — extend — which
is the one an Element never performs.** And there are ~80,200 Elements per fire at `G=200 W=200`
against a handful of Tokens.

So the discriminator was never rule width or a size threshold. It is **which operations the object
performs.** Element and Token want different representations because they do different things.

## ★ THE ONE CONTRACT DECISION

**`matcher`'s binding readers become generic over a `Bindings` trait; they are NOT given a converter.**

`matcher.rs` demonstrably reads **both** kinds — `build_insert_fact` (`:472`) is token-side,
`alpha_match_inner`'s accumulator (`:343`) is element-side, and `eval_test_core` (`:967`) can be
either, because a `:test` clause may sit after a join. So the two representations meet inside matcher
whether we like it or not, and there are only two ways to resolve it:

- **Convert at the boundary** — build an rpds map from the array wherever matcher wants one. This
  **reintroduces the exact allocation the stone exists to remove**, in the hot path, and would make
  the whole change worthless while still looking green. Rejected.
- **A minimal `Bindings` trait** (`get(&Value) -> Option<&Value>`, `iter()`), implemented by both,
  with matcher's readers taking `impl Bindings`. Monomorphised — **no vtable, no dispatch cost** —
  and each call site compiles to the concrete representation it was handed.

Take the second. The trait must stay **read-only**: the moment it grows an `insert`, the two
representations are being forced into one interface again and the array will be made to pay for the
trie's operation.

## Blast radius

`src/rete/kernel/session.rs` (the `Element` struct's field, the encode/decode pair) and `src/rete/matcher.rs`
(7 bindings-taking signatures, 13 mentions of the concrete type). **Not one file** — the earlier claim
that this was contained to `kernel.rs` was wrong and is corrected here.

**Unblocked by `32142f8a`:** until Element became a native struct this morning, its bindings were a
`Value::wat__core__PersistentMap` and no other representation was expressible. That stone was the
prerequisite, exactly as `03ec041e` predicted.

## RESOLVED — no spill. A plain array at any width.

*(This section was "deliberately not pre-decided" and pointed a rider at
`binding_cardinality_distribution` to settle it. That was a **false claim**: the census shared one
bucket set across both kinds, so it separated the TOTALS and not the histogram. Instrument fixed —
buckets are now per-kind — and the question is answered below rather than handed on.)*

```
ELEMENTS   accum axis: 1 -> 50.4%, 2 -> 49.6%     max 2
           2-cond join: 2 -> 100%                 max 2
TOKENS     accum axis: 1 -> 33.3%, 2 -> 66.7%
           2-cond join: 2 -> 50%,  3 -> 50%       max 3
```

An element's bindings come from **one condition**, so its width is *fields destructured in that one
condition* — not rule width. That is a far tighter structural bound than the rule-level one, and it is
why elements sit at 1–2 here. It is **not** a cap: a wide packet condition binding five or six fields
is ordinary, and nothing forbids more.

**No spill is needed anyway.** The array's only losing operation is lookup past 8 — and at n=16 that
loss is `54.9 vs 41.1`, **1.3×**, while build is `583 vs 4228` (7.3× the other way) and drop is
`198 vs 2831` (14×). Lookup is two orders of magnitude cheaper than build. The array wins the
aggregate at **every size measured**, so a plain `Arc<[(Value, Value)]>` is correct at any width and
a spill path would be complexity bought for nothing.

The 8-crossover stays interesting for `Token.bindings` — which extends, and where the trie wins —
but Token is out of scope here.

## The gate

A representation change with **no behaviour change** — nothing fire derives may move. What would turn
it red:

- **`probe_arc278_2b_insert_alpha`** — reads alpha off `fire-once'` and asserts a binding's *value*
  (`?t = 25`). It exercises the element-bindings encode path end to end. Load-bearing.
- **`round_trip_fired_session`** — both directions over a populated alpha.
- **The ~24 count differentials** — any change to what fire derives.
- **`binding_cardinality_distribution`** — still reports both populations, so a representation that
  silently lost bindings would show up as a shifted histogram rather than a crash.

**Measured expectation:** at n=2 (elements' modal), build `345.9 → 92.8` and drop `188.6 → 55.5` —
≈386 ns × 80,200 ≈ **31 ms**, against a ~113 ms release fire. Call it **~20–27%**, and say the real
number afterwards: this is an isolated-microbenchmark extrapolation and in-situ gains are always
smaller.

## Out of scope = REJECTED (affirmative cuts)

- **Changing `Token.bindings`.** The trie wins extend 3.4× at n=8, and extending is all a Token does.
  Leaving it is the finding, not an omission.
- **Keyword interning.** Measured at 1.0–1.2× on lookup — not the lever here. It is a language-level
  correctness matter, filed at `109-kill-std/NOTE-keyword-storage-must-intern.md`, and must not be
  smuggled into this diff where it would destroy the attribution.
- **Touching wat's `PersistentMap` value type.** That is the language, not the kernel — its own arc.
- **Sizing anything to our rule corpus.** Rejected at the root: our tests do not predict users.
- **`wat/rete.wat`.** The oracle is never optimized.
