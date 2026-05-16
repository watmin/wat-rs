# Arc 200 — Macro-layer List/Vector splice symmetry

**Direction:** relax the existing macro substrate so unquote-splicing (`~@`) treats `WatAST::Vector` and `WatAST::List` interchangeably in template-splice contexts. NOT a new primitive — an existing primitive accepting a wider input set.

**Status:** DESIGN. Slice 1 BRIEF drafted alongside.

**Originating signal:** Arc 170 Stone D2 (commit `64cc793`) STOPPED on substrate-gap probe. The D2 BRIEF mandated a `(:wat::kernel::run-threads [[:I :O f] ...] client-fn)` call form per four-questions outcome (2026-05-16). Variadic iteration over the user-typed `[[:I :O f] ...]` Vector AST at expand time bumped into two minimal substrate asymmetries:

- **Gap 1** (`src/macros.rs:1081-1086`): `~@sym` rejects `WatAST::Vector` — only `WatAST::List` accepted. Probe error: `unquote-splicing (,@xs) requires a List argument; got vector`
- **Gap 2** (`src/macros.rs:926-941`): `walk_template`'s Vector branch lacks unquote-splicing dispatch. List branch handles it at `:860-898`. Cascading symptom: `Vector` literal survives macro expansion and hits arc 167's runtime "vectors at value position" error.

D2's probes (`tests/probe_stone_d2_splice_vector.rs`) PASS as expected-failure assertions; the diagnosis is sealed. After arc 200 lands, the probes flip from expected-failure to expected-success regression tests.

---

## Why this is a defect, not a feature

Neither asymmetry has a DESIGN doc justifying it as intentional. The List branch of `walk_template` was extended for unquote-splicing per arc 029 slice 1; the Vector branch was added later (arc 167 slice 1) for fn-signature hygiene but never extended to splice-dispatch. The asymmetry is an inadvertent oversight of the staged substrate evolution — vectors arrived after splice was wired for lists, and the splice-handling code wasn't generalized.

Lispers expect Vector and List to be interchangeable for template-splice contexts. The existing `:wat::test::deftest` prelude (parens) and the D2 brackets shape (vectors) both express "a sequence of items to splice"; the macro layer should treat them uniformly.

## Scope

**In scope:**
- Gap 1: relax `splice_argument` (`src/macros.rs:1081-1086`) to clone children from `WatAST::Vector` the same way it does from `WatAST::List`. ~5-line change.
- Gap 2: extend `walk_template`'s Vector branch (`:926-941`) to dispatch unquote-splicing on List children, mirroring the List branch (`:860-898`). ~15-line change.
- Flip the two D2 probes (`tests/probe_stone_d2_splice_vector.rs`) from expected-failure to expected-success regression tests.
- Add at least one positive test demonstrating the brackets+splice form expands as expected (mirror of the D2-mandated `[[:I :O f] ...]` call shape).

**Out of scope (separate arcs):**
- Gap 3 (cascading): arc 167's "vectors at value position not supported" runtime limitation. After Gap 2 fixes the splice-dispatch in macro expansion, the Vector literal doesn't survive to runtime — so arc 167 is unaffected. Future enhancement, but NOT needed for D2 to proceed.
- Fresh-name generation (gensym): arc 173 slice 1 territory. D2 may use the Tuple+drain-tuple pattern to avoid fresh-name dependency.
- Any other macro-layer asymmetries not surfaced by D2's probes.

## Four-questions (informal; formal pass at BRIEF time)

**Arc 200 = Gap 1 + Gap 2 only (macro-layer Vector-splice symmetry)**

- Obvious: **YES** — Lispers expect Vector ↔ List interchangeable for template-splice; both fixes are macro-layer; both surfaced by D2 probes; cohesive scope
- Simple: **YES** — ~20 lines total; single file (`src/macros.rs`); mirror existing List-branch logic in Vector branch
- Honest: **YES** — fixes asymmetry without expanding to unrelated concerns; the diagnosis is sealed by D2 probes
- Good UX: **YES** — Lispers expect `~@` to work uniformly inside `[...]` or `(...)`

→ YES YES YES YES.

**Larger scope (e.g., Gap 1 + Gap 2 + Gap 3 vectors-at-value-position)** disqualified:
- Obvious: NO — Gap 3 is a separate arc 167 concern; not what D2 surfaced
- Simple: NO — Gap 3 is a much bigger runtime surface
- Honest: NO — conflates macro-layer fix with runtime extension

**Smaller scope (just Gap 1 OR just Gap 2)** disqualified:
- Simple: NO — both gaps together produce the broken Vector-splice experience; fixing one without the other leaves a half-broken state

## Slice plan

**Slice 1 — the two relaxations + regression tests.** Single slice; both fixes are cohesive macro-layer work; D2 probes already exist to flip; positive test added.

No further slices anticipated. The arc closes on slice 1 SCORE.

## Discipline anchors

- `feedback_attack_foundation_cracks` — the foundation must be trusted; the fix is also diagnostic; D2 probes sealed the diagnosis
- `feedback_no_known_defect_left_unfixed` — we know the gaps, we have probes demonstrating them, the fixes are small; discipline says fix now
- `feedback_no_new_types` — relaxing existing primitives is NOT minting new ones; the discipline holds
- `feedback_simple_is_uniform_composition` — the substrate becomes more uniform; Lispers' intuition aligns with substrate behavior

## Consumer arc affected

**Arc 170 Stone D2** — currently STOPPED on these gaps. Post-arc-200:
- D2 restarts with the BRIEF's bracket form `[[:I :O f] ...]` working as intended
- D3 (panic cascade) follows D2
- Stone E (`run-processes`) inherits the cleaner shape

No other arcs known to be blocked. The macro-layer asymmetry hasn't surfaced elsewhere because D2 is the first variadic-iteration-over-vector-spec consumer.
