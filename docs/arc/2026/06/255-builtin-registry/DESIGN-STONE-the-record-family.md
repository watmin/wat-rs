# DESIGN — STONE: the record family gets homes (and the struct pair needs a RULING)

> **Builder, 2026-08-31:** *"record/struct family next"*
>
> ⚠ **Five of the seven. The struct pair is a ruling, not a stone** — measured in pre-flight.

## The split

| verb | scheme? | verdict |
|---|---|---|
| `Record/assoc` · `Record/same-data?` · `record->map` | YES | ✅ in scope |
| `to-record` · `variant` | no | ✅ in scope |
| **`struct-new` · `struct-field`** | no | ⛔ **NEEDS A RULING** — see below |

## ⛔ THE STRUCT PAIR — a fork the builder owns

`src/rete/purity.rs:940`, the substrate's own declaration:

> *"a Record/HolonRecord accessor is pure ∧ deterministic, **a Struct accessor is impure (a struct
> can hold a live resource, arc 293.W)**"*

★ **But the impurity is a claim about the TYPE, not the verb.** Measured: `eval_struct_field` and
`eval_struct_new` contain no `Mutex`, no `RefCell`, no `borrow`, no `apply_function` — they read a
field and build a value. The claim is that a *struct* may hold a live resource, so a struct **value**
is not a pure value.

**And the fork bites, because either answer costs something:**

- **`Effectful`** → ⛔ **fails the census.** `declared_purity_vs_effectful_by_prefix_census` asserts
  `Effectful ⇒ effectful_by_prefix`, and `:wat::core::` is **not** in that prefix list
  (`kernel · io · holon · eval- · load · config · stream · rete`). Widening it is the option the W7
  NOTE already disqualified — it makes the guess vacuous for the largest namespace in the language.
- **`Pure`** → two authorities disagree: the registry would say pure for a verb `accessor_meta`
  calls impure, and `intrinsic_meta` consults the registry FIRST — so the registry's answer would
  silently win.

⚠ **This is the same wall as W7, reached by a different road.** It is not "a struct verb is hard";
it is that `:wat::core::` cannot express `Effectful` today, and three families now queue behind that.

## The rulings for the five — the rider measures each

All five are expected `Pure ∧ Deterministic`; none applies caller code (verified). `@Totality` is
**per-verb and measured** — the last stone proved why: `assoc`/`conj` turned out `Partial` on inner
helpers (`hashmap_assoc_inner`'s unhashable-key check) that a container-gate reading never reaches.
⚠ `Record/assoc` is `assoc`'s sibling and likely shares that shape. **Read the inner helpers.**

## ★ THE MIXED PREDICTION — and the lesson from last stone

```
Record/assoc · Record/same-data? · record->map   scheme YES  ->  NO debt row
to-record · variant                              scheme no   ->  a debt row each
```

**FROZEN_CHECKER_DEBT 64 → 66**, rows for `to-record` and `variant` only.

★★ **And the corollary that cost two floor rounds last stone:** a verb WITH a scheme is one the doc
gate verifies. `doc_arg_ret_types_match_checker_scheme` went red twice — `conj`'s `@arg` was narrower
than its scheme, `assoc`'s was more precise than its scheme. **For the three with schemes, read the
registered `TypeScheme` FIRST and write `@arg`/`@ret` to match it**, then put the real meaning in the
prose. Having a scheme is what makes a verb checkable; the price of being checkable is being checked.

## Out of scope = REJECTED (not deferred)

- **`struct-new` · `struct-field`** — the fork above. A ruling, then a stone.
- **The rest of `KNOWN_UNREVIEWED`** (41 today; 36 after this).

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the five; the struct pair to a ruling** | YES | YES | YES | YES | ✅ **ADMITTED** |
| all seven, struct pair `Pure` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| all seven, struct pair `Effectful` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| all seven, struct pair `Unreviewed` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **struct-`Pure` Honest? NO** — it contradicts `accessor_meta`'s live-resource declaration, and the
  registry answers first, so the contradiction resolves silently in the registry's favour.
- **struct-`Effectful` Honest? NO** — it fails the census, and the only way to pass is widening
  `:wat::core::`, which the W7 NOTE disqualified for making the guess vacuous.
- **struct-`Unreviewed` Honest? NO** — the bodies were read. `Unreviewed` means *nobody looked*, and
  recording it about a measured verb is the lie the fourth variant exists to prevent. **The honest
  state is "measured, and the answer needs a ruling"** — which is a NOTE, not a declaration.

## Acceptance

| what | expected |
|---|---|
| the five are registered | `lookup_entry` `Some` for each |
| each `@Totality` measured | per-verb, cited; `Record/assoc`'s inner helpers read |
| the ratchet | `KNOWN_UNREVIEWED` 41 → **36** |
| the mixed prediction | `FROZEN_CHECKER_DEBT` 64 → **66**, `to-record`/`variant` only |
| ★ the doc gate passes FIRST TIME | `@arg`/`@ret` written from each registered scheme |
| floor | 5110/5110, 0 failed |
| clippy | 0 under `-D warnings --all-targets` |
