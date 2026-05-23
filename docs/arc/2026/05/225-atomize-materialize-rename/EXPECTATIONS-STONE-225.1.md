# EXPECTATIONS — Arc 225 Stone 225.1 (v3) — Bridge naming family substrate-wide rename + mint

Mode A target: 16/16 PASS.

## v3 supersedes earlier drafts

The v1 EXPECTATIONS (commit 898f2ed) targeted `:wat::holon::atomize` rename — rejected. v3 captures the resolved family naming (to-wat/from-wat/to-holon/from-holon).

| # | Row | Expectation |
|---|---|---|
| 1 | Rust fn `eval_algebra_atom` → `eval_holon_atom_constructor` | `src/runtime.rs:13820` renamed; verb dispatcher arm intact |
| 2 | Rust fn `value_to_atom` → `wrap_holon_as_atom` + NARROW body | `src/runtime.rs:13838` renamed; body accepts ONLY Value::holon__HolonAST input; returns HolonAST::Atom(inner); all other input-arm branches DELETED |
| 3 | TypeScheme for `:wat::holon::Atom` narrowed | `src/check.rs:13558` updated from `∀T. T → HolonAST` to `HolonAST → HolonAST`; `infer_list` special-case at `:5326` updated accordingly |
| 4 | NEW Rust fn `eval_holon_to_holon` + verb registration | New polymorphic UP verb; absorbs retired Atom arms (primitives, WatAST, collections, Uuid, HolonAST→Atom-wrap); dispatch table entry added; TypeScheme `∀T. T → HolonAST` |
| 5 | Rust fn `eval_atom_value` → `eval_holon_from_holon` + verb rename | `src/runtime.rs:13633` renamed; verb registration `:wat::core::atom-value` → `:wat::holon::from-holon` (namespace move); body semantics unchanged |
| 6 | L1-3 doc-lie fix on `eval_holon_from_holon` | `src/runtime.rs:13619-13629` doc refreshed to honestly describe the polymorphic decode (currently says "Composite (Bundle/...) → error" but body handles Bundle three-way; refresh) |
| 7 | Rust helper `holon_item_to_value` → `from_holon_item` + `op: &str` param | Helper renamed; `op: &str` parameter threaded through signature; all callers updated to pass own op name (closes arc 224 L1-runtime-3 latent lie) |
| 8 | TypeScheme for `:wat::holon::from-holon` | `src/check.rs:13591` updated to new verb-name string; special-case at `:5362` updated |
| 9 | Rust fn `eval_holon_from_watast` → `eval_holon_from_wat` + verb rename | Function + verb renamed; TypeScheme + special-case updated; semantics unchanged |
| 10 | Rust fn `eval_holon_to_watast` → `eval_holon_to_wat` + verb rename | Function + verb renamed; TypeScheme + special-case updated; semantics unchanged |
| 11 | Substrate-as-teacher cascade — Rust callers | All ~31 `:wat::holon::Atom` literal + ~10 `:wat::core::atom-value` literal + all `:wat::holon::from-watast` / `to-watast` literal call sites in `src/` + `tests/` updated to new verb names; `cargo build --release -p wat` 0 errors |
| 12 | Substrate-as-teacher cascade — wat-side callers | All caller sites in `wat/**/*.wat` + `wat-tests/**/*.wat` updated to new verb names (~54+ sites for Atom + atom-value; plus from-watast/to-watast callers); substrate startup loads cleanly |
| 13 | Polymorphic Atom callers redirected correctly | Callers passing HolonAST input keep `:wat::holon::Atom` (now narrow); callers passing non-HolonAST input change to `:wat::holon::to-holon` (the new polymorphic UP verb); type-system dispatches correctly |
| 14 | Adjacent doc comments refreshed | Doc comments naming retired verbs in touched files updated as discovered (no global hunt — fix what you touch) |
| 15 | All test suites green | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5 signal tests]` PASS; integration tests for arcs 220/221/221b/143 PASS; `cargo test -p wat-edn` PASS; `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings |
| 16 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty |

## Independent prediction (calibration record)

**Target runtime:** 180-300 min Mode A
**Upper bound:** 360 min
**Confidence:** medium-high

**Rationale:**
- Closest precedent Stone 221.4b: ~100 min for ~100 sites of dispatcher + cascade rename in wat-rs
- This stone is ~1.5-2x the scope (150-200 sites) + adds 2 new verbs + 2 cosmetic renames
- Substrate-as-teacher cascade well-understood; pattern locked
- Polymorphic-Atom-caller redirection (row 13) is the trickiest part — needs type analysis at each call site

**Risks:**
- Polymorphic Atom callers in wat-side may be ambiguous about input type; sonnet may need to inspect adjacent code or err toward `to-holon` (the more general verb) when uncertain
- Verb-namespace move (`:wat::core::atom-value` → `:wat::holon::from-holon`) requires updating both the namespace + the verb name; mechanical but error-prone

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- holon-rs changes (algebra primitives stay)
- wat-edn changes
- Collection classifier-wrap behavior (arc 228 — `to-holon`'s collection arm ships bare Bundle for now)
- Type predicates (arc 226)
- INSCRIPTION (Stone 225.2; blocked on arc 228 closing)
- Deprecation aliases (HARD CUT)
- Quasiquote evaluator (arc 229; deferred)

## Honesty deltas accepted

- Rust function rename targets may vary slightly if sonnet finds a more honest name during the sweep — sonnet documents the choice in SCORE
- Doc-comment refresh wording — sonnet picks; load-bearing point is "no longer references retired verb names"
- For polymorphic Atom callers where input type is genuinely ambiguous, sonnet picks `to-holon` and notes the decision in SCORE
- Number of caller sites may exceed pre-flight estimate; substrate-as-teacher cascade absorbs

## Honesty deltas NOT accepted

- "Pre-existing failure" framing for tests broken by this stone — STOP per Stone 221.3 Delta 1a; honest framing required (broken-by-this-stone IS the cascade we expect)
- Skipping any rename per "didn't want to touch that test" — STOP. Hard-cut means hard-cut.
- Touching holon-rs — STOP per STOP-4
- Adding deprecation aliases for old names — STOP. The "fractal of correctness" principle: dishonesty is illegal; aliases would BE dishonest. Hard-cut.
- Extending scope to other polymorphic verbs found during the sweep — STOP per STOP-5; surface as finding; orchestrator decides whether to spawn additional fix-arcs

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors (not from rename cascade)
- **STOP-2:** test failure beyond cascade-rename consequences after green build
- **STOP-3:** 360 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** additional polymorphic-name verbs found beyond Atom + atom-value
- **STOP-7:** bash discipline — cargo hang from accidental pipes
