# NOTE (arc 109 vocabulary) — type names go SHORT and lowercase, and the collections collapse to one family

**Filed 2026-08-24. A DIRECTION, not a decision — and it REVERSES a measured position from four days
ago. Read the reversal section before acting on either note.**

Builder, verbatim:

> *"i want to standardize to… `wat.type/i64` `wat.type/f64` `wat.type/bool` `wat.type/str`
> `wat.type/map` `wat.type/set` `wat.type/vec` `wat.type/list` `wat.type/tuple` `wat.type/u8` …
> and so on… i think we should move all the collection types to persistent map, vec, list etc etc"*

## ⚠ THIS REVERSES `NOTE-type-name-casing-is-90-percent-done.md` (2026-08-20)

That note is four days old, it is MEASURED, and it concluded the opposite. It was opened by the
builder's own *"i think we'll mimic what rust does…. i64 f64 String HashMap HashSet"*, censused the
corpus, and found:

> lowercase `i64` 3037 · `bool` 802 · `keyword` 483 · `nil` 433 · `f64` 279
> Uppercase `String` 1400 · `Vector` 1143 · `PersistentVector` 1009 · `PersistentMap` 430 …
>
> **"That IS the Rust convention already — lowercase primitives, Uppercase containers and
> aggregates."** Its scope was two renames (`keyword`→`Keyword`, `symbol`→`Symbol`) and one open
> question (`nil`).

**Both notes must not read as live.** This one supersedes the casing direction: the target is now
all-lowercase and SHORT (`str`, `vec`, `map`), which is Clojure's register rather than Rust's. The
older note's *measurements* remain good and are cited below; its *direction* does not. Whoever draws
the stone amends that file to point here — a superseded direction left unmarked is how a rejected
option comes back in new clothes.

## TWO QUESTIONS ARE BUNDLED HERE, AND THEY ARE NOT THE SAME SIZE

### (a) The rename — mechanical

`String` → `str`, `Tuple` → `tuple`, `HashSet` → `set`, and the primitives keep their spelling. A
prefix/exact codemod, the same shape as `rename-kernel-to-spawn.wat`. Large but ordinary.

### (b) The collection collapse — a SEMANTIC change, and it is the real stone

*"move all the collection types to persistent map, vec, list"* is not a rename, because
**`Vector` and `PersistentVector` are two DISTINCT LIVE TYPES today.** Measured this session:

```clojure
(:wat::core::type (vector-returning-fn))            →  "wat::core::Vector"
(:wat::core::type (persistent-vector-returning-fn)) →  "wat::core::PersistentVector"
```

Both type-check, both are inhabited, both are used. Same for `HashMap` / `PersistentMap`. So
`wat.type/vec` cannot simply be a new spelling — **one family has to absorb the other**, and every
call site of the loser changes meaning, not just name.

## MEASURED TODAY — bare type heads, all `git ls-files '*.wat'`

| name | sites | | name | sites |
|---|---|---|---|---|
| `String` | 4408 | | `Tuple` | 857 |
| `Vector` | 2634 | | `Option` | 760 |
| `PersistentVector` | 1697 | | `HashMap` | 451 |
| `PersistentMap` | 685 | | `HashSet` | 131 |
| `Result` | 127 | | `List` | 109 |
| `Bytes` | 12 | | | |

⚠ These are NOT comparable to the 08-20 census and neither is wrong: that one counted `wat/` +
`tests/**/*.wat`; this counts every tracked `.wat`. Different populations, stated so nobody reads a
discrepancy as drift.

**The collection collapse alone is ~5,700 sites across four pairs.** That is larger than stone E.

## OPEN — the proposal does not cover these, and a stone cannot guess

1. **Which family wins, and what happens to the loser's semantics?** If `Vector` is the mutable-ish
   builder and `PersistentVector` the immutable one, collapsing them is a behaviour decision, not a
   naming one. **Measure what actually differs before scoping.** Nothing in this note establishes
   that they differ only in name — and if they turn out to be the same thing wearing two names, the
   collapse is mechanical after all and this paragraph is the finding.
2. **`Option` / `Result`** are not in the list. Do they become `wat.type/option` / `wat.type/result`,
   or stay capitalised as enums? They are ENUMS, not containers — and `296/DESIGN-STONE-H` is
   simultaneously moving their VARIANTS to `#wat.core/Option.Some {…}`. The two must agree.
3. **`nil`** — still open from the 08-20 note, still open here.
4. **`keyword` / `symbol`** — the 08-20 note scoped them UP to `Keyword`/`Symbol`. Under this note's
   all-lowercase direction they stay put and that scope evaporates. Confirm which.
5. **`Bytes`** (12 sites) — `bytes`? It is a primitive-ish container.

## WHAT THIS DOES NOT COLLIDE WITH

**Stone E, in flight as this is filed**, moves the string OP namespace `:wat::core::string::` →
`:wat::string::`. That is verbs. This note moves the TYPE `:wat::core::String` → `wat.type/str`.
Different namespaces, no collision — `wat.string/join` and `wat.type/str` coexist cleanly, which is
the same separation `NOTE-typed-form-and-type-namespace.md` proposed in the first place.

⚠ But note the ORDER dependency: stone E's central control is that
**`:wat::core::String` is UNTOUCHED** (4741 occurrences, counted before it ran) because the type and
the op-namespace share a parent and only the trailing `::` separates them. This note's rename must
land AFTER E, or E's control is measuring a moving target.

## SEQUENCING

This is the `wat.type/` half of `NOTE-typed-form-and-type-namespace.md`, whose three coupled moves
were: `<-` → `:-` (**DONE**, arc 109), a `:wat::type::` namespace (this), and the dotted clojure
render (251's flip). The dotted form `wat.type/i64` the builder writes above is 251's spelling —
so this note's CONTENT can land in the current `::` spelling first, or wait and land once with the
flip. **Landing it twice is the thing to avoid**, and that is the same argument that put home #4 at
the end of the string chain.

**Not scoped, not queued.** A direction with two measurements attached and five open questions.
