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
              concat · conj                          to add

TERMINAL      foldl · foldr · count · contains? · get · reverse
```

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

## Unruled, flagged not decided

- **`empty?` on a Stream.** It needs only ONE element — cheap — but on a read-once value that element
  is *gone*. Clojure allows it (realizes one, cached, harmless); Ruby's Enumerator has no `empty?`;
  Java has no `isEmpty`. Under our read-once semantics it is a **terminal op that looks free**. Allow
  it as terminal, or refuse it like `length`? Genuinely open.
- **`reverse` over a Stream** materializes. Permitted under the standing ruling — *"single-pass is a
  property of the value, never of the surface"* — but worth naming as terminal so nobody reads it as
  lazy.
- **The seven `-stream` twins** (`wat/seq.wat`) are the workaround this contract deletes. Their own
  stone, after this.

## What is measured vs what is knowledge

**Measured on disk this session:** Stream's full capability row; `mappable()`/`ordered()` marking
Stream `○ gap`; `foldl` refusing `Stream<T>`; `map`/`filter`/`take`/`drop` bypassing those tables via
`extract_lazyable_elem`; `count`/`counted?`/`seq` all free; `count` present only in the purity
allowlist.

**Model knowledge, NOT verified against a live interpreter, and checkable:** Clojure's `LazySeq`
caching, Clojure's `counted?`, Java's terminal/intermediate split and single-use `Stream`. Ruby's
three behaviours were **demonstrated live by the builder** (`length` → NoMethodError, `size` → nil,
`count` → 1) and are not model knowledge.
