# NOTE — a defservice may `:satisfies` a surface and not implement all of it

**Found 2026-09-01**, while grading item (c) stone A. Not chased further; recorded so it is not
re-derived.

## The observation

`Span` gained a fifth op (`flush`). `tests/services/probe_arc278_span_surface.wat`'s toy satisfier
declares `:satisfies :wat::telemetry::Span` and implements **four** arms — `incr`/`timed`/`log`/`close`,
no `flush`. **It type-checks, freezes, and the floor is green.**

The mechanism is visible in the macro: `serve-op-arms` (`wat/service.wat:1433`) is a `foldl` over
`:impls`. A surface op with no impl simply produces no arm. Nothing compares `:impls` against the
surface's `:features`.

Grep for the check, this session: `missing.*impl` / `impls.*complete` / `does not implement` across
`src/check.rs` and `src/types.rs` — **zero hits.**

## Why it matters, and why it is not obviously severe

`:satisfies` reads as a promise that the satisfier answers the whole surface. A caller holding a
`(Peer :- [Span::Op Span::Reply])` has no way to know which satisfier is behind it, and the surface
is the contract it was handed. A partial satisfier makes that contract locally false.

What it is NOT: silent success. A client that calls the unimplemented op does not get a wrong answer
— `retag-op` canonicalizes the wire op into the SERVICE's own `Op`, and an op with no arm has no
variant there, so the call fails rather than misbehaves. **The hole is that it fails at RUNTIME, on a
call, rather than at the `defservice` that made the promise.**

## Why this is the arc's own shape

This is the sibling of the `:messages` completeness guard that excursus 001 stone 5 widened — that
one asks "does every type this surface's ops mention cross a fork?", and it exists because a type
absent from `:messages` fails at a fork rather than at the declaration. Same failure geometry, one
clause over: `:features` versus `:impls`.

★ And it has the property that makes this class expensive: **a green floor proves nothing about it.**
The surface probe — whose entire job is "every declared op is reachable and replies" — went on
passing while driving four of five ops, because it only ever checked what it always checked. It was
the op COUNT in the test's own name (`..._all_four_ops_reply`) that gave it away, and only because a
human read it.

## What was done here

The toy satisfier gained its `flush` arm and the probe drives it, so that gate means what its name
says again. The test is renamed off the count (`..._every_declared_op_replies`) — a name carrying a
number goes stale silently the moment the surface grows.

**The guard itself is NOT built.** It would be: at `defservice` registration, every op in the
`:satisfies` surface's `:features` must have an arm in `:impls`, or a located error naming the
missing ops. That is a checker stone, undrawn.
