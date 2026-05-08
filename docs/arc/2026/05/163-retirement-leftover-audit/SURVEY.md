# Arc 163 — Soft-retirement surface SURVEY

**Drafted 2026-05-07 by orchestrator.** Comprehensive inventory of
every retirement surface that still has SOFT scaffolding (typealias
fall-through, runtime alias arm, transitional acceptance) when it
should be HARD-retired per user direction *"this arc has as many
sub pieces as necessary - do not close this current arc until all
retired forms are actually retired."*

Built by enumerating every `BareLegacy*` CheckError variant in
`src/check.rs` + cross-checking for soft fall-throughs.

## The 12 surfaces

| # | Surface (`BareLegacy*`) | Retired by | Refs in check.rs | Soft state? | Cost |
|---|---|---|---|---|---|
| 1 | `BareLegacyLetStar` | arc 154 | 9 | NONE — hard since arc 154 slice 2 (walker body retired) | done |
| 2 | `BareLegacyLambda` | arc 155 | 7 | NONE — hard since arc 155 slice 2 (Path B full retirement) | done |
| 3 | `BareLegacyLowercaseFn` | arc 155 | 6 | NONE — hard, same arc 155 slice 2 | done |
| 4 | `BareLegacyConsolePath` | arc 109 K.console | 6 | TBV — verify no live runtime arm | cheap |
| 5 | `BareLegacyTelemetryServicePath` | arc 109 K.telemetry | 5 | TBV | cheap |
| 6 | `BareLegacyLruCacheServicePath` | arc 109 K.lru | 6 | TBV | cheap |
| 7 | `BareLegacyStreamPath` | arc 109 slice 9d | 6 | TBV — slice 2 of 163 swept consumers | cheap |
| 8 | `BareLegacyKernelQueuePath` | arc 109 K.kernel-channel | 5 | TBV — slice 2 of 163 swept consumers | cheap |
| 9 | `BareLegacyUnitName` (value) | arc 153 | 7 | TBV — `unit` → `nil` value rename | cheap |
| 10 | `BareLegacyUnitType` (type) | arc 153 | 5 | TBV — `:wat::core::unit` type → `:wat::core::nil` (or removed?) | cheap-medium |
| 11 | `BareLegacyContainerHead` | arc 109 slice 1f | 5 | SOFT — `:Vec<T>` parses as typealias to internal `head: "Vec"`; walker fires but parser succeeds | medium |
| 12 | `BareLegacyPrimitive` | arc 109 slice 1c | 9 | SOFT — bare `:i64`, `:f64`, `:String`, `:bool` etc. Substrate accepts; ~4040 sites in tree | expensive |

PLUS two non-`BareLegacy*` surfaces (runtime arms):

| Surface | Retired by | Soft state | Cost |
|---|---|---|---|
| `:wat::core::list` runtime arm (`runtime.rs:3088`) | arc 109 slice 1g | SOFT alias arm | cheap |
| `:wat::core::vec` runtime arm (`runtime.rs:3082`) + substrate-internal canonicalization | arc 109 slice 1f | SOFT alias arm + substrate-internal `head: "Vec"` matches the legacy spelling, not canonical `Vector` | medium |

## Cost-ordered slice plan

Stepping stones first; the cheap surfaces build confidence + the
audit framework is reusable on harder surfaces.

### Slice 3a — `:wat::core::list` runtime arm (CHEAP)

Surgical: delete one runtime arm + migrate 37 test sites
`(:wat::core::list ...)` → `(:wat::core::Vector ...)` (NOT `vec`
— `vec` is also retired). Type-checker Pattern 2 poison stays as
the user-facing diagnostic; runtime arm gone for defense-in-depth.

**Status:** sonnet's first attempt (`a464358b...`) shipped the
deletion but migrated tests to `vec` (retired target — my BRIEF
error). I attempted fix-forward; broke substrate-internal
canonicalization. Stash holds the WIP work. Need to re-do cleanly.

**Action:** revert to clean main, re-spawn sonnet on corrected
BRIEF (target `Vector`).

### Slice 3b — Service path retirements (CHEAP × 4)

Verify each has zero soft fall-through then sweep any remaining
Bucket B comments:
- `BareLegacyConsolePath` (arc 109 K.console — `:wat::std::service::Console::*` → `:wat::console::Console::*`)
- `BareLegacyTelemetryServicePath` (arc 109 K.telemetry)
- `BareLegacyLruCacheServicePath` (arc 109 K.lru)
- `BareLegacyStreamPath` (arc 109 slice 9d)
- `BareLegacyKernelQueuePath` (arc 109 K.kernel-channel)

Audit each: walker present (yes per arc INSCRIPTIONS); consumer
code clean (slice 2 confirmed Stream + Queue); no runtime alias
arm. If all confirmed: just verify, no edits. Maybe sweep small
Bucket B comments.

### Slice 3c — Unit name + type retirements (CHEAP)

`:wat::core::unit` retired by arc 153 in favor of `:wat::core::nil`.
Both `BareLegacyUnitName` (value position) + `BareLegacyUnitType`
(type position). Verify substrate has no soft acceptance + sweep
any consumer leftovers.

### Slice 3d — `:wat::core::vec` runtime arm + substrate-internal canonicalization (MEDIUM)

Architectural — internal `head: "Vec"` representation. Two paths:

**Path A** — change internal representation: `head: "Vec"` →
`head: "Vector"` substrate-wide. ~50-100 sites of `head == "Vec"`
matches updated. Most surgical for hard-retire but biggest mass
edit.

**Path B** — keep internal `head: "Vec"` as substrate impl detail;
only delete the user-facing `:wat::core::vec` keyword arm at
runtime.rs:3082; update canonicalize step at 16811 to recognize
`Vector` (the canonical user-facing spelling); update doc
comments + test fixtures.

Path B is cheaper. Start with Path B; revisit Path A only if it's
required for honest hard-retire.

### Slice 3e — `:Vec<T>` walker firmness (MEDIUM)

`BareLegacyContainerHead` walker fires on `:Vec<T>`, but the
parser accepts `:Vec<T>` syntactically (per the typealias). Verify
the walker emission is FATAL (fails check), not warning. If soft:
make it fatal.

### Slice 3f — Bare primitive retirement (EXPENSIVE — last)

`BareLegacyPrimitive` covers bare `:i64`, `:f64`, `:String`, `:bool`,
maybe more. ~4040 sites total in tree (most in test fixtures).

Two-step:
1. Substrate audit: confirm walker present + emission level (warn
   vs fatal). Make fatal if not already.
2. Mass test-fixture sweep: bare → FQDN'd. Sonnet edits-only;
   orchestrator verifies.

This is the biggest piece; do last to leverage all prior slice
patterns.

### Slice 3z — closure (INSCRIPTION + 058 row)

After all slices ship: write closure. INSCRIPTION names every
slice; 058 row summarizes total surfaces hardened.

## Discipline going forward

After EACH slice ships:
- `cargo build --release` clean
- `cargo test --release --workspace` 2041+ passing / 0 failed
- Audit grep for the surface confirms residual is Bucket C/D only
- Commit + push BEFORE starting next slice (durable checkpoint)

If ANY slice surfaces a previously-unknown soft retirement:
- Add it to this SURVEY as a new slice (3g, 3h, ...)
- Sequence by cost (cheap first)
- DO NOT CLOSE arc 163 until SURVEY shows zero soft surfaces
