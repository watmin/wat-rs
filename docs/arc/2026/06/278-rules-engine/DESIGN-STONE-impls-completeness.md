# DESIGN STONE — the `:impls` completeness guard

**Commissioned 2026-09-01.** Builds the guard recorded in
`NOTE-impls-completeness-is-unenforced.md`, found while grading item (c) stone A.

## The defect

A `defservice` may declare `:satisfies <Surface>` and implement only part of it. **This compiles,
freezes, and the floor stays green.** `serve-op-arms` (`wat/service.wat:1433`) is a `foldl` over
`:impls`: a surface op with no impl simply produces no arm, and nothing compares the two.

The verified instance: `tests/services/probe_arc278_span_surface.wat`'s toy satisfier declared
`:satisfies :wat::telemetry::Span` and implemented four of five arms after `flush` joined the
surface. It passed. The test's own name — `..._all_four_ops_reply` — was the only tell, and only
because a human read it.

## Why it matters

`:satisfies` reads as a promise to answer the whole surface. A caller holding a
`(Peer :- [S::Op S::Reply])` cannot know which satisfier is behind it; the surface is the contract it
was handed.

It is not silent success — `retag-op` canonicalises the wire op into the service's own `Op`, which
has no variant for an unimplemented op, so the call fails rather than misbehaves. **The hole is that
it fails at RUNTIME, on a call, instead of at the `defservice` that made the promise.**

Same geometry as excursus 001 stone 5's `:messages` completeness guard — that one asks whether every
type a surface's ops mention crosses a fork, and exists because a type absent from `:messages` fails
*at the fork* rather than at the declaration. One clause over.

## The rule

> Every op in the `:satisfies` surface's `:features` must have an arm in `:impls`.

**One-directional, and this is the subtlety that matters.** The converse is false: `:impls` may carry
arms the surface does not declare — the reactor-internal ops, leading dash, `-flush-logs` /
`-flush-metrics` / `-tick`. Those are deliberately not on the surface (no client can name them) and a
symmetric "every arm must be a feature" check would reject every self-scheduling service in the tree.

The guard requires **`features ⊆ impls`**, never equality.

## ⚠ I do not have a trustworthy census, and I am not supplying a fake one

I attempted one and it was noise: it reported `:wat::telemetry::journal` missing `close`/`flush`/
`incr`/`log`/`timed` — which are **Span** ops, not Journal's — and listed `:wat::core::let` as a
missing op, having matched `let` bindings inside impl bodies. A regex over `.wat` text cannot reliably
pair a surface's `:features` with a service's `:impls`.

**So the census is the guard's own output, not an input to this brief.** Build it, run `--check`
across the corpus, and report what it names. That number is a finding, and it decides whether the
stone ships as drawn:

- **only the deliberate red probe** → ship.
- **a handful of real partial satisfiers** → they are the finding; report before fixing.
- **broad rejection** → the rule is wrong (most likely: internal ops leaking into the comparison,
  or a parametric surface whose features resolve differently per satisfier). STOP and report.

The one thing I will not do is hand over a number I could not produce. Every specification error in
this campaign came from a row written out of memory or generalisation instead of a read.

## Fires-on-nothing is the live risk

The single verified instance is already fixed. So a guard that never fires would pass every row that
does not deliberately construct a partial satisfier — which means the **red probe is not optional
colour, it is the only proof the guard works at all.**

It goes in `probes/`, never `wat-scripts/`: that tree's loader gate type-checks every file, so a
must-be-rejected file there turns the floor red for as long as the guard works. That was excursus
002 stone 1's specification error and it is not being repeated.

## What the error must name

The service, the surface, and **every** missing op — not the first. A guard that names one op per
run turns a five-op gap into five edit-compile cycles.

## Out of scope = REJECTED

- Requiring `:impls ⊆ :features` (would reject every internal op).
- Any change to `serve-op-arms` or the dispatch. This is a declaration-time check; the runtime
  behaviour of a complete service does not change.
- Arity or type checking of an arm against its feature. A separate concern with its own evidence.
