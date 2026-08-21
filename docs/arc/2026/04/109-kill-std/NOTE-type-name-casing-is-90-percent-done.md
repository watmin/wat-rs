# NOTE (arc 109) — type-name casing: the convention is ALREADY ~90% held. Do NOT bundle it with the bracket cut.

**Filed 2026-08-20. MEASURED, with a control.** Opened by the builder while the bracket-annihilation
stone was being drawn: *"we haven't been consistent on String vs string… i think we'll mimic what
rust does…. i64 f64 String HashMap HashSet … then we need Keyword and Symbol …. hrm….. do we need to
solve this naming problem now?"*

## The answer: NO, and the reason is a doctrine already on disk

**It does not block the bracket cut.** The bracket cut changes the SHAPE of a parametric's type-arg
group; casing changes the NAME of the head. Orthogonal — the codemod's discriminator (`<` preceded by
alphanumeric/`_`/`'`) never reads the name.

Some sites would be touched by both passes. **That cost is zero.** 255's own realization:

> ***IVDICIVM SEMEL, MACHINA SAEPE*** — *"a plan is NOT to be scored by its number of passes — that
> quantity carries no cost information at all — but by how many times it spends JUDGMENT."*

Both passes are expressible as rules, so both are machine work. Bundling them is what costs: a rider
holding two rewrite rules at once over 3,236 sites is where mistakes come from, and a red floor could
not say which rule caused it.

## The measurement — it is far better than it looks

Bare type names in `wat/` + `tests/**/*.wat` (⚠ counted with `::`-suffixed NAMESPACE uses excluded —
a first pass that did not exclude them reported `string` 810 and `i64` 3606, both contaminated by
`string::length` / `i64::+`):

```
lowercase   i64 3037 · bool 802 · keyword 483 · nil 433 · f64 279 · u8 · char
Uppercase   String 1400 · Vector 1143 · PersistentVector 1009 · PersistentMap 430 ·
            Option 380 · HashMap 351 · Tuple 142 · HashSet 106 · Result 90 · List 62 · Bytes 7
```

**That IS the Rust convention already** — lowercase primitives, Uppercase containers and aggregates.

★ **There is no `String`/`string` duplicate.** `:wat::core::string` as a bare type: **0 occurrences.**
All 810 hits are the `string::` OP NAMESPACE (`string::length`, `string::trim`). The builder's worry
was well-founded in shape and does not exist in fact.

## What is actually left

Two names, and one open question:

| name | sites | note |
|---|---|---|
| `keyword` → `Keyword` | 483 | a wat value-kind, not a Rust primitive |
| `symbol` → `Symbol` | ~89 | ⚠ COUNT UNVERIFIED — the census that produced it was the contaminated one; re-measure before scoping |
| `nil` | 433 | **open** — Rust has no `nil`. Lowercase may be correct as a primitive-ish, or it becomes `Nil`. Not decided. |

## Control — capital `Keyword` is NOT a type today, and the checker is right about that

```
:wat::core::keyword       check exit 0   ← the real type
:wat::core::Keyword       check exit 1
:wat::core::TotallyBogus  check exit 1   ← CONTROL: unknown names ARE rejected
:wat::core::Symbol        check exit 1
:wat::core::symbol        check exit 1   ← neither casing is a type
```

So there is **no uninhabited-type defect** — `Keyword` is rejected exactly like any unregistered name.
The rename is a straight rename, not a reconciliation of two live types.

⚠ **One thing measured but NOT run down:** `:wat::core::Keyword` was *accepted* as a HashMap type-ARG
(it produced `expects :wat::core::Keyword; got :wat::core::keyword` rather than an unknown-type error)
while being *rejected* as a param type. That suggests type ARGS may not be validated against the type
registry the way param types are. **Not investigated. Do not act on this sentence — measure it.**

## Sequencing

After the bracket cut's ③ and the Fn-bracket stone. Same reason both of those are separated from each
other: one structural churn at a time, so a red floor names its own cause.
