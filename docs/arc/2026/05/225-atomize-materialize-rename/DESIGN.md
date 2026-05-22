# DESIGN — Arc 225 — `:wat::holon::Atom` → `atomize` / `:wat::core::atom-value` → `materialize` substrate-wide rename

> **SPAWN-BLOCK STATUS (2026-05-23 morning):** Arc 225 is spawned by arc 224 (substrate naming honesty audit) per `feedback_spawn_block_winding`:
> - **Arc 225 BLOCKS arc 224's closure** — arc 224's INSCRIPTION (Stone 224.7) cannot fire until arc 225 closes
> - Arc 224's spawn tree as of 2026-05-23 morning: arc 224 → arc 225
> - The chain: arc 220 ← arc 221 ← arc 224 ← arc 225 (depth 4)
>
> Arc 225 is the FIX-ARC for the load-bearing Level 1 lie surfaced by arc 224 Stone 224.2 (runtime.rs cast): `:wat::holon::Atom` verb is polymorphic across 9 input arms; sibling `:wat::core::atom-value` decodes Bundles too. The honest pair `atomize` / `materialize` names the boundary-crossing direction.

## Triggering observation

User-articulated 2026-05-22 very-late mid-doctrine-dialogue: *"Atom is meant to be holder of something - semantically its a quote.. just as (quote (quote :foo)) is holder of things"* + *"we have found a flaw in our foundation - we need intueri to find our way out -- our names are lying to us."*

Intueri cast on `wat-rs/src/runtime.rs` (Stone 224.2) confirmed the L1 lie at line 13820 (`:wat::holon::Atom` polymorphic across 9 arms; most produce shapes that are NOT HolonAST::Atom) + identified the family pattern extending to `:wat::core::atom-value` (decodes Bundles too).

The honest pair per intueri's family-pattern finding:

| Current (lying) | Proposed (honest) | Direction |
|---|---|---|
| `:wat::holon::Atom` | `:wat::holon::atomize` | lift any runtime value INTO the algebra |
| `:wat::core::atom-value` | `:wat::holon::materialize` | lower any HolonAST OUT of the algebra to runtime |

Each verb names DIRECTION across the algebraic boundary; polymorphism admitted; no borrowed variant names. The arc 065/221 splits (`:wat::holon::leaf`, `:wat::holon::from-watast`) are the existing right-shape examples — one verb, one input type, one output behavior.

## Mission

Rename the boundary-crossing verb-pair substrate-wide:
- `:wat::holon::Atom` → `:wat::holon::atomize`
- `:wat::core::atom-value` → `:wat::holon::materialize` (note namespace move from `:wat::core` to `:wat::holon` — both verbs land in the same namespace, signaling they're a pair)

Wat-rs side: verb registrations (runtime.rs + check.rs), `eval_*` function names, doc comments, TypeScheme registrations, all wat-side caller sites (wat/*.wat, wat-tests/**/*.wat), Rust-side test fixtures, USER-GUIDE, BOOK, README references.

Holon-rs side: no changes expected (the algebra primitive `HolonAST::Atom` STAYS as the variant name — intueri's 224.1 cast confirmed it's honest at the substrate; the lie is at the wat-rs verb layer above).

## Scope

### Phase 1 — Substrate rename (Stone 225.1)

- `src/runtime.rs` — `eval_algebra_atom` → `eval_holon_atomize`; `value_to_atom` → `value_to_holon` OR `atomize_value` (TBD per intueri's earlier note about overlap with `value_to_holon` at line 20983); dispatch table entry; doc comments
- `src/runtime.rs` — `eval_atom_value` → `eval_holon_materialize`; `holon_item_to_value` doc updates; dispatch table entry
- `src/check.rs` — TypeScheme registration entries (lines around 13558 + 13591) renamed; special-case handlers in `infer_list` (lines 5326 + 5362) renamed; doc comments
- Move both verbs to `:wat::holon::*` namespace (atom-value migrates from `:wat::core::*`)
- Verify cascade arms in adjacent dispatchers (`eval_holon_leaf`, `eval_holon_from_watast` — these stay; they're the existing right-shape splits)

### Phase 2 — Wat-side caller sweep (Stone 225.2)

Substrate-as-teacher cascade. After Phase 1 renames the verb-registry entries, `cargo test --release` will emit cascading errors from every wat source using the old verbs. Iterate until clean. Expected sites:
- `wat/**/*.wat` — substrate-bundled wat files
- `wat-tests/**/*.wat` — test fixtures
- Per arc 159 precedent (~951 sites for `let*` retirement), expect tens-to-hundreds of caller sites; sweep mechanically; trust the cascade

### Phase 3 — Doc + USER-GUIDE + BOOK (Stone 225.3)

- USER-GUIDE.md — verb reference updates
- BOOK.md (if it references these verbs) — narrative updates
- 058 spec — `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/` — verb spec updates
- README.md — any verb-name references
- CONVENTIONS.md / WAT-CHEATSHEET.md — if verb-name examples cite these

### Phase 4 — INSCRIPTION (Stone 225.4)

- arc 225 INSCRIPTION
- Cross-references back to arc 224 (the audit that surfaced the lie) + arc 221 (the doctrine dialogue) + arc 065/221 (the existing right-shape splits)
- Closes arc 225; unblocks arc 224's Stone 224.7 INSCRIPTION

## Calibration

| Stone | Scope | Predicted | Notes |
|---|---|---|---|
| 225.1 | substrate rename | 60-120 min Mode A | mechanical; verb-registry + handlers + TypeSchemes; cascade tests EXPECTED |
| 225.2 | wat-side caller sweep | 90-180 min Mode A | substrate-as-teacher cascade; iterate until cargo test green |
| 225.3 | doc + USER-GUIDE + BOOK | 30-60 min Mode A | reference updates; low risk |
| 225.4 | INSCRIPTION | 30 min paperwork |  |

**Total estimate:** 3.5-6.5 hours sonnet wall-clock across 4 stones.

## What this arc does NOT do

- Touch holon-rs (the algebra primitive `HolonAST::Atom` STAYS — intueri 224.1 confirmed it's honest)
- Touch wat-edn (the EDN wire format doesn't directly reference these verbs)
- Rename other verbs surfaced in arc 224 findings (Group A small fixes — that's Stone 224.5's job)
- Address L2 mumbles (Stone 224.6 / future maintenance arc)

## Cross-references

- arc 224 DESIGN.md — the audit that surfaced this fix-arc
- arc 224 FINDINGS-INTUERI-RUNTIME.md — L1-1 + family pattern finding (verb body shows 9-arm polymorphism)
- arc 224 FINDINGS-INTUERI-CHECK.md — TypeScheme registrations are honest; verb-name layer is what needs rename
- arc 224 AGGREGATE-FINDINGS.md — Stone 224.4 categorization that surfaced this arc
- arc 065 — `Atom` was originally split into `leaf` / `from-watast` / narrowed Atom; this arc completes that splitting work
- arc 221 — substrate-doctrine arc that surfaced the verb-naming honesty question via doctrine dialogue
- INTERSTITIAL § 2026-05-22 very-late → 2026-05-23 — the realization narrative
- [[atom-is-holder]] memory — substrate doctrine + verb-pair direction
- [[spawn-block-winding]] — arc 225 parentage discipline
- `feedback_substrate_as_teacher` — Phase 2 sweep methodology (cargo errors = brief)

## Open questions for the BRIEF

1. **Naming alternatives reviewed?** `atomize` / `materialize` is the proposed pair. Alternatives considered but NOT chosen: `lift`/`lower`, `encode`/`decode`, `to-holon`/`from-holon`. Pending user confirmation.
2. **`value_to_atom` Rust function rename target?** Function does what `:wat::holon::atomize` verb dispatches to. Candidates: `atomize_value`, `value_to_holon` (collides with existing fn at line 20983), `lift_value_to_holon`. Pending Phase 1 BRIEF decision.
3. **`:wat::core::atom-value` namespace move?** Proposed move from `:wat::core::*` to `:wat::holon::*` (so both pair verbs live in same namespace). Pending user confirmation.
4. **Backwards compatibility?** Should the old verb names be retained as deprecated aliases for one release cycle? OR hard-cut per `feedback_no_known_defect_left_unfixed`? Recommendation: hard-cut (consistent with arc 159 / arc 162 retirement discipline).
