# Arc 206 — UUID primitive promotion to substrate (`:wat::core::uuid::v4`)

**Status:** OPEN 2026-05-17.

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
EOF
echo "DESIGN written"