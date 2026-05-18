# BRIEF — Arc 209 Slice 1: audit + implementation strategy decision for defservice

**Predecessors:** Arc 208 SHIPPED at `f1157f1`; arc 203 DESIGN updated at `f67f7ac` consolidating demands; arc 209 DESIGN at this slice's parent dir.

**This is an AUDIT slice — pure investigation. NO substrate code edits. NO new test files. NO Cargo.toml changes. Slice 1 produces ONE artifact: `SCORE-SLICE-1.md` carrying the implementation strategy decision that slice 2 builds on.**

## Why this slice exists separate from slice 2

Per `feedback_diagnose_before_spec`: read the existing code paths before specifying the substrate change. defservice could be implemented as:
- (a) **Pure defmacro** — expands at parse time into all the hand-rolled artifacts (struct decls + dispatch loop + wrappers); arc 200 + arc 150's macro infrastructure does the heavy lifting
- (b) **Pure substrate special form** — registered in check.rs / runtime.rs with custom validation + synthesis at freeze time; bypasses defmacro
- (c) **Hybrid** — a thin defmacro that calls into substrate-side helpers for the heavy synthesis (the macro is the user-facing surface; substrate code does the actual generation)

Slice 2's BRIEF can only be honest after slice 1's audit names which strategy fits the existing substrate.

## Audits required

Each audit cites file:line for evidence per `feedback_assertion_demands_evidence`.

### Audit 1 — defmacro infrastructure

Read these + tabulate capabilities:
- `src/macros.rs` — the defmacro engine
- Arc 150 INSCRIPTION + slice docs — variadic macros (`& rest` parameters)
- Arc 200 INSCRIPTION — Vector/List splice symmetry (macro splice in both `~@` and `~`-into-Vector contexts)
- Arc 143 INSCRIPTION — computed-unquote (`~(:fn args)` evaluated at expand time)
- Arc 091 slice 8 — runtime quasiquote + struct→form

Question slice 1 answers: can defmacro (option a) generate the full set of artifacts defservice needs, given current infrastructure? Specifically:
- Can a defmacro synthesize `:wat::core::struct-restricted` declarations from a protocol map?
- Can a defmacro synthesize `:wat::core::enum` Wire + WireResp from operation lists?
- Can a defmacro synthesize wrapper defns with proper depth-3 decomposition?
- Can defmacro composition (one macro calls another) avoid blowing the depth-3 budget in the GENERATED code?

### Audit 2 — arc 203 hand-rolled pattern's structural anatomy

Read `wat-tests/counter-service-capability-N3.wat` + `counter-service-process-N3.wat` end-to-end. Catalog EVERY generated artifact:
- struct-restricted declarations (with field types + accessor restrictions)
- enum declarations (Wire variants + payloads; WireResp variants + payloads)
- ServiceError enum (variants + Vector<TypedError> chains)
- Dispatch loop (select + route + validate + handler call); identify reusable sub-shapes
- Per-operation wrapper functions (send + recv pattern; Result match arms; ServerDied/PeerDied propagation)
- Server spawn + setup code

Tabulate: for each artifact, what does the consumer write today vs what would defservice generate? What's the "domain-specific" portion (operations + handlers) vs the "boilerplate" portion (everything else)?

### Audit 3 — arc 146 dispatch infrastructure

Read `src/dispatch.rs` (or equivalent). Question: does defservice's operation-name → handler-fn routing benefit from arc 146's dispatch mechanism, or is it static enough that direct match-on-keyword in generated code suffices?

### Audit 4 — arc 198 restricted_to

Read `src/check.rs:7478+` (`infer_def_restricted`) per arc 203 DESIGN line 152. Confirm: defservice's generated Admin/User struct accessor restrictions can use the same machinery (declaring `:wat::core::struct-restricted` synthesized within the defservice expansion is sufficient; no new restriction mechanism needed).

### Audit 5 — freeze-time validation strategy

How does defservice validate that every operation in `:admin` + `:user` has a corresponding handler in `:handlers`?

Options:
- **At macro expand time** — the macro pattern-matches and PANICs if a handler is missing. Cleanest; pure user-facing diagnostic.
- **At freeze/check time** — substrate-side code walks the expanded definition (struct + handlers); if mismatch, freeze panics. Useful if the validation needs to inspect type info from `check.rs`.
- **At runtime first-call** — dispatch into a missing handler raises a runtime error. Last-resort; bad UX.

Slice 1 names the right strategy. Reasonable default: expand-time (option a) because the user-facing operation list IS the macro's input; checking handler-map keys against it is mechanical at expand time.

### Audit 6 — depth-3 decomposition strategy

Per arc 203 DESIGN line 281+: substrate-generated wrappers MUST follow depth-3 by construction. What's the synthesis pattern that produces correctly-decomposed wrappers?

Likely shape: each operation's wrapper expands to a top-level function that calls 2-3 small helpers (send-and-handle, recv-and-decode, dispatch-response). The defservice macro generates the helpers + the top-level wrapper. Sonnet confirms this is feasible with current infrastructure.

## Required deliverable: `SCORE-SLICE-1.md`

Write `docs/arc/2026/05/209-defservice/SCORE-SLICE-1.md` with these sections:

### Audit findings (cite file:line for every claim)

- Audit 1: defmacro infrastructure capabilities tabulated
- Audit 2: arc 203 generated-artifact inventory; consumer-vs-generated split
- Audit 3: dispatch infrastructure relevance
- Audit 4: restricted_to mechanism confirmation
- Audit 5: validation-strategy options + recommendation
- Audit 6: depth-3 synthesis pattern feasibility

### Implementation strategy decision

The chosen strategy — (a), (b), or (c) — with explicit four-questions verdict (Obvious/Simple/Honest/Good UX; atomic YES/NO per candidate per `feedback_four_questions_yes_no`). Run the four questions on each candidate; the YES-YES-YES-YES winner is the call.

### Slice 2 surface area checklist

Concrete checklist for slice 2's implementation, derived from the strategy decision:
- Specific files to touch (defmacro source if option a; check.rs/runtime.rs if option b/c)
- Specific synthesis patterns for each generated artifact
- Specific freeze-time validation hook (if any)
- Specific tests to write for the Hello-world defservice proof

This becomes slice 2 BRIEF's source of truth — the orchestrator drafts slice 2 from this list.

### Honest deltas

Anything in arc 209 DESIGN that the audit surfaces as wrong, optimistic, or under-specified. Surface honestly; DESIGN is living per FM 13.

## HARD constraints

- DO NOT touch ANY source file (`*.rs`, `*.wat`, `*.toml`). Pure investigation slice.
- DO NOT commit. Orchestrator commits the SCORE after independently verifying.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`.
- For any cross-repo grep / read, use absolute paths or `git -C <path>`; do NOT cd.
- DO NOT extend scope to other meta-form concerns (this slice is defservice-only).
- DO NOT pre-decide the surface keywords (`:admin`, `:user`, `:state`, `:handlers`) — arc 209 DESIGN locked that. Slice 1 audits IMPLEMENTATION, not interface naming.
- If the audit surfaces an existing meta-form similar to defservice anywhere in the substrate, surface as STOP-trigger 1 — orchestrator decides whether to extend that vs mint new.

## STOP triggers

1. **Audit reveals an existing service-meta-form** anywhere on disk (substrate, wat-side, tests). If exists, slice 1's decision is moot; surface what exists + propose what arc 209 actually does instead.
2. **The three strategy candidates (a/b/c) are not the only viable shapes** — if audit surfaces a fourth, run four-questions on it inline.
3. **Defmacro infrastructure cannot synthesize one or more of the required artifacts** — slice 1 surfaces the gap; arc 209 may need a substrate-prerequisite slice (e.g., extend defmacro to handle X) before slice 2.
4. **Freeze-time validation requires new substrate hooks** beyond what exists — slice 1 surfaces; orchestrator decides extend-arc vs pre-arc.
5. **Depth-3 decomposition synthesis is structurally infeasible** at expand time — surface; orchestrator decides whether to relax the rule for generated code OR add a substrate facility.

## SCORE methodology

3 high-level rows (atomic YES/NO; no "medium"):

| Row | Evidence |
|---|---|
| A — All 6 audits completed with file:line citations | Each audit section in SCORE shows specific file:line refs |
| B — Implementation strategy decision made + four-questions verdict captured | Decision (a/b/c) named + YES/NO per question with audit citations |
| C — Slice 2 substrate surface checklist produced | Concrete checklist sonnet-2 can implement against without re-deciding strategy |

## Time-box

Predicted 45-75 min sonnet. Hard stop 90 min. Pure investigation; bigger surface than arc 207/208 slice 1 because the meta-form has more moving parts.

## On completion

Return summary: strategy decision (a/b/c), 1-paragraph rationale, surface checklist count, any STOP-triggers fired, any honest deltas.

You are launching now. T-minus 0.
