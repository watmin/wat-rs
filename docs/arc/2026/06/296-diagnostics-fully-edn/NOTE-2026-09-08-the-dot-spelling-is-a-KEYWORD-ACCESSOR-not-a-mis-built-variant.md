# NOTE — the dot spelling is a KEYWORD ACCESSOR MISS, not a mis-built variant

**Measured 2026-09-08**, answering the caveat carried in the seam and in
`NOTE-the-dot-spelling-silently-builds-the-wrong-variant.md`:

> *"Why it yields `Option.None` specifically is **unmeasured** — read the expansion before
> theorising."* `[[feedback_an_adjacent_implementation_is_not_the_subject]]`

It has now been measured, and **the mechanism is not the one the name implies. Nothing constructs a
variant.** Recorded beside the original, not in it — what is written stays written.

## The measurement

`wat-scripts/scratch-pad/keyword-accessor-vs-enum-map-ctor.wat`, run against `target/release/wat`:

```
(:wat::core::Option::Some {:value 7})                      ->  #wat.core/Option.Some {:value 7}
(:wat::core::Option.Some  {:value 7})                      ->  #wat.core/Option.None {}
(:wat::core::Option.Some  {:wat::core::Option.Some 42})     ->  #wat.core/Option.Some {:value 42}   ★
(:anything-at-all         {:value 7})                      ->  #wat.core/Option.None {}            ★
(:value                   {:value 7})                      ->  #wat.core/Option.Some {:value 7}    ★
(:wat::core::Nope 7)                                       ->  UnknownFunction (LOUD, correct)
```

★ The third line finds the key **when the key is actually present**. The fourth needs no `:wat::`
prefix at all. The fifth is a plain hit. This is a **map lookup**, and `None` is the honest answer
to the question that was asked.

## The mechanism — `src/runtime.rs:3655`, arc 234 Stone 234.3c

```rust
// Arc 234 Stone 234.3c — keyword-as-accessor fall-through.
// When head is an unknown verb AND args.len() == 1 AND receiver is
// {Value::Aggregate (Record/HolonRecord/Struct), wat__std__HashMap}, dispatch as field accessor.
```

```rust
Value::wat__std__HashMap(map) => {
    // Never errors on miss — missing key = None (per D5 / T7).
    let key = Value::wat__core__keyword(Arc::new(other.to_string()));
    return match map.get(&key) { Some(v) => Ok(…Some…), None => Ok(…None…) };
}
```

The accessor is **total by design** — a miss is `None`, never an error — and that design is correct
for an accessor. It fires *last*, only for a head that resolved nowhere.

## ★★★ THE ACTUAL DEFECT — 296 M gave the ctor the shape 234 had already given the accessor

```
(K {…})   enum map ctor        arc 296 M, 2026-09-07     K resolves   → the map is the PAYLOAD
(K {…})   keyword accessor     arc 234.3c, months older  K is unknown → the map is the RECEIVER
```

**One syntax, two meanings, and which one you get is decided by whether `K` resolves.** So a
*misspelled constructor is not a misspelling at all* — it is a well-formed accessor over the
payload it meant to build. The ctor was positional until yesterday; this collision did not exist
before 296 M and was not seen when M landed.

## Two conditions, and breaking EITHER makes the failure loud

```
1  the :wat::* blanket        lets the head past resolve.  A :usr:: head is REFUSED first:
                              (:usr::Box.Full {:payload 7})  -> UnresolvedReference, check=1
                              (:usr::whatever {:payload 7})  -> the same. The dot is irrelevant;
                              resolve refuses EVERY namespaced non-reserved keyword head.
2  the accessor fall-through  converts "unknown verb + one map arg" from UnknownFunction into a
                              total lookup. Give it a NON-map receiver and it refuses correctly.
```

⚠ **This is the correction to the queue's stated reason.** The seam said the dot spelling *"is
accepted under `:wat::*` and SILENTLY BUILDS THE WRONG VARIANT."* Half of that is right — it is
accepted under `:wat::*`, and the value is silently wrong. The **"builds the wrong variant"** half
is false: it builds nothing, it *reads*. The blanket is one of two necessary conditions, not the
whole mechanism, and the second condition lives in arc 234, not in 255.

## What this does NOT change

- **The blanket is still a correctness defect and still gates the dot flip.** Condition 1 is real
  and it is 255's founding target. A user namespace catches this today; `:wat::*` does not.
- **The consequence is unchanged**: `(:wat::core::Option.Some {:value 7})` checks clean and yields
  a wrong value, and that spelling is the one the head migration moves toward.

## What it opens — for the builder to rule

★ Once heads are **real symbols** (`wat.core/Option.Some`) rather than keywords, the collision
**cannot be written down**: a symbol head is not a keyword, so it can never be an accessor key. The
crusade's own destination dissolves this by construction — which makes the ordering question
"which do we do first," not "how do we patch the accessor."

⬜ And an independent question the measurement raises, which does not need the blanket at all:
**should the accessor fall-through fire on a head that is NAMESPACED?** `(:value {…})` is the
accessor's honest use. `(:wat::core::Option.Some {…})` is not plausibly a field name — no record
has a `:wat::core::`-qualified field. A namespaced key is the shape of a *verb that failed to
resolve*, and that is the population where the totality hides a defect rather than serving a caller.
