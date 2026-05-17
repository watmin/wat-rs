# Arc 207 — `:wat::core::Uuid` typed primitive

**Status:** OPEN 2026-05-17.

**Priority:** **BLOCKING.** Per user direction 2026-05-17: *"we're blocked on all other prior arcs until this done."* Arc 170 closure paperwork, arc 203 closure paperwork, lab reconstruction, and any new substrate work all wait until arc 207 ships.

**Pedigree:** Arc 206 promoted `:wat::core::uuid::v4` + `:wat::core::uuid::v5` to substrate-core (slices 1, 1.5, 2, 3 — last commit `ced6ac2`). Both verbs return `:wat::core::String`. Arc 206's INSCRIPTION-SLICE-2 named typed `:wat::core::Uuid` as "out of scope; no current consumer demands it." That framing was wrong on two counts.

**The crack arc 207 closes.** Two pieces of consumer pressure were already on disk when arc 206 closed:

1. **`src/edn_shim.rs:404` rejects `Edn::Uuid(_)` with `EdnReadError::Other(...)`.** When user wat code calls `(:wat::edn::read "#uuid \"550e8400-...\"")`, the substrate errors instead of producing a wat-level Uuid value — because there's no wat-typed Uuid for the EDN read path to land on. The substrate has a literal arm waiting to be filled. That IS consumer pressure, sitting on disk uncomplained-of until the user named it.
2. **`:wat::core::uuid::v5`'s `namespace` parameter is `:wat::core::String` and runtime-panics on invalid UUID input.** A typed Uuid parameter would push the validation gate to check time. Currently the substrate carries a foot-gun documented in the USER-GUIDE.

User direction 2026-05-17, naming the doctrine: *"deferral is a dishonest term."* Arc 206's "no current consumer" framing was deferral dressed in affirmative language; the consumer pressure existed, we hadn't named it. Arc 207 names it and closes it.

## What "proper UUID support" means (the Clojure reference)

Clojure's pattern (the great whose footprints we're stepping into per `user_no_literature`):

- UUIDs are a distinct type (`java.util.UUID`), NOT strings
- `(random-uuid)` returns a UUID instance
- `(parse-uuid s)` → UUID or nil
- `(uuid? x)` predicate
- `#uuid "..."` reader literal produces a UUID
- Equality: UUID-to-UUID; a UUID does NOT equal its String representation
- Strings stringify via `(str u)`

wat-edn already mirrors this internally: `Value::Uuid(uuid::Uuid)` distinct variant (`crates/wat-edn/src/value.rs:55`), full `#uuid` parser + writer, `as_uuid()` accessor. **The gap is that this typed surface stops at the wat-edn Rust crate boundary** — the wat substrate above it ships String at the substrate-API level.

## Goal

Mint `:wat::core::Uuid` as a typed primitive at the wat substrate level. Surface mirrors Clojure's `java.util.UUID`:

| Verb | Signature | Notes |
|---|---|---|
| `:wat::core::uuid::v4` | `[] -> :wat::core::Uuid` | FLIPPED from `:String`; random 122-bit |
| `:wat::core::uuid::v5` | `[namespace :wat::core::Uuid, name :wat::core::String] -> :wat::core::Uuid` | FLIPPED; namespace becomes type-enforced |
| `:wat::core::Uuid/from-string` | `[s :wat::core::String] -> :Option<wat::core::Uuid>` | Parse; None on invalid |
| `:wat::core::Uuid/to-string` | `[u :wat::core::Uuid] -> :wat::core::String` | Render canonical hyphenated form |

Plus the substrate fix at `src/edn_shim.rs:404`: `Edn::Uuid(u)` → produce a wat-level `:wat::core::Uuid` value (no more EdnReadError).

The four substrate verbs cover Clojure's surface: v4 is `random-uuid`; v5 covers content-addressing (no Clojure equivalent — it's our extension); `Uuid/from-string` is `parse-uuid`; `Uuid/to-string` is `str`. Equality + comparison fall out from the existing wat substrate dispatch infrastructure (arc 146) — if the substrate's existing equality dispatch needs a `:wat::core::Uuid` arm, that's part of slice 2/3.

## Out of scope (affirmatively, not deferral)

Per the lesson the user just inscribed ("deferral is a dishonest term"), what arc 207 does NOT cover and why:

- **Other UUID versions (v1 / v3 / v7 / v8).** Arc 206 documented the 3-step mechanical pattern in DESIGN; flipping their return type from `:String` to `:wat::core::Uuid` is a one-line edit per version when the version arrives. No version other than v4 and v5 ships today; flipping non-existent verbs is anti-work.
- **Direct UUID literal at the wat-syntax level** (e.g., `#uuid "..."` as wat reader literal alongside `(:wat::core::Uuid/from-string "...")`). The EDN read path covers reader-literal semantics; a wat-syntax-level literal is a separate concern (parser change, not substrate change). If a consumer surfaces wanting it inline in wat source, that opens a new arc.
- **UUID equality verb as substrate primitive** (e.g., `:wat::core::Uuid/equal?`). The substrate's dispatch infrastructure (arc 146) handles equality polymorphically; minting a type-specific verb when polymorphic dispatch covers it is wrong shape per `feedback_no_new_types`. Slice 2/3 confirms via test that `(= u1 u2)` works through the existing dispatch.

## Slicing (proactive stepping stones per recovery doc § 5)

| Slice | Status | What | Notes |
|---|---|---|---|
| **1 — substrate audit + shape decision** | OPEN | Audit how existing typed primitives are registered at the wat substrate (`Bytes`? `Symbol`? other typealiases / newtypes / opaque types?). Decide: typealias-of-String vs newtype-over-String vs new `Value::wat__core__Uuid` variant. Decision committed in slice 1 SCORE; subsequent slices build on it. NO substrate code edits in slice 1 — pure investigation. | Per `feedback_diagnose_before_spec`: read the actual code path before specifying. Slice 1 produces the implementation shape that slice 2 ships. |
| **2 — mint `:wat::core::Uuid` type + `Uuid/from-string` + `Uuid/to-string`** | BLOCKS on 1 | Substrate-register the type per slice 1's decision; mint the two parse/render verbs; tests cover round-trip. No verb-flip yet — v4/v5 still return `:String`. | Stepping stone: the type exists + can be constructed/destructed before any consumer-facing API changes. |
| **3 — flip `:wat::core::uuid::v4` + `v5` return types + v5 namespace type** | BLOCKS on 2 | v4: `[] -> :wat::core::Uuid`. v5: `[namespace :wat::core::Uuid, name :String] -> :wat::core::Uuid`. v5's namespace runtime-panic on invalid String retires (type system enforces). Update arc 206 USER-GUIDE entry to reflect typed surface. | Consumer-facing API change. Telemetry alias + arc 203 demo consumers ripple. |
| **4 — fix `src/edn_shim.rs:404`** | BLOCKS on 2 | `Edn::Uuid(u)` arm produces wat-level `:wat::core::Uuid` value instead of erroring. Test: `(:wat::edn::read "#uuid \"...\"")` returns a typed Uuid. | Parallel-with-3-possible; either slice closes the substrate gap independently. |
| **5 — consumer ripple** | BLOCKS on 3 + 4 | wat-telemetry alias (`:wat::telemetry::uuid::v4` return type flips), arc 203 demos (counter-service-{capability,process}-N3.wat — server-id type changes), any other String-typed consumer that holds a Uuid-shaped value. | Mechanical sweep guided by grep. |
| **6 — closure paperwork** | BLOCKS on 5 | INSCRIPTION; DESIGN status CLOSED; USER-GUIDE § 11 rewrite (replaces arc 206's String-typed entry with typed Uuid entry); 058 row in lab repo. | Arc 207's INSCRIPTION explicitly retires the "no current consumer demands" framing as a discipline failure pattern. Arc 206's INSCRIPTIONs stay immutable. |

After arc 207 closes: arc 170, arc 203, lab reconstruction all unblock.

## Substrate touchpoints (preliminary; slice 1's audit refines)

- `src/runtime.rs` — wat substrate Value enum; check for typealias / newtype / variant patterns
- `src/check.rs` — type scheme registrations for `:wat::core::uuid::*` verbs
- `src/string_ops.rs` — current `eval_uuid_v4` + `eval_uuid_v5` handlers
- `src/edn_shim.rs:404` — the `Edn::Uuid` rejection arm
- `crates/wat-edn/src/value.rs` — `Value::Uuid(uuid::Uuid)` variant (substrate-of-substrate; should NOT need edits)
- `Cargo.toml` (root wat crate) — `uuid` dep already present (added arc 206 slice 1.5)
- `docs/USER-GUIDE.md` § 11 — backward-compat section rewrites
- `docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION.md` + `INSCRIPTION-SLICE-3.md` — immutable; arc 207 INSCRIPTION cross-references them as "forward-correcting the type-level honesty gap"

## Connection to broader work

Arc 207 is the third (and final, modulo future versions) UUID-substrate arc:

| Arc | Theme | Ships |
|---|---|---|
| 092 | Initial mint | `wat_edn::new_uuid_v4` in wat-edn crate |
| 206 | Substrate promotion + telemetry de-dup | `:wat::core::uuid::v4` + `v5` (String-returning) |
| **207** | **Type-level honesty** | **`:wat::core::Uuid` typed primitive; EDN read gap closed** |

After 207 ships, the UUID story is complete at every layer: substrate verbs are type-honest, EDN reads work end-to-end, capability-token consumers can express UUID-shaped identity at check time, telemetry/arc-203/lab-reconstruction unblock.

## Discipline lesson (carried forward into INSCRIPTION when arc closes)

Arc 206's "out of scope; no current consumer demands it" framing for typed Uuid was **deferral dressed in affirmative language**. The consumer pressure was on disk at `src/edn_shim.rs:404` (literal `Edn::Uuid` rejection arm) AND in arc 206 INSCRIPTION's own admission that v5's `namespace: :String` runtime-panics on invalid input. Both are concrete substrate gaps the orchestrator chose not to see.

The discipline carry-forward: **before naming anything "out of scope; no consumer demands it," grep the substrate for arms / errors / panics that name the missing type.** If they exist, that IS the consumer pressure; the type belongs in scope.

User direction 2026-05-17 (load-bearing for arc 207's existence): *"deferral is a dishonest term."* When the right shape is known and the consumer pressure exists on disk, ship it. Do not paper over with affirmative scope-bounding.

This lesson lands in arc 207's INSCRIPTION at closure time, not here in DESIGN. DESIGN states it for the slice authors so the slicing reflects the doctrine.
