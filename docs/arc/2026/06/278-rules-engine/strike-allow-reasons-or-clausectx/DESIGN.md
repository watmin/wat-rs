# DESIGN — eleven `too_many_arguments` allows, and the codebase already wrote the answer

**Status:** drawn 2026-09-08. Resolves vigilia rows **`X1`** and **`X2`** (excusare, both L1).

## Why

`excusare` found five bare `#[allow(clippy::too_many_arguments)]` in `src/rete/` and struck them:
`X1` for `alpha.rs:338` (*"least defensible of the six"*), `X2` for four more, with the argument that
lands the whole strike — **"each has a doc comment above describing what the fn DOES; a comment's
presence is not a reason's presence."**

## Re-derived, and both halves move

**There are ELEVEN, not five** (`grep -rn 'allow(clippy::too_many_arguments)' src/rete/`, excluding
one doc-comment mention). The rows cite where the ward noticed them, not where they live —
`[[a-finding-names-one-site-enumerate-the-rest]]`. Tree-wide there are 22; **this strike scopes to
`src/rete/` only.**

**And only ONE is genuinely bare:** `kernel/fire/pass/alpha.rs:338`, whose preceding line is
`#[inline(never)]` — an attribute, not a comment. The other ten carry prose, which is exactly why
`X2`'s distinction is the load-bearing one. Read against *argument count* specifically:

| site | the line above | answers arg-count? |
|---|---|---|
| `pass/alpha.rs:338` | `#[inline(never)]` | ⛔ **bare** |
| `pass/hash_join.rs:427` | *"…than a thirteen-parameter explosion. See `AlphaNews::of`."* | ✅ **yes, explicitly** |
| `pass/hash_join.rs:41`, `pass/join_after_filter.rs:31` | *"already the fire pass's working set rather than an accident…"* | likely — verify |
| `compiled_cond.rs:246` | *"A builder would be ceremony for a constructor…"* | likely — verify |
| `fire/mod.rs:2128` | *"Exists/Not Leaf: probe the token's bucket…"* | ⛔ describes the fn |
| `pass/filter_after_join.rs:17` | *"Drain the frontier pass 3.6 produced…"* | ⛔ describes the fn |
| `pass/round_census.rs:25` | *"`round_no` and advances it."* | ⛔ describes the fn |
| `fire/mod.rs:469`, `validate/typing.rs:742` | fragments | verify |

## ⭐⭐ The codebase already contains the argument AND the worked cure

`src/rete/validate/mod.rs:388-393`, on `ClauseCtx`:

> *"Everything a clause check needs about the CONDITION it sits in — invariant across the clause
> walk, so it travels as one value rather than seven positional arguments. … the alternative here
> was an `#[allow(clippy::too_many_arguments)]`, **which silences the signal instead of answering
> it**."*

**So this is not a strike inventing a policy.** The policy is written, the counter-example is built,
and eleven sites predate or ignore it. `hash_join.rs:427` shows the other honest answer — keep the
allow and *say why*, naming the alternative you rejected.

## What this delivers

Every one of the eleven ends in one of exactly two states:

1. **The allow carries a reason that addresses ARGUMENT COUNT** — naming what travels together and
   why a struct would be worse. `hash_join.rs:427` is the model.
2. **The allow is gone**, because the arguments became a context struct, `ClauseCtx`-style.

## The one contract decision, pinned

⛔ **DEFAULT TO (1), NOT (2).** This is a *diagnostic* strike, not a refactor: eleven fire-path
signature rewrites in one pass would be a large, silent behaviour risk on the hottest code in the
subsystem, and the vigilia's own evidence is that the sites are mostly *unexplained*, not
*unjustified*. **Take (2) only where the arguments visibly already travel as a unit** and the
executor can say so — and if that is nowhere, that is a fine outcome.

## Out of scope = rejected

- **The 11 sites outside `src/rete/`.** Same class, different subsystem; this vigilia is rete-scoped.
- **Widening to other `#[allow]` lints.** `excusare` weighed those separately.
- **Changing any signature on a hot path to satisfy the lint alone.** See the pinned decision.
