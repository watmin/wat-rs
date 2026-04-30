# Resolution: Arc 109 (kill-std)

**Status:** Both designers APPROVED, 2026-04-29.

| Designer | Verdict | Pushback |
|---|---|---|
| Rich Hickey | APPROVED | Tighten the `:wat::poly::*` admission rule so the tier doesn't rot into a new `std`. |
| Brian Beckman | APPROVED | None. |

## Where they agreed

- **The three-tier partition is honest** because it tracks dispatch
  shape, not topic. Three tiers, three categorical signatures:
  mono arrows (`core`), runtime-tagged case analysis (`poly`), HOFs
  functorial in the collection (`list`). Each op answers a
  different question; no overlap.
- **Killing `:wat::std::*` is right.** "std" was a place — the
  name told you nothing about what was inside. The replacement
  tiers (`list`, `math`, `stat`) are concerns. Place-naming was
  the rot; concern-naming is honest.
- **`:wat::poly::*` belongs.** Beckman frames it as the universal
  arrow out of the coproduct `i64 + f64`; the mono `core::i64::+`
  and `core::f64::+` are the injections-into-action; the
  polymorphic `+` is the case analysis. Categorically clean.
  Hickey: "naming the tier where 'the same operation across many
  types' lives is more honest than scattering the dispatch
  through `core`."
- **`Type/verb` (slash) shape for `Option/expect`, `Result/try`
  etc. is right.** Both call out that arc 108's interim
  `option::expect` (lowercase, double-colon) hid the dispatch by
  making it look like a sub-namespace. The slash signals
  method-on-type explicitly.
- **`Option/try` closes a missing natural transformation.** Arc
  108 had `Result/try` (Result-only `try`) but not `Option/try`.
  Arc 109's addition makes the `try` family symmetric across
  both sum types — same shape (left-strict monadic propagation
  with non-local return), parameterized by which sum type is
  unwrapped.
- **Algebra at `:wat::holon::*` is untouched.** The reorganization
  is structural; the algebraic kernel (`bind` / `bundle` /
  `cosine` / etc.) is independent and stays put. Both confirmed.
- **Bare vs FQDN tracks the syntax/semantics boundary.** `_` and
  `->` stay bare because they are grammar, not values. FQDN'ing
  them would be a category error.
- **Verbosity is solved at the right layer.** The substrate has
  no namespace mechanism; the FQDN IS the name. Editors complete;
  humans don't type. The verbosity worry was the wrong worry.

## Where they pushed (Hickey)

> "What I'd watch: don't let `poly` grow into a place where
> everything-someone-found-handy gets dropped. The tier earns its
> keep ONLY for ops that genuinely dispatch on operand type. The
> moment `poly::frob` shows up because it 'felt convenient' with
> no type-driven story, the tier has rotted. The doctrine in the
> doc — 'if it exists ONLY because it makes the surface less
> verbose, this is its home' — is dangerously close to that
> failure mode. Tighten it: 'if it dispatches on operand type to
> give one name across many types, this is its home.' Otherwise
> you've reinvented `std`."

## Resolution of the pushback

The INVENTORY's Section G (the three-tier table) has been updated.
The `:wat::poly::*` admission rule was rewritten (verbatim):

> **Admission rule (Hickey, 2026-04-29 review):** an op earns
> `:wat::poly::*` ONLY if it dispatches on operand type to give
> one name across many types. "It feels convenient" is NOT enough
> — that's the rule that turns this tier into a new `std`. Every
> member must answer: which type tag selects which mono-typed
> primitive? If the op has no type-driven story, it does not
> belong here.

The tier description itself was sharpened from "polymorphic
conveniences" to "**runtime-polymorphic dispatchers** — one name;
runtime selects the implementation based on operand type."

The fifteen ops currently slated for `:wat::poly::*` (numeric
operators, comparisons, polymorphic `empty?` / `length` /
`contains?` / `get` / `show`) all pass the tightened rule —
each has a type-driven story. The rule is the gate for future
admissions.

## Open follow-ups (none blocking)

None. Both verdicts are APPROVED with one rule-tightening
pushback that is now closed in the inventory.

## Cross-references

- `INVENTORY.md` — the proposal under review (now reflects
  Hickey's pushback).
- `review-hickey.md` — Rich Hickey's full review.
- `review-beckman.md` — Brian Beckman's full review.
- `feedback_fqdn_is_the_namespace.md` (memory) — the doctrine
  this proposal implements.
