# SCORE 4 — replay batch 1 STOP-4 at checkpoint #60

Branch: `replay/grok-rete`. **Not pushed.** Main untouched.
2a4 `c184348f7`. Addendum `b342b13a2`. **#11–#60 committed (50 REPLAY).** Checkpoint #60 floor RED — do not re-run.

`git log --oneline` `REPLAY(grok-rete #` count in range #11–#60: **50**. HEAD `2528967fb`.

## STOP-4 at checkpoint #60

`scripts/floor.sh` after #60. Captured, **not re-run**.

```
floor  scripts/floor.sh   .floor/2026-09-14T01-43-05Z
       Summary [ 216.702s] 5490 tests run: 5485 passed, 5 failed, 22 skipped   exit=100
```

Failing tests:

1. `wat::lint one_variant_separator::only_identifier_rs_spells_the_variant_separator`
2. `wat::cli metadata_of_example_formats::example_from_the_lookup_formats_widest_le_120`
3. `wat::cli pprintln_doc_row::doc_row_pprintln_matches_byte_golden`
4. `wat::cli pprintln_doc_row::printed_row_edn_read_round_trips`
5. `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`

Whole ARM: `.floor/2026-09-14T01-43-05Z/ARM.txt` (and `clean.log`). Clippy not run (floor red first).

### Arm 1 — `one_variant_separator` (`src/config.rs:415`, replayed #59)

```
        FAIL [   0.075s] ( 126/5490) wat::lint one_variant_separator::only_identifier_rs_spells_the_variant_separator
  stdout ───

    running 1 test
    test one_variant_separator::only_identifier_rs_spells_the_variant_separator ... FAILED

    failures:

    failures:
        one_variant_separator::only_identifier_rs_spells_the_variant_separator

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 138 filtered out; finished in 0.07s

  stderr ───

    thread 'one_variant_separator::only_identifier_rs_spells_the_variant_separator' (3339325) panicked at /home/john/work/holon/wat-rs/tests/lint/one_variant_separator.rs:249:5:


    🔥🔥🔥 A SECOND VARIANT SEPARATOR — 1 site(s) spell the `::` between an enum and 
    its variant OUTSIDE `crates/wat-reader/src/identifier.rs`.

    A variant's fully-qualified name is composed and decomposed in exactly ONE place, or two 
    spellings WILL disagree — nine of these hid for months inside `rsplit_once("::")`, a 
    shape `one_name_grammar.rs` bans for `'/'` and does not name for `"::"`.

    THE FIX — route through the pair:

      compose_variant(enum_path, variant)   -> `{enum}.{variant}`
      decompose_variant(name) -> Option<(&str, &str)>   the exact inverse

    If this site does NOT separate an enum from its variant, add a co-located
    `// rune:lint(one-variant-separator, <category>) — <reason>` on the line or the one
    above, with <category> one of: namespace | type-path | display | edn | not-a-name.
    ⛔ `variant` is NOT a category — a variant site routes through the door.

    Offenders:

    src/config.rs:415  [ACCESSOR]  && wat_reader::identifier::leaf(head).starts_with("set-") =>

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

The line is a **namespace** prefix check (`head.starts_with(":wat::config::")` / `leaf(head).starts_with("set-")`), not a variant separator. Replayed #59 (config setter for `set-max-fire-rounds!`).

### Arms 2–5 — `#50 UnconsumedWrapperBind` vs main's fmt rules

Same mechanism. #50's wall (`UnconsumedWrapperBind`: a bind under `:not` consumed nowhere) is rete behaviour that survived. At freeze of the stdlib/fmt rules it now refuses **24** binds in main's `wat-scripts/fmt/rules/kwargs.wat` and `defrecord.wat` (first: `fmt::kwargs-claim-from-1` `?more` line 31). CLI tests that spawn `wat` and `every_wat_scripts_file_loads` die at startup:

```
thread 'metadata_of_example_formats::example_from_the_lookup_formats_widest_le_120' (3345970) panicked at /home/john/work/holon/wat-rs/tests/cli/metadata_of_example_formats.rs:24:5:
expected clean run; stdout:

stderr:
[#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "... 24 rete rule validation errors ..." ... :errors [#wat.rete/UnconsumedWrapperBind {:rule "fmt::kwargs-claim-from-1" :var "?more" :fact-type "wat::grep::Node" :span ...kwargs.wat line 31...} ... #wat.rete/UnconsumedWrapperBind {:rule "fmt::defenum-empty-vec" :var "?v" ...defrecord.wat line 100...}]}}]
```

(pprintln_doc_row both tests, every_wat_scripts_file_loads: same StartupError. Full EDN in ARM.txt.)

#50's own named tests (the 5 UnconsumedWrapperBind cases, including `an_unconsumed_bind_inside_exists_is_left_alone_because_exists_binds_outward`) **PASS**. The wall is doing what C specified. The checkpoint is red because the wall now sees main's fmt corpus, which C's grok-rete tree never loaded as stdlib-adjacent grep rules in the same way — or which grok-rete also would have refused, untested on this floor.

STOP-4: the red is inside replayed files (`src/rete/validate.rs` #50, `src/config.rs` #59). Did not re-run. Did not commit further.

## Checkpoint #35 (green, before STOP-4)

```
floor  .floor/2026-09-14T01-00-58Z
       Summary [ 216.862s] 5463 tests run: 5463 passed, 25 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

## 2a4 + #22 (the bar that unblocked the batch)

`wat/gen.wat` convert: **0 UNREGISTERABLE ReservedPrefix**. Binary starts. `gen_library_satisfies_its_own_laws` PASS [0.315s], `rete_fuzzer_finds_no_native_oracle_divergence` PASS [4.925s]. Two-phase. Committed `a8a95400f`.

run5 on 2a4 `c184348f7`: MA LOSING=0 GAINING=1; PC LOSING=1 GAINING=7 UNRESOLVED 173; VS LOSING=0 GAINING=67; chain vs main identical **1370**, CHAIN-FAILS **28**. Unchanged vs 2a2-REFUTE.

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **PASS count.** 50 `REPLAY(grok-rete #` #11→#60, each `-x` trailer. Then STOP-4 at the #60 floor. |
| E2 | **PASS.** #26 `ea5163aa6`, #48 `3c925fa17`, #56 `50240389c` docs cherry-picks. |
| E3 | **held through the steps' `--check`.** Checkpoint floor cannot start fmt rules (#50 wall). |
| E4 | sample of named tests PASS at their steps (see REPLAY-LOG). |
| E5 | shared steps #27 #29 #39 #43 #47 #50 #53 #58 #59 #60: named tests green at the step. #50/#59 hand re-expression logged. |
| E6 | **FAIL.** #35 green. #60 floor RED (this STOP). Clippy at #60 not run. |
| E7 | this SCORE + REPLAY-LOG |
| E8 | **PASS.** no `wat-scripts/fixes/` or main-deleted path. |
| E9 | no hand-edited corpus `.wat`. Hand edits: wat in `.rs` strings (#50 `:wat::rete::i64::{<,>=}`; #59 `fixpoint_error` map envelope; CharLit leaf). |

## Blast radius

50 REPLAY commits + 2a4. SCORE-4 + REPLAY-LOG uncommitted. Not pushed.
