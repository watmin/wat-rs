# NOTE — character case carries no meaning in this language

**Ruled by the builder 2026-09-10**, mid dot-flip landing, filed here because arc 109 is where the
name grammar lives.

> *"we should never do any character case based logic… char case does not matter"*

```wat
(wat.core/defenum user/some-enum wat.enum/Pure
  first []
  second [])
```

is a legal declaration producing `user/some-enum.first` and `user/some-enum.second`. **We have a
BIAS toward `Enum.Variant` over `enum.variant` — a bias is not a rule, and nothing may branch on
it.**

⚠ **NOT FIXED. Recorded, deliberately, so the dot flip can land first.** This note is the tracker;
the work is a separate arc.

## Where the substrate violates it

### ⛔ Case decides SEMANTICS — three copies of one rule, plus a fourth site

```
src/check.rs:1577           a bare name is a TYPE VARIABLE iff its first alphabetic char is uppercase
src/function/subsume.rs:60  the same rule, second copy
src/function/subsume.rs:95  the same rule again
src/declare/parse.rs:1048   the same rule, third copy
```

`check.rs`'s own comment names the fourth: *"same rule as `collect_free_type_vars::is_type_var` in
`runtime.rs`"*. Under it, the builder's `some-enum` can never be a type variable and `T` always is —
purely because of a letter's case.

```
crates/wat-macros/src/edn_doc.rs:266-267
    let type_is_type   = type_seg.starts_with(|c: char| c.is_uppercase());
    let name_is_method = !name.starts_with(|c: char| c.is_uppercase());
```

decides *"is this a type, is this a method"* by case, and its own doc admits the limit: *"A record
type spelled lowercase, or a method spelled uppercase, defeats it."*

★ **I made that one worse on 2026-09-10.** Fixing a doc-block breakage during the dot flip I added
`if type_is_type { compose_variant(…) }` — a **variant-vs-namespace-join decision on case**. It
works only because every doc axis happens to read `Purity/Pure`, `Determinism/Deterministic`. A
lowercase-named enum in a doc block breaks it silently, which is exactly the shape the builder's
example declares legal.

### ✓ Case as DATA — legitimate, not in scope

`string::to-lowercase`/`to-uppercase` (verbs a caller asked for), `string/mod.rs`'s kebab↔pascal
converter (transforming case, never branching semantics on it), and `{}-MAX-REQUEST-BYTES` name
minting (generating a name).

## The failure shape this is an instance of

**Reading a SHAPE and inferring a KIND.** Three times in one session I called
`:wat::core::Record::def` *"a surface method"* because it looked like one. A surface method in this
substrate is `Type/name` — with a SLASH. And `Record::def` has not existed since
`wat-scripts/fixes/rename-record-def-to-defrecord.wat` retired it; it survives only in ~14 stale
comments, in that migration (which must keep the byte-exact old name), and in a probe I wrote that
same day. My four-name discrimination probe answered `None` for it for the WRONG REASON — not
because it is a method, but because **it does not exist** — while I claimed it proved the
method-vs-variant discrimination the whole codemod design rests on.

The live discrimination, measured after the correction:

```
wat::cache::HolographicLru::put     a live defn (wat/cache.wat:294), namespaced, NOT a variant
wat::program::PeerKind.thread       VARIANT of wat::program::PeerKind
wat::cache::Cache::GetRequest       a live defrecord (wat/cache.wat:171), NOT a variant
wat::cache::Cache::GetResponse.Ok   VARIANT of wat::cache::Cache::GetResponse
```

★★ **And it leaked into my own instruments.** The class-⑥ and class-⑦ censuses were written with
`[A-Z]` baked into their predicates. Measured against ground truth they missed nothing *in this
corpus* — but only by luck: `PeerKind::thread` and `PeerKind::process` are live lowercase-leaf
variants, and I had already found them in phase ① as the counterexample to a capitalised-leaf grep,
written that finding into the design, and then built the next predicate on the same assumption
anyway.

★★★ The instrument that has never been wrong in this campaign is the one that stopped encoding an
assumption and asked: over-generate freely, let `variant-parent-of` decide. Every case-shaped
predicate needed correcting; the 382-pair census was right the first time and was confirmed by an
independent derivation. `[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`

## Also stale, found by the same pull

~14 `.wat` files carry `Record::def` / `Record::of` prose for a form retired months ago. Prose only —
it cannot break a build, and it is what fed me the dead example. Same class as the `:wat::std::`
correction in `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 8.
