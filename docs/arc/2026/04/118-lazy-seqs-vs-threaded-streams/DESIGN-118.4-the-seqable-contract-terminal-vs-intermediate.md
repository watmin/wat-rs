# DESIGN — 118.4 · the `Seqable` contract: intermediate vs terminal, and why `length` stays refused

**Builder ruled D 2026-08-17** (*"D has been reasoned and i don't see a reason against it… seqs have
been a thing we've been working towards for months"*), then set the reference frame: *"we should
prefer whatever clojure does, influenced by ruby."*

Following that literally does not work, and the reason is the whole design.

## ⛔ Clojure's answer is bought with caching wat does not have

| language | lazy seq | `count` / `length` |
|---|---|---|
| **Clojure** | `LazySeq` is **CACHED** — realizing is idempotent | `(count s)` realizes, then `(map f s)` still works |
| **Ruby** | `Enumerator` is **read-once** | `#length` **does not exist** · `#size` → `nil` · `#count` → walks |
| **Java** | `Stream` is **read-once** | `count()` is a **terminal** op; reuse throws |
| **wat** | `Stream` is **read-once** (builder, 2026-08-17: *"wat does not cache... its read once"*) | — |

**Clojure's semantics depend on a cache.** wat is read-once, so Clojure's answer is not available to
us at any price short of building the cache. What Clojure gives us is **names**. Ruby and Java give
us the **semantics**, and — independently — they agree.

⚠ **A probe of mine appeared to show caching and did not.** `(map f v)` over a *Vector* produces a
stream over a re-walkable source, so two `into`s both returned 3. That measures the **source**, not
the stream. The builder corrected it. A genuine single-pass source is required to measure this, and
`3,3` must not be read as "wat caches."

## The rule

> **`Seqable` is what you can walk. An operation is INTERMEDIATE (returns a Stream, stays lazy) or
> TERMINAL (consumes, returns a value). On a read-once value, consumed means gone.**

That is Java's frame, and it explains the shape of the existing corpus: **arc 118 shipped exactly the
intermediate half and stopped.**

```
INTERMEDIATE  map ✓  filter ✓  take ✓  drop ✓        already done, arc 118.2a
              concat · conj                          to add  (ordered / has_append)

TERMINAL      foldl · foldr                          to add  (mappable)
              count                                  to MINT (new verb)
              contains?                              to add  (searchable)
              reverse                                to add  (ordered, materializes)

REFUSED — correctly, NO CHANGE          length · empty?  (measurable)
                                        get              (gettable)

ALREADY TRUE                            first/second/third (indexable) · rest (has_tail)
```

★ `first` and `rest` are already `true` for Stream — **Clojure's ISeq primitives are already under
it.** The foundation was there the whole time.

## ★ `length` STAYS REFUSED — and this corrects my own earlier worklist

I listed `length`/`empty?` as gaps to fill. **Ruby says no, and Ruby is right.**

`Enumerator#length` raises `NoMethodError` — deliberately. `length` promises a size known *without
walking*. A read-once stream cannot promise that. So `measurable() => false` for Stream is **the
correct answer, not the `○ gap`** the table's comment calls it.

The split is Ruby's, exactly:

| verb | status | meaning |
|---|---|---|
| `length` | **unchanged** — Vector/List/PersistentVector only | "size, known without walking" |
| `count` | **NEW, terminal** — any `Seqable` including Stream | "walk it and tell me how many" |
| `counted?` | optional | Clojure's name for Ruby's `size → nil`: is the length known without walking? |

★ **`:wat::core::count` is already free AND already blessed pure-total** — it appears exactly once in
the tree, in `macros/eval.rs:563`'s `is_pure_total` allowlist, with **no `TypeScheme` and no dispatch
arm**. Someone intended it and never built it. Minting it needs no allowlist change.

`:wat::core::counted?` and `:wat::core::seq` are both entirely free (0 hits, Rust and wat).

## What this buys, stated as the synthesis it is

- **Clojure's names** — `seq`, `count`, `counted?`
- **Ruby's split** — `length` ≠ `count`; the verb that cannot answer cheaply does not pretend to
- **Java's structure** — intermediate vs terminal, one rule with no per-verb exceptions

And one rule with no exceptions is what the bytecode compiler wants as input. Builder, 2026-08-17:
*"the surface will be our expression language for optimized code it produces."*

## ★ RULED 2026-08-17 — `empty?` refuses too, and the rule becomes uniform

Builder demonstrated it live, same session, same shape as `length`:

```
[1] e = Enumerator.new { |y| y << 1 }
[2] e.empty?     NoMethodError: undefined method `empty?' for an instance of Enumerator
[3] [].empty?    => true
```

Ruby refuses **both** `length` and `empty?` on a read-once sequence, and refuses them for one reason:
they are **cheap structural queries**, and a stream cannot answer either without consuming.

Clojure's `empty?` *does* work on a lazy seq — and it works for exactly the reason its `count` works:
**caching makes realizing one element harmless.** We have no cache. So the same argument that took
Clojure's `count` off the table takes its `empty?` too. Builder: *"ruby's bias."*

> **A read-once value gets no cheap structural queries.** `length` and `empty?` both refuse Stream.
> The terminal `count` is how you find out, and it costs you the stream.

★ **Consequence: `measurable()` needs NO CHANGE AT ALL.** Its doc says *"`length` / `empty?` — element
count"*, and `Stream => false` is now the correct answer for both. What the table calls a `○ gap`
there is not a gap; it is the contract.

## The same argument disposes of `get`

`gettable()` gates `get` — index lookup. Ruby's `Enumerator` has no `[]`; Java's `Stream` has no
`get`. **Random access implies a shape a stream does not have**, so `gettable() => false` for Stream
is also correct and also needs no change.

Note the substrate already draws this line correctly: `indexable()` is **true** for Stream
(`first`/`second`/`third` — O(1)-ish prefix walks) while `gettable()` is **false** (`get n` is O(n)
random access). That distinction was already right.
- **`reverse` over a Stream** materializes. Permitted under the standing ruling — *"single-pass is a
  property of the value, never of the surface"* — but worth naming as terminal so nobody reads it as
  lazy.
- **The seven `-stream` twins** (`wat/seq.wat`) are the workaround this contract deletes. Their own
  stone, after this.

## ★★ THE REFUSAL IS A FORCING FUNCTION, NOT A LIMITATION — and the escape hatch already exists

Builder, 2026-08-17: *"a user could shove a stream into a vec and then do the empty check?.. and any
others.. right?"* **Yes — and `into` already has the clauses.** Measured live:

```wat
(:wat::core::into (:wat::core::Vector :wat::core::i64) some-stream)
(:wat::core::into (:wat::core::PersistentVector)       some-stream)
;=> "vec-len=3 vec-empty=false vec-get=#wat.core.Option/Some [2] pv-len=2"
```

`into`'s clause list carries **`(Vector<T>, Stream<T>)`** and **`(PersistentVector<T>, Stream<T>)`**
today. So the complete story is:

> **A Stream refuses `length`/`empty?`/`get`. `into` materializes it. The materialized value answers
> everything.**

That is Java's `collect(toList())` then `.size()`/`.isEmpty()`/`.get(i)`, and Ruby's `to_a` then
`.length`/`.empty?`/`[]`. Every read-once design lands here.

★ **This is what upgrades the refusal from Honest to Good UX.** The cost of walking a read-once
sequence is made **visible at the call site** — the user writes `into` and thereby says *"I am
consuming this"* — and after that one obvious verb, nothing is restricted. The wrong path (a silent
consuming `length`) is not there; the right path is one word.

⚠ The one real `into` gap found this session is **`(Vector<T>, List)`** — List, not Stream. Sibling
of task #45's already-shipped `(PersistentVector, Vector)`. Independent of this contract.

## What is measured vs what is knowledge

**Measured on disk this session:** Stream's full capability row; `mappable()`/`ordered()` marking
Stream `○ gap`; `foldl` refusing `Stream<T>`; `map`/`filter`/`take`/`drop` bypassing those tables via
`extract_lazyable_elem`; `count`/`counted?`/`seq` all free; `count` present only in the purity
allowlist.

**Model knowledge, NOT verified against a live interpreter, and checkable:** Clojure's `LazySeq`
caching, Clojure's `counted?`, Java's terminal/intermediate split and single-use `Stream`. Ruby's
three behaviours were **demonstrated live by the builder** (`length` → NoMethodError, `size` → nil,
`count` → 1) and are not model knowledge.
