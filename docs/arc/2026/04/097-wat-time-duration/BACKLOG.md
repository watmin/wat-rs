# Arc 097 — BACKLOG

DESIGN settled 2026-04-29 (all 5 open questions resolved). Slice
order matches the DESIGN's slice plan. Each slice is one substrate
add; each ships with green tests before the next opens.

## Slice 1 — Duration variant + unit constructors — *ready*

- **Status:** ready.
- **Adds:** `Value::Duration(i64)` runtime variant + 7 unit
  constructors (`Nanosecond`, `Microsecond`, `Millisecond`,
  `Second`, `Minute`, `Hour`, `Day`).
- **Guards:** negative input panics (§2); i64-overflow panics (§4).
- **Substrate touches:**
  - `src/runtime.rs` — new variant, type_name dispatch, EDN write,
    eq/hash, 7 constructor functions.
  - `src/check.rs` — type registration for `:wat::time::Duration`,
    schemes for the 7 constructors.
  - `wat-tests/std/time/Duration.wat` — wat-level smoke tests.
- **Done when:** `cargo test --workspace` green; `(:wat::time::Hour 1)`
  produces a Duration with `3_600_000_000_000` nanos.

## Slice 2 — Polymorphic - / + — *obvious in shape, fog-until-1*

- **Status:** obvious in shape; depends on slice 1 landing.
- **Adds:** `:wat::time::-` (Instant-Duration → Instant; Instant-Instant
  → Duration) and `:wat::time::+` (Instant + Duration → Instant).
- **Dispatch:** match on RHS Value variant; reject Duration LHS in
  this slice (no Duration arithmetic).
- **Guards:** Instant-Instant where LHS < RHS panics (would produce
  negative duration; per §2 not allowed).
- **Sub-fog:** does the type checker need scheme overloading, or do
  we register `-` as a single polymorphic scheme that matches both
  shapes? Verify against arc 025's polymorphic `get` precedent.

## Slice 3 — `ago` / `from-now` composers — *ready when 2 lands*

- **Status:** ready once slice 2 ships.
- **Adds:** `(:wat::time::ago duration) -> Instant` and
  `(:wat::time::from-now duration) -> Instant`.
- **Implementation:** one-line each over `(now)` + slice 2 arithmetic.
  Probably wat-side, not Rust.
- **Sub-fog:** wat-side or Rust-side? Wat-side is cheaper to maintain;
  Rust-side is one fewer function-call indirection. Lean wat unless
  arc 091's perf budget cares.

## Slice 4 — Pre-composed unit-ago / unit-from-now — *ready when 3 lands*

- **Status:** ready once slice 3 ships.
- **Adds:** 14 helpers (7 units × ago/from-now): `nanoseconds-ago`,
  `microseconds-ago`, ..., `days-ago`, plus the from-now variants.
- **Implementation:** wat-side `define`s on top of slices 1+3.
  Each is `(define (X-ago n) (ago (UnitConstructor n)))`.
- **Sub-fog:** none — pure mechanical composition.

## Slice 5 — INSCRIPTION + USER-GUIDE — *ready when 4 lands*

- **Status:** ready when 4 ships.
- **Adds:** INSCRIPTION.md sealing the arc, USER-GUIDE.md section
  on time helpers, 058 FOUNDATION-CHANGELOG row, arc 093's
  Predecessors entry resolved (sibling shipped).

## Cross-cutting fog

- **`:wat::time::Instant` already ships as a runtime variant** (per
  arc 056). Existing primitives (`now`, `epoch-nanos`, etc.)
  operate on it. Arc 097 doesn't re-shape Instant — just adds
  Duration alongside.
- **Type-checker scheme registration for new variants** — verify
  arc 048's enum machinery covers what we need, or whether
  Duration as a primitive (not enum) needs different handling.
  Resolves in slice 1's first commit.
- **EDN serialization** — Duration's EDN rendering. `(Duration 3600000000000)`?
  `#wat-time/duration "1h"`? Decide in slice 1 alongside the
  variant add.
