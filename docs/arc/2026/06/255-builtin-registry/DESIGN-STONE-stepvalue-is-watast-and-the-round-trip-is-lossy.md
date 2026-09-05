# DESIGN — STONE: `StepValue` faces `WatAST`, and the holon round trip is LOSSY TODAY

> **Builder, 2026-09-04:** *"what are the remaining 'holon to wat' conversions?.. are they
> acceptable?"* → *"brief and release the StepValue strike"*
>
> The inventory said 18 conversions: 3 legitimate, 2 hidden residue, **4 that exist only because
> the CEK stepper's own internal enum still speaks holon**. Measuring those four turned up a bug.

## ⛔ THIS IS NOT A TIDY-UP. `eval-step!` CORRUPTS TWO LITERAL KINDS.

Measured this session, against the current build:

```
(:wat::core::quote 1/2)                        → renders  1/2                     RationalLit
(:wat::eval-step! (quote 1/2))     terminal    → renders  "1/2"                   StringLit  ⛔
(:wat::core::quote 1234…7890N)                 → renders  1234…7890N              BigIntLit
(:wat::eval-step! (quote 1234…N))  terminal    → renders  "1234…7890N"            StringLit  ⛔
CONTROL  (:wat::eval-step! (quote 42))         → renders  42                      i64        ✅
```

**A rational goes in and a string comes out.** The cause is in `try_recognize_holon_value`
(`src/holon/ast.rs:928`), and its own comments name it without calling it a defect:

```rust
// Arc 300 stone B — SURPRISE: holon-rs has no native rational leaf; lower to its
//                   canonical rendered string.
WatAST::RationalLit(r, _) => Some(HolonAST::string(format!("{}/{}", r.numer(), r.denom()))),
// Arc 300 stone C1 — same SURPRISE as Rational immediately above.
WatAST::BigIntLit(n, _)   => Some(HolonAST::string(format!("{}N", n))),
```

★★★ **And today's stone made it worse in one specific way**: `StepResult::StepTerminal` now
declares `:wat::WatAST`, so the corrupted value is presented under an honest-looking type. Before,
a caller got a holon and might reasonably distrust it. Now the signature says `WatAST` and hands
back a `StringLit` where a `RationalLit` went in. **A lossy conversion behind an honest signature is
exactly what `STONE-eval-walk-faces-watast`'s own STOP-1 forbade — and we shipped one, one layer
down, because the loss was inside a function that stone never touched.**

## The mechanism — the holon is a BYPRODUCT, not the answer

`try_recognize_holon_value`'s stated job is a **predicate**: *"try to recognize a WatAST as a
holon-value shape… This is what lets `eval-step!` distinguish 'input was already a value'
(`AlreadyTerminal`) from 'this step reduced a redex' (`Terminal`)."*

The question is *"is this already a value?"* — a **bool**. It answers by **building a HolonAST**, and
the build is where the loss happens. The stepper then carries that lossy holon:

```rust
enum StepValue {
    Next(WatAST),               ← already WatAST
    Terminal(HolonAST),         ⛔
    AlreadyTerminal(HolonAST),  ⛔
}
```

and converts back at four sites (`runtime.rs:12203, 12297, 12303, 12578`). **The input WatAST was
right all along; it was destroyed and imperfectly rebuilt.**

## The shape

```
StepValue::Terminal / ::AlreadyTerminal  →  carry WatAST
try_recognize_holon_value                →  the recognition stays; what it RETURNS changes.
                                            The caller needs "is it a value?", not a holon.
the four conversion sites                →  DISAPPEAR
```

⚠ **`try_recognize_holon_value` has other callers.** It is `pub(crate)` in `src/holon/ast.rs` and
`runtime.rs:12322` is not necessarily its only user. Whatever shape it takes must keep working for
callers that genuinely want a holon — this stone must not break the VSA path to fix the eval path.

★ **rustc is the census here, unlike this morning.** `StepValue` is a Rust enum, so changing its
field types produces real compiler errors at every site. That is the opposite of `types.rs`, where
wat types are declared as data and rustc was blind.

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **carry WatAST through the stepper** | YES | YES | YES | YES |

- **Obvious? YES** — `Next` already does it; the input is a `WatAST`; the output is declared
  `:wat::WatAST`. Only the middle disagrees.
- **Simple? YES** — one enum, four call sites that get *deleted*, and rustc enumerates the rest.
- **Honest? YES**, and it is the whole stone: today `eval-step!` returns a value it did not receive,
  under a type that says otherwise.
- **Good UX? YES** — `(eval-step! (quote 1/2))` gives back `1/2`.

## Scope

**In:** `StepValue`'s two fields · whatever `try_recognize_holon_value` must become to serve a
value-predicate without a lossy build · the four conversion sites · a probe proving `1/2` and a
bigint survive · every site rustc names.

**Out, affirmatively:** the VSA surface (`BundleResult`, `Holons`, `Reckoner/new-discrete`,
`wat/holon*.wat`, `assert-coincident`, `cache.wat`'s `hologram-svc`) · `:wat::holon::to-wat` and the
`Value::holon__HolonAST` coercion arm at `runtime.rs:7047`, both legitimate · `reflect/verbs.rs`'s
two conversions — sized out, **not unrelated**: see below.

## ★ WHY THESE KEEP APPEARING — the builder, 2026-09-04

> *"the reflect tooling... they are in our targets for the registry.... **the registry is forcing the
> discovery of bad practices**... reflection should never have used holon... but.. at the early days
> of wat... we did not have a mature wat-ast..... holon-ast was our crutch.... we built holon-ast
> (like... 6 months ago or longer...) to hold the same data as edn ... **holon-ast is hypervector of
> data ... edn is a wire format of data**.... the data both have can be represented in either."*

That names the category error underneath every site this session found. `HolonAST` is a **VSA
encoding** of data; EDN is a **wire encoding** of the same data. Neither is a syntax tree. Using one
as the substrate's AST was a **crutch taken while `WatAST` was immature**, and it has been quietly
lowering literals into whatever the hypervector representation could hold ever since — which is
exactly why `RationalLit` and `BigIntLit` arrive as strings: holon-rs has no leaf for them **because
it was never meant to carry syntax.**

⭐ **And the reason the campaign keeps finding them is structural, not luck.** The registry's
demand — every name answerable, every property declared and gated — forces each surface to state
what it actually is. A crutch survives indefinitely while nothing asks it a question. `reflect/`'s
two conversions are therefore **registry work, deferred for size only**, and belong on the campaign's
board rather than in a list of unrelated debris.
