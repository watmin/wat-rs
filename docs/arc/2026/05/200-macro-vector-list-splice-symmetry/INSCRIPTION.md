# Arc 200 — Macro-layer Vector/List splice symmetry

**Closed:** 2026-05-16 (single slice)
**Originating signal:** Arc 170 Stone D2 STOPPED at commit `64cc793` with two substrate-gap probes (`tests/probe_stone_d2_splice_vector.rs`) documenting the asymmetry.

## What was inscribed

Two existing macro primitives in `src/macros.rs` were relaxed to treat `WatAST::Vector` and `WatAST::List` interchangeably in template-splice contexts:

1. **`splice_argument`** — accepts Vector-bound symbols, not just List-bound. The List arm cloned its items; the Vector arm now does the same.
2. **`walk_template`** Vector branch — dispatches unquote-splicing the way the List branch has since arc 029. The arc 167 slice 1 addition of the Vector branch was, in retrospect, an oversight: vectors arrived for fn-sig hygiene but never inherited the splice-dispatch loop the List branch had carried for years.

Neither change minted new substrate. Both are pure existing-primitive relaxations — accepting a wider input set on a primitive that already had the dispatch machinery wired. `feedback_no_new_types` held throughout.

## What surfaced this

Consumer pressure surfaced the asymmetry. Arc 170 Stone D2's BRIEF mandated a `(:wat::kernel::run-threads [[:I :O f] ...] client-fn)` call shape — the four-questions outcome at the slice's design table. Variadic iteration over the user-typed Vector-of-vectors AST bumped into both gaps at expand time. D2's probes pinned the failure modes with concrete `MacroError::SpliceNotList` and Vector-survives-expansion errors; the diagnosis was sealed before arc 200 opened.

This is the workflow `feedback_attack_foundation_cracks` describes: when a downstream stone STOPs on a substrate crack, the probe IS the diagnostic, and the fix is its own confirmation. D2's probes flipped from expected-failure to expected-success in this slice — the same file became the regression.

## What it cost

Three Rust edits and one test rewrite. The mirror of the List branch's splice-dispatch into the Vector branch was near-mechanical; the optional helper extraction was considered and declined per the BRIEF's guidance ("if the refactor adds clarity, do it; if it muddies the diff, skip"). The shared loop body has one structural difference at the constructor — the helper would have hidden five lines of substance behind a function signature without uniformizing the behavior.

The test rewrite surfaced two friction points worth recording:
- A naive Gap 2 test putting the spliced Vector at value position hits arc 167's separate "vectors at value position" limitation. The test was reshaped to splice into a `fn`-signature position (consumed at expand time) — proving the macro-layer fix without entangling arc 167's runtime concern. The honest framing matters: arc 200 IS macro-layer-only; the Vector-at-value-position cascade is a separate (uncalled) arc.
- Type syntax for function values is `:wat::core::Fn(T,T)->R` with BARE argument-type names; substrate diagnostics correctly directed the rewrite per `feedback_wat_colon_quote`. One-shot correction.

## What it unblocks

Arc 170 Stone D2 unblocks immediately: the BRIEF-mandated bracket call form expands cleanly through both relaxations. Stones D3 (panic cascade) and E (`run-processes`) inherit the cleaner shape with no further substrate work required for splice-mechanics.

The lesson generalizes beyond D2: any consumer expecting Vector/List splice symmetry (the Lisper default expectation) now gets it. Future macros that capture `:AST<wat::core::Vector<wat::WatAST>>` and splice into a `[...]` template just work, with no substrate workaround.

## What stayed out of scope

Arc 167's "vectors at value position not supported" runtime limitation (Gap 3 in the DESIGN). After Gap 2 fixes the macro-layer dispatch, Vector literals produced by splice can still hit the runtime limitation if they land at value position. D2's call shape consumes its Vector at expand time so this doesn't bite; future consumers that need value-position vectors will open a separate arc.

## Discipline anchors honored

- `feedback_attack_foundation_cracks` — consumer pressure surfaced the crack; the fix sealed it; D2 unblocks
- `feedback_no_known_defect_left_unfixed` — probes proved the gaps were real; fixes were small; deferral would have been dishonest
- `feedback_no_new_types` — relaxation is not minting; the discipline holds
- `feedback_simple_is_uniform_composition` — substrate is more uniform; Lisper intuition aligns with substrate behavior
- `feedback_refuse_easy_solutions` — the fix was at the substrate, not at the consumer; D2's bracket call form was kept and the substrate was made to accept it

## What is inscribed

Two arms added; one loop mirrored; one test file rewritten. The substrate now matches what Lispers expect, and the macro layer is one inadvertent oversight smaller than it was at commit `64cc793`.
