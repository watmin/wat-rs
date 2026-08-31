# DESIGN — STONE: the record family gets homes — ALL SEVEN

> **Builder, 2026-08-31:** *"record/struct family next"* → then, on the struct fork:
> *"what makes struct-field impure other than that we haven't declared it in the old world?"* and
> *"a function to say 'give me this field's value' is constant when applied to a constant input?"*
>
> ⛔ **REVISED 2026-08-31.** This DESIGN first drew FIVE and sent `struct-new`/`struct-field` to a
> ruling. **The fork was posed on a measurement I never took.** All seven are in scope.

## The seven

| verb | scheme? | @Purity | @Determinism |
|---|---|---|---|
| `Record/assoc` · `Record/same-data?` · `record->map` | YES | Pure | Deterministic |
| `to-record` · `variant` | no | Pure | Deterministic |
| `struct-new` · `struct-field` | no | **Pure** | **Deterministic** |

## ⛔ THE STRUCT FORK — SUPERSEDED, and every one of its three costs evaporated

The first draft said the pair could not be ruled: `Effectful` fails the census, `Pure` contradicts
`accessor_meta`, `Unreviewed` is a lie. **Two of those three were false.** Measured:

**1. `Pure` contradicts nothing.** `accessor_meta`'s FIRST GUARD (`src/rete/purity.rs`) is
`if !head.contains('/') { return None; }`. The verb is `:wat::core::struct-field` — **no slash.**
`accessor_meta` never speaks about this name. I compared two different verbs.

**2. `Effectful` was never the honest answer**, so `effectful_by_prefix` is untouched and there is
nothing to widen. ★ **The wall the seam said three families queue behind never applied to this one.**

**3. `struct-field` is not a struct verb.** `runtime.rs`, arc 293.R2.2, in the body itself:

> *"accept ANY `Value::Aggregate` … **the old `Nature::Struct` guard was a pre-unification
> artifact**; record + holon-record field accessors now use this same primitive."*

Arc 293 **deleted** its struct guard. It is the unified field-read every record, holon-record and
struct accessor calls. The name is a fossil, and the fork was drawn against the fossil.

### What actually made it "impure": nothing. It was undeclared.

`eval_struct_field`, read whole: evaluate the receiver → match `Value::Aggregate` → bounds-check the
index → `Ok(inner.fields[index].clone())`. No `Mutex`, no `RefCell`, no `borrow`, no
`apply_function`. **A pure indexed read.**

⚠ **Two refusals were wearing one message.** The fence prints the same sentence for both:

```
:u::Conn/fd                REFUSED — accessor_meta MEASURED it (a.nature.is_pure() == false)
:wat::core::struct-field   REFUSED — KNOWN_UNREVIEWED default-deny: NOBODY DECLARED IT
```

*"is not pure"* is the fence's phrasing for both *"I measured this and it isn't"* and *"I have no
idea what this is."* Only the second applied. `[[NOTE-a-nature-is-a-transport-fact-not-a-purity-verdict]]`

## ★★ THE EVIDENCE — a projection is constant, and the resource is guarded elsewhere

A `defstruct` holding a LIVE `Lru` handle plus a plain `i64`, read across two mutations:

```
plain field: read twice, equal?          true
handle len when FIRST read:              0
handle len via the SECOND read:          2
handle len via the FIRST read, now:      2     <- SAME OBJECT, both names
```

Same input, same answer, every time. What moved was *behind* the handle.

**And admitting it opens no hole — measured, not argued:**

```
(:wat::cache::Lru::len …)  in a where   => REFUSED "':rust::cache::Lru::len' is not pure"
(:wat::core::= h1 h2)      on handles   => REFUSED TypeMismatch, not a comparable pair
```

A handle in hand inside a fence is **inert**. Every verb that could *do* anything with it is refused
one step later by the recursive walk — which is where the effect lives. ⛔ The first draft worried
about a bypass **without ever checking whether there was anything on the other side of it.**

## THE ONE CONTRACT DECISION — pinned

**Purity is a property of the VERB — same input, same output — never of what the verb hands back.**
A pure function may return an impure *thing*; guarding the thing is the next op's job, and the
recursive walk already does it.

## The rulings — the rider measures `@Totality` per verb

All seven are `Pure ∧ Deterministic`; none applies caller code (verified). ⚠ **`@Totality` is
per-verb and measured**, never copied across the family — the collection readers proved why
(`assoc`/`conj` were `Partial` on inner helpers a container-gate reading never reaches).
★ `Record/assoc` is `assoc`'s sibling and likely shares that shape — **read the inner helpers.**
★ `struct-field` has a bounds-check on the field index and a `TypeMismatch` on a non-Aggregate
receiver — the rider decides what each means for this axis, citing the line.

## ★ THE MIXED PREDICTION — measured for all seven, uneven in both directions

```
Record/assoc · Record/same-data? · record->map    register_builtins: YES  ->  NO debt row
to-record · variant · struct-new · struct-field   register_builtins: NO   ->  a row each
```

**FROZEN_CHECKER_DEBT 64 → 68. KNOWN_UNREVIEWED 41 → 34.** Verified against
`register_builtins` (`src/check.rs:16569–21711`), not inherited from the prior draft.
⚠ If any of the four DOES carry a scheme, or any of the three does NOT, the measurement is wrong and
that is a finding — not a row to add or skip quietly.

★★ **And the corollary that cost two floor rounds on the collection readers:** a verb WITH a scheme
is one `doc_arg_ret_types_match_checker_scheme` verifies. `conj`'s `@arg` was narrower than its
scheme; `assoc`'s was more precise. **For the three with schemes, read the registered `TypeScheme`
FIRST and write `@arg`/`@ret` to match it**, then put the real meaning in prose.

## Out of scope = REJECTED (not deferred)

- **Re-ruling `accessor_meta`'s `pure: a.nature.is_pure()`.** Arc 293.W owns that wall. This stone
  neither changes it nor depends on it; the conflation is recorded in the NOTE with its probes so
  the arc that re-rules it inherits the measurement. ⚠ It is refusing legitimate pure reads **today**
  (`:u::Conn/fd` on an `i64` field), and that cost is named, not hidden.
- **The W7 HOFs and the stream forcers.** Genuinely different: they run code they did not write.
  Untouched by this ruling, and `effectful_by_prefix` is untouched with them.
- **The rest of `KNOWN_UNREVIEWED`** (41 today; 34 after this).
- **`:layer`**, untouched and un-guessed.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **all seven, `Pure ∧ Deterministic`, measured** | YES | YES | YES | YES | ✅ **ADMITTED** |
| the five; struct pair to a NOTE (the first draft) | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| all seven, struct pair `Effectful` | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| all seven, struct pair `Unreviewed` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| seven + re-rule `accessor_meta` in this stone | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |

- **the-first-draft Honest? NO** — it parked a verb as unruleable on a contradiction that does not
  exist (`accessor_meta` never keys on a slashless head) and a census that never applied. Parking a
  MEASURED verb is the same lie as `Unreviewed`, wearing a NOTE.
- **`Effectful` Simple? NO** — needs `:wat::core::` in `effectful_by_prefix`, which the W7 NOTE
  disqualified for making the guess vacuous across the largest namespace. **Honest? NO** — the body
  has no effect, and `(struct-field a-record :x)` never had one.
- **`Unreviewed` Honest? NO** — the bodies were read and the probes were run.
- **re-rule-here Simple? NO** — seven mechanical homings plus a change to arc 293's containment
  semantics in one stone; a red could not be attributed.

## Acceptance

| what | command | expected |
|---|---|---|
| the seven are registered | `lookup_entry` each | `Some` |
| each `@Totality` is its own | the seven declarations | measured per verb, cited |
| ★ the fence flips for the raw form | `struct-field` in a `where`, via `compile-all` | was REFUSED → **ADMITTED** |
| ★ the typed struct accessor is UNCHANGED | `:u::Conn/fd` in a `where` | still REFUSED (293.W's wall, untouched) |
| the ratchet | `KNOWN_UNREVIEWED` | 41 → **34** |
| the uneven prediction | `FROZEN_CHECKER_DEBT` | 64 → **68**, and only those four |
| ★ the doc gate passes FIRST TIME | `@arg`/`@ret` written from each registered scheme | green on the first floor |
| behaviour unchanged | records/structs built and read as today | as today |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5110/5110, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
