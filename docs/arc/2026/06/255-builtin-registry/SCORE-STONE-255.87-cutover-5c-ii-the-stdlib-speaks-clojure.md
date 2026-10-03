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
