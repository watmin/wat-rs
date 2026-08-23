# BRIEF — annihilate the angle bracket, WAVE 2: the Rust side

Wave 1 is **in the tree, uncommitted** — the wall is applied and 27 corpus files are migrated. The
floor then went **4855 passed / 26 failed**, and all 26 are one family. This brief closes them.
Read `DESIGN-STONE-annihilate-the-angle-bracket.md` and
`BRIEF-STONE-annihilate-the-angle-bracket.md` first; wave 1's rules still bind.

⚠ **Work against the DIRTY tree. Do not revert, stash, or rebuild from main** — wave 1's changes are
your foundation. This is the documented atomic-commit pattern: sweep A leaves the tree deliberately
broken, sweep B runs against it, the orchestrator commits both together when the floor is green.

## Why these 26 existed and wave 1's census could not see them — read this, it is the whole shape

Wave 1's census imposed the wall over all 1798 `.wat`/`.wat.bad` files and asked each *"does your own
text lex?"*. That is a census of **source**. It is blind to two other populations that also feed the
reader:

1. **Text a program hands the reader at RUNTIME** — a `.wat` file with no angle syntax of its own
   that passes `"…Vector<t::Old>…"` as a string to `read-string` / `rename-keyword-prefix`.
2. **Rust `&str` literals** compiled into unit tests, and **the reader's own tests of the permission**.

**The floor is the only instrument that sees either.** It saw both. Nothing here is a defect in wave
1's work — the boundary was drawn from the wrong population, and that is the orchestrator's error,
recorded so you do not inherit the assumption.

## The 26, by class — every one is `LexError { kind: AngleTypeHeadInName }` reaching a caller that expected `Ok`

### Class 1 — the reader's OWN tests; their SUBJECT is the permission (11)

```
crates/wat-reader/src/lexer.rs:1166   keyword_parametric_type
                            :1212   keyword_crate_path
                            :1260   keyword_vec_parametric
                            :1296   keyword_nested_parametric_with_fn_type
                            :1348   keyword_apostrophe_after_parametric_close
                            :1363   keyword_primed_generic_single_param
                            :1372   keyword_unprimed_generic_single_param_control
                            :1409   keyword_comma_in_angle_brackets_rejected
                            :1425   keyword_single_param_generic_and_tuple_still_lex
crates/wat-reader/src/parser.rs:843   internal_colons_lex_as_single_keyword
                              :976   parametric_keyword_survives_in_call
```

Representative arm, verbatim from the floor:

```
thread 'lexer::tests::keyword_vec_parametric' panicked at crates/wat-reader/src/lexer.rs:1260:35:
called `Result::unwrap()` on an `Err` value: LexError { position: 4, kind: AngleTypeHeadInName }
```

These assert that `Vector<i64>` lexes as ONE keyword. **That is the permission, and it is gone**, so
each is kind-(ii) in wave 1's vocabulary: the subject no longer exists.

**Re-point them, do not delete them.** A negative control that CAN be kept MUST be kept — each becomes
an assertion that the angle head is REFUSED, naming `AngleTypeHeadInName`. Judge each individually:
- `keyword_comma_in_angle_brackets_rejected` already asserted a refusal — it now trips the ANGLE wall
  one step before the comma wall. Move its expectation to the mechanism that actually fires.
- `keyword_unprimed_generic_single_param_control` is a CONTROL for a primed/unprimed pair. Keep the
  pairing: both halves now refuse, and the control still discriminates.
- ⚠ **Whatever a test asserted about `<` and `>` that is NOT the type-head permission must survive.**
  `internal_colons_lex_as_single_keyword` is about `::` in a keyword path, not about angles — keep its
  real subject and give it an input that still lexes.

### Class 2 — inline-wat Rust unit tests (6)

```
src/runtime.rs:34343  (the shared `eval_expr` helper)   hashmap_accepts_composite_key
                                                        hashset_accepts_composite_element
                                                        keyword_to_string_strips_leading_colon
                                                        keyword_reflection_round_trip
src/check.rs:21435    (the shared `check` helper)       user_parametric_define_passes
                                                        user_parametric_wrong_return_rejected
```

The panic is in the shared helper; the angle text is in each test's own `&str`. Migrate the literals
to the `:-` spelling. These are ordinary class-A/class-B migrations, just written in Rust.

### Class 3 — the migration TOOLING's own tests (7)

```
tests/resolve/probe_arc251_decl_migrator.rs:51        c02_defn_generic_name_drops_type_params
                                          :69        c03_generic_decl_name_and_parametric_target
tests/resolve/probe_arc251_fix_macro_param_types.rs:19
tests/resolve/probe_arc251_fix_source_local_rules.rs:43  contract_03_structural_parametric_type
tests/resolve/probe_arc251_stone5_roundtrip.rs:28        contract_01_edn_roundtrip_is_faithful
tests/resolve/probe_arc251_stone5a_read_string.rs:28     contract_02_read_string_reads_the_dirty_surface
src/test_runner.rs:487   deftest_wat_tests_lint_rename_keyword_prefix_type_arg_and_boundary
```

★ **These need JUDGEMENT, not a rewrite, and this is the interesting class.** They test tooling
*over angle input*. Some subjects survive the migration and some are annihilated by it. The worked
example is `tests/types/probe_arc283_1_rename_typearg` (already handled in the tree): its premise was
*"a type argument is a name embedded inside another keyword's text, so start-anchored rename misses
it."* In `(:wat::core::Vector :- [:t::Old])` the type argument is an ordinary keyword **leaf** — the
embedding does not exist, so the premise is void even though the test can be made green.

For each of the seven, decide and SAY WHICH:
- **(a) the subject survives** — migrate the input to `:-`, keep the assertion;
- **(b) the subject was the angle form itself** — re-point it as a refusal control;
- **(c) the subject is annihilated** (the tooling handled a problem the angle bracket created) — keep
  the test green under (a) if it can be, and **report it as annihilated** so the sibling purge stone
  knows the machinery underneath is a purge candidate. Do NOT delete the machinery here.

### Class 4 — the lint, and it is wave 1's own doing (1)

```
tests/lint/no_loose_string_assert.rs:112   3 site(s) assert with contains/starts_with/ends_with
```

Wave 1 re-pointed three tests to `.contains("annihilate the angle bracket")`. The repo's documented
escape is the `rune:lint(loose-assert)` marker with a REASON — `tests/types/probe_arc232_generic_
method_type_application.rs` carries a correct example. Apply it where the loose check is right
(a targeted presence over a large structured diagnostic), or move to an `.edn` golden where it is not.
Read `docs/CONVENTIONS.md` § 'Test idioms'.

### Class 5 (1)

```
tests/function/probe_arc241_stone1_argspec_canonical.rs:31   contract_07_rest_binder_rejected
```

## STOP triggers

- **STOP-1 — the floor's failure count does not fall monotonically as you work.** Each class you close
  should drop it. A NEW failure name appearing means a fix reached past its site; report it.
- **STOP-2 — a class-1 test cannot be re-pointed honestly** because its real subject was something
  other than the permission and no still-lexing input exercises it. Report it; do not delete the control.
- **STOP-3 — a class-3 test can only be made green by weakening what it asserts.** That is annihilation
  wearing a migration's clothes. Report it as (c) rather than shipping a weaker assertion.
- **STOP-4 — you find yourself editing the wall** (`crates/wat-reader/src/lexer.rs`'s
  `AngleTypeHeadInName` arms) to make a test pass. The wall is the specification; the tests move.

## Boundaries

- The 26 named sites, their fixtures and goldens, and `crates/wat-reader/src/lexer.rs` + `parser.rs`
  **test modules only** — not the lexer's production code.
- **Do NOT delete the downstream machinery** (`canonical_callable_name`, `split_type_params`,
  `split_name_and_type_params`, `split_method_name_type_params`, the `find('<')` splits in
  `check.rs`/`runtime.rs`/`types.rs`). That is the sibling purge stone.
- **Do NOT touch `src/types.rs:4631`** — ③'s type-parser wall stays.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the full floor and clippy centrally. Use scoped checks —
  `cargo nextest run --release -E 'test(<name>)'` and `./target/release/wat --check <file>` (~0.2s).

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

The disposition of every one of the 26, grouped by class, each with one line saying what you did and
why. For class 3, the explicit (a)/(b)/(c) call per test — the (c) list is what the purge stone
inherits. Any STOP that fired, with the arm captured verbatim BEFORE you diagnosed it. What surprised
you.
