# EXPECTATIONS — Arc 209 Slice 1

## Mode prediction

**Mode A — clean audit, strategy decided, surface checklist produced (~65%).** Sonnet reads the 6 audit targets, names the substrate's current capabilities, picks strategy (a/b/c) with four-questions YES YES YES YES, produces concrete slice-2 checklist. ~50-70 min wall-clock. Likely outcome: option (c) hybrid — defmacro for the user-facing surface (`:wat::service::defservice` is a macro); substrate-side helper functions in `src/macros.rs` or new `src/service.rs` handle the heavy synthesis (struct-restricted + enum + dispatch + wrappers); freeze-time validation in `src/check.rs` cross-checks handlers against operations.

**Mode B — STOP-trigger fires (~25%).** Most likely:
- Audit 5 (validation strategy) reveals freeze-time validation needs substrate hooks beyond what exists — prerequisite slice needed
- Audit 6 (depth-3 synthesis) is harder than expected at expand time — orchestrator decides whether to relax for generated code or add substrate facility
- Audit 2 (arc 203 anatomy) surfaces a structural element that doesn't compose cleanly into a meta-form (e.g., a per-service customization that can't be parameterized)

Sonnet surfaces, orchestrator decides, sonnet completes the audit with the decision encoded.

**Mode C — fourth strategy candidate surfaces (~7%).** Audit reveals a wat-specific substrate pattern the BRIEF didn't enumerate (e.g., a substrate-level "service template" mechanism that already exists in proto form). Sonnet runs four questions inline on the new candidate; if it wins, surface to orchestrator for confirmation.

**Mode D — existing meta-form found (~3%).** STOP-trigger 1 fires; arc 209 redefines around the existing surface rather than minting new.

## Strategy-decision prediction

Orchestrator's guess: **option (c) — hybrid (defmacro user-facing surface + substrate-side synthesis helpers).**

Reasoning sonnet's audit should either confirm or overturn:
- The user-facing surface is naturally macro-shaped (compile-time AST transformation of operation maps into multiple synthesized definitions)
- The synthesis logic is non-trivial (multiple inter-related artifacts; depth-3 decomposition; ServiceError variant generation; per-tier transport adapter selection) — pure defmacro might produce un-readable macro body
- Freeze-time validation benefits from substrate-side check.rs access (validating handler signatures against operation signatures requires type-system access)
- Pure substrate special form (option b) bypasses defmacro's existing infrastructure (variadic params + splice symmetry + quasiquote) — duplicates effort
- Hybrid mirrors how complex Clojure macros work in practice: thin user-facing macro that calls into a more substantial implementation

Sonnet's audit may overturn — if synthesis is mechanically simple enough that defmacro alone handles it cleanly, option (a) is honest. Trust the audit.

## Workspace baseline expected

Sonnet should NOT run cargo test (no compile changes). Sonnet may run `cargo check` if needed to verify substrate's current state, but shouldn't burn time on the test suite.

## Out-of-scope findings (surface, don't act)

These may surface during audit; surface in SCORE honest-deltas, don't act:
- Other arc 203 hand-rolled patterns (lab-side, holon-rs-side) — slice 5 ripple territory, not slice 1
- Macro infrastructure gaps that could benefit defservice but aren't blocking — surface as potential future arc
- Naming convention drift in `:wat::service::*` namespace (none today; first occupant) — slice 1 doesn't decide naming

## Failure-mode catches

- FM 1 (grep before claiming): every audit assertion cites file:line
- FM 11 (deferral language): N/A for this slice — no INSCRIPTION yet
- FM 14 (surface retirement leaving leftovers): N/A — this is mint slice, not retirement
- FM 16 (no tool preamble): BRIEF doesn't mention Bash/cargo availability
- `feedback_assertion_demands_evidence`: every audit claim cites file:line; if sonnet writes "the substrate has X" without a file:line, that's the failure pattern to catch

## What "done" means for this slice

SCORE-SLICE-1.md exists; 3 SCORE rows YES; strategy decision (a/b/c) named with four-questions verdict; slice 2 surface checklist concrete enough that BRIEF-SLICE-2 can be drafted from it without re-deciding strategy.

## Atomic commit shape

NO commit by sonnet. Orchestrator commits BRIEF + EXPECTATIONS + SCORE atomically when sonnet returns.

## Calibration record

Arc 207 slice 1 (audit, similar shape): 36 min. Arc 209 slice 1 predicted larger (more audit surface; 6 audits vs 5). Honest range: 50-75 min. Hard stop 90 min.

If sonnet returns under 40 min on 6 audits, that's calibration data for over-specification.

Sonnet: trust the audit; trust the disk; produce the checklist; return.
