# Arc 268 (STUB) — audit all prior arcs for unbuilt / deferred tooling

> **Status: STUB — a banked audit task, 2026-06-14.** Not blocking anything. Build when we want the map
> (or when a leg keeps tripping over deferred slots, as the host-parity leg did twice in one session).

## Why

A recurring substrate pattern: an arc **builds a mechanism, names a user-facing slot over it, and
deliberately defers the slot "until a caller needs it."** The slot sits pre-described-but-unbuilt
until the first real consumer surfaces — then it's a thin block-and-build, not a fresh design.

The host-parity leg hit this **twice in one session**, both predicted by their arcs' own deferral notes:
- **arc 267 — parametric protocol bounds.** arc 232 built the protocol/`extend-type` mechanism and
  scoped out parametric bounds: *"Parametric protocols — OUT of v1 unless a strike proves them
  load-bearing … if/when [a caller] does, a NEW arc opens"* (232 DESIGN.md:99 / INSCRIPTION.md:41).
  The host seam surfaced it → built (`0a649ffd`).
- **`derive` verb** (arc-237 follow-on). arc 237 S-A built the `typesub` hierarchy engine
  (`register_subtype`/`is_subtype`/`subtype?`/roots) and deferred the user-facing verb:
  *"A user-facing derive verb ships only when a caller needs it"* (DESIGN-STONE-S-A:178). The
  `:Spawned` handle marker surfaced it → built (`2caf01f6`).

Pairs [[feedback_deferred_dep_becomes_necessary_block_and_build]]. We keep rediscovering these the
hard way (reach for the slot, find it unbuilt). A catalog makes them visible up front.

## The task

Sweep **all** prior arc docs (`docs/arc/**`) for deferred/unbuilt-slot language and produce a catalog:
`(arc · the slot · the trigger condition it named · where the mechanism already lives · status)`.

Grep idioms to seed the sweep (each marks a likely deferred slot):
- "ships only when", "until a caller", "when a caller needs it", "if/when … a NEW arc opens"
- "OUT of v1 unless", "out of scope unless a strike proves", "deferred", "banked"
- "perpetually awaiting", "awaiting its key/definition", "forcing function", "`:remote`"
- "not built", "deliberately absent", "minimal-form … later", "FORWARD-MAP", "ships when we need it"

## The KEY distinction (don't conflate two opposite kinds)

1. **Latent-ready slots** — mechanism built, slot deferred *until a caller needs it*. BUILD when a
   caller surfaces (derive, parametric bounds). These are the audit's payoff: a ready-to-pull list.
2. **Forcing-function / deliberately-never-built slots** — left unbuilt *on purpose* to keep the
   design honest/general; the absence IS the design (`:remote`/`RemoteOpts`, the perpetually-awaiting
   host key — spawn.wat:116). These must NOT be "completed" — [[feedback_dont_build_the_forcing_function]].

The catalog must TAG each slot kind (1) or (2). A type-(1) slot is a TODO-when-needed; a type-(2) slot
is a standing invariant to leave alone.

## Output

A catalog doc (`docs/DEFERRED-TOOLING.md` or similar) — the live map of pre-described tooling, each
row tagged latent-ready vs forcing-function, with its trigger and its mechanism's location. Refresh it
when an arc adds a new deferral or a caller pulls one through.

## To investigate when picked up
- Decide the catalog's home + whether it's generated (grep-driven, gated) or hand-curated.
- Run the sweep; tag each hit (1)/(2); note any latent-ready slot a current leg could already use.
- Whether to add an `exigere`-style rune at each deferral site pointing to its catalog row (so the
  deferral and the map can't drift).
