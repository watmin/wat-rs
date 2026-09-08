# BRIEF — STONE Q2: `subtype?` belongs in the registry

> **Builder:** *"this belongs in the registry — this is just another occurrence reminding us why
> the registry is necessary."*

## HOW IT SURFACED — a wall firing on the stone that describes its own disease

Stone Q added `:wat::runtime::is-type?` — the verb that answers *is this a type?* across three
mechanisms because no single set could. Its first `@see` pointed at its natural sibling,
`:wat::core::subtype?`, and the `@see` wall went red:

```
dangling @see `:wat::core::subtype?` on `:wat::runtime::is-type?`      src/intrinsic/mod.rs:2006
```

The wall is **right**, and the `@see` is **also right**. `subtype?` is live: the corpus calls it 9
times, `check.rs:2791` has its arm, `runtime.rs:9938` implements it, and it answers `"true"` when
run. It is simply **not registered**.

## ⛔ IT IS NOT A CLASS. IT IS ONE OF A PAIR, AND 255 TOOK THE OTHER.

```
:wat::core::conforms?   arc 237 Stone 237.5   REGISTERED #[wat_intrinsic] by arc 255 Stone 1c-a-ii
:wat::core::subtype?    arc 237 Stone S-A     runtime.rs:9938 · check.rs:2791 · NOT registered
```

`runtime.rs:9885` names them in one breath — *"`conforms?` and `subtype?` consult…"*. They are twins:
both arc 237, both type-level predicates taking type keywords rather than values, both with runtime
implementations and checker arms. **255's registration stone registered one of them.**

Sampled siblings, for shape: `conforms?` **registered** · `variant-name` **registered** ·
`is-type?` (Q's own) **registered** · `subtype?` **not**.

## THE WORK

**1. ⛔ FIRST — THE CENSUS, because I sampled FOUR of 143.** `check.rs` carries **143** string-literal
verb arms. I checked four and concluded "an omission, not a class." **Measure the full set: which
arms have a `check.rs` arm but NO intrinsic registration?** Commit the list.

```
if the count is SMALL   -> subtype? is an omission; register it; report the others as a short list
if the count is LARGE   -> ⛔ STOP. It is arc 255's special-forms bucket, the disposition flips,
                           and this stone is redrawn as a 255 stone. SAY SO; do not register 143
                           rows on a brief that assumed one.
```

**2. Register `subtype?` mirroring `conforms?`.** Same mechanics — arc 255 Stone 1c-a-ii is the
worked precedent, three arms away in the same file. Copy its shape, not a new one.

**3. The `@see` stays.** It is a correct reference to a live verb. ⛔ Do NOT remove or repoint it —
that would delete a true cross-reference to silence a true report.

**4. Verify the wall goes green for the RIGHT reason**: because `subtype?` now resolves, not because
the reference moved.

## READ IN ORDER

```
src/intrinsic/mod.rs:1998-2010   the @see wall + its doc: it already unions TWO sources (a
                                 registered Rust intrinsic OR a wat verb that DECLARES — arc 255's
                                 "@see can cross the boundary" stone). subtype? is in neither.
src/runtime.rs:9938              subtype?'s implementation (arc 237 Stone S-A)
src/runtime.rs:2159              its dispatch arm
src/runtime.rs:9468-9500         ★ conforms? — THE PRECEDENT. Registered #[wat_intrinsic] by 255
                                 Stone 1c-a-ii, with the variadic-sniff/single-@arg mechanics
                                 spelled out in its doc block. Mirror this.
src/check.rs:2791                subtype?'s checker arm — both args are Keyword, inference skipped
                                 on both. That is why it is not a value-level intrinsic in shape.
```

## STOP TRIGGERS

- **STOP-1 — registration proceeds before the census is committed.** Four of 143 is a sample, not a
  measurement. The census decides whether this stone is one row or 255's bucket.
- **STOP-2 — the `@see` is removed or repointed.** It is correct. The fact it reported is the finding.
- **STOP-3 — the wall is loosened.** It already unions two sources and it is asking the right
  question. Widening it to admit unregistered verbs would delete the only instrument that found this.
- **STOP-4 — a new registration shape is invented.** `conforms?` is the twin and the precedent.
- **STOP-5 — Doctrine 1 is touched.** `subtype?` takes type keywords, not values; that is why its
  checker arm skips inference on both args. Register it as it is.
