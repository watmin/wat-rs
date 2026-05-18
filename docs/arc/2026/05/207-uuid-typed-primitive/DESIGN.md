# Arc 207 — `:wat::core::Uuid` typed primitive

**Status:** CLOSED 2026-05-17 — INSCRIPTION at INSCRIPTION.md.

**Priority:** **BLOCKING.** Per user direction 2026-05-17: *"we're blocked on all other prior arcs until this done."* Arc 170 closure paperwork, arc 203 closure paperwork, lab reconstruction, and any new substrate work all wait until arc 207 ships.

**Pedigree:** Arc 206 promoted `:wat::core::uuid::v4` + `:wat::core::uuid::v5` to substrate-core (slices 1, 1.5, 2, 3 — last commit `ced6ac2`). Both verbs return `:wat::core::String`. Arc 206's INSCRIPTION-SLICE-2 named typed `:wat::core::Uuid` as "out of scope; no current consumer demands it." That framing was wrong on two counts.

**The crack arc 207 closes.** Two pieces of consumer pressure were already on disk when arc 206 closed:

1. **`src/edn_shim.rs:404` rejects `Edn::Uuid(_)` with `EdnReadError::Other(...)`.** When user wat code calls `(:wat::edn::read "#uuid \"550e8400-...\"")`, the substrate errors instead of producing a wat-level Uuid value — because there's no wat-typed Uuid for the EDN read path to land on. The substrate has a literal arm waiting to be filled. That IS consumer pressure, sitting on disk uncomplained-of until the user named it.
2. **`:wat::core::uuid::v5`'s `namespace` parameter is `:wat::core::String` and runtime-panics on invalid UUID input.** A typed Uuid parameter would push the validation gate to check time. Currently the substrate carries a foot-gun documented in the USER-GUIDE.

User direction 2026-05-17, naming the doctrine: *"deferral is a dishonest term."* Arc 206's "no current consumer" framing was deferral dressed in affirmative language; the consumer pressure existed on disk, we hadn't named it. Arc 207 names it and closes it.

## What "proper UUID support" means (the Clojure reference)

Clojure's pattern (the great whose footprints we're stepping into per `user_no_literature`):

- UUIDs are a distinct type (`java.util.UUID`), NOT strings
- `(random-uuid)` returns a UUID instance
- `(parse-uuid s)` → UUID or nil
- `(uuid? x)` predicate
- `#uuid "..."` reader literal produces a UUID
- Equality: UUID-to-UUID; a UUID does NOT equal its String representation
- Nil-uuid `(UUID. 0 0)` is honest — sentinel for "no id yet" cases

wat-edn already mirrors this internally: `Value::Uuid(uuid::Uuid)` distinct variant (`crates/wat-edn/src/value.rs:55`), full `#uuid` parser + writer, `as_uuid()` accessor. **The gap is that this typed surface stops at the wat-edn Rust crate boundary** — the wat substrate above it ships String at the substrate-API level.

## Goal — user-facing surface (locked 2026-05-17)

Mint `:wat::core::Uuid` as a typed primitive at the wat substrate level. Six entries on the type using Type/verb naming per `feedback_wat_namespace_principle` (CONVENTIONS.md: `::` separates namespaces; `/` separates type from method/constructor):

| Verb | Signature | Notes |
|---|---|---|
| `:wat::core::Uuid` | TYPE | The primitive; minted at substrate level |
| `:wat::core::Uuid/v4` | `[] -> :wat::core::Uuid` | Random 122-bit; constructor |
| `:wat::core::Uuid/v5` | `[ns :wat::core::Uuid, name :wat::core::String] -> :wat::core::Uuid` | Deterministic SHA-1; namespace is type-enforced (no runtime panic on invalid String) |
| `:wat::core::Uuid/from-string` | `[s :wat::core::String] -> :Option<:wat::core::Uuid>` | Parse; None on invalid (no panic) |
| `:wat::core::Uuid/to-string` | `[u :wat::core::Uuid] -> :wat::core::String` | Render canonical 8-4-4-4-12 hyphenated form |
| `:wat::core::Uuid/nil` | the nil-uuid (`00000000-0000-0000-0000-000000000000`) | Whether 0-arg verb or constant is implementation detail per slice 1 |

Plus the substrate fix at `src/edn_shim.rs:404`: `Edn::Uuid(u)` → produce a wat-level `:wat::core::Uuid` value (no more EdnReadError).

**Why six and not more — version-check is out (affirmative):**

User check 2026-05-17: *"do we need a version check?.. what's the utility in that?... if i have a uuid do i care how it was constructed?.."* No. UUIDs are identifiers; construction technique is implementation detail not visible to comparators. Clojure exposes `.version` because java.util.UUID does, not because it was designed in. Real consumer demand: audit-trail debugging, narrow security validation, v7 timestamp-ordering — none exist today in wat. If v7 lands or an audit consumer surfaces, open new arc. Arc 207 does NOT mint `Uuid/version`.

**v4-vs-v5 type distinction is out (affirmative):**

User check 2026-05-17: *"do we need a v4 and v5 type?.. they are all uuid with just a different construction technique?.."* Confirmed. ONE `:wat::core::Uuid` type; v4 and v5 are constructors, indistinguishable post-construction. Matches Clojure: `(random-uuid)` and `(UUID/fromString s)` return the SAME type. The "v4-ness" / "v5-ness" lives at construction time, not type level. Equality between a v4 Uuid and a v5 Uuid with the same value works; they are the same Uuid.

## Naming convention notes

- `from-string` / `to-string` matches existing substrate convention (`:wat::core::keyword/from-string` + `keyword/to-string`). NOT `from-str` / `to-str` (abbreviation form not used elsewhere).
- Arc 207 RENAMES arc 206's namespace-form verbs (`:wat::core::uuid::v4` → `:wat::core::Uuid/v4`; same for v5). Per `feedback_refuse_easy_solutions`: NO parallel namespace+Type forms kept for "compatibility." The namespace-form was correct when UUIDs were String-typed (free functions in a namespace); now that Uuid is a type, constructors on the type IS the honest form. Telemetry's wat-side alias (`:wat::telemetry::uuid::v4`) updates its target accordingly.
- Equality requires ONE explicit `values_equal` arm addition (Pattern B's mechanical extension, per slice 1 audit). NOT automatic — DESIGN's initial "falls out from dispatch" framing was over-optimistic; corrected. Comparison (ordering) intentionally absent — UUIDs are identifiers not ordinals (same as keyword, Enum, Struct). Hash (`hashmap_key`) intentionally absent from slice 2 — latent gap, in-scope ADD in slice 4 if consumer ripple surfaces `HashMap<:Uuid, T>` demand.

- `Uuid/from-string` accepts CANONICAL form only (8-4-4-4-12 lowercase hyphenated). Returns `None` for uppercase, `urn:uuid:` prefix, braced, or otherwise non-canonical input. Matches EDN-layer strictness per slice 1 audit 4. If lenient parsing surfaces as a consumer need, a future arc adds it; arc 207 ships strict.

## Out of scope (affirmatively, not deferral)

Per the doctrine the user inscribed ("deferral is a dishonest term"), what arc 207 does NOT cover and why:

- **Other UUID versions (v1 / v3 / v7 / v8).** Arc 206 documented the 3-step mechanical pattern for adding versions; flipping their return type from `:String` to `:wat::core::Uuid` is a one-line edit per version when the version arrives. No version other than v4 and v5 ships today; flipping non-existent verbs is anti-work.
- **Direct UUID literal at the wat-syntax level** (e.g., `#uuid "..."` as wat reader literal alongside `(:wat::core::Uuid/from-string "...")`). The EDN read path covers reader-literal semantics; a wat-syntax-level literal is a separate concern (parser change, not substrate change). No consumer pressure surfaces today.
- **UUID equality verb as substrate primitive** (e.g., `:wat::core::Uuid/equal?`). The substrate's dispatch infrastructure (arc 146) handles equality polymorphically; minting a type-specific verb when polymorphic dispatch covers it is wrong shape per `feedback_no_new_types`. Slice 2 confirms via test that `(= u1 u2)` works through the existing dispatch.
- **`Uuid/version` extraction.** Per the "what's the utility" check above — no consumer pressure today.
- **`uuid?` predicate verb.** Substrate dispatch handles type-predicates polymorphically; no Uuid-specific predicate verb needed.

## Slicing (proactive stepping stones per recovery doc § 5)

| Slice | Status | What | Notes |
|---|---|---|---|
| **1 — substrate audit + shape decision** | SHIPPED 2026-05-17 `1aed75e` | Audit complete (SCORE-SLICE-1.md). Shape decision: option (c) — new `Value::wat__core__Uuid(uuid::Uuid)` variant. Pattern B (dedicated Value variant) per Instant/Duration/keyword precedent. Four-questions YES YES YES YES. | Slice 1's audit also surfaced: (a) edn_shim fix is mechanically inseparable from the Value variant addition (folded into slice 2); (b) equality requires an explicit `values_equal` arm (NOT automatic dispatch as DESIGN initially claimed — corrected below); (c) `Uuid/from-string` accepts canonical-only form (matches EDN strictness; strict canonical-only is the design, not a gap); (d) `hashmap_key` arm latent gap named for in-scope ADD if telemetry consumer surfaces it. |
| **2 — mint `:wat::core::Uuid` + 5 verbs + edn_shim fix** | SHIPPED 2026-05-17 `a961112` | Substrate-register the Value variant; mint `Uuid/v4` + `Uuid/v5` + `Uuid/from-string` + `Uuid/to-string` + `Uuid/nil`; add `values_equal` arm; fix `edn_shim.rs:404` Edn::Uuid arm (read) + add `value_to_edn_with` Uuid arm (write). 21-item substrate surface area (20 from SCORE-SLICE-1 + render_value arm surfaced by Rust exhaustiveness). v5 namespace param is `:Uuid` (type-enforced; eliminates current runtime-panic foot-gun). 10 tests pass. | One slice because all 21 items build the same coherent type; splitting creates broken intermediates. Per `feedback_simple_is_uniform_composition`: N uniform additions IS simple. |
| **3 — retire `:wat::core::uuid::*` namespace verbs** | SHIPPED 2026-05-17 `5f9d370` | Arc 206's `:wat::core::uuid::v4` + `v5` namespace verbs retired entirely. Telemetry's `:wat::telemetry::uuid::v4` alias retargets to `:wat::core::Uuid/v4`. STOP trigger 3 fired (telemetry test consumer outside arc 203 scope); resolved in-scope: `hashmap_key` arm added + telemetry test updated. | Per `feedback_refuse_easy_solutions`: no parallel keep-both. Namespace form was correct for String era; type era demands Type/verb. |
| **4 — consumer ripple** | SHIPPED 2026-05-17 `3865569` | Arc 203 demos (`wat-tests/counter-service-{capability,process}-N3.wat` + `counter-client-capability-proof.wat`): server-id + user-id types flip from `:String` to `:wat::core::Uuid`. Wire protocol stays EDN-readable via slice 2's edn_shim fix. Arc 206 uuid tests deleted (namespace verbs retired). USER-GUIDE § 11 fully rewritten. Mode D latent gap fixed in-slice: `edn_to_typed_value_inner` missing `:wat::core::Uuid` arm (subprocess `readln -> :T` typed coercion). | Mechanical sweep guided by grep `:wat::core::uuid::` + `:wat::telemetry::uuid::`. Mode D cascade was bounded: 7-line substrate fix + subprocess test passes green. |
| **5 — closure paperwork** | SHIPPED 2026-05-17 (this commit) | INSCRIPTION (FM 11 pre-grep returns ZERO matches); DESIGN status CLOSED; 058 row in lab repo; SCORE-SLICE-5. INSCRIPTION names arc 206's wrong scope framing as the discipline failure; doctrine "grep the substrate for arms/errors/panics that name the missing type" lands as carry-forward. | Arc 206's INSCRIPTIONs stay immutable; arc 207's INSCRIPTION forward-corrects them. |

After arc 207 closes: arc 170, arc 203, lab reconstruction all unblock.

## Substrate touchpoints (preliminary; slice 1's audit refines)

- `src/runtime.rs` — wat substrate Value enum; check for typealias / newtype / variant patterns (audit target for slice 1)
- `src/check.rs` — type scheme registrations for `:wat::core::uuid::*` verbs (retired in slice 4) + `:wat::core::Uuid/*` verbs (added in slice 2)
- `src/string_ops.rs` — current `eval_uuid_v4` + `eval_uuid_v5` handlers (rewritten in slice 2 to return typed Uuid)
- `src/edn_shim.rs:404` — the `Edn::Uuid` rejection arm (fixed in slice 3)
- `crates/wat-edn/src/value.rs` — `Value::Uuid(uuid::Uuid)` variant (substrate-of-substrate; should NOT need edits unless slice 1's audit surfaces something)
- `Cargo.toml` (root wat crate) — `uuid` dep already present (added arc 206 slice 1.5)
- `docs/USER-GUIDE.md` § 11 — backward-compat section rewrites in slice 5
- `crates/wat-telemetry/wat/telemetry/uuid.wat` — alias target update in slice 4 (one-liner)
- `wat-tests/counter-service-{capability,process}-N3.wat` + `counter-client-capability-proof.wat` — server-id/user-id type flips in slice 5
- `tests/wat_arc206_uuid_substrate.rs` + `tests/wat_arc206_uuid_v5.rs` — return-type assertions update in slice 5

## Connection to broader work

Arc 207 is the third (and final, modulo future versions) UUID-substrate arc:

| Arc | Theme | Ships |
|---|---|---|
| 092 | Initial mint | `wat_edn::new_uuid_v4` in wat-edn crate |
| 206 | Substrate promotion + telemetry de-dup | `:wat::core::uuid::v4` + `v5` (String-returning) |
| **207** | **Type-level honesty** | **`:wat::core::Uuid` typed primitive; EDN read gap closed; namespace verbs retired** |

After 207 ships, the UUID story is complete at every layer: substrate verbs are type-honest, EDN reads work end-to-end, capability-token consumers can express UUID-shaped identity at check time, telemetry/arc-203/lab-reconstruction unblock.

## Discipline lesson (carried forward into INSCRIPTION when arc closes)

Arc 206's "out of scope; no current consumer demands it" framing for typed Uuid was **deferral dressed in affirmative language**. The consumer pressure was on disk at `src/edn_shim.rs:404` (literal `Edn::Uuid` rejection arm) AND in arc 206 INSCRIPTION's own admission that v5's `namespace: :String` runtime-panics on invalid input. Both are concrete substrate gaps the orchestrator chose not to see.

The discipline carry-forward: **before naming anything "out of scope; no consumer demands it," grep the substrate for arms / errors / panics that name the missing type.** If they exist, that IS the consumer pressure; the type belongs in scope.

User direction 2026-05-17 (load-bearing for arc 207's existence): *"deferral is a dishonest term."* When the right shape is known and the consumer pressure exists on disk, ship it. Do not paper over with affirmative scope-bounding.

This lesson lands in arc 207's INSCRIPTION at closure time, not here in DESIGN. DESIGN states it for the slice authors so the slicing reflects the doctrine.
