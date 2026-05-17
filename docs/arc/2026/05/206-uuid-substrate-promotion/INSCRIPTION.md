# Arc 206 INSCRIPTION — Substrate-level UUID primitives

**Status:** SHIPPED 2026-05-17. v4 + v5 minted at `:wat::core::uuid::*`; both versions tested 4/4; workspace baseline preserved; backward-compat preserved for `:wat::telemetry::uuid::v4`.

## What this arc gave the substrate

Promoted UUID minting from `:wat::telemetry::uuid::v4` (categorically miscategorized as telemetry; pulled telemetry dep into every consumer needing unguessable ids) to substrate-level primitives at `:wat::core::uuid::*`:

| Verb | Signature | Backing | Use case |
|---|---|---|---|
| `:wat::core::uuid::v4` | `[] -> :wat::core::String` | `wat_edn::new_uuid_v4` (random 122 bits) | Unguessable secret-witness; capability tokens; per-actor secret server-ids |
| `:wat::core::uuid::v5` | `[namespace :String, name :String] -> :wat::core::String` | `wat_edn::new_uuid_v5` (SHA-1 namespace+name) | Content addressing; hierarchical deterministic derivation; cross-process consistent ids |

Both return canonical 8-4-4-4-12 hyphenated hex String. Both available to all wat code without `wat::telemetry` dep. Backward compat: `:wat::telemetry::uuid::v4` continues to work (separate independent impl; both wrap `wat_edn::new_uuid_v4`).

## Slices

| Slice | Commit | What |
|---|---|---|
| 1 — v4 promotion | `4ff2b72` | `:wat::core::uuid::v4` substrate registration; backward-compat alias for telemetry |
| 1.5 — v5 promotion | `b56e272` | `:wat::core::uuid::v5` substrate registration; SHA-1 backend pulled via `uuid/v5` feature flag |
| 2 — this closure | (current) | INSCRIPTION + 058 changelog row + USER-GUIDE entry |

## Substrate touchpoints (final)

| File | Change |
|---|---|
| `crates/wat-edn/Cargo.toml` | `uuid` deps: `mint` feature now includes `uuid/v5` (SHA-1 backend via `sha1_smol`) |
| `crates/wat-edn/src/lib.rs` | `new_uuid_v4` (pre-existing); `new_uuid_v5(namespace, name)` (new) |
| `Cargo.toml` (root wat crate) | Added `uuid = "1"` direct dep (eval handler calls `parse_str` directly) |
| `src/string_ops.rs` | `eval_uuid_v4` + `eval_uuid_v5` handlers; v5 panics on invalid namespace via `assertion-failed!` |
| `src/runtime.rs` | Dispatch arms for both verbs |
| `src/check.rs` | Type schemes registered |
| `tests/wat_arc206_uuid_substrate.rs` | 4 tests (v4: basic / entropy / canonical positions / no-telemetry-dep) |
| `tests/wat_arc206_uuid_v5.rs` | 4 tests (v5: basic / deterministic / namespace-affects / name-affects) |

## Out of arc 206's scope (affirmatively named)

- **Typed `:wat::core::Uuid`** — would distinguish from arbitrary String at type-system level; would make v5's `namespace` parameter honest (currently String; must contain valid UUID; runtime panic on invalid). Out of arc 206 scope; not tracked elsewhere because no current consumer demands it. If/when a consumer surfaces (e.g., heavy UUID-handling code where String-typed namespace becomes a documented foot-gun), a new arc opens with concrete shape pressure.
- **UUID parsing/validation verbs** (`:wat::core::Uuid/from-string`, `/parse`, `/valid?`) — eval handler for v5 already does the parsing internally; user-facing parsing primitives have no current consumer. Same affirmative scope-bound as typed Uuid.
- **Other UUID versions** (v1 MAC-time, v3 MD5, v7 timestamp-ordered, v8 custom) — arc 206 does NOT commit to them. Future versions follow the same 3-step mechanical pattern documented in DESIGN.md § "Why this positions for future versions":
  1. Enable version's feature flag in `wat-edn/Cargo.toml`
  2. Mint `wat_edn::new_uuid_v<N>` thin wrapper around `uuid::Uuid::new_v<N>`
  3. Register `:wat::core::uuid::v<N>` substrate verb mirroring v4/v5 pattern
  
  Per user direction 2026-05-17: *"i've only ever needed v4 and v5 in 10+ years of hyperscaler deliverables — if need more we know how to add them."* v4 + v5 cover the dominant use cases; further versions open new arcs when a real consumer demands them, with concrete shape pressure informing the design.

## Discipline lessons inscribed

### Categorical placement matters

Substrate primitives sit where their CATEGORY says they belong, not where they were historically introduced. UUIDs are foundational utility (used by capability minting, content addressing, distributed identity, telemetry — among others). They belong at `:wat::core::*`, not under any single consumer's namespace.

The miscategorization happened via arc 091 (minted `uuid::v4` under `wat-measure` because measurement was the immediate consumer) → arc 096 (folded `wat-measure` into `wat-telemetry`). Both moves were correct for their immediate scope; neither moved the primitive to its right category. Arc 203's capability pattern surfaced the friction (wat-tests test runner couldn't reach UUIDs without pulling telemetry dep); arc 206 corrected it.

The pattern: when consumer pressure surfaces that a primitive needs to be reachable from contexts that shouldn't pull the host crate's deps, the primitive is in the wrong category. Promote to substrate-core; preserve backward-compat alias at the original site.

### Separate-impl vs alias-chain backward-compat

`:wat::core::uuid::v4` and `:wat::telemetry::uuid::v4` are SEPARATE independent impls — both thin wrappers around the same `wat_edn::new_uuid_v4`. NOT an alias chain where one is registered as pointing at the other.

Reason: alias chains add a dispatch lookup + indirection; the verbs are one-liners; duplication costs nothing. Direct impls are simpler to reason about; either can evolve independently without breaking the other.

This is the cleaner backward-compat pattern when the underlying implementation is trivial. Adopt for similar future promotions.

## Cross-references

- `docs/arc/2026/04/091-substrate-quality-gates/INSCRIPTION.md` — slice 2 minted `uuid::v4` under `wat-measure`
- `docs/arc/2026/04/096-telemetry-crate-consolidation/INSCRIPTION.md` — folded `wat-measure` into `wat-telemetry`
- `docs/arc/2026/05/203-struct-restricted/DESIGN.md` — post-3e expansion; arc 203 slice 3f-uuid will use the new substrate UUIDs to eliminate constant-string honesty gap in counter-service demos
- `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-2.md` Delta 1 — the original honesty gap that surfaced consumer pressure for arc 206

---

Arc 206 inscribed.
