# BRIEF — Stone 1: the `RequestMalformed` sanitization wall, proven end-to-end on ONE service

> Design: `DESIGN-request-malformed-input-sanitization.md` (committed `28701476`, read it first).
> Builder-ruled 2026-07-25: *"we must have a request-malformed … the whitelist of what we accept is already
> explicit — a bad caller (malicious or dumb) cannot crash anything."*
>
> **This is a proven, live denial of service.** One malformed frame from any client kills a service for
> everyone; a second, innocent client cannot even `connect'`. Fixing it outranks the cache campaign.

## PHASE 0 — GROUND THIS FIRST, IT DECIDES WHAT YOU BUILD

**Do `ServiceEvent::Malformed` / `ServiceEvent::Rejected` already reach the CLIENT as a matchable value, or
are they owner-side only?**

Grep them in `src/runtime.rs` and follow both to their consumers. This decides the whole shape:

- **If a protocol-tier rejection already reaches the client** → `RequestMalformed` may ride *that* rail, and
  the corpus never needs a per-op variant. Much smaller, and it means Stone 2 barely exists.
- **If they are owner-side only** → the variant is per-op, mirroring `:RequestTooLarge` (ruling A), and
  Stone 2 is a codemod sweep.

**Report which, with `file:line`, BEFORE building.** If it is ambiguous, STOP and surface it — do not pick.

## The vulnerability (proven; reproduce it first as your RED gate)

`wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat` is the live reproduction:

```
"attacker good  => Ok"
"attacker BAD   => LOST (peer gone)"
victim: connect REFUSED — service is GONE
```

The frame is well-formed EDN with a wrong-typed body under a correct tag —
`#dos.Bag/PutRequest {:items [1 2 3]}` against `items <- Vector<String>`. The handler uses the field at its
declared type and detonates. Sibling probes: `probe-arc278-wire-type-enforcement.wat` (the measurement,
both tiers) and `-detonate.wat` (the handler crash).

## Both halves already exist — connect them, do not invent

- **The whitelist:** the op's declared request record. Already authored, per op. **Nothing new to declare.**
- **The validator:** `edn_to_typed_value` (`src/edn_shim.rs:1741`) — walks a declared `TypeExpr`, recurses per
  element (`Vector<T>` arm `:1892-1905`), rejects `Integer` against `String`, and yields the offending path
  (`.items.[0]`). **Zero production callers** since arc 258 Stone 258.5b deleted its last one on the
  trusted-wire premise.

## The placement — mirror `guarded-arm` EXACTLY

`wat/service.wat` ~`:1060` enforces `:max-request-bytes` **post-decode, inside the generated dispatch arm,
before the handler**, and on violation `send'`s the named variant then **recurses into `serve`** — keeping
the service alive, and treating a gone client as non-fatal too.

**Put the shape guard in that same slot.** This is the load-bearing reason: it is post-decode in the serve
loop, so it covers **BOTH TIERS**. A Rust-side decode fix would miss the thread tier entirely — that tier
does no decode at all (`ReactorClass::InMemory`, `src/runtime.rs:27585-27591`, `Value` passed verbatim
through crossbeam).

## What to build

1. **A wat-callable shape validator.** The guard is generated *wat*; `edn_to_typed_value` is Rust with no wat
   surface. **Ground whether one already exists before minting one** — check the `:wat::edn::*` surface, and
   whether arc 299's constraint system (`conforms`) already answers "does this value match this shape."
   Reuse beats minting; say what you found either way.
2. **The refusal variant** — shaped by Phase 0. If per-op, `:RequestMalformed` as a mandatory sibling of
   `:RequestTooLarge`, carrying the coordinate the validator already computes:
   `path <- Vector<String>` (segments, e.g. `["items" "[0]"]`), plus expected/got.
3. **The guard** in `guarded-arm`, before the handler, keeping the service alive on violation.

**Scope: ONE service.** Do NOT sweep the corpus — that is Stone 2, and it is a `wat/fix.wat` codemod plus a
compiler-driven worklist (the exhaustive-match rule self-identifies every site). Nothing here should fear it.

## THE ACCEPTANCE BAR — the DoS probe, INVERTED

A `deftest` in which:
1. the attacker sends the malformed frame and receives a **named `RequestMalformed`** (not a crash, not a raise);
2. **a subsequent innocent client `connect'`s and is served successfully.**

Both tiers — thread and process. (2) is the point of the whole strike: *a bad caller cannot crash anything.*

## Open — decide with four-questions, do not assume

`expected`/`got` as `String` versus structured type forms. A `TypeExpr` is EDN-expressible, so the builder's
prose-vs-structured rule says structure it — but that collides with the `format_type` question already noted
for arc 296. **Decide it once, here, with the four questions, and record the reasoning** so 296 inherits it
rather than re-deriving it.

## STOP triggers

1. Phase 0 ambiguous → STOP, report, do not pick a shape.
2. No wat-callable validator exists and minting one exceeds a thin wrapper over `edn_to_typed_value` → STOP,
   report the real cost.
3. The guard cannot be made tier-general in `guarded-arm` → STOP. Tier-general is the requirement, not a nice-to-have.
4. Blast radius beyond `wat/service.wat` + the validator surface + one service's enum + the new gate → STOP.

## Gate

- The inverted-DoS `deftest` green on both tiers.
- The three existing scratch-pad probes still GREEN (loader-gated).
- `cargo build --release` clean; `cargo nextest run --release` — **Summary line VERBATIM**. Floor: **4173
  passed, 314 skipped**.
- FOREGROUND only. **Do NOT commit** — the orchestrator weighs by their own re-run and commits.

## Your report

The Phase-0 verdict with citations and how it shaped what you built; whether a wat validator existed or was
minted; the inverted-DoS evidence (attacker's reply AND the victim being served, quoted); the four-questions
call on `expected`/`got`; the verbatim Summary line; any STOP.
