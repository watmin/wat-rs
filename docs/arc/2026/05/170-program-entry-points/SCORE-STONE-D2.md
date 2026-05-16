# SCORE — Arc 170 Stone D2 (multi-factory heterogeneous `run-threads`)

**Date:** 2026-05-16
**BRIEF:** `BRIEF-STONE-D2.md`
**Status:** **STOPPED — substrate gap surfaced per BRIEF § STOP triggers #1.** No D2 code shipped. Two substrate-gap probes added as evidence (`tests/probe_stone_d2_splice_vector.rs`, +2 passing tests). D1 macro + test untouched. Workspace baseline preserved.
**Time:** ~85 min (within 90 min hard stop; investigation-heavy due to the gap surfacing).

## Headline

Arc 199 REJECTED 2026-05-16 framed the substrate as already sufficient — based on the parametric-keyword-construction surface D1 needed. D2's surface (variadic iteration over a user-typed `[[:I :O f] ...]` Vector AST at expand time) crosses a DIFFERENT substrate boundary that arc 199's investigation did not cover. Two minimal substrate gaps were surfaced via concrete `freeze_result` failures:

- **Gap 1:** `~@sym` errors when `sym` is bound to `WatAST::Vector` (only `WatAST::List` accepted). Concrete error: `unquote-splicing (,@xs) requires a List argument; got vector`.
- **Gap 2:** `~@(...)` inside a `[...]` Vector template does not fire splice — Vector branch of `walk_template` lacks the unquote-splicing dispatch that the List branch has. Concrete error (different surface): `malformed <vector literal> form: vector literals at value position are not supported in arc 167`.

The BRIEF-mandated D2 call form `(:wat::kernel::run-threads [[:I :O f] ...] client-fn)` requires the macro to walk that outer Vector at expand time and emit per-spec binding clauses. With both Vector-splice paths blocked, the only ways through are: (a) RELAX the existing splice machinery to also accept Vector inputs (~5 lines each at `src/macros.rs:1081` + `src/macros.rs:926-941`) — small substrate change, NOT a new primitive; (b) ship gensym (arc 173, not yet landed) so a wat-side helper fn could fabricate fresh symbol names from indices; (c) revise the BRIEF's call form to use parens `((:I :O f) ...)` instead of brackets — Lists splice cleanly today.

Per `feedback_no_new_types` + BRIEF § STOP triggers #1 ("variadic iteration at expand time requires a new substrate primitive — STOP, surface"): I STOPPED rather than papering over with hand-rolled tricks (e.g., `extract-arg-names` against a hand-written quoted signature to mint fresh symbols; macro-driven recursion via tuple-accumulation). Both tricks "work" in narrow cases but rely on shape-fragile substrate behavior not documented as a public-facing path, and the second one runs into the same Vector-splice block at the spec-list iteration step.

## Rows

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | `:wat::kernel::run-threads` macro refactored to unified 2-arg shape `[[specs...] client-fn]` | **NO** | Macro not refactored — STOP triggered. Macro at `wat/kernel/run_threads.wat:85-112` remains the D1 4-arg shape. See § Substrate-gap narrative for the two concrete `freeze_result` failures blocking the refactor. |
| B | D1's test updated to new call form; still passes | **NO (test untouched; still green)** | D1 test `tests/wat_run_threads_d1.rs` not modified. `cargo test --release -p wat --test wat_run_threads_d1` → `test result: ok. 1 passed; 0 failed`. Refactor blocked at row A. |
| C | D2 test added; 3 heterogeneous factories round-trip via the bracket | **NO** | No D2 test added — the 3-heterogeneous-factories test cannot be written until the macro accepts the unified call form. STOPped before writing speculative code. |
| D | No new substrate types/verbs/structs minted | **YES** | `git diff src/` → only `tests/probe_stone_d2_splice_vector.rs` added (substrate-gap probe; 2 expected-failure tests demonstrating the gaps). Zero modifications under `src/`. The discipline anchor held — when the obvious path required a new substrate primitive (or a substrate relaxation outside D2 scope), I stopped rather than minted. |
| E | Workspace test failure count ≤ baseline (4) | **YES** | `cargo test --release --workspace --no-fail-fast` → exactly 4 failing test names, identical to baseline: `lifeline_pipe_zero_orphans_across_100_trials` (lifeline flake), `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`. Zero new failures attributable to D2. The probe file added +2 passes (both expected-failure assertions on substrate gaps). |

## Substrate-gap narrative

### Investigation path

1. Read D1's current macro (`wat/kernel/run_threads.wat`, post-arc-199-rejection clean form) and the precedent macro `:wat::runtime::define-alias` (`wat/runtime.wat:22-29`). Both use the arc 143 slice 2 computed-unquote pattern.
2. Walked the macro expander code in `src/macros.rs`:
   - `unquote_argument` (`:1010-1052`) — accepts a parameter symbol OR a `WatAST::List` whose head is a Keyword (computed unquote, evaluates at expand time, value_to_watast back to WatAST).
   - `splice_argument` (`:1064-1150`) — three branches: (a) Symbol bound to **List** (`:1080`); (b) **List** with Keyword head (computed unquote-splicing, evaluates, requires `Value::Vec` result, splices via `value_to_watast` per element, `:1088-1138`); (c) already-substituted **List** (`:1140`). **NO branch accepts `WatAST::Vector`** — error at `:1141-1149` for any other shape.
   - `walk_template` (`:795-946`) — List branch at `:805-911` DOES dispatch unquote-splicing per child (`:860-898`); **Vector branch at `:926-941` recurses children but does NOT dispatch splice**.
3. Confirmed the runtime side: `WatAST::Vector` at value position errors with `"vector literals at value position are not supported in arc 167"` (`src/runtime.rs:3706-3714`). So even if a helper fn received a quoted Vector via `(:wat::core::quote ~specs)`, it couldn't convert the Vector AST into a runtime Vec (no Vector→Vec primitive).
4. Confirmed wat lacks a string→Symbol primitive: keyword/from-string exists (`:wat::core::keyword/from-string`, `src/runtime.rs:4117`) but only produces `Value::wat__core__keyword`, which `value_to_watast` lowers to `WatAST::Keyword`, NOT `WatAST::Symbol`. Symbol fabrication paths in the codebase are: `extract-arg-names` (extracts user-defined arg names from a signature head AST); `rename-callable-name` (substitutes a renamed function head). Neither accepts arbitrary strings.
5. Confirmed gensym is arc 173 territory (planned, not shipped). See `docs/arc/2026/05/173-clojure-macro-features/DESIGN.md` § Slice 1 — `gensym` primitive (atomic; standalone).

### Concrete probe failures

Added `tests/probe_stone_d2_splice_vector.rs` with two tests asserting the EXPECTED-FAILURE mode. Both pass:

```
running 2 tests
Gap 1 actual error: macro: <entry>:6:26: unquote-splicing (,@xs) requires a List argument; got vector
test splice_of_vector_bound_symbol_errors_with_splice_not_list ... ok
Gap 2 actual error: check:
1 type-check error(s):
  - <entry>:7:11: malformed <vector literal> form: vector literals at value position are not supported in arc 167.
    Vectors are currently consumed only in :wat::core::fn / :wat::core::defn signature positions
    (slice 2 wires those consumers). A future arc enables vector literals as `Value::Vec` values.

test splice_inside_vector_template_does_not_fire ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Gap 1's probe macro: `(:my::splice-vec [1 2 3])` with template `` `(:wat::core::Vector :wat::core::i64 ~@xs) `` — splice on a Vector-bound symbol → `SpliceNotList`.

Gap 2's probe macro: zero-arg `` `[~@(:wat::core::forms 10 20 30)] `` — `(:wat::core::forms ...)` returns `Value::Vec` at expand time, but the surrounding `[...]` template's Vector branch never dispatches splice, so the unquote-splicing form survives literally — and the substituted Vector reaches runtime where vector-at-value-position errors. Two cascading manifestations of the same gap.

### Why a wat-side helper fn doesn't close the gap

The BRIEF offered two paths: (1) wat helper fn; (2) inline-recursive-quasiquote. Both face the same primitive-shortfall:

- **Helper fn approach.** The helper would receive specs as `:wat::WatAST` (via `(:wat::core::quote ~specs)`) and need to iterate the outer `WatAST::Vector` to emit per-spec binding clauses. `:wat::core::map` / `:wat::core::foldl` require `Value::Vec`, NOT `WatAST::Vector` (`src/runtime.rs:7479-7506` for `Vector/get`; same `require_vec` pattern for map/foldl). Converting the WatAST::Vector to a Vec at runtime is blocked by the same "vectors at value position not supported" error. Converting via `:wat::holon::from-watast` yields `HolonAST::Bundle`, which has NO iteration primitive at the wat layer (only `extract-arg-names` walks Bundle children, and only in the strict 2-element arg-pair shape — spec triples `[:I :O f]` are 3-element Bundles).
- **Inline-recursive-quasiquote approach.** A recursive macro can pattern-match the first spec and recurse with the remaining specs — but pattern-matching the FIRST item of a Vector AST at macro-expansion time requires either AST destructuring (no wat-side primitive) or splice (Vector-splice blocked per Gap 1/2). Recursive accumulation via `:wat::core::Tuple` value-position works ONLY if the accumulator has a stable type — which would require either monomorphic specs (homogeneous types — but D2's premise is heterogeneous) or runtime-typed Tuple growth (substrate's tuple types are static).
- **Macro-driven fresh-name generation.** Even if iteration were possible, generating per-spec fresh names (`thread-0`, `thread-1`, `peer-0`, `peer-1`, …) requires either gensym (arc 173) or the `extract-arg-names`-on-hand-rolled-quoted-signature trick — the latter caps the bound names at a hand-written maximum N and depends on `extract-arg-names`' shape-specific Bundle walk that documents itself as "arg names from a signature head."

### Minimal substrate paths forward (NOT in D2 scope)

Three orthogonal options, in order of smallest-blast-radius:

1. **Relax `splice_argument`** (`src/macros.rs:1081-1086`) to clone children from `WatAST::Vector` the same way it does from `WatAST::List`. ~5-line change. AND extend `walk_template`'s Vector branch (`:926-941`) to dispatch unquote-splicing on List children, mirroring the List branch (`:860-898`). ~15-line change. Together: lets `~@vec` and `~@(...)` inside `[...]` work as Lispers expect — Vector and List are interchangeable in template-splice contexts. Unblocks D2 immediately AND aligns the macro layer with the BOOK chapter's "splice composes" intuition.
2. **Ship arc 173 slice 1** (`gensym` primitive). Per `docs/arc/2026/05/173-clojure-macro-features/DESIGN.md` § Slice 1. Doesn't unblock D2 by itself (still needs iteration), but combined with a Vector-iteration helper would close the surface.
3. **Revise D2's call form** to `((:I :O f) (:I :O f) ...)` — outer parens (List), inner parens (List). The four-questions outcome from 2026-05-16 settled on nested-vector brackets because they read as "grouped pairs" visually; revising to nested-parens-pairs would re-litigate that decision. Lower cost than option 1 only if user is comfortable with re-litigation.

The BRIEF's anchor is `feedback_no_new_types` — and the relaxation at option 1 is NOT a new type/verb/struct/special-form. It's an existing primitive accepting a wider input set. The decision belongs to the orchestrator + user; D2 cannot proceed without one of the three.

### What this means for arc 170

- D2 stays open until the substrate-side fix lands (option 1 most likely).
- D3 (panic cascade) is blocked behind D2.
- Stone E (run-processes) inherits the D-family shape and the same blocker if it ships before resolution.
- D1's clean call form is unaffected (it doesn't iterate at expand time).

## Honest deltas to capture

### Delta 1 — Investigation depth vs implementation output

The 85-minute clock is heavily skewed toward READING (`src/macros.rs`, `src/runtime.rs`, `wat/runtime.wat`, `wat/core.wat`, `wat/test.wat`, arc 173 DESIGN, arc 199 DESIGN) and HYPOTHESIS PROBING (six distinct candidate macro shapes considered + rejected before STOP). Output is one 90-line probe file + this SCORE. Per `feedback_attack_foundation_cracks`: when a crack surfaces, the fix is also diagnostic. The two probes seal the diagnosis even though they don't seal the underlying gap.

### Delta 2 — Fresh-name strategy: not picked (would have been needed)

Had iteration been possible, the BRIEF asked sonnet to pick between index-suffix vs gensym. Neither is reachable today: gensym is arc 173 unshipped, and index-suffix requires either string→Symbol (no primitive) or the `extract-arg-names`-on-hand-rolled-quoted-signature trick (works narrowly, fragile). The honest answer per the four-questions: NONE of the candidates pass YES on `simple` because all require shape-fragile composition. Surface the gap, don't push through with a brittle trick — the BRIEF's STOP triggers explicitly call this out as the correct move.

### Delta 3 — Helper-fn vs inline-recursive-quasiquote: not chosen (would have been needed)

The BRIEF offered both paths. Both fail at the same primitive-shortfall (outlined above § "Why a wat-side helper fn doesn't close the gap"). Picking either to "show work" would have produced code that doesn't compile or papers over the gap with a fragile trick. The honest answer: surface the gap and let the orchestrator decide on the substrate relaxation.

### Delta 4 — Hygiene quirks: none surfaced

Because no fresh names were minted, no hygiene clash surfaced.

### Delta 5 — Workspace baseline preserved cleanly

`cargo test --release --workspace --no-fail-fast` produces exactly 4 failing tests, all in the pre-D2 baseline set:
- `lifeline_pipe_zero_orphans_across_100_trials` (lifeline flake)
- `deftest_wat_tests_tmp_totally_bogus`
- `t6_spawn_process_factory_with_capture_round_trips`
- `startup_error_bubbles_up_as_exit_3`

Zero new failures. The probe file adds +2 passes (both expected-failure assertions). D1's test continues to pass under the unchanged macro.

### Delta 6 — Why STOP is the right call here, framed against `feedback_pivot_not_defer`

`feedback_pivot_not_defer`: STOP signal before writing "substrate is missing X" or "future fix is open." The trap is reflexive; check convention first.

I checked convention first. The convention says splice accepts Lists — and the wat source convention for "this group of pairs" is parens (every existing variadic-collector macro uses parens, e.g. `:wat::test::deftest` prelude). The BRIEF mandated brackets per a four-questions decision settled 2026-05-16 with the user; the brackets-vs-parens question IS the convention question, and it was already resolved. The substrate gap that this surfaces is real: brackets in let-bindings are the established Vector-shape convention, splice inside Vector templates doesn't fire, and the macro layer's List/Vector asymmetry isn't documented as intentional anywhere I could find.

So this is NOT "substrate is missing X, future fix is open" — it's "the brackets-vs-parens four-questions outcome puts D2 in a position where the macro layer's existing List/Vector asymmetry blocks the implementation, and the asymmetry has no DESIGN doc justifying it as intentional." That's a foundation crack per `feedback_attack_foundation_cracks`. Surfacing it is forward progress: the diagnostic IS the fix's starting point. The honest framing per `feedback_no_known_defect_left_unfixed` is also relevant — once the gap is named, "we know how to surface a failure" and the next arc closes it.
