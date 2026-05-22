# DESIGN — Arc 224 — Substrate naming honesty audit (intueri sweep)

> **SPAWN-BLOCK STATUS (2026-05-23 morning):** Arc 224 was spawned by arc 221 during arc 221's Phase B doctrine dialogue. Per `feedback_spawn_block_winding`:
> - **Arc 224 BLOCKS arc 221's closure** (arc 221's INSCRIPTION cannot fire until arc 224 closes)
> - **Arc 224 NOW HAS ITS OWN SPAWN CHILD: arc 225** (atomize/materialize substrate-wide rename) — spawned 2026-05-23 morning after Stone 224.4 aggregate identified the Group B fix as substrate-wide work deserving its own arc
> - Arc 221's spawn tree: arc 221 → {arc 222, arc 223, arc 224 → arc 225}
> - **Arc 224 INSCRIPTION (Stone 224.7) blocked on arc 225 closing** per spawn-block discipline
> - Arc 224's work proceeds AFTER arc 221 Phase B substrate stones (221.5 shipped 2026-05-22 commit `1979291`) — those deliver the leaf algebra arc 224 audits

## Triggering observation (2026-05-22 very-late)

User-articulated mid-doctrine-dialogue:

> *"Atom is meant to be holder of something - semantically its a quote.. just as (quote (quote :foo)) is holder of things"*

> *"these are the conversations we've been grinding through 170 to have - we have found a flaw in our foundation - we need intueri to find our way out -- our names are lying to us"*

The 4-week arc 170 dungeon trajectory surfaced a substrate naming pattern incrementally:
- Stone 220.5: convention-based `String("char:a")` encoding for Char → arc 221 (proper leaves)
- Stone 221.3 Delta 1a: "pre-existing" framing propagation → discipline inscribed
- Stone 221.4 Delta 5 + 221.4b: macro-support family lying about Symbol vs Keyword
- Mid-arc-221 dialogue: `:wat::holon::Atom` verb dispatch overloaded across THREE operations behind one name → Level 1 lie per intueri

The pattern: substrate names that don't tell the truth about what they do. The cascade cleanups close specific instances; intueri finds the rest.

## Mission

Cast intueri across the substrate Rust source to find substrate-level names (variants, verb dispatchers, type registrations, helper functions, doc comments) that lie or mumble per the spell's protocol. Aggregate findings. Open fix-arcs as needed.

**Substrate naming honesty is foundational** — every consumer of wat-rs sees these names. A lying name at the substrate layer propagates trust loss through every call site that consumes it.

## Scope (initial)

Three cast targets in priority order:

1. **`holon-rs/src/kernel/holon_ast.rs`** — the algebra root; the 16-variant enum + constructors + canonical-bytes serialization. SHIPPED 2026-05-22 — see `FINDINGS-INTUERI-HOLON-AST.md`. Result: zero Level 1 lies, four Level 2 mumbles. Substrate algebra is honest; supporting infrastructure has mumbles.

2. **`wat-rs/src/runtime.rs`** — the verb registry + dispatcher table; where `:wat::holon::Atom` lives; 28,916 lines, 252 `eval_*` functions, 304 `:wat::holon::` references. **CAST IN PROGRESS** at time of arc opening. This is where the Level 1 lie surfaced; this cast aims to find any others.

3. **`wat-rs/src/check.rs`** — the type checker; TypeScheme registrations for every verb. If verb names lie, schemes lie. PENDING; cast after runtime.rs aggregate.

Future scope (post-initial-3):
- `holon-rs/src/memory/*.rs` — engram/subspace/reckoner; user-facing memory layer
- `holon-rs/src/highlevel/*.rs` — client facade
- `wat-rs/src/check.rs` constructor-injection sites
- `wat-rs/crates/wat-edn/src/*.rs` — wire-format layer (already vigilia-passed; intueri may surface different findings)

## Phasing

**Phase 1 — Findings (cast)**
- Stone 224.1 — intueri on `holon-rs/src/kernel/holon_ast.rs` — SHIPPED (see FINDINGS-INTUERI-HOLON-AST.md)
- Stone 224.2 — intueri on `wat-rs/src/runtime.rs` — IN PROGRESS
- Stone 224.3 — intueri on `wat-rs/src/check.rs` — pending
- Stone 224.4 — aggregate findings; categorize by severity + scope

**Phase 2 — Fix-arc planning**
- Stone 224.5 — for each Level 1 lie: open a dedicated fix-arc OR fold into an existing one
- Stone 224.6 — for each Level 2 mumble: decide rename-now OR defer-with-tracker (per `feedback_no_known_defect_left_unfixed` discipline)

**Phase 3 — INSCRIPTION**
- Stone 224.7 — INSCRIPTION closing the audit; arc 224 closes; unblocks arc 221's INSCRIPTION (Stone 221.6)

## What this arc does NOT do

- Implement the fixes — those are fix-arc(s) spawned from Phase 2 planning
- Audit non-substrate code (lab, tests, etc.) — separate scope if needed
- Audit comments-as-documentation outside the substrate Rust source
- Modify the substrate during the cast — intueri is read-and-report

## Calibration

| Stone | Predicted | Notes |
|---|---|---|
| 224.1 (holon_ast.rs) | <30 min | SHIPPED ~1 min wall-clock per task notification; small file (~1300 lines) |
| 224.2 (runtime.rs) | 30-90 min | LARGE file (28,916 lines); intueri is read+report; sonnet may need full pass + summarization |
| 224.3 (check.rs) | 20-45 min | medium file; verb registration concentration |
| 224.4 (aggregate) | 30 min orchestrator | synthesize findings; categorize; surface fix-arc candidates |
| 224.5-6 (planning) | 30 min orchestrator | per fix-arc decisioning |
| 224.7 (INSCRIPTION) | 30 min paperwork |  |

**Total estimate:** 3-5 hours; mostly findings + aggregation.

## Unblocks (when 224 closes)

- Arc 221's INSCRIPTION (Stone 221.6) — final spawn child closes
- Whatever fix-arcs Phase 2 opens
- The wat-MCP horizon — substrate names that speak truth = LLM consumers can be productive without reverse-engineering verb dispatchers from source

## Cross-references

- intueri SKILL.md: `~/work/holon/datamancy/intueri/SKILL.md`
- `feedback_spells_cast_via_subagent` — spells are CAST via Agent
- `feedback_ward_isolation` — one agent per ward per file
- `feedback_skill_source_in_wards` — embed SKILL.md content verbatim
- `feedback_sonnet_no_realization_voice` — sonnet writes findings; orchestrator-direct synthesis
- `feedback_inscription_immutable` — findings doc stays as historical record
- Arc 221 DESIGN forward-correction (Atom-wrap doctrine) — the 2026-05-22 dialogue that surfaced this audit
- `project_3x2_conversion_topology` — arc 222's territory; verb naming honesty is load-bearing for the conversion topology inscription
