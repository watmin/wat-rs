# AMEND-2 — STONE 255.89: one door, and rete reads a head by identity

**Drawn 2026-10-04.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `2b566f772`. Commit
locally on `main`; **do not push**.

## Accepted from the amend

- **The vanished tests run:** the new floor's test-name set against `.floor/2026-10-03T13-09-11Z` is MISSING 0, ADDED
  11; the deftest lookup goes through `canonical_identity`; the both-spellings probes are good shapes.
- `e969cf456` (a nonexistent variant refused in both spellings), `2fe5ff0ed` (the `:i64` proof restored onto KEEP).
- **Ignore ledger:** the SEAM's command reads **18** (it counts Rust `#[ignore]` attributes). Nextest's 19 also
  lists one deftest ignored by a `.wat` annotation (`deftest_wat_tests_lint_lint_stdlib_runs`). That's a different
  instrument, and the skipped total is 24 both before and after. Nothing to cure.
- The STOP was right. The regrouping by deciding site is the shape asked for.

## 1. One door, not two

`2459375cc` hand-copies `canonical_identity` into `crates/wat-macros/src/discover.rs`. Two copies of the door drift apart
(`[[feedback_routing_through_a_door_inherits_what_it_encodes]]`). The function is pure string work
(`src/edn/render.rs:3740-3777`, with `ns_to_wat_path`). **Move it, with `ns_to_wat_path`, into `crates/wat-reader`
(beside `Identifier`)**. `src/edn/render.rs` re-exports it, so its many callers do not move, and `discover.rs` calls the
wat-reader one. Delete the copy. A unit test in `wat-reader` holds the slash-rule cases (`u/a/b`,
`u/pathological/name//foo`, `wat.core//`, a `::` keyword, a parametric `(…)` rendering) with the values they map to today.

## 2. The rete family: a head or a cond decided by its spelling or its printed text

This is 5b's class, in places 5b's ledger cannot see: `WatAST::Keyword` match arms and text keys are its stated blind
shapes. From your table and the unfiled fourteen, the sites are:

| site | what decides |
|---|---|
| `src/rete/kernel/arm.rs:118` (+ `node.rs:214` `cond_text`) | an alpha is looked up by the **EDN text** of its cond. A keyword-minted alpha and a symbol cond are different strings. **A string standing in for data.** |
| `src/rete/kernel/arm.rs:374` | `compile_condition_local` returns none on a converted cond |
| `wat/rete/compile.wat` (the totality decision upstream of `:345`/`:609`) | `:wat::core::length` read as not total |
| `src/rete/expr_ir/mod.rs:745` | `cannot lower head` |
| `src/intrinsic/rete.rs:301` | `vocabulary-admitted?` matches only `WatAST::Keyword` |
| `src/rete/purity.rs` 1132, 1254, 1262, 1283, 1311, 1356, 1411, 2097 | `WatAST::Keyword` arms (the purity/determinism unfiled tests) |
| `src/rete/eval_insert.rs:254` | `:then` head `cg/make-rate` missed |
| `src/check.rs:14664` | a `wat.form/matches?` pattern head must be a `Keyword` |
| `src/runtime.rs:8996-9006` | a nested variant pattern accepted only as a keyword `is_namespaced_variant` |

**The cure for each:** decide by identity (`canonical_identity`, or the `Identifier`'s `(ns, name)`), in both spellings
until 5d. The alpha table is **keyed by data**: the cond with every name in it canonicalized, compared structurally,
never its printed text. Each site gets a probe with **the same rule or form in each spelling**, asserting the **same
value** (a derived fact or a count witness), not merely "no error". Check each site against the brief's STOP-1: a keyword
program's behaviour must not change.

**Name the blind spot:** after the cures, list every other `WatAST::Keyword(` arm in `src/rete/` and `wat/rete/` that
compares against a name literal, and say for each whether it decides a head (then cure it here) or reads a value
keyword (then leave it).

## Not in this amend (each its own row after the next floor)

The member-join image (rows 6, 10: `wat.core.i64/to-string` canonicalizes to a retired path; `my.Counter/Op` vs
`:my::Counter::Op`; `:u::Demo::Has::has`), `normalize.rs:991` unresolved references, the acronym LAW (8), the
test-text kinds (11–15), sift, peers, and the four files `every_wat_scripts_file_loads` refuses. Do not cure them here;
the next floor regroups what is left.

## Then

The floor, in the foreground, `git status` clean. **The test-name set against `.floor/2026-10-03T13-09-11Z`: MISSING 0.**
Then regroup what is still red by deciding site, as before, with the rete family's residue named. Clippy, and the ignore
ledger at 18. A red caused by this amend's own change is captured verbatim, cured, and followed by a **new** floor; any
other red that is new against `.floor/2026-10-04T00-08-26Z` is a STOP. Append to the SCORE, commit, **do not push**.
