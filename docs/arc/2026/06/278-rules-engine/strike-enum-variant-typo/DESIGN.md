# DESIGN-STONE — rete's answer must agree with core's for the same input

> **Origin.** Vigilia Class D1, found by `solvere` **by reading**, and recorded in the work list as
> *"mechanism verified by reading; NOT yet driven."* Driven 2026-08-31, and driving found a
> **second arm the reading did not name.**

## Why

`validate/typing.rs`'s `keyword_constant_segment` types a bare keyword constant by **PREFIX ONLY**:
`rsplit_once("::")`, then "is that path a `TypeDef::Enum`" → `"enum"`. **It never checks the
variant exists.** The runtime resolves the same keyword through `expr_ir::keyword_value` →
`sym.unit_variant`, an **exact lookup over UNIT variants only**. The two disagree, `enum::=`
compares an Enum against a plain keyword, and the answer is always false.

**The rule compiles, fires, and matches nothing — with no diagnostic.**

`matcher.rs:130`'s `enum_variant_ctor` already exists to be the one resolution. Its own doc:
*"ONE COPY. This resolution … was hand-written at THREE independent sites."* `typing.rs` is the
**fourth**, and unlike the other three it disagrees with the runtime.

## ⛔ THE CONTRACT IS AGREEMENT WITH CORE, AND CORE REFUSES BOTH ARMS

This is not "rete has a hole." **The same expression is refused by core and silently accepted by
rete.** Driven at HEAD `c75b0152c`:

| input | core | rete |
|---|---|---|
| `(= :evt::G::Hii :evt::G::Hi)` — misspelled | **CheckErrors** — *"parameter #2 expects `:wat::core::keyword`; got `:evt::G`"* | — |
| `(enum::= :grade :evt::G::Hii)` in a rule | — | **compiles, fires, `0`, exit 0** |
| `(= :tg::P::Hi (:tg::P::Hi 7))` — bare tagged | **CheckErrors** — *"expects `[:wat::core::i64 :-> :tg::P]`"* | — |
| `(enum::= :grade :tg::P::Hi)` in a rule | — | **compiles, fires, `0`, exit 0** |

Core types the typo **honestly as a keyword** — confirmed directly: `(println :evt::G::Hii)` prints
`:d1.G/Hii`, a plain keyword — sees keyword-vs-enum, and refuses. **Rete's prefix shortcut makes it
LESS correct than core for the identical input.**

The arc's own ruling, recorded in the breadcrumb: *"When adding a rete form, drive CORE's answer for
the same input before deciding rete's — agreement is the contract, and 'it didn't match' is the
easiest wrong answer to ship."* Rete is shipping exactly that wrong answer.

## ⚠ THE ARM TABLE — TWO ARMS, and the obvious fix closes only one

| arm | `enum_variant_ctor` says | routing through it alone |
|---|---|---|
| **1 — misspelled** (`:G::Hii`, enum has `Hi`/`Lo`) | `None` | **FIXED** — types as `keyword`, mismatch fires |
| **2 — bare TAGGED** (`:P::Hi` where `Hi` takes a payload) | `Some(arity = 1)` | **STILL BROKEN** — types as `enum`, runtime still yields a keyword |

`enum_variant_ctor` resolves Unit **and** Tagged; `sym.unit_variant` is **unit-only**. So the typing
must additionally require **arity == 0**: a bare keyword is an `enum` value only if it names a
UNIT variant that exists. A tagged variant has no bare value form — `(:P::Hi 7)` is the only way to
write one — which is why core refuses it too.

**Both arms were driven before this stone was written.** Arm 2 is the one `solvere`'s reading did
not name, and it is exactly the half a rider would ship as "done".

## The algorithm

Replace `keyword_constant_segment`'s hand-rolled resolution with
`matcher::enum_variant_ctor(types, k)`, and return `"enum"` **only when it resolves AND its arity
is 0**. Everything else falls to `"keyword"`, where the existing `UnknownField` /
`ConstraintTypeMismatch` machinery already produces a located diagnostic — the same shape core
gives.

## ★ THE ONE CONTRACT DECISION

**The fix is measured against CORE's answer, not against "it now refuses."**

A refusal that lands with a different reason, at a different phase, or on a different set of inputs
than core's is not agreement — it is a second divergence replacing the first. For each of the two
arms the report must show **what core does** and **what rete now does**, side by side. If they
cannot be made to agree, that is a finding to surface, not a gap to paper over with any refusal
that happens to be red.

## Blast radius

`src/rete/validate/typing.rs` — one function — plus the probe pair into `tests/rete/`. **No new
types, no wire change.** ⚠ `keyword_constant_segment` has two callers (`typing.rs:51` and the
`check_constraint_head` path at `:319`); confirm both before assuming the radius holds.

## Out of scope — AFFIRMATIVELY CUT

- **The residual coarsening `solvere` named**: segment strings collapse every enum to `"enum"`, so
  `(enum::= :v :F::A)` against a field typed `:E` also types clean. That wants the enum PATH
  carried in `OperandType::Resolved` rather than a `&'static str`, which is a type change and its
  own strike. **Named so it is not mistaken for done.**
- **The other three `enum_variant_ctor` callers.** They already route correctly.
