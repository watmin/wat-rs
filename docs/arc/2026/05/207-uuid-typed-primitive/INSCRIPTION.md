# Arc 207 INSCRIPTION — `:wat::core::Uuid` typed primitive

**Status:** SHIPPED 2026-05-17. Typed `:wat::core::Uuid` primitive minted end-to-end: `Value::wat__core__Uuid` variant, 5 verbs, `hashmap_key` arm, full EDN roundtrip, arc 206 namespace verbs retired, arc 203 demos rippled, Mode D latent gap closed.

## What arc 207 gave the substrate

Arc 207 completed the UUID story at every layer. Where arc 206 promoted UUID minting to substrate-core but returned `:String`, arc 207 makes the type honest:

| Verb | Signature | Notes |
|---|---|---|
| `:wat::core::Uuid/v4` | `[] -> :wat::core::Uuid` | Random 122-bit; replaces `:wat::core::uuid::v4` |
| `:wat::core::Uuid/v5` | `[:wat::core::Uuid, :wat::core::String] -> :wat::core::Uuid` | Deterministic SHA-1; namespace is type-enforced (eliminates runtime-panic foot-gun) |
| `:wat::core::Uuid/from-string` | `[:wat::core::String] -> :Option<:wat::core::Uuid>` | Parse; canonical-only; `None` on invalid (no panic) |
| `:wat::core::Uuid/to-string` | `[:wat::core::Uuid] -> :wat::core::String` | Renders canonical 8-4-4-4-12 hyphenated form |
| `:wat::core::Uuid/nil` | `[] -> :wat::core::Uuid` | Well-known nil UUID; useful in process-tier where forms block cannot capture runtime-minted values |

Plus:

- `Value::wat__core__Uuid(uuid::Uuid)` variant — Pattern B per `keyword`/`Instant`/`Duration` precedent; dedicated opaque variant, no TypeDef alias in `types.rs`
- `hashmap_key` arm for typed Uuid — surfaced in-scope when telemetry test demanded `HashSet<:Uuid>` (slice 3; substrate-as-teacher cascade ran correctly)
- `values_equal` arm — `(= u1 u2)` works through existing dispatch; equality is by Uuid content; a `:String` holding a UUID canonical form does NOT equal a `:Uuid` holding the same value
- EDN `#uuid "..."` reader-literal end-to-end roundtrip — read arm (`Edn::Uuid → Value::wat__core__Uuid`) and write arm (`Value::wat__core__Uuid → OwnedValue::Uuid`) both wired in `edn_shim.rs`
- `edn_to_typed_value_inner` `:wat::core::Uuid` arm — the typed coercion path used by `readln -> :T` (slice 2's latent gap; surfaced and closed in slice 4 when subprocess `readln -> :counter::Wire` decoded `#uuid "..."` for a typed Uuid field)
- Arc 206's `:wat::core::uuid::v4` + `:wat::core::uuid::v5` namespace verbs RETIRED entirely — no parallel keep-both per `feedback_refuse_easy_solutions`
- `:wat::telemetry::uuid::v4` alias retargeted to `:wat::core::Uuid/v4` — return type `:String` → `:Uuid` is a breaking change for the caller, but honest: the alias now says what it returns
- Arc 203 capability + process demos rippled: `server-id` and `user-id` fields in `counter-service-capability-N3.wat`, `counter-service-process-N3.wat`, `counter-client-capability-proof.wat` are now typed `:wat::core::Uuid` throughout; secret-witness security model is type-honest in test setup
- USER-GUIDE § 11 rewritten to teach the typed surface: constructors, accessors, nil-uuid strategy for process-tier, EDN roundtrip, v4-vs-v5 guidance, backward-compat retirement note

## Slices

| Slice | Commit | What |
|---|---|---|
| **1 — substrate audit + shape decision** | `1aed75e` (slice 1 SHIPPED, in slice 2 BRIEF commit) | Audit complete (SCORE-SLICE-1.md). Shape decision: option (c) — `Value::wat__core__Uuid(uuid::Uuid)` variant (Pattern B). Four-questions YES YES YES YES. Audit also surfaced: edn_shim fix mechanically inseparable from slice 2; equality requires explicit `values_equal` arm (not automatic); `from-string` strict canonical-only; `hashmap_key` latent gap named for slice 3 in-scope if telemetry demands it. |
| **2 — mint type + 5 verbs + edn_shim fix** | `a961112` | 21-item surface area: `Value::wat__core__Uuid` variant, `type_name()` arm, `values_equal` arm, dispatch wiring (5 arms), `render_value` arm, `closure_extract.rs` arm, `edn_to_value` read fix (replaces `EdnReadError::Other`), `value_to_edn_with` write arm, 5 type scheme registrations in `check.rs`, 5 eval handlers in `string_ops.rs`. 10 tests pass (`tests/wat_arc207_uuid_typed.rs`). |
| **3 — retire namespace verbs + retarget telemetry + hashmap_key in-scope** | `5f9d370` | `check.rs` uuid::v4 + v5 registrations removed; `string_ops.rs` `eval_uuid_v4` + `eval_uuid_v5` handlers removed; `runtime.rs` dispatch arms removed; `hashmap_key` arm added (`"U:{canonical}"` key format) when telemetry test (`HashSet<:Uuid>`) surfaced demand (STOP trigger 3; in-scope ADD per BRIEF direction); telemetry alias retargeted; arc 206 test files deleted. |
| **4 — consumer ripple + USER-GUIDE + Mode D latent gap fix** | `3865569` | USER-GUIDE § 11 rewritten (~58 → ~103 lines); arc 203 capability-N3, process-N3, client-capability-proof flipped to typed `:Uuid`; constant-id strategy: capability-tier uses `Uuid/v4` mint captured in closure, process-tier uses `Uuid/nil` as forms-block-safe constant; `edn_to_typed_value_inner` `:wat::core::Uuid` arm added (Mode D latent gap; subprocess `readln -> :T` typed coercion was missing the arm). |
| **5 — closure paperwork** | (this commit) | INSCRIPTION + DESIGN status CLOSED + 058 changelog row + SCORE-SLICE-5. |

## Substrate touchpoints (final inventory)

| File | Arc 207 change | Commit |
|---|---|---|
| `src/runtime.rs` | `Value::wat__core__Uuid(uuid::Uuid)` variant + `type_name()` arm + `values_equal` arm + dispatch wiring (5 arms) + `render_value` arm + `hashmap_key` arm | `a961112`, `5f9d370` |
| `src/edn_shim.rs` | `edn_to_value` read arm (fixes `edn_shim.rs:404` `EdnReadError`); `value_to_edn_with` write arm; `edn_to_typed_value_inner` `:wat::core::Uuid` arm (Mode D) | `a961112`, `3865569` |
| `src/check.rs` | 5 type scheme registrations (`Uuid/v4`, `Uuid/v5`, `Uuid/from-string`, `Uuid/to-string`, `Uuid/nil`); removed arc 206 uuid::v4 + v5 registrations | `a961112`, `5f9d370` |
| `src/string_ops.rs` | `is_canonical_uuid_string` helper; 5 eval handlers (`eval_uuid_typed_v4/v5/from_string/to_string/nil`); removed `eval_uuid_v4` + `eval_uuid_v5`; module doc updated | `a961112`, `5f9d370` |
| `src/closure_extract.rs` | `Value::wat__core__Uuid` arm — portable encoding as `(:wat::core::Uuid/from-string "canonical")` WatAST | `a961112` |
| `src/types.rs` | NOT touched — Pattern B: no `register_builtin` for opaque-variant typed primitives | — |
| `crates/wat-telemetry/wat/telemetry/uuid.wat` | Alias body + type sig retargeted to `Uuid/v4` (`:String` → `:Uuid`) | `5f9d370` |
| `crates/wat-telemetry/wat-tests/telemetry/uuid.wat` | `HashSet<:String>` → `HashSet<:Uuid>` (STOP trigger 3 in-scope ripple) | `5f9d370` |
| `crates/wat-telemetry/src/lib.rs` | Prose comments updated | `5f9d370` |
| `docs/USER-GUIDE.md` § 11 | Full subsection rewrite — typed surface, constructors, accessors, nil strategy, EDN roundtrip, backward-compat note | `3865569` |
| `wat-tests/counter-service-capability-N3.wat` | Type flips (server-id, user-id, Wire, AdminReq/Resp, RegistryEntry); closure-captured `Uuid/v4` mint; forge test uses `Uuid/nil` | `3865569` |
| `wat-tests/counter-service-process-N3.wat` | Type flips; process-tier uses `Uuid/nil` as forms-block-safe constant server-id | `3865569` |
| `wat-tests/counter-client-capability-proof.wat` | server-id + user-id field type flips; `Uuid/v4` mints in test setup | `3865569` |
| `tests/wat_arc207_uuid_typed.rs` | NEW — 10 tests covering all typed surface | `a961112` |
| `tests/wat_arc206_uuid_substrate.rs` | DELETED (arc 206 namespace verbs retired) | `5f9d370` |
| `tests/wat_arc206_uuid_v5.rs` | DELETED (arc 206 namespace verbs retired) | `5f9d370` |

## Arc 207 intentionally does NOT cover

- **Other UUID versions (v1/v3/v7/v8).** Arc 206's DESIGN documented the 3-step mechanical pattern for adding versions (feature flag + `new_uuid_v<N>` wrapper + substrate verb registration); the type-return-flip is one line per version. Flipping non-existent verbs is anti-work. Arc 207 ships v4 and v5 because those are the only versions with live consumers. No arc number is reserved for other versions; a new arc opens only when a concrete consumer arrives with its own shape.
- **`#uuid "..."` as wat-syntax-level reader literal.** Arc 207 intentionally does NOT cover this because the EDN read path already handles reader-literal semantics end-to-end (`(:wat::edn::read "#uuid \"...\"")` → typed `:Uuid`). A wat-syntax-level literal is a parser-layer concern entirely separate from the substrate work arc 207 performs. No consumer has brought concrete demand for a wat-syntax `#uuid` form distinct from the EDN path.
- **`Uuid/version` extraction.** Arc 207 intentionally does NOT cover this because UUID is an identifier; its construction technique is invisible to comparators and consumers. Per the user's own reasoning 2026-05-17: *"if i have a uuid do i care how it was constructed?"* — No. The concern is architectural, not a gap waiting to be closed.
- **`uuid?` predicate verb.** Arc 207 intentionally does NOT cover this because substrate dispatch handles type-predicates polymorphically; minting a type-specific predicate verb when polymorphic dispatch covers it is wrong shape per `feedback_no_new_types`.
- **`values_compare` arm (Uuid ordering).** Arc 207 intentionally does NOT cover this because UUIDs are identifiers not ordinals. The same architectural reason applies to `keyword`, `Enum`, and `Struct` — none have `values_compare` arms. A new arc opens only when a concrete consumer arrives needing lexicographic-UUID ordering with its own design input.
- **Lenient `Uuid/from-string` parsing (URN, braced, simple-form, uppercase).** Arc 207 intentionally does NOT cover this because strict canonical-only (`from-string`) matches the EDN-layer's own strictness policy (wat-edn's `is_canonical_uuid` enforces the same constraint). The round-trip invariant holds: `to-string` always produces canonical form; `from-string` accepts only canonical form. A consumer needing URN-form parsing opens a new arc with that specific input shape as the contract.

## Discipline lessons inscribed

### The forward-correction of arc 206

Arc 206's INSCRIPTION (`docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION.md`, immutable per `feedback_inscription_immutable`) named typed `:wat::core::Uuid` as:

> "Out of arc 206 scope; not tracked elsewhere because no current consumer demands it."

That framing was wrong. The consumer pressure was on disk when arc 206 closed:

1. **`src/edn_shim.rs:404` rejected `Edn::Uuid(_)` with `EdnReadError::Other(...)`** — a substrate arm literally waiting to be filled. When user wat code calls `(:wat::edn::read "#uuid \"...\"")`, the substrate errored instead of producing a typed Uuid. That IS consumer pressure; the arm named the missing type.
2. **`:wat::core::uuid::v5`'s `namespace` parameter was `:wat::core::String` and runtime-panicked on invalid UUID input** — a foot-gun documented in the USER-GUIDE. A typed `:Uuid` parameter would have pushed the validation gate to check time (no panic). The GUIDE itself named the missing type.

User direction 2026-05-17: *"that is naming something dishonestly."* Arc 207 forward-corrects the discipline failure.

Arc 206's INSCRIPTIONs stay immutable as historical record per `feedback_inscription_immutable`. This INSCRIPTION carries the correction forward.

### Carry-forward doctrine (quotable)

> **Before marking any type as having no consumer pressure, grep the substrate for arms / errors / panics that name the missing type. If they exist, that IS the consumer pressure; the type belongs in scope.**

The concrete test: `grep -rn "EdnReadError\|assertion-failed!\|runtime-panic\|TypeMismatch" <relevant-files>` near any arm where a not-yet-typed value is handled. An error arm that rejects a value type because the type does not exist yet is the substrate telling you what it needs.

### Substrate-as-teacher cascade ran cleanly across slices 3 and 4

Per `feedback_no_known_defect_left_unfixed`: when a consumer's actual code path surfaces a substrate gap, the right move is in-scope fix at the slice that surfaced it, not a punt to a subsequent arc.

**Slice 3 cascade:** Telemetry test (`crates/wat-telemetry/wat-tests/telemetry/uuid.wat`) stored alias results in `HashSet<:String>`. When the alias retargeted to return `:Uuid`, the element type mismatched. The test surfaced the `hashmap_key` gap named in SCORE-SLICE-1 Delta 3 as "latent, no active consumer." Consumer arrived in slice 3. In-scope ADD: `hashmap_key` arm for `Value::wat__core__Uuid` + telemetry test type flip. STOP trigger 3 fired and resolved correctly.

**Slice 4 cascade (Mode D):** Arc 203 process-tier demo called `(:wat::kernel::readln -> :counter::Wire)` where `Wire::Admin.server-id` is `:wat::core::Uuid`. The subprocess read a `#uuid "..."` EDN string from the wire, the typed coercion path (`edn_to_typed_value_inner`) hit the `_` wildcard → mismatch → subprocess exited → parent's `recv` returned `channel disconnected`. The symptom was indirect; the root was `edn_to_typed_value_inner` missing a `:wat::core::Uuid` arm (slice 2 latent gap — `edn_to_value` was fixed but not its typed-coercion sibling). In-scope fix: 7 lines adding the `:wat::core::Uuid` arm to `edn_to_typed_value_inner`. Subprocess now decodes `#uuid "..."` into a typed Uuid; wire protocol survives the full roundtrip.

Both cascades demonstrate the same pattern: the substrate surfaces a gap through a downstream consumer's failure; the gap is fixed at the substrate, not bridged at the test. This is the discipline.

## Cross-references

- **Arc 092** (`docs/arc/2026/04/091-substrate-quality-gates/INSCRIPTION.md`) — initial `uuid::v4` mint under `wat-measure`
- **Arc 206** (`docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION.md` + `INSCRIPTION-SLICE-3.md`) — substrate promotion + telemetry de-dup; these INSCRIPTIONs are immutable historical record; arc 207 forward-corrects arc 206's wrong scope framing
- **Arc 203** (`docs/arc/2026/05/203-struct-restricted/DESIGN.md`) — capability pattern; consumer ripple target for arc 207 slice 4; unblocked by arc 207
- **Arc 170** — unblocked by arc 207 closure; closure paperwork resumes
- **INTERSTITIAL § seven-greats convergences** — wat-MCP entry; HolonAST-as-universal-AST strange-loop note
- `feedback_inscription_immutable` — arc 206 INSCRIPTIONs stay unchanged; this INSCRIPTION forward-corrects
- `feedback_refuse_easy_solutions` — no parallel keep-both of namespace verbs; alias chain is the honest backward-compat pattern (arc 206 slice 3's corrected lesson applied here)
- `feedback_no_known_defect_left_unfixed` — slice 3 hashmap_key in-scope ADD; slice 4 edn_to_typed_value_inner in-scope fix
- `feedback_wat_namespace_principle` — Type/verb naming (`Uuid/v4`, `Uuid/from-string`) per CONVENTIONS.md

---

Arc 207 inscribed. 2026-05-17.
