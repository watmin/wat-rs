# SCORE — STONE 255.87: STOP-3 — the converted stdlib, fifteen mechanisms

Branch `main`. Conversion commit `50f4c0a52` (parent draw `e08fe7349`). **Not pushed.** No cure commit. 5c-iii, 5c-iv, and 5d were not started.

## Conversion

The recorded converter `wat-scripts/fixes/to-faithful-clojure.wat`, driven by the pristine binary copied to `/tmp/wat-pristine-25587` before any rewrite, over all 65 `wat/*.wat` paths. Log `/tmp/g1-87-convert.log`: 65 `[to-faithful-clojure]` lines, `RC=0`. No file refused.

Keyword call heads (the `call_heads` scanner: first token of a list, outside strings and `;;` comments, text starts with `:` and contains `::`):

| | heads |
|---|---|
| before, on `e08fe7349` | 11131 |
| after `50f4c0a52` | 0 |

Every file went to 0. Before-counts:

| before | file |
|---:|---|
| 130 | `wat/Record.wat` |
| 523 | `wat/bracket.wat` |
| 141 | `wat/cache.wat` |
| 9 | `wat/capability.wat` |
| 54 | `wat/class.wat` |
| 969 | `wat/core.wat` |
| 163 | `wat/deporder.wat` |
| 33 | `wat/doc.wat` |
| 45 | `wat/doctest.wat` |
| 3 | `wat/edn.wat` |
| 3 | `wat/eval.wat` |
| 1427 | `wat/fix.wat` |
| 873 | `wat/fmt.wat` |
| 323 | `wat/gen.wat` |
| 232 | `wat/grep.wat` |
| 25 | `wat/holon.wat` |
| 2 | `wat/holon/Amplify.wat` |
| 2 | `wat/holon/Bigram.wat` |
| 11 | `wat/holon/Circular.wat` |
| 5 | `wat/holon/Log.wat` |
| 6 | `wat/holon/Ngram.wat` |
| 3 | `wat/holon/Project.wat` |
| 3 | `wat/holon/ReciprocalLog.wat` |
| 13 | `wat/holon/Reject.wat` |
| 15 | `wat/holon/Sequential.wat` |
| 2 | `wat/holon/Subtract.wat` |
| 2 | `wat/holon/Trigram.wat` |
| 16 | `wat/io.wat` |
| 101 | `wat/kernel/assertion.wat` |
| 14 | `wat/kernel/channel.wat` |
| 14 | `wat/kernel/diagnostics.wat` |
| 14 | `wat/kernel/outcomes.wat` |
| 25 | `wat/kernel/readln.wat` |
| 178 | `wat/kernel/services/stdio.wat` |
| 371 | `wat/lint.wat` |
| 2 | `wat/process.wat` |
| 3 | `wat/program.wat` |
| 276 | `wat/query.wat` |
| 111 | `wat/query/mem.wat` |
| 263 | `wat/query/sqlite-store.wat` |
| 27 | `wat/repl.wat` |
| 111 | `wat/rete.wat` |
| 82 | `wat/rete/acc.wat` |
| 676 | `wat/rete/compile.wat` |
| 52 | `wat/rete/factbag.wat` |
| 208 | `wat/rete/oracle/accum-pass.wat` |
| 54 | `wat/rete/oracle/explain.wat` |
| 259 | `wat/rete/oracle/fire.wat` |
| 41 | `wat/rete/oracle/insert.wat` |
| 422 | `wat/rete/oracle/pass.wat` |
| 225 | `wat/rete/oracle/stratify.wat` |
| 154 | `wat/rete/syntax.wat` |
| 8 | `wat/runtime-meta.wat` |
| 10 | `wat/runtime-typeinfo.wat` |
| 251 | `wat/seq.wat` |
| 1216 | `wat/service.wat` |
| 1 | `wat/source.wat` |
| 149 | `wat/spawn.wat` |
| 85 | `wat/sqlite.wat` |
| 2 | `wat/stream.wat` |
| 21 | `wat/string.wat` |
| 65 | `wat/telemetry.wat` |
| 312 | `wat/telemetry/journal.wat` |
| 193 | `wat/telemetry/span.wat` |
| 102 | `wat/test.wat` |

A second run of the same pristine binary on the same 65 paths: `RC=0`. `git diff` sha256 before that run and after it: `f2d5e650e69ac7ab200d6660e6e7c9f1fa3bfa248b821fbeec12edf2b3aa40e3` both times. The conversion commit is those 65 files only: 11644 insertions, 11644 deletions.

Release build after that commit: `RC=0`, `Finished release profile in 24.46s`. `git status` was clean before the floor.

## Floor

`.floor/2026-10-03T06-32-08Z` on `50f4c0a52`, `scripts/floor.sh`, exit 100. Not re-run.

```
Summary [ 546.885s] 6397 tests run: 6181 passed (29 slow), 214 failed, 2 timed out, 24 skipped
```

6397 run, same count as `.floor/2026-10-03T05-24-19Z`. 216 failure blocks (214 FAIL + 2 TIMEOUT). Fifteen mechanisms. STOP-3 is twelve, so no cure was started.

## Mechanisms

| n | mechanism | witness |
|---:|---|---|
| 155 | keyword `:wat::capability::Capability` is an unresolved reference | `probe_arc278_per_op_enforcement_codegen::codegen_flags_over_op_request_the_body_does_not` |
| 24 | keyword `defn` leaves `:wat::core::def` in expression position | `runtime::tests::define_and_call` |
| 9 | `wat.core/first` on an empty sequence at `wat/rete/compile.wat:576` | `metadata_of_example_formats::example_from_the_lookup_formats_widest_le_120` |
| 7 | diagnostic goldens: the reason text matches and the stdlib span columns moved | `wat_core_cond::cond_refuses_missing_else` |
| 3 | nested-program census: `checked=0` | `nested_program_literals_start_on_the_child_path` |
| 3 | journal / sqlite probes die `disconnected` | `probe_arc278_sqlite_store_differential::sqlite_store_differential` |
| 3 | killed by a time limit, no assertion text | `rete::reachability::reachability_shard_0_of_6` |
| 3 | closure extraction's type-name list is not the names the assertions require | `wat_arc170_closure_extraction::t3_toplevel_defn_uses_user_types` |
| 2 | negative probes print 0 lines before the gap | `probe_arc255_75_negative_probes::cap2_peer_pid_on_unified_peer_is_an_honest_error` |
| 2 | stdio gate looks up keyword service names and finds none | `probe_1_stdio_service_messages_carry_no_handles` |
| 1 | `:t::work::Kwargs` is not a registered aggregate | `check::tests::declared_types_kwargs_defn_mints_kwargs` |
| 1 | faithful-surface non-vacuity: 1 slash-joined declaration name, gate requires 9 | `every_stdlib_declaration_name_survives_the_faithful_surface` |
| 1 | doc-row byte golden | `pprintln_doc_row::doc_row_pprintln_matches_byte_golden` |
| 1 | lost-arm expansion text uses symbol heads | `probe_arc255_32_lost_arm_expansion::the_emitted_lost_arm_reaps_and_does_not_raise` |
| 1 | defstruct body must contain the keyword `:wat::core::structtype` | `wat_arc144_special_forms::lookup_form_struct_returns_special_form` |

The seven span goldens are `cond_refuses_missing_else`, `contract_02_non_exhaustive_cond_names_else`, `format_strict_unused_kwarg_is_macro_error`, `peers_bijection_form_spelling_undeclared_peer_names_the_surface`, `peers_bijection_old_spelling_undeclared_peer_is_rejected`, `probe_two_arg_form_only_one_arg_errors`, and `witness_thread_first_empty_step_panics_at_expansion`. Stripping spans and whitespace, each actual reason tail matches the expected reason tail (`cond: non-exhaustive — needs a terminal :else arm`, `format: kwarg :y is unused — no {y} in template`, the defservice ephemeral-peer sentence, `cannot take rest of empty Vec`, and `:wat::core::first: WatAST List has 0 child(ren); no child at index 0`). The threading witness's actual location is `wat/core.wat` line 1460 col 30; the golden names col 33 on that line. The defrecord witness's actual location is `wat/Record.wat` line 145 col 52; the golden names col 58.

`cond-is-fact-bind` / `bind-arrow?` in `wat/rete/compile.wat` compare the arrow node to the string `":-"`. That predicate is the same text on `e08fe7349` (then line 882). The new failure is the `first` at the accumulate branch, line 576 of the converted file, which runs when `acc-form` has no children.

### 1. Capability keyword is unresolved

`wat/capability.wat` declares `(wat.core/defsurface wat.capability/Capability ...)`. The witness:

```
thread 'probe_arc278_per_op_enforcement_codegen::codegen_flags_over_op_request_the_body_does_not' (150212) panicked at src/freeze.rs:1176:9:
call_beside_value: fixture beside "/home/john/work/holon/wat-rs/tests/services/probe_arc278_per_op_enforcement_codegen.rs" failed to freeze: #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":wat::capability::Capability" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/services/probe_arc278_per_op_enforcement_codegen.wat" :line 27 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 33 :col 96}}}}]}
```

### 2. `defn` leaves `def` in expression position

The witness program is the keyword form `(:wat::core::defn :my::app::inc [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))`.

```
thread 'runtime::tests::define_and_call' (108692) panicked at src/runtime.rs:15785:10:
called `Result::unwrap()` on an `Err` value: Diagnostic(#wat.runtime/DeclarationInExpressionPosition {:message ":wat::core::def is consumed before evaluation — it is registered or spliced at freeze time and never evaluated, so it cannot appear in expression position" :location #wat.core/Span {:file "src/runtime.rs:14962" :line 2 :col 13 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 2 :col 100}}} :causes [] :head ":wat::core::def"})
```

### 3. `first` on an empty sequence in `compile.wat`

```
thread 'metadata_of_example_formats::example_from_the_lookup_formats_widest_le_120' (113410) panicked at /home/john/work/holon/wat-rs/tests/cli/metadata_of_example_formats.rs:24:5:
expected clean run; stdout:

stderr:
[#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::core::first form: :wat::core::first: sequence has 0 element(s); no element at index 0\" :location #wat.core/Span {:file \"wat/rete/compile.wat\" :line 576 :col 58 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 576 :col 64}}} :causes [] :head \":wat::core::first\" :reason \":wat::core::first: sequence has 0 element(s); no element at index 0\"}"}]
```

The same `compile.wat:576` text is the stderr of the eight `wat_grep` failures and of `every_wat_scripts_grep_program_matches_a_known_target`.

### 4. Diagnostic span columns

```
thread 'wat_core_cond::cond_refuses_missing_else' (166725) panicked at /home/john/work/holon/wat-rs/tests/wat_lang/wat_core_cond.rs:69:5:
assertion `left == right` failed: EDN data mismatch (expected missing-:else diagnostic)
--- actual (raw) ---
#wat.macro/ProgramBodyEvalFailed {:message "macro :wat::core::cond — program body eval failed" :location #wat.core/Span {:file "tests/wat_lang/wat_core_cond_no_else.wat.bad" :line 5 :col 3 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 7 :col 36}}} :causes [] :macro-name ":wat::core::cond" :cause #wat.macro/MalformedTemplate {:message "malformed template: cond: non-exhaustive — needs a terminal :else arm" :location #wat.core/Span {:file "wat/core.wat" :line 1512 :col 5 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 1512 :col 79}}} :causes [] :reason "cond: non-exhaustive — needs a terminal :else arm"}}
--- expected (raw) ---
#wat.macro/ProgramBodyEvalFailed {
  :message "macro :wat::core::cond — program body eval failed"
  :location #wat.core/Span {
```

The expected body continues as the pretty-printed record in `.floor/2026-10-03T06-32-08Z/clean.log` from that line. Its `:reason` is the same sentence, and its `:cause` location is `wat/core.wat` line 1512 col 5.

### 5. Nested-program census

```
thread 'nested_program_starts::nested_program_literals_start_on_the_child_path' (81038) panicked at tests/lint/nested_program_starts.rs:574:5:
census was 141 literals; got checked=0 assembled=0 templates=0 data=139 total=139
```

The two sibling failures in the same file: `old erase should have a checked child` (`nested_program_starts.rs:647`) and `sabotaged erase should have a checked child` (`nested_program_starts.rs:706`).

### 6. `disconnected`

```
sqlite_store_differential deftest must pass (mem-store' and sqlite-store' must return IDENTICAL Pages for the same op sequence)
  deftest FAILED: #wat.kernel/Failure {:error #wat.core/Fault {:message "disconnected" :location #wat.kernel/Location {:file "tests/rete/probe_arc278_sqlite_store_differential.wat" :line 58 :col 15} :causes []} :frames [#wat.kernel/Frame {:file "tests/rete/probe_arc278_sqlite_store_differential.wat" :line 58 :symbol ":probe::expect-scan"} #wat.kernel/Frame {:file "tests/rete/probe_arc278_sqlite_store_differential.wat" :line 99 :symbol ":probe::run-ops"} #wat.kernel/Frame {:file "wat/spawn.wat" :line 375 :symbol ":wat::type::Fn"}] :actual #wat.core/Option.None {} :expected #wat.core/Option.None {}}
```

The same `"disconnected"` message is the arm of `journal_persists_identically_across_mem_and_sqlite_backends_on_a_thread` (wat file line 50, symbol `:user::journal-roundtrip`) and of `journal_writes_a_metric_through_a_held_sqlite_store_peer_on_a_process`.

### 7. Time limits

```
     TIMEOUT [  30.015s] (2318/6397) wat rete::reachability::reachability_shard_0_of_6
  stdout ───

    running 1 test

    (test timed out)
```

`reachability_shard_1_of_6` is `TIMEOUT [  30.033s]` with the same `(test timed out)` body. Shards 2, 3, and 4 passed in that run at 28.650s, 28.978s, and 29.275s. The third kill is a FAIL, not a nextest TIMEOUT:

```
deftest_wat_tests_rete_fuzz_test_native_matches_oracle: exceeded time-limit of 90000ms — deftest :wat-tests::rete::fuzz::test-native-matches-oracle at wat-tests/rete/differential-fuzz.wat:422:1 (test thread leaked — process exit will reap)
```

### 8. Closure-extraction names

```
thread 'wat_arc170_closure_extraction::t3_toplevel_defn_uses_user_types' panicked at tests/function/wat_arc170_closure_extraction.rs:299:5:
Point struct must be extracted; got [":my::Side", ":my::PriceUsd", ":my::Coord"]
```

`t5_inline_lambda_captures_let_scope_struct` asserts `type_decls` contains `":my::Config"` and that assertion failed. `t11_captures_with_recursive_struct`: `Tree must appear exactly once; got []`, left 0, right 1.

### 9. Println absent

```
thread 'probe_arc255_75_negative_probes::cap2_peer_pid_on_unified_peer_is_an_honest_error' (143991) panicked at tests/process/probe_arc255_75_negative_probes.rs:86:5:
assertion `left == right` failed: the println right before the gap must still have printed:
  left: 0
 right: 1
```

`m1_addr_roundtrip_is_refused_by_the_capability_wall` at line 186 of the same file: `the wire-form println before the gap must still have printed`, left 0, right 1.

### 10. Stdio name gate

```
thread 'probe_arc214_stone82_stdio_services_no_handle_passing::probe_1_stdio_service_messages_carry_no_handles' panicked at tests/services/probe_arc214_stone82_stdio_services_no_handle_passing.rs:105:5:
assertion `left == right` failed: wat/kernel/services/stdio.wat no longer declares the expected stdio services — this gate's subject has moved again. Re-point it at the live services rather than letting it pass on a file that no longer holds them.
  left: []
 right: [":wat::kernel::stdout-svc", ":wat::kernel::stderr-svc", ":wat::kernel::stdin-svc"]
```

`probe_2_stdio_services_have_no_add_remove_protocol` at line 142: the same left `[]` and the same right list.

### 11. Kwargs aggregate

The witness source is the keyword program `(:wat::core::defn :t::work [x <- wat.type/i64 & [n <- wat.type/i64]] -> wat.type/i64 (:wat::i64::+ x n))`.

```
thread 'check::tests::declared_types_kwargs_defn_mints_kwargs' panicked at src/check.rs:24345:22:
:t::work::Kwargs is not a registered aggregate: None
```

### 12. Faithful-surface slash count

```
thread 'freeze::env::faithful_surface_round_trip::every_stdlib_declaration_name_survives_the_faithful_surface' panicked at src/freeze/env.rs:1166:9:
only 1 stdlib declaration names carry a `/` member join — the population this gate discriminates ON has vanished, so a pass proves nothing
```

The assert above it (`measured > 500`) did not fire. The `/` count is taken after `ns_to_wat_path` on a symbol declaration name (`src/freeze/env.rs` around the `name.contains('/')` check).

### 13. Doc-row bytes

```
thread 'pprintln_doc_row::doc_row_pprintln_matches_byte_golden' panicked at tests/cli/pprintln_doc_row.rs:32:5:
assertion `left == right` failed: byte golden: #wat.doc/Row from a record value
```

The left and right values are the rest of that stderr block in the floor log. In the `:examples` entry, left is a single line beginning `(:wat.core/do (:wat.core/defrecord :probe/StepPayloadExampleTemp` and right breaks that same `(:wat.core/do` form across lines. Both sides, in the portion read, use `wat.core/do` as that head.

### 14. Lost-arm spelling

```
thread 'probe_arc255_32_lost_arm_expansion::the_emitted_lost_arm_reaps_and_does_not_raise' panicked at tests/services/probe_arc255_32_lost_arm_expansion.rs:30:5:
assertion `left == right` failed: stderr:

  left: "\"ServiceEvent.Lost {:idx idx :cause _cause} (:p32.echo/serve self l (wat.seq/remove-at selectables idx) next-id state)] [wat.spawn/\""
 right: "\"ServiceEvent.Lost {:idx idx :cause _cause} (:p32.echo/serve self l (:wat.seq/remove-at selectables idx) next-id state)] [:wat.spawn/\""
```

### 15. `structtype` text

```
thread 'wat_arc144_special_forms::lookup_form_struct_returns_special_form' panicked at tests/wat_lang/wat_arc144_special_forms.rs:189:5:
Arc 293.2-parity: defstruct's macro body must expand through to :wat::core::structtype (the low-level primitive)
```

## What was measured before the conversion, and what was not run after

Pre-census `.census/2026-10-03T05-56-40Z.txt`: 2286 files, rc buckets 0×2076, 1×208, 101×2.

Pre-delta on the unconverted tree, binary `/tmp/wat-pristine-25587`, list sha `da1aa882e86e3590150f23eff755a11d8f46a96f5e3b5e9718d1714518a24e8d`, 178 paths, 0 missing: ORIG-CLEAN 159/178, CONV-CLEAN 157/178, NEW 2 (`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat`, `wat/holon/Ngram.wat`), RECOVERY 0, exit 0. That delta checked converted copies against the unconverted binary. It was not re-run against the converted stdlib. `census.sh --diff` was not run. Clippy was not run. Those three are the gates after a green floor, and this floor is the STOP.

## STOP

STOP-3. Fifteen mechanisms, the table above, no cure. The floor log is `.floor/2026-10-03T06-32-08Z/`.

## Amend — group A, and the cost

The amend is `d66138c46`. Group A is cured, each mechanism in its own commit, probe in that commit. Group B was left standing. 5c-iii, 5c-iv, and 5d are unstarted. No timeout was raised. The temporary phase timers used for the profile were removed before the floor; they are not in the tree.

### What each cure changed

- **#1** `977efcbfd`. A symbol declaration name registers as its canonical identity, for every declaration form the door covers.
- **#2** `1600e83b6`. `wat.core/def` in expression position takes the same freeze/eval skip as `:wat::core::def`. The heresy ledger missed the old keyword compares: `matches!` is not a recorded compare, both floor sites sit in `mod tests`, and `head_of` had no `:wat::` literal.
- **#3** `bad14adb4`. `cond-is-fact-bind` now recognizes a symbol type by canonical identity, so the accumulate branch at `wat/rete/compile.wat:576` no longer takes `first` of the atom. On this floor `wat_grep::g3_malformed_file_is_loud_and_nonzero` passes. On `.floor/2026-10-03T06-32-08Z` that test died at `tests/cli/wat_grep.rs:105` with `malformed :wat::core::first form` at `wat/rete/compile.wat:576`.
- **#6** `731e24740`. The service died before the client ever asserted. The death, from the pre-cure eprintln that was reverted before the commit (`/tmp/g1-a6.log`):

```
A6-CRASH runtime [#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/UnknownFunction {:message \"unknown function: :rust::sqlite::Connection::select is not registered in the rust-deps registry\" :location #wat.core/Span {:file \"wat/sqlite.wat\" :line 158 :col 21 ...
```

`reconstruct_call_path` wrote `:rust::sqlite::Connection::select` because `Connection` is not a `TypeEnv` type. The registry key is `:rust::sqlite::Connection/select`. The client panic stayed `"disconnected"` at `tests/rete/probe_arc278_sqlite_store_differential.wat:58` (`:probe::expect-scan`). Journal thread and process failed the same death and pass after the same helper. The client view stays scrubbed.

- **#8** `6cb520c3e`. The closure-extraction collector keeps a symbol `wat.core/structtype` under the canonical type name. t3 Point, t5 Config, and t11 Tree pass.
- **#11** `a4e155089`. `expand_once` expands a symbol `wat.core/defstruct` the way it already expanded the keyword. The kwargs record name was already `:t::work::Kwargs`; the one-step walk had been dropping the unexpanded symbol.
- **#9** has no cure commit. After #1, the negative-probe module passed. On this floor the module is PASS, including `cap2_peer_pid_on_unified_peer_is_an_honest_error` and `m1_addr_roundtrip_is_refused_by_the_capability_wall`.

### The cost

The pre-cure converted test binary (23550584 bytes, Oct 2 23:32, matching `50f4c0a52`) was overwritten. There is no preserved copy, so this table has no pre-cure converted column. `/tmp/wat-pristine-25587` (23635912 bytes, Oct 2 22:15) is an ELF CLI from before the draw and cannot run these lib tests. The unconverted column is the rust at `731e24740` with `wat/` from the draw `e08fe7349`, in the worktree `/tmp/wat-cost-unconv`. Both columns use that same rust. The clock is nextest's own `Summary` line (one test, 6426 skipped). All 24 runs returned RC=0. None was killed.

Shard 2 was the reachability workload because on `.floor/2026-10-03T06-32-08Z` it completed (`PASS [  28.650s]`) while shard 0 timed out at 30.015s and shard 1 at 30.033s. Shards 3, 4, and 5 also completed on that floor (28.978s, 29.275s, 29.255s). The other heavy test is `rete::kernel::tests::rank_and_instrument::keyed_gather_visits_match_the_keyed_prediction`, which the green floor `.floor/2026-10-03T05-24-19Z` marked SLOW and passed in 16.074s.

| run | shard 2 unconverted | shard 2 converted | keyed unconverted | keyed converted |
|---:|---:|---:|---:|---:|
| 1 | 9.772 | 14.499 | 7.738 | 11.419 |
| 2 | 9.791 | 14.492 | 7.679 | 11.220 |
| 3 | 9.699 | 14.406 | 7.651 | 11.371 |
| 4 | 9.664 | 14.552 | 7.606 | 11.272 |
| 5 | 9.695 | 14.462 | 7.610 | 11.298 |
| 6 | 9.620 | 14.461 | 7.596 | 11.402 |
| sum | 58.241 | 85.872 | 45.880 | 67.982 |
| mean | 9.707 | 14.312 | 7.647 | 11.330 |

Ratios of the sums: shard 2 is 85.872/58.241 = 1.474. keyed_gather is 67.982/45.880 = 1.482. The slowdown is still there after group A. It gets its own stone. This amend does not cure it.

`perf` is not installed (`command -v perf` empty; no `/usr/bin/perf`). The profile is one extra converted shard-2 run with the existing pass-order names plus temporary durations. `Summary [  14.404s]`, RC=0. Printed totals: `startup_ns=13857220245`, `apply_ns=39281472`. Phase nanoseconds, longest first:

| phase | ns |
|---|---:|
| 4-register-defmacros | 6927731123 |
| 8-check-program | 3362114410 |
| 7-normalize-stored-function-bodies | 1292046863 |
| 7.7-normalize-stored-function-bodies | 556239200 |
| 3-resolve-loads | 481242103 |
| 7-resolve-references | 350162499 |
| 6-register-stdlib-defines | 268647951 |
| 6-register-defines | 200267055 |
| 9-freeze | 148426742 |
| 5-register-stdlib-types | 107638063 |
| 4-register-stdlib-defmacros | 104778237 |
| 4-expand-all | 39060527 |
| 5-register-types | 13173494 |
| 7-normalize-symbol-refs | 3487355 |
| 1-parse | 1780814 |
| 3b-extract-rete-defn-names | 85308 |
| 2-collect-entry-file | 55272 |

The phase sum is 13856937016 ns. Startup minus that sum is 283229 ns. The hot region is the span opened by `record("4-register-defmacros")` at `src/freeze/env.rs:423` and closed by `record("4-expand-all")` at line 476: 6.928 s of 13.857 s of startup. The calls inside that span, in order, are `register_defmacros`, `register_aggregate_kwargs_companions`, `seed_declared_type_names` twice, `preregister_acronyms`, and `expand_all_with` of the stdlib. The timer does not split them. `4-expand-all` itself, the user `expand_all`, is 0.039 s. `8-check-program` is the next region, 3.362 s; on this run a temporary `9-freeze` record sat immediately before `FrozenWorld::freeze`, so that bucket is `check_program`. `apply_function` is 0.039 s.

### Floor after group A

```
Summary [ 582.441s] 6403 tests run: 6373 passed (30 slow), 26 failed, 4 timed out, 24 skipped
```

Exit 100. Log: `.floor/2026-10-03T08-02-27Z/`. Do not re-run it. Do not re-run `.floor/2026-10-03T06-32-08Z`. Doctests exited 0. The post-cure floor clock is 582.441 s. The pre-cure converted floor was 546.885 s. The last green floor before the conversion, `.floor/2026-10-03T05-24-19Z`, was 394.450 s.

`cargo clippy --release --all-targets -- -D warnings` returned RC=0. The tree was clean before this floor and before this commit. This amend's Then list does not include census or delta; neither ran.

### Remaining reds

26 failures and 4 timeouts. The group A arms are absent: no `freeze.rs:1176` capability panic, no `DeclarationInExpressionPosition`, no `first` of empty at `wat/rete/compile.wat:576`, no journal or sqlite `disconnected`, no missing Point/Config/Tree, no unregistered `:t::work::Kwargs`.

| remaining | n | arm on this floor |
|---|---:|---|
| #4 diagnostic goldens. The reason text is the old reason. The EDN golden differs, and the span sits in the converted stdlib. | 7 | `cond_refuses_missing_else` and `contract_02_non_exhaustive_cond_names_else`: reason `cond: non-exhaustive — needs a terminal :else arm`, span `wat/core.wat:1512`. `format_strict_unused_kwarg_is_macro_error`: reason `format: kwarg :y is unused`, span `wat/core.wat:2037`. `witness_thread_first_empty_step_panics_at_expansion`: reason `:wat::core::first: WatAST List has 0 child(ren)`, span `wat/core.wat:1460`. `probe_two_arg_form_only_one_arg_errors`: reason `cannot take rest of empty Vec`, span `wat/Record.wat:145`. Both `peers_bijection_*_undeclared_peer_*`: reason `surface :probe::Echo is not declared in :peers`, span `wat/service.wat:927`. |
| #5 nested-program census | 3 | `nested_program_literals_start_on_the_child_path` at `tests/lint/nested_program_starts.rs:574`: `census was 141 literals; got checked=0 assembled=0 templates=0 data=139 total=139`. The other two fail `should have a checked child` (`:647`, `:706`). |
| grep matches nothing | 7 | These died inside #3's `first`-of-empty on the pre-cure floor, except `g3`, which now passes. `g1` at `wat_grep.rs:73`: `sample fixture must have at least one node; got 0`. `g4` at `:126`: `balanced input has symbols; the rule should fire`. `g5` `:140`, `g6` `:157`, `written_refuses_a_string_literal` `:189`: `fixture must have named nodes`. `g7` `:209`: `left: 0` `right: 1` Match lines. `every_wat_scripts_grep_program_matches_a_known_target` at `grep_programs_still_match.rs:74`: `matches=0` on the smoke programs, exit status 0. |
| metadata width | 1 | `example_from_the_lookup_formats_widest_le_120` at `metadata_of_example_formats.rs:48`: `got 1154`. |
| #10 stdio gate | 2 | `probe_1` `:105` and `probe_2` `:142`: `left: []` `right: [":wat::kernel::stdout-svc", ":wat::kernel::stderr-svc", ":wat::kernel::stdin-svc"]`. |
| #12 faithful-surface non-vacuity | 1 | `every_stdlib_declaration_name_survives_the_faithful_surface` at `src/freeze/env.rs:1166`: `only 1 stdlib declaration names carry a / member join`. |
| #13 doc-row bytes | 1 | `doc_row_pprintln_matches_byte_golden` at `pprintln_doc_row.rs:32`: `assertion left == right` failed, `byte golden: #wat.doc/Row from a record value`. |
| #14 lost-arm spelling | 1 | `the_emitted_lost_arm_reaps_and_does_not_raise` at `probe_arc255_32_lost_arm_expansion.rs:30`. Left contains `(wat.seq/remove-at` and `[wat.spawn/`. Right contains `(:wat.seq/remove-at` and `[:wat.spawn/`. |
| #15 structtype text | 1 | `lookup_form_struct_returns_special_form` at `wat_arc144_special_forms.rs:189`: `defstruct's macro body must expand through to :wat::core::structtype`. |
| emitted defn binders empty | 1 | `arc255_24_every_emitted_defn_declares_its_letters` at `probe_arc255_24_defservice_declares_what_it_emits.rs:87`: `left: []` against the eighteen expected binders. On `.floor/2026-10-03T06-32-08Z` this test died at `src/freeze.rs:1176` inside #1, so the binder assert never ran. |
| time limit, no assertion raised | 4 timeouts + the fuzz failure | `reachability_shard_0_of_6` TIMEOUT 30.023s, shard 1 TIMEOUT 30.012s, shard 4 TIMEOUT 30.016s. Shard 2 PASS 29.754s, shard 3 PASS 29.592s, shard 5 PASS 29.131s. `deftest_wat_tests_rete_fuzz_test_native_matches_oracle` FAIL 90.022s: `exceeded time-limit of 90000ms` at `tests/kernel/test.rs:17`. `retirement_table_is_fully_reachable` TIMEOUT 240.004s. On `.floor/2026-10-03T06-32-08Z` that retirement test passed in 231.462s. |

26 + 4 = 30. The seven diagnostic goldens, the three nested-program failures, the two stdio failures, and mechanisms 12, 13, 14, and 15 are group B. The grep-zero, metadata-width, and empty-binder failures are arms the pre-cure floor never reached, because those tests died inside #1 or #3. The time-limit rows are the cost, measured above, and they stay on the default kill.

## STOP (amend)

Group A is cured. Group B and the three unmasked arms remain. The converted stdlib is slower on the two isolated workloads, and the floor after the cures is 582.441 s. That slowdown is its own stone. No performance cure, no timeout change. The floor log is `.floor/2026-10-03T08-02-27Z/`.

## Amend 2 — group B, and the arms group A unmasked

No STOP. The cost is untouched. No time limit was raised. 5c-iii, 5c-iv, and 5d are unstarted.

### Cures

- **grep** `281cdd483`. Two doors. The alpha index stored the raw head, so a keyword fact `wat::grep::Node` never met a symbol pattern `wat.grep/Node`. `collect-rules` required the return type `:wat::rete::Rule` and the converted `defrule` emits `wat.rete/Rule`. Both doors now use `fact_class_key` / `canonical_identity`. A keyword class string is the old colon-stripped key (`fact_class_key(":wat::grep::Node")` is `wat::grep::Node`). The symbol fixture is `tests/cli/wat_grep__count_rules_symbol.wat`. `:user::grep` stays the driver entry. Before the floor, `cargo test --release --test cli grep` returned RC=0 and the nine grep tests passed, including the seven that had matched nothing and the symbol pair.
- **width.** No separate commit. `format-source` lays the example out by rete rules. With `collect-rules` blind to `wat.rete/Rule` the rule vector was empty, so the example stayed one line (1154 on `.floor/2026-10-03T08-02-27Z`). The same collect-rules cure is what makes the rules fire. `example_from_the_lookup_formats_widest_le_120` then passed. The limit stays 120.
- **binders** `21299f095`. The emitted defns were not empty. `defn-row` and `is-list-headed?` compared `ast-name` to the keyword `":wat::core::defn"`, and the emitted head is the symbol `wat.core/defn`, so the census returned `[]`. Both compares now go through `canonical-identity`. `cargo test --release -p wat --test services -- arc255_24` returned RC=0. The eighteen expected rows matched, so the emitted names are still those keyword strings and the binders are the letters the test already named.
- **#5** `a1c2c03dc`. `head_kw` returned only a keyword, and `param_names` counted a reference symbol as a parameter. After the conversion a type in `[locus :- wat.spawn/ThreadOpts prog :- …]` shifted `prog`'s index, so `:wat::test::spawn-peer`'s forms argument was no longer the carrying argument. Binders are the only names that count. Heads and declaration names go through `canonical_identity`. The census floor stays `>= 141`. The keyword/symbol pair is `head_ident_keyword_and_symbol_are_one`.
- **#10** `b590541b4`. `declared_services` stripped the keyword prefix `:wat::service::defservice `. The live lines are `(wat.service/defservice wat.kernel/stdout-svc`. Head and name are `canonical_identity`. The expected three names are unchanged. `declared_services_reads_keyword_and_symbol` builds both spellings.
- **#15** `e41d7d917`. `watast_carries_keyword` matched only `WatAST::Keyword`. The macro body at `wat/core.wat` calls `(wat.core/structtype ~@args)`. A reference symbol whose canonical identity is `:wat::core::structtype` carries that name. `structtype_keyword_and_symbol_are_one_carrier` parses both nodes. `lookup_form_struct_returns_special_form` passes.
- **#12** `758a7b626`. The gate now counts member joins: a clojure declaration name whose receiver's last segment is a type (`wat.cache.Lru/get`). It does not count a `/` in the rust key. Symbol declarations are stored by `ns_to_wat_path`, which writes `::`, and that is why the `/` population had fallen to 1. The run that still expected `:wat::core::Fault/of` printed `801 names measured, 10 of them member joins` and `left: []` against that one name. The floor stays `>= 9`. `:wat::core::Fault/of` left `UNSPELLABLE_IN_THE_FAITHFUL_SURFACE`. The side that moved is the declaration: `wat/core.wat` declares `(wat.core/defmacro wat.core.Fault/of`, and the key the round trip stores is `:wat::core::Fault::of`.
- **#13.** No recapture. The doc-row golden's example was the formatted form, and the actual was the same example on one line, the empty-rules render. After the collect-rules cure, `doc_row_pprintln_matches_byte_golden` passed against the existing golden.
- **#14** `86b6a11d8`. The emitted window changed from `(:wat.seq/remove-at` and `[:wat.spawn/` to `(wat.seq/remove-at` and `[wat.spawn/`. That is the only difference. The expansion should emit the symbol. `:wat.seq/remove-at` is the dotted-keyword pre-image; `canonical_identity` of both is `:wat::seq::remove-at`. The converted macro calls the symbol, so the symbol is the spelling it emits.
- **#4** `f3e7e2c5f`. Whitespace-stripped, each golden differs only by span columns in the converted stdlib. Reason text is unchanged. Recaptured with `UPDATE_EDN=1`, then the seven tests passed without it.

| test | file:line | was | is |
|---|---|---|---|
| `witness_thread_first_empty_step_panics_at_expansion` | `wat/core.wat:1460` | col 33, end col 37 | col 30, end col 34 |
| `cond_refuses_missing_else` | `wat/core.wat:1512` | end col 82 | end col 79 |
| `contract_02_non_exhaustive_cond_names_else` | `wat/core.wat:1512` | end col 82 | end col 79 |
| `format_strict_unused_kwarg_is_macro_error` | `wat/core.wat:2041` | end col 78 | end col 75 |
| both `peers_bijection_*` | `wat/service.wat:927`, end line 935 | end col 113 | end col 110 |
| `probe_two_arg_form_only_one_arg_errors` | `wat/Record.wat:145` | col 58, end col 103 | col 52, end col 91 |

- **lint gates** `0f10d248b`. The heresy ledger shrank 64 → 63 because `eval_collect_rules` is no longer an Ex1 keyword compare. The frozen row is gone. The identity-pair literals were rebuilt so they are not inlined EDN or inlined wat. The three `::` splits in `carrying_keys` are namespace spelling (`rune:lint(one-variant-separator, namespace)`), not a variant separator.

### Floors

`.floor/2026-10-03T08-45-43Z` is red. Do not re-run it.

```
Summary [ 583.131s] 6408 tests run: 6399 passed (31 slow), 5 failed, 4 timed out, 24 skipped
```

Exit 100. Doctests exited 0. The five failures are the four lint gates fixed in `0f10d248b` (`tests_carry_no_inlined_edn`, `tests_carry_no_inlined_wat`, `only_identifier_rs_spells_the_variant_separator`, `the_heresy_ledger_matches_its_frozen_census`) and the fuzz time limit. Timeouts: reachability shards 0 (30.008s), 1 (30.009s), and 3 (30.014s), and `retirement_table_is_fully_reachable` (240.005s). Shards 2, 4, and 5 passed at 29.920s, 29.921s, and 29.369s. The fuzz failure is `deftest_wat_tests_rete_fuzz_test_native_matches_oracle` at 90.022s, `exceeded time-limit of 90000ms`.

`.floor/2026-10-03T09-00-08Z` is the floor after that commit. Do not re-run it. Its only reds are the time limits this amend leaves for the cost stone.

```
Summary [ 580.065s] 6408 tests run: 6404 passed (32 slow), 1 failed, 3 timed out, 24 skipped
```

Exit 100. Doctests exited 0. The failure is the fuzz test at 90.029s, same limit. Timeouts: shard 0 at 30.009s, shard 1 at 30.024s, `retirement_table_is_fully_reachable` at 240.008s. Shards 2, 3, 4, and 5 passed at 29.792s, 29.437s, 29.726s, and 29.706s.

`cargo clippy --release --all-targets -- -D warnings` returned RC=0 after the lint-gate commit, before this floor. The tree was clean. Census and delta did not run.

## STOP (amend 2)

Group B and the three unmasked arms are cured. The time-limit rows remain, and they are the cost stone. No performance cure, no timeout change. The floor log is `.floor/2026-10-03T09-00-08Z/`.

## Amend 3 — the cost of the conversion

Same rust both columns. Unconverted `wat/` is `e08fe7349`, checked out for that run and restored. The clock is one nextest Summary of `reachability_shard_2_of_6`, not a six-run mean. `keyed_gather_visits_match_the_keyed_prediction` was not run. No time limit was raised. The phase probe was in the measured binaries and is not in the tree.

The span amendment 1 left between `4-register-defmacros` and `4-expand-all` is stdlib `expand_all_with`. Before the cure, probe on:

| | unconverted | converted |
|---|---:|---:|
| Summary | 9.893s | 14.695s |
| `4f-expand-stdlib` | 3593066497 ns | 7026309758 ns |
| `7-normalize-stored-function-bodies` | 386383670 ns | 1273225796 ns |
| `8-check-program` | 3287712899 ns | 3363393619 ns |
| `reconstruct` calls | 253360 | 3643738 |
| `ns_subtype_parent` | 243407022 ns | 2078176478 ns |
| `classify` calls | 428566 | 3819438 |
| `expand_kw_head` / `expand_sym_head` | 543426 / 62002 | 67262 / 538218 |

Both RC=0. Logs `/tmp/g1-a3-shard2-unconv.log`, `/tmp/g1-a3-shard2-conv2.log`. `8-check-program` did not grow. The converted stdlib is symbol-headed, so expand and the first normalize of stored bodies did.

`is_known_type` on a miss called `classify`, and `is_subtype_parent` walked every `subtype_edges` value. That walk is the 2.078s. A `subtype_parents` set, filled in `register_subtype` and dropped in `unindex_subtype_child`, makes `is_subtype_parent` a membership test. An already-canonical `:ns::name` with no `/` asks `is_builtin_primitive` and does not allocate. After that, converted only, probe still on: Summary 12.090s, `4f-expand-stdlib` 5340755714 ns, `ns_subtype_parent` 0, `classify` 0, `reconstruct` still 3643738. Log `/tmp/g1-a3-cured-conv.log`. RC=0.

The remaining reconstructs are non-macro call heads. A clojure spelling is stored beside the keyword key at registration (`symbol_alias` points at the one `MacroDef`; both joins registered drop the alias and record the spelling as ambiguous). Expand reconstructs only an ambiguous spelling, or a symbol that is not `ns/name`. `eval_list` borrows the cached `Arc<str>` from `reconstruct_call_path_shared` unless the receiver's last segment is PascalCase, in which case it still asks `join_the_registry_holds`. `boundary_of_node` answers the six stored heads for a symbol without `canonical_identity`. Normalize builds the other join only when the primary is not a held, resolvable name; a retired primary (resolvable, no binding) still loses to an alt that has a binding. `Identifier`'s three spellings are `Arc<str>`, so a template clone shares the bytes.

Same rust, probe on, after those doors. One run each.

| | unconverted `e08fe7349` | converted |
|---|---:|---:|
| Summary | 8.493s | 9.379s |
| `4f-expand-stdlib` | 2753880835 ns | 3224000542 ns |
| `7-normalize-stored-function-bodies` | 255281416 ns | 425603939 ns |
| `8-check-program` | 3204403683 ns | 3276822621 ns |
| `ident_clones` | 13218751 | 19396028 |

Both RC=0. Logs `/tmp/g1-a3-cured7-unconv.log`, `/tmp/g1-a3-cured7-conv.log`. Converted is 0.886s slower. Expand is 0.470s of that. The first normalize of stored bodies is 0.170s. Check is 0.072s.

Sharing one scope-set allocation across an expansion was measured on the converted tree (Summary 9.441s, `4f-expand-stdlib` 3215025137 ns, `/tmp/g1-a3-cured8-conv.log`, RC=0) and is not in the tree.

`cargo clippy --release --all-targets -- -D warnings` returned RC=0. The floor, census, and delta did not run.

## STOP (amend 3)

STOP-2. The converted startup does not reach the unconverted one on this rust. The gap that is left is the shape of the tree, not a lookup that still misses.

A namespaced call head in the converted stdlib is a symbol. `walk_template` adds a hygiene scope to every template identifier, and normalize then rebuilds each surviving symbol into a keyword. The unconverted call head is already that keyword: the copy is one string, and it has no scope set. `binder_and_reference_carry_identical_scope_sets` requires the scope on a bare binder and on the bare body reference (`tmp`). Putting the same set on a namespaced head, and then rewriting that head into a keyword after expansion, is the work the 0.886s is. Two designs would remove it, and neither is in this commit. One: a namespaced head becomes its keyword once, at the door, before the template is copied, and a bare binder keeps `add_scope`. That changes `env_key` of a namespaced binder, which the keyword tree never had. Two: one frozen stdlib is shared across worlds, so this startup is not repeated per process. That is the builder's call.

No time limit was raised. 5c-iii, 5c-iv, and 5d were not started.
