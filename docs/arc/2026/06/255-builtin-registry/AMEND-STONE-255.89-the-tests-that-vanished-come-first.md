# AMEND — STONE 255.89: the tests that vanished come first

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `1c9fd486d`. Commit
locally on `main`; **do not push**.

The STOP was right to fire, and the KEEP list, the conversion commit (2205 files, heads 85,072 → 0, second run 0) and the
replay (18/18, no golden changed) are accepted as they stand. Two things change before any other cure.

## 1. The floor shrank by 443 tests, and discovery is why (orchestrator, measured)

`.floor/2026-10-03T23-07-54Z` ran **5969** tests against **6412** at `30cd8b69c`. Diffing the two floors' test-name lists:
**442 `wat::kernel test::…` tests are gone** (every deftest under `wat-tests/`), plus
`with-loader-example::test deftest_user_with_loader_test_test_loader_wiring`. Only the "no deftests found" guard went red
(`tests/kernel/test.rs:17`, and its twin in `with-loader-example`), because there the count reached zero. **Wherever a
count only shrinks, the loss is silent:** a test that does not exist cannot fail.

The mechanism is `crates/wat-macros/src/discover.rs:334-388`: the deftest discovery matches **keyword spellings**
(`":wat::test::deftest"`, `-hermetic`, the primed forms, `make-deftest`, and the annotations `:wat::test::ignore`,
`should-panic`, `time-limit`). A symbol head (`wat.test/deftest`) is not seen. This is 5b's class (a head decided by its
spelling) in a crate the heresy ledger does not scan (`tests/lint/keyword_heresy_ledger.rs:86`: "anything outside
`src/` … not `crates/`").

**The cure, first:**
- **Every build-time or test-time enumerator that finds `.wat` forms by their text reads them by identity**, in both
  spellings until 5d: the deftest discovery and its annotations; and every other one you find by asking which proc
  macros, `build.rs` scripts, lints and test generators walk `.wat` and match heads. The floor already names some
  candidates: `rete_compile_gate::the_rule_declaring_population_is_not_vacuous` ("found 18 files"), `every_probe_runs`,
  `nested_program_starts`, and the grid tests (`wat_scripts_grid_*`). Name each one you find, cured or not.
- **The witness is the set of tests, not a count.** Extract the test-name list from `.floor/2026-10-03T13-09-11Z` and
  from your new floor. Every name in the old list is in the new one, unless you show its test was renamed by this stone
  (old name → new name, one row each). A discovered deftest's sanitized Rust name should not change with its head's
  spelling; if it does, say why.
- Each enumerator you cure carries a probe holding **one deftest in each spelling** in one file, and asserting both are
  found.

## 2. Regroup by mechanism, not by message

The SCORE's 95 rows group by the error sentence. A mechanism is **the code site that decides wrong** (a `file:line` in
`src/`, `crates/`, `wat/`, or a test's own reader), and many sentences share one. For example, "fact-shaped cond has no
minted alpha", "accumulator expr is not total", "call canonicalizes to a retired verb", "comparison head is not a rete
primitive", "cannot lower head" and "vocabulary-admitted? expects a Keyword" all come from rete compiling converted
rules. Find out whether they share a reader in `wat/rete/` or `src/rete/`. After item 1 lands, run a new floor (it
will run the 443 too), and give the table again: one row per **deciding site**, the sentences it produces, the tests it
fails, and one verbatim block each.

**The test-text class:** a Rust test that asserts on an error message or a fixture's text (a keyword spelling in an
expected string) is one mechanism per **kind** of assertion (an error renders the name as written; a reader slices
text). Name the kind. Cure it by asserting on data, never by rewriting the expected string to the new spelling.

## 3. The census's three recoveries

- `tests/types/probe_arc296_p1_annotation_names_a_type__bare_legacy_primitive.wat` is a negative proof of a retired
  keyword form (the bare `:i64`): it goes on the KEEP list. Restore its pre-conversion bytes from `9d85da5a5`
  (`git checkout 9d85da5a5 -- <path>`) and record that in the KEEP commit's message. Restoring the original bytes is not a
  hand-edit.
- `probe_arc278_enum_variant_typo_bad.wat` and `_tagged.wat`: the symbol spelling `evt.G/Hii` of a variant that does not
  exist is **admitted**. That is a wall with a hole in the new spelling, not a file to keep. Find the deciding site. It is
  a mechanism to cure (the wall refuses a nonexistent variant in both spellings), with a probe holding both spellings.

## Then

If, after item 1's floor, the deciding sites number **12 or fewer**, cure them by mechanism, as the brief says. If more,
STOP with the regrouped table. Clippy and the ignore ledger (18) run before the SCORE. The brief's other rules stand:
`git status` clean before the floor; a red caused by this stone's own change is captured verbatim, cured, and followed by
a **new** floor; any other red is a STOP. Append to the SCORE, commit, **do not push**.
