# NOTE — a DECLARED TYPE can be coarser than the shape a body actually handles

> Found 2026-08-31 by the wave-2 rider, re-verified by the orchestrator. **No row, nothing drawn.**
> This records a recurring `@Totality` mechanism so later waves inherit it.

## The shape

A verb declares a parameter type. Its body then `match`es on a NARROWER shape inside that type and
**raises** on the rest. The call is well-typed; the raise is reachable; the verb is `Partial`.

```
:wat::core::type-equal?             declares :wat::WatAST, body needs a node that PARSES AS A TYPE
:wat::core::type-params-used-in     declares :wat::WatAST, body needs a Symbol or Keyword
:wat::rete::vocabulary-admitted?    declares :wat::WatAST, body needs a Keyword specifically
```

Three of wave 2's seventeen, all overturning a hand verdict of `total: true`. `type-equal?`'s own
doc says it outright:

> *"★ Contract: given a node that does not parse as a type at all, **this RAISES rather than
> returning `false`**."*

Verified empirically against the pre-stone binary: each passes `--check` (exit 0) and raises
`TypeMismatch` at run.

## ⛔ Why the obvious tell does NOT find these

Two plausible screens both miss it:

- **"Is it on `FROZEN_CHECKER_DEBT_LEDGER`?"** `type-equal?` and `type-params-used-in` are — no
  `TypeScheme` at all — which *worsens* their exposure. But **`vocabulary-admitted?` is checked
  normally and still overturned.** A registered scheme does not help when the scheme's own type is
  the coarse one.
- **"Is the arity variadic?"** That was wave 1's mechanism (`:wat::string::concat`, arity 0 admitted
  by `check.rs:14944`). It is a *different* mechanism and finds none of these three.

★ The only reliable instrument is the one the contract already mandates: **read the body and ask
what shape it actually accepts, then compare that against the DECLARED type.** Where the declared
type is broader, the gap is a domain hole.

## The generalization — a sibling of a class this repo already names

`holon/CLAUDE.md` records a recurring class: *"when a generic form misbehaves, suspect a string
comparison with one side normalized and the other not."* **This is the same failure one level up the
stack** — not a string compared against a normalized string, but a **TYPE compared against a shape**,
where the type is the coarser of the two. `:wat::WatAST` is the repeat offender because it is a sum
over every node kind, and almost no body handles every node kind.

⚠ **Predict it for the rest of the campaign:** any verb declaring `:wat::WatAST` (or another wide sum
type) and immediately `match`ing on one variant is a `Partial` candidate, whatever a hand verdict or
a prior comment asserts about it.
