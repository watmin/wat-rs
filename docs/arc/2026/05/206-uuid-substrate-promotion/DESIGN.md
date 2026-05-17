# Arc 206 — UUID primitive promotion to substrate (`:wat::core::uuid::v4`)

**Status:** CLOSED 2026-05-17 (slice 3).

**Pedigree:** Arc 091 slice 2 minted `uuid::v4` under `wat-measure`; arc 096 folded `wat-measure` into `wat-telemetry`. UUIDs landed under `:wat::telemetry::uuid::v4` by historical accident, not by design — they were minted next to other telemetry primitives because that's where measurement infrastructure lived.

**Defect:** UUIDs are foundational substrate (capability ids, content addressing, distributed identity, secret-witness tokens). Every consumer that needs unguessable ids must pull a telemetry dep — even consumers that don't emit telemetry. The categorical placement makes UUIDs needlessly hard to reach.

**Surfaced consumer pressure:** Arc 203's capability pattern (struct-restricted Admin + User) requires server-id + user-id minting. Slice 2 SCORE Delta 1 documented the gap — wat-tests/ test runner doesn't have wat::telemetry dep wired; the demos used constant strings (`"server-counter-thread-0"`, `"client-1"`) with prose comments saying "in production use uuid::v4." That's documentation, not implementation.

## Goal

Promote uuid::v4 from telemetry-crate verb to substrate-core primitive. Mint `:wat::core::uuid::v4` (or `:wat::kernel::uuid::v4` — namespace TBD per slice 1 grep). The promoted primitive returns `:wat::core::String` (canonical 8-4-4-4-12 hyphenated hex) — same shape as existing `:wat::telemetry::uuid::v4` return.

Keep `:wat::telemetry::uuid::v4` as an alias (or deprecation arrow pointing at the new substrate-level name) for backward compatibility with existing telemetry consumers; arc 091's original telemetry uses continue to work.

## Out of scope (deferred to future arcs)

- **Typed `:wat::core::Uuid`** — distinguishes from arbitrary String at the type system level. Worth doing eventually but separate concern; this arc keeps return type as String per existing shape
- **UUID parsing/validation** — `:wat::core::Uuid::from-string` etc.; not needed for capability minting (servers only generate, never parse)
- **Other UUID versions (v5, v7)** — only v4 (random) needed for unguessability

## Slicing (per "very short arc" direction)

| Slice | Status | What |
|---|---|---|
| **1 — substrate promotion** | OPEN | Mint `:wat::core::uuid::v4` (or `:wat::kernel::uuid::v4`); thin wrapper around existing `wat_edn::new_uuid_v4`; available without telemetry dep; existing telemetry alias remains |
| **2 — closure** | BLOCKED on 1 | INSCRIPTION + 058 row + USER-GUIDE entry pointing at canonical generation pattern |

After 2 closes: arc 203 demos flip to use substrate-level UUIDs (separate small slice in arc 203 — `slice 3f-uuid` — refactors counter-service demos to use unguessable ids; eliminates the constant-string honest gap).

## Substrate touchpoints

- `crates/wat-telemetry/src/shim.rs` — `:rust::telemetry::uuid::v4` (the actual minter; uses `wat_edn::new_uuid_v4`)
- `crates/wat-telemetry/wat/telemetry/uuid.wat` — current wat-side wrapper (`:wat::telemetry::uuid::v4`)
- `wat_edn::new_uuid_v4` (per arc 092) — the underlying RNG-backed canonical-string mint
- src/runtime.rs — likely where substrate-core verbs register (similar pattern to other `:wat::core::*` verbs)
- tests/test.rs — the wat-tests test runner; after promotion, `:wat::core::uuid::v4` available to all wat-tests without additional dep

## Connection to next steps

After arc 206 closes:
- **arc 203 slice 3f-uuid** — refactor `wat-tests/counter-service-{capability,process}-N3.wat` to mint server-id + user-id via `:wat::core::uuid::v4`; eliminates the constant-string + monotonic-counter honesty gap
- **Protocols arc (still pending)** — `defservice` meta-form uses substrate-level UUID for capability minting
- **Arc 203 services refactor (3g/3h/3i)** — cache + holon-cache + stdio services use defservice → use UUIDs
- **Arc 203 closure** → unblocks
- **Arc 170 closure** → unblocks

---

## Scope expansion 2026-05-17 — v4 AND v5 in same arc; positioned for future versions

User direction: *"we grow the current uuid arc to handle this proper.. no new arc.. the uuid arc handles v4 and v5 (we'll position ourselves for other versions being added in if we need them...)"*

### Revised scope

Arc 206 covers BOTH UUIDv4 and UUIDv5 at substrate level. Structured so future versions (v7, etc.) are mechanical additions.

| Version | Use case | Substrate-side signature |
|---|---|---|
| **v4** (random) | Unguessable secret-witness; capability tokens; per-actor secret server-ids | `:wat::core::uuid::v4` `[] -> :wat::core::String` |
| **v5** (SHA-1 namespace+name) | Content addressing; hierarchical-derivation (parent-id + counter → deterministic child-id); cross-process consistent ids | `:wat::core::uuid::v5` `[namespace :String, name :String] -> :wat::core::String` |
| Future (v7 timestamp-ordered, etc.) | Open | Same `:wat::core::uuid::v<N>` namespace; mechanical addition |

### Revised slicing

| Slice | Status | What |
|---|---|---|
| **1 — v4 promotion** | DONE (`4ff2b72`) | Mint `:wat::core::uuid::v4` wrapping existing `wat_edn::new_uuid_v4`; backward-compat alias for telemetry |
| **1.5 — v5 promotion** | DONE (`b56e272`) | Mint `wat_edn::new_uuid_v5(namespace, name)` (adds `v5` feature to `uuid = "1"` dep); register `:wat::core::uuid::v5` at substrate (same pattern as v4) |
| **2 — closure (premature)** | DONE (`74d7fea`) | INSCRIPTION + USER-GUIDE + 058 row — **forward-corrected by slice 3**: telemetry's duplicate impl was NOT retired; slice 2's lesson ("separate-impl wins over alias-chain") was WRONG per user review |
| **3 — telemetry de-dup + honest closure** | DONE | Retire `:rust::telemetry::uuid::v4` shim + `wat-edn` dep from `wat-telemetry`; delegate `:wat::telemetry::uuid::v4` to substrate-core at the wat layer; EDN roundtrip proven; arc 206 reaches honest closure |

### Why this positions for future versions

The substrate-registration pattern is per-version (each `:wat::core::uuid::v<N>` is its own verb). No central dispatch. Adding a new version = three mechanical steps:
1. Enable the version's feature flag in `wat-edn/Cargo.toml` (uuid crate already supports v1/v3/v4/v5/v7/v8)
2. Mint `wat_edn::new_uuid_v<N>(args)` thin wrapper around `uuid::Uuid::new_v<N>(...)`
3. Register `:wat::core::uuid::v<N>` substrate verb (mirror v4/v5 pattern)

No architectural decisions to re-litigate. Future arc can add v7 as one small slice when a consumer needs timestamp-ordered ids.

### Out of scope (still)

- Typed `:wat::core::Uuid` — future arc if demand surfaces (would make v5's namespace parameter honest at type-system level)
- UUID parsing/validation verbs (`:wat::core::Uuid/from-string`, `/parse`) — future arc
- UUID equality/comparison primitives — String equality suffices today
