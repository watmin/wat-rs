# FINDING — the message alias is two jobs; one duplicates a law, one is load-bearing

**Measured 2026-09-23 by the orchestrator** (read-only — a 255.15 executor was working in `src/` at the
time; no build, no probe run). Question from the one-way ruling: can `<S>::<op>/Request` be retired so
each message type has **one** name, `<S>::<OpPascal>Request`?

## Why the alias exists — a pipeline-ordering workaround

`src/types.rs` (~4132-4170), Arc 278 surface-minted op alias:

> *"`wat/service.wat` names these aliases instead of guessing a message's type name by concatenation —
> its only prior channel, since `expand_all` runs before `register_types` and the registry is empty at
> expand time."*

`defservice` must name each op's request/response type, but it expands before any type is registered.
Rust mints `<Surface>::<op>/Request` (a name the macro can compute from the op name alone), with
`type_params: surf.type_params` and target *"the request/response type EXACTLY as `:features`
declared it"*. The macro applies it to `proto-args` (the surface's own params).

## Job 1 — the NAME. ⛔ Duplicated by a law: two paths to one thing.

`src/types.rs` (~3578, ~3647), **Arc 278 #74 / #74b — `<Op>Response` and `<Op>Request` are LAW**
(builder, 2026-08-05: *"convention is law — enforce it"*), checker-forced as
`<surface-base>::<OpPascal>Request`:

> *"so the wat macros can go back to splicing a literal ctor keyword, guaranteed correct by
> construction."*

The alias and the law solve **the same problem** — naming a message without the registry. Under the
ruling *exactly one way to do things*, that duplication is itself a defect.

## Job 2 — the ARGUMENT MAPPING. ⭐ Load-bearing. The law does not cover it.

The law compares the **base name only** (*"`(GetResponse :- [K V])` CONFORMS: type args are stripped
from both sides"*). But a message's type arguments are **not** a uniform function of the surface's:

| surface | message | type args |
|---|---|---|
| `wat/cache.wat:177` `Cache :- [K V]` | `PutRequest` | `[K V]` — all |
| `wat/cache.wat:177` `Cache :- [K V]` | `GetResponse` | **`[V]` — a subset** |
| `wat-tests/service-parametric-bare-messages.wat:31` `BareBox :- [T]` | `PutRequest`, `PutResponse` | **none — bare** |
| `wat-tests/service-parametric-two-params.wat:36` `Pair :- [K V]` | `PutRequest`, `PutResponse` | **none — bare** |

The macro cannot know `GetResponse` takes `[V]` rather than `[K V]` at expand time. **The alias is where
that mapping lives.** Retiring it without replacing that is a regression, not a cleanup.

## What one name would take — three routes, not yet weighed

1. **Fix the root: register surfaces before `defservice` expands**, so the macro reads the message's real
   type (with its args) by the law's name. `extirpare`: *never construct the situation that needs the
   patch* — the alias is a patch for a pipeline order. ⚠ This is a deliberate pass-order change, and
   255.12's `src/freeze/pass_order.rs` pins that order; its own instruction is to re-read every
   post-step-7 disposition before re-ordering.
2. **Make the mapping uniform by law**: every message takes exactly the surface's params, in order. Then
   `(<OpPascal>Request :- <surface params>)` is deterministic and the alias is redundant. ⛔ Costs phantom
   type parameters: `GetResponse :- [K V]` with `K` unused, bare messages forced to `:- [T]`.
3. **Keep the alias as the only name the macro uses** and demote it to internal plumbing. ⛔ It is still a
   registered, user-visible second name for one type — the thing the ruling forbids.

**Not measured:** how many surfaces in the whole corpus have a non-identity mapping (only four were read).
