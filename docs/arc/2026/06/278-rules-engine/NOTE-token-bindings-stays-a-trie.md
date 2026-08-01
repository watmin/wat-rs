# NOTE — `Token.bindings` stays a trie, and now on evidence rather than inheritance

> **The challenge (builder, 2026-08-01):** *"does this still hold up?... as we've made forward
> progress earlier decisions have been overruled once a new path was revealed."* Asked of my
> deferral to R60 while ranking `hj:catchup:probe` (18.3 ms, 21% of the fire, the largest single
> item) — whose cost is dominated by `extend_token`'s bindings fold.

## What R60 actually settled, and what I wrongly claimed it settled

`41c59cde`'s own message:

> *"Element.bindings: rpds::HashTrieMapSync → Arc<[(Value, Value)]>. **Token.bindings stays a
> trie.** … an Element is built once and then only read/cloned/dropped, and **the trie's sole
> advantage is extend, which an Element never does.**"*

Airtight in the direction it was used: an Element never extends → a trie buys it nothing → array.
That stone bought 42%.

But I invoked the **converse** — *"Token extends, therefore a trie is right for Token"* — and it
does not follow. *"The trie's advantage is extend"* + *"Token extends"* yields only **"a trie is not
pointless for Token,"** which is far weaker. `Token.bindings` was never changed; it was **left**,
with a rationale attached, and I later quoted that rationale back as a measured verdict. That is the
chronicle read as ground instead of as history — `feedback_ground_the_substrate_not_just_the_chronicle`,
and R61's lesson wearing a new costume.

## The trap the measurement had to avoid

R60 killed a **corpus-derived threshold**: 91% of our rules bind ≤3 variables was *true, measured,
and favourable*, and the builder threw it out — *"you have no fucking clue what are our users … are
going to do."* Re-deciding the representation from our corpus's cardinality would re-run exactly
that error.

So the probe asks a **dominance** question, which needs no corpus: *does one representation win
across the whole plausible cardinality range?* If yes, there is no constant to tune. If it wins only
below some N, that N **is** a corpus threshold and R60's cut applies.

## The probe

`token_bindings_representation_dominance` (`src/rete/kernel.rs`, committed). Measures the real
`extend_token` bindings fold (`parent.clone()`, per-key `get`-guard, `insert_mut`) against an array
twin with identical semantics. The shape is the real one — **one parent, twenty children** — which is
exactly where a trie's structural sharing is supposed to pay, since every child shares the parent's
nodes while an array copies the whole prefix into each. GET is measured alongside EXTEND, on the
**worst** key (last inserted), because the matcher reads bindings constantly and a representation
that extends faster but reads slower is not a win.

A **faithfulness gate runs before any timing**: both representations must produce the same logical
binding set (same size, every key resolving to the same value) or the test fails before reporting a
number. Nothing about the timing is asserted, so the probe cannot flake.

## The result — three runs, consistent

```
card    EXTEND trie   EXTEND array   ratio      GET trie    GET array   ratio
   1       178.7ns         83.9ns    2.13x        19.7ns         4.2ns    4.67x
   2       182.0ns        100.7ns    1.81x        19.6ns         9.2ns    2.12x
   3       185.7ns        116.0ns    1.60x        20.4ns        13.9ns    1.46x
   4       197.0ns        136.3ns    1.44x        20.2ns        16.6ns    1.22x
   8       238.9ns        216.9ns    1.10x        19.8ns        29.7ns    0.67x
  16       320.4ns        421.8ns    0.76x        19.7ns        64.0ns    0.31x
  32       426.6ns        752.7ns    0.57x        19.3ns       135.3ns    0.14x
  64       592.3ns       1412.5ns    0.42x        23.4ns       233.7ns    0.10x
```

**NO DOMINANCE.** The array wins extend below ~12 and loses above; it wins read below ~6 and loses
catastrophically above. Both crossovers are thresholds, so R60's cut stands and the trie is retained.

**The structural fact that decides it:** the trie's GET is **flat — ~19–20 ns from cardinality 1 to
64** — while the array's climbs linearly from 4 ns to 234 ns. The matcher reads bindings constantly,
and R25's chaos engine is the many-condition regime where that line keeps climbing.

## The number that had to be refused

At the fanout cell's actual cardinality — a Token binding `?k`, `?l`, `?r`, i.e. **three** — the
array extends **1.6× faster**. Real number, real workload, and taking it would be tuning the engine
to our own test suite. `QVOD FAVET, PRIMVM CADIT`: the measurement that flatters the change I was
drifting toward is the one that falls first.

## What this does NOT settle

- **`extend_token`'s `matches` field.** `tok.matches.clone()` allocates at exactly `len`, then
  `push` reallocs — two allocations where `Vec::with_capacity(len + 1)` would do one. ~2–3 ms on the
  fanout cell, independent of the bindings question, still open.
- **Whether the `get`-guard before `insert_mut` pays.** It costs a hash + compare to avoid a path
  copy on an already-equal shared join key. Not measured; a separate question.
- **Any cardinality above 64.** The ladder stops there because the crossovers are already three
  doublings behind it; the trend is not in doubt.
