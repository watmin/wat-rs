# BRIEF — Arc 207 Slice 1: substrate audit + shape decision for `:wat::core::Uuid`

**Predecessor:** Arc 206 closed (slice 3 INSCRIPTION at `3dd2505`). Arc 207 DESIGN at this slice's parent dir.

**This is an AUDIT slice — pure investigation. NO substrate code edits. NO new test files. NO Cargo.toml changes. Slice 1 produces ONE artifact: `SCORE-SLICE-1.md` carrying the shape decision that slice 2 implements.**

## Why this slice exists separate from slice 2

Per `feedback_diagnose_before_spec`: read the actual code path before specifying the substrate change. The wat substrate has multiple ways to mint typed primitives (typealiases, newtypes, opaque Value variants, struct-restricted forms). Slice 2's BRIEF can only be honest after slice 1's audit names which pattern fits Uuid.

## The decision slice 1 settles

How should `:wat::core::Uuid` be registered at the wat substrate level? Three candidate shapes:

- **(a) Typealias of `:String`.** `:wat::core::Uuid` is a type-system alias for `:wat::core::String`. Runtime: same Value variant. Check-time: distinct type name; can't be passed where `:String` is expected (and vice versa). Lightest weight. Cheapest implementation.
- **(b) Newtype over `:String`.** `:wat::core::Uuid` wraps a `:String` with opaque-to-consumers semantics. Construction + destruction explicit (must wrap / unwrap). Runtime: same Value variant. Check-time: distinct nominal type with stronger discrimination than alias.
- **(c) New `Value::wat__core__Uuid(uuid::Uuid)` variant.** True distinct runtime value, mirroring wat-edn's `Value::Uuid(uuid::Uuid)`. Heaviest weight. Closest to Clojure's `java.util.UUID` (distinct type at runtime, not just compile time).

Per Clojure precedent (the great whose path we're following per `user_no_literature`), java.util.UUID is a distinct runtime type — Clojure's `(uuid? x)` discriminates at runtime, not just at compile time. That favors (c). But the wat substrate may have constraints / patterns that make (a) or (b) the right wat-shaped answer (the same destination via different mechanism). The audit decides.

## Audit work

Read these files + answer the questions below. NO assumptions; answers must cite file:line for the evidence.

### Audit 1 — Existing typed primitives in the substrate

Investigate how each of these is registered at the wat substrate level. For each: which Value variant carries it at runtime? How is the type registered at check time? Is it typealias-of-X / newtype-over-X / opaque substrate variant / something else?

- `:wat::core::Bytes` (typealias per arc 062 + arc 063)
- `:wat::core::Symbol` (substrate primitive — check how)
- `:wat::core::keyword` (substrate primitive — `keyword/from-string` + `keyword/to-string` per arc 109; check the value variant + check-time registration)
- `:wat::core::nil` / `:wat::core::Unit` (arc 109 slice 1d minted; check how)
- Any other typealias declared in `src/types.rs` or registered in `src/check.rs`

Suggested entrypoints: `grep -n "pub enum Value" src/runtime.rs`, `grep -n "typealias\|TypeAlias" src/types.rs src/check.rs`, `grep -n "wat__core__\|wat__std__" src/runtime.rs | head -40`.

### Audit 2 — wat-edn's `Value::Uuid` variant

`crates/wat-edn/src/value.rs:55` declares `Uuid(uuid::Uuid)`. Read the surrounding context:

- How is wat-edn's `Value` type related to the wat substrate's `Value` type? (Different types? Same? Conversion path?)
- The substrate's `src/edn_shim.rs:404` arm errors on `Edn::Uuid(_)` — read it; understand the conversion the substrate does for OTHER `Edn::*` variants (e.g., `Edn::String`, `Edn::Keyword`) to see the pattern that `Edn::Uuid` would follow.
- Does `wat-edn` impose any constraint on what shape the substrate must mint for the typed Uuid to be roundtrippable?

### Audit 3 — Dispatch infrastructure (arc 146)

Read `src/check.rs` + relevant runtime dispatch code for how generic operations (equality, comparison, hash) discover per-type implementations. The DESIGN claims "equality, comparison, hash all fall out from the substrate's dispatch infrastructure" — verify this by:

- Finding where `=` (equality) dispatches across types
- Confirming that adding a new Value variant (option c) OR a new typealias (option a) automatically lands in dispatch
- If dispatch needs an explicit registration for the new type, naming that registration in slice 2's surface area

### Audit 4 — Round-trip considerations

The DESIGN claims `:wat::core::Uuid/to-string` + `Uuid/from-string` make conversion explicit. Audit:

- The exact canonical string form `uuid::Uuid::to_string()` produces (should be `8-4-4-4-12` lowercase hyphenated; verify)
- `uuid::Uuid::parse_str()`'s tolerance (does it accept uppercase? `urn:uuid:` prefix? braces? — confirm + decide which forms `:wat::core::Uuid/from-string` accepts vs returns None)
- Whether the existing `tests/wat_arc206_uuid_substrate.rs::uuid_v4_edn_roundtrip` test's invariant changes meaning under typed Uuid (it currently asserts String equality; under typed Uuid it asserts Uuid equality — same algebraic invariant, different value comparison)

### Audit 5 — Nil-uuid shape

User confirmed nil-uuid in scope. Two implementation shapes for `:wat::core::Uuid/nil`:

- **0-arg verb** `:wat::core::Uuid/nil -> :wat::core::Uuid` — consistent with `Uuid/v4` (also 0-arg); each call produces the canonical nil value
- **Substrate constant** — registered as a typed value at substrate-init; consumers use it as an identifier expression

Audit how `uuid::Uuid::nil()` is exposed by the uuid crate (likely an associated function). Decide which shape fits wat's substrate conventions per audit 1 + 2 findings. SCORE row names the decision + reason.

## Required deliverable: `SCORE-SLICE-1.md`

Write `docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-1.md` with these sections:

### Audit findings (cite file:line for every claim)

- Audit 1 findings: per-primitive registration patterns; tabulate
- Audit 2 findings: wat-edn Value vs substrate Value; the `Edn::*` conversion pattern at `src/edn_shim.rs`
- Audit 3 findings: dispatch infrastructure verification
- Audit 4 findings: canonical string form + `parse_str` tolerance
- Audit 5 findings: nil-uuid shape recommendation

### Shape decision

The chosen shape for `:wat::core::Uuid` — (a), (b), or (c) — with explicit rationale tying back to audit findings. Run the four questions (Obvious/Simple/Honest/Good UX; atomic YES/NO per candidate per `feedback_four_questions_yes_no`) on the chosen candidate. The four-questions verdict is the load-bearing evidence; the audit citations are the supporting context.

### Slice 2 substrate surface area

Concrete checklist for slice 2's implementation, derived from the shape decision:

- Specific Value variant changes (if any) to `src/runtime.rs`
- Specific type registrations to `src/types.rs` / `src/check.rs`
- Specific eval handler changes in `src/string_ops.rs` (or wherever Uuid handlers land)
- Whether dispatch registration is needed and where
- Whether nil-uuid is a 0-arg verb or substrate constant

This becomes the source of truth for slice 2's BRIEF — the orchestrator drafts slice 2 from this list.

### Honest deltas

Anything in the DESIGN that the audit surfaces as wrong, optimistic, or under-specified. Surface honestly; the DESIGN is living per FM 13 and gets corrected forward.

## HARD constraints

- DO NOT touch ANY source file (`*.rs`, `*.wat`, `*.toml`). This is a pure investigation slice.
- DO NOT commit. The orchestrator commits the SCORE after independently verifying it.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/` (illegal per FM 7-bis).
- For any cross-repo grep / read (e.g., lab repo), use absolute paths or `git -C <path>`; do NOT cd.
- DO NOT extend scope to other typed primitives (this slice is Uuid-only).
- DO NOT pre-decide naming questions the DESIGN already locked. Naming surface (`Uuid/v4`, `Uuid/v5`, `Uuid/from-string`, `Uuid/to-string`, `Uuid/nil`) is settled. Audit informs HOW to register them, not WHAT they're called.
- If you find a substrate ambiguity that the audit doesn't resolve (e.g., two equally-honest registration patterns), surface as STOP-trigger; orchestrator decides which to pick before slice 2.

## STOP triggers (surface immediately, do NOT pick a path on your own)

1. **Audit reveals an existing `:wat::core::Uuid` registration** anywhere on disk (substrate, wat-side, tests). If it exists, slice 1's decision is moot; surface the existing registration + propose what arc 207 actually does instead.
2. **The three candidate shapes (a/b/c) are not the only viable shapes.** If audit surfaces a fourth substrate pattern that's a better fit, surface + run four questions on the new candidate inline.
3. **`Value::Uuid` at wat-edn cannot roundtrip cleanly** to whatever substrate shape you propose. If the proposed substrate shape requires a wat-edn change, surface; arc 207 should NOT modify wat-edn (per DESIGN substrate touchpoints).
4. **`Uuid/from-string`'s parse semantics are ambiguous** in a way that affects user surface (e.g., should it accept `urn:uuid:` prefix?). Surface; user direction needed.
5. **Dispatch infrastructure does NOT cover Uuid equality automatically** under the chosen shape. If so, surface + propose what slice 2 needs to add for dispatch coverage.

## SCORE methodology

3 high-level rows (atomic YES/NO; no "medium"):

| Row | Evidence |
|---|---|
| A — All 5 audits completed with file:line citations | Each audit section in SCORE shows specific file:line refs |
| B — Shape decision made + four-questions verdict captured | Decision (a/b/c) named + YES/NO per question with audit citations as evidence |
| C — Slice 2 substrate surface checklist produced | Concrete checklist sonnet-2 can implement against without re-deciding shape |

## Time-box

Predicted 30-45 min sonnet. Hard stop 60 min. Pure read + write; no compile cycles needed.

## On completion

Return summary: shape decision (a/b/c), 1-paragraph rationale, surface checklist count, any honest deltas the audit surfaced. Orchestrator reads SCORE-SLICE-1 + independently verifies before drafting BRIEF-SLICE-2.

You are launching now. T-minus 0.
