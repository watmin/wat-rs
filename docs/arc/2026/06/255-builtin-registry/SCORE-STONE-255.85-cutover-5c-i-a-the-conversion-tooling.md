# SCORE — STONE 255.85: cutover 5c-i (a) — the conversion tooling

Branch `main` @ `85e51366f`. The brief's pin is `c9fab1869`. The commits of this stone, oldest first:

| commit | what |
|---|---|
| `b07f09866` | one deleted path removed from the delta list |
| `e2181bae8` | the renderer asks `WAT_TYPE_HARD_PRIMITIVES`; the seven gates read by identity; `wat-fix-rust` retries a failed literal and skips prose; `fix-seq` leaves `:fn(` |
| `5a8e24c38` | the three foldl goldens expect `wat.core/Seqable` |
| `e935b3093` | `fix-text-leaf-edits` leaves `:fn(` (the census walker) |
| `85e51366f` | a Session field binder may be the keyword `:-` |

**Not pushed.** `main` is ahead of `origin/main` by 5. Main `wat/` is the unconverted tree. The converted stdlib lives only on the shared clone `/tmp/wat-5c85b`.

STOP-2 does not apply. `pub(crate) const WAT_TYPE_HARD_PRIMITIVES` in `src/types.rs` (lines 246–271) is the 24 tails. `type_expr_to_clojure_form` reads that const. `wat/fix.wat` keeps no second list. The tails are `i64`, `f64`, `u8`, `bigint`, `rational`, `char`, `String`, `bool`, `keyword`, `nil`, `Value`, `Never`, `Fn`, `Record`, `Struct`, `Vector`, `HashMap`, `HashSet`, `List`, `Tuple`, `PersistentVector`, `PersistentMap`, `Bytes`, `AST`. `Seqable` is not among them.

## 1. The type rule asks the closed set

A member renders `wat.type/<tail>`. Every other type keeps its home spelling. Replay fixture `wat-scripts/fixes/replay/to-faithful-clojure/after.post` (byte-exact against the rebuilt binary) ends:

```
(wat.core/defn user/closed
  [m :- wat.type/i64
   e :- wat.core/Error
   h :- wat.uuid/UUID]
  :- wat.type/nil
  nil)
```

`:wat::core::Tuple(i64)` is a path whose tail is the text `Tuple(i64)`, which is not one of the 24 exact tails, so the same fixture renders `wat.core/Tuple(i64)`. The same one line is in `to-faithful-clojure-net/after.post` and `to-faithful-clojure-rete/after.post`.

`cargo test --release --test cli every_recorded_migration_replays` after the text-walker edit: `/tmp/wat-25585-replay3.log`, `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 9.43s`, `RC=0`.

`clojure_type_form_asks_the_closed_set`: `/tmp/wat-25585-closed.log`, `1 passed`, `RC=0`.

## 2. The seven gates read a declaration by identity

On the unconverted main tree, `cargo test --release --test lint` (`/tmp/wat-25585-lint5.log`): `376 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 355.13s`. That run includes:

| gate | test |
|---|---|
| `ast_kind_nodekind_sync` | `ast_kind_arms_match_nodekind_variants` |
| `gen_doc_surface_matches` | `every_exported_gen_verb_is_documented`, `every_gen_name_the_doc_writes_actually_exists` |
| `rete_header_claims_are_asserted` | `session_record_field_count_matches_its_doc` |
| `no_raw_network_keys_in_oracle` | `no_raw_network_keys_walk_outside_topological_node_ids` |
| `rete_bind_generators` | the tests in that file, including `a_converted_symbol_node_is_the_same_mint` |
| `rete_names_in_wat_scripts_resolve` | `known_forms_are_real` |

Siblings in those files passed in the same 376. `every_rete_name_in_wat_scripts_code_resolves` passed. The string scanner no longer mints a `wat.rete` name from inside a string (`"wat.rete.core.i64/"` and `"wat.rete.core.f64/"` in `wat-scripts/grep/rete-numerics-ops.wat`). The assertion is still that every real `:wat::rete::` name resolves.

The converted-`wat/` proof is below, under the re-measure. One reader change landed after that first lint run: the Session field binder.

## 3. The delta list

`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt` is **178** lines. `sha256sum` is `da1aa882e86e3590150f23eff755a11d8f46a96f5e3b5e9718d1714518a24e8d`. The filename is unchanged. `docs/SEAM.md` line 632 records `da1aa882…` and that `probe-arc278-57-persistentmap-contains-key.wat` was removed because `581478c9c` deleted the file; the list was not rebuilt by index. Commit `b07f09866`. The G1/R-a/H2 section (`0a9b2effb`) was not edited.

## 4. `wat-fix-rust`

A literal is a candidate when all of the following hold (`is_candidate_wat` in `src/codemod_driver.rs`):

1. `parse_all_with_file` succeeds and yields at least one form.
2. Every form is a list whose first child is a keyword, or a symbol whose text contains `/` or `::`.
3. The head's source span text equals the head's spelled name. A reader-synthesized quasiquote head (the span is the backtick) fails.
4. Outside those spans, only whitespace and `;;` comments remain.

`a_literal_is_a_candidate_only_when_it_is_a_wat_program` passed in `/tmp/wat-25585-lib3.log` (`codemod_driver::driver_tests`, 6 passed, `RC=0`).

A codemod failure on a whole-file batch is retried per literal. A single-literal failure is `CodemodFailed { raw_lo, raw_hi, first_error }`. The scan continues. The summary line counts `codemod-failed`. Dry-run writes nothing.

The full dry-run is under the re-measure.

## 5. The two `:fn(` fixtures

Both subjects are the retired keyword-bodied fn type. The fixtures were not respellled and `fn-keyword-to-bracket.wat` was not run. `tests/function/fn_rename.rs` asserts `TypeErrorKind::MalformedTypeExpr` and the reason `a keyword-bodied fn type is retired; write the bracket `[A :-> R]` (stone 251.4c). `:fn(A)->R` and `:wat::core::Fn(A)->R` no longer parse`. `/tmp/wat-25585-fn.log`: `fn_rename::` 14 passed, including `bare_fn_type_fixture_is_the_retired_keyword_refusal` and `mixed_legacy_fixture_is_the_retired_keyword_refusal`, `finished in 0.50s`.

`fix-seq` already returned the node unchanged for `retired-keyword-fn?`. The census walker is `fix-text-leaf-edits`, and that path still called `keyword/to-type-form`. The first clone census (below) is that miss. `e935b3093` emits no edit for that keyword, so one retired token does not abort the file.

## 6. Re-measure

Fresh shared clone `/tmp/wat-5c85b` at `e935b3093` (`git clone --shared`). `cargo build --release`: `/tmp/wat-5c85b-build.log`, `Finished release profile [optimized] target(s) in 52.81s`, `RC=0`. The lint test binary was built before `wat/` was converted (`/tmp/wat-5c85b-lint-build.log`, `RC=0`, executable `target/release/deps/lint-23e0527f2e4014e8`).

### Corpus census

Driver `/tmp/census-5c85b/drive.py` (the 255.84 batch driver, pointed at this clone). Tracked `.wat` outside `wat/`: 2219. `/tmp/census-5c85b/results.tsv` is 2220 lines.

| | files | heads before | heads after |
|---|---:|---:|---:|
| OK | 2219 | 86593 | 0 |
| FAIL | 0 | | |

2215 files changed bytes. Four did not: `tests/cli/mode_parity__empty.wat`, `tests/resolve/probe_arc251_type_namespace_fix__c02-core-parametric.wat`, `tests/types/typed_if_match__if_wrong_arity_needle.wat`, `wat-scripts/scratch-pad/probe-qq-arm-shape-src.wat`. 56 batches, sum of batch `elapsed=` fields **659.87s**. Driver `RC=0`.

Both `:fn(` fixtures are OK (heads 3 → 0). The converted copies still contain the token:

- `/tmp/census-5c85b/tree/tests/function/fn_rename_bare_fn_type.wat:4` `[g :- :fn(wat::core::i64)->wat::core::i64]`
- `/tmp/census-5c85b/tree/tests/function/fn_rename_mixed_legacy.wat:4` `((g :fn(wat::core::i64)->wat::core::i64)`

The same driver against `/tmp/wat-5c85` at `5a8e24c38`, before `fix-text-leaf-edits` learned the skip: 2217 OK, **2 FAIL**, class `other`, heads 86593 → 6, 56 batches, sum **657.03s**, driver `RC=0`. Both fails are those two fixtures. First error is `MalformedTypeExpr` on `:fn(wat::core::i64)->wat::core::i64`, `wat/fix.wat` line 376 col 42 (bare) and line 381 col 44 (mixed). That is the census walker calling `keyword/to-type-form`. `e935b3093` is the cure. The 0-FAIL census above is the one this score reports.

### Embedded dry-run

`git ls-files '*.rs'` on the clone: **1301** paths (`/tmp/wat-5c85b-rs.txt`). Command, cwd `/tmp/wat-5c85b`:

`./target/release/wat-fix-rust wat-scripts/fixes/to-faithful-clojure.wat --dry-run --wat-binary ./target/release/wat --list /tmp/wat-5c85b-rs.txt`

Log `/tmp/wat-5c85b-dry.log`. Wall time of that command: 72.15s. Summary line:

```
[wat-fix-rust] 1301 file(s) scanned, 103 changed, 6559 edit(s) found, 18 refused, 1 codemod-failed
```

`RC=0`. The run finished after recording the failures.

`crates/wat-edn/tests/spec_strict.rs` is line 32 of the list and has no per-file line in the log, so the scan found 0 edits, 0 refused, 0 codemod-failed there. `crates/wat-doc/src/lib.rs` printed `5 edit(s), 0 refused, 0 codemod-failed` (real programs). The backtick fence is not a candidate: the head span is the backtick.

The 19 recorded rows (`[wat-fix-rust] refused splices:`):

| where | reason |
|---|---|
| `src/freeze.rs:104888..105022` | codemod failed: `AnyBanned { raw: ":Any" }` from `keyword/to-type-form` (`wat/fix.wat` line 380 col 42 in that binary). Full line: `/tmp/wat-5c85b-dry.log:6535` |
| `src/rete/kernel/tests/rank_and_instrument.rs` 44..55, three times | `old=":xxxx::seed"` `raw=":{ns}::seed"` — raw source under this span is not char-for-char the decoded old text |
| `src/rete/kernel/tests/where_tree_branch_differential.rs` 1..12, 58..77, 136..148, 395..407 | same reason; raws `:{ns}::seed`, `:{ns}::{rules_verb}`, `:{ns}::q-Hit`, `:{ns}::items` |
| `tests/lint/no_inlined_wat_in_tests.rs` 1..18, 19..40 | same reason; raws `:{ns}::run-counts`, `:wat::rete::{fire_fn}` |
| `tests/rete/probe_arc278_6b_ii_b_where_native_differential.rs` 1236..1257 | same reason; raw `:wat::rete::{fire_fn}` |
| `tests/rete/probe_arc278_8b_accumulate_native_differential.rs` 836..857 | same reason; raw `:wat::rete::{fire_fn}` |
| `tests/rete/probe_arc278_8custom_native_differential.rs` 839..860 | same reason; raw `:wat::rete::{fire_fn}` |
| `tests/rete/probe_arc278_P6_delta_asymmetric_join.rs` 1..16 | same reason; raw `:{ns}::q-{name}` |
| `tests/rete/probe_arc278_deep_cascade.rs` 23..38, 84..97, 22..43, 63..82, 533..554 | same reason; raws `:casc::Stage{k}`, `:casc::Tag{k}`, `:casc::q-Stage{depth}`, `:casc::Stage{depth}`, `:casc::q-Stage{depth}` |

### Converted `wat/` and the seven gates

The clone's 65 `wat/*.wat` files were converted in place by `/tmp/wat-5c85b/target/release/wat` (the binary built before that conversion). `/tmp/wat-5c85b-stdlib.log`: seven batches, every one `rc=0 ok=1`, `STDLIB_DONE fails=0`. Batch elapsed fields sum to **1347.77s** (277.51, 370.94, 0.86, 31.92, 79.74, 572.87, 13.93). `git diff --stat` in the clone: `65 files changed, 11659 insertions(+), 11659 deletions(-)`. Main porcelain stayed empty.

The pre-conversion lint binary, run on that tree, finished in 0.18s: **43 passed, 1 failed** (`/tmp/wat-5c85b-gates.log`, `RC=101`). The one failure:

```
thread 'rete_header_claims_are_asserted::session_record_field_count_matches_its_doc' panicked at
  /tmp/wat-5c85b/tests/lint/rete_header_claims_are_asserted.rs:478:22:
Session field binder is not a symbol: Keyword(":-", Span { file: "wat/rete.wat", line: 200, col: 22, end: Some(Pos { line: 200, col: 24 }) })
```

Unconverted `wat/rete.wat:200` is the symbol `<-`. The converted line is `network           :- wat.type/PersistentMap`. The lexer reads a leading colon as a keyword, so `:-` is `Keyword(":-")`. The assertion is still 8 fields whose binder text is `<-` or `:-`. `85e51366f` accepts the keyword. On main, after that edit and before the commit, `session_record_field_count_matches_its_doc` passed (`/tmp/wat-25585-session.log`, 1 passed, `RC=0`).

The clone's copy of that test was edited the same way and the lint binary was rebuilt there (`/tmp/wat-5c85b-lint-build2.log`: `Compiling wat`, `Finished release profile [optimized] target(s) in 30.83s`, `RC=0`). That rebuild embedded the converted `wat/` via `include_str!`. The same seven filters then passed: `/tmp/wat-5c85b-gates2.log`, `44 passed; 0 failed`, `finished in 0.18s`, `RC=0`. `session_record_field_count_matches_its_doc` is `ok`. This is the gate proof. It is not the stdlib floor; that floor was not run.

## Gates

| what | result |
|---|---|
| converter replay | 18 passed, 9.43s, `RC=0` (`/tmp/wat-25585-replay3.log`) |
| seven gates, main tree | 376 passed in the lint crate at the identity-reader commit; the Session binder test passed again after `85e51366f`; the floor below includes both |
| seven gates, clone converted `wat/` | 44 passed, 0 failed (`/tmp/wat-5c85b-gates2.log`) |
| census, main tree | pre `.census/2026-10-02T23-42-35Z.txt` and post `.census/2026-10-03T00-24-43Z.txt`, each `files=2284`, `RC=0`. `--diff` printed `census-diff: no STOP-8`, `RC=0`. Recounted here: both `{'0': 2074, '1': 208, '101': 2}`, only_pre 0, only_post 0, flips 0 |
| corpus census (clone) | 2219 OK, 0 FAIL |
| embedded dry-run (clone) | summary line above; 18 refusals and 1 codemod-failed, each with its reason |
| release floor | `.floor/2026-10-03T01-50-48Z` at `85e51366f`. Pre-status 0 bytes. `Summary [ 404.839s] 6394 tests run: 6394 passed (28 slow), 24 skipped`. `RC=0` |
| clippy | `/tmp/wat-25585-clippy4.log`, `Finished release profile [optimized] target(s) in 12.53s`, `RC=0`, after the Session binder edit |

Baseline `.floor/2026-10-02T22-04-08Z` at `af6577c3e` was `6389 passed (27 slow), 24 skipped`. This floor ran 6394.

### The red floor this stone caused

`.floor/2026-10-03T00-26-14Z` at `e2181bae8`. Do not re-run it. `Summary [ 404.458s] 6394 tests run: 6391 passed (28 slow), 3 failed, 24 skipped`. `RC=100`. All three left-hand sides rendered `wat.core/Seqable` where the golden expected `wat.type/Seqable`:

- `tests/reflection/wat_arc201_structured_signature_types.rs:155` (`signature_of_defn_foldl_emits_structured_parametric_and_fn`)
- `tests/wat_lang/wat_arc143_lookup.rs:132` (`signature_of_defn_foldl_renders_synthesised_shape`)
- `tests/wat_lang/wat_arc143_manipulation.rs:66` (`rename_callable_name_happy_path_foldl_to_reduce`)

`Seqable` is not in `WAT_TYPE_HARD_PRIMITIVES`, so the home spelling is what the renderer emits. `5a8e24c38` updates the three `.edn` expectations. The floor of that commit, `.floor/2026-10-03T00-37-54Z`: `Summary [ 406.721s] 6394 tests run: 6394 passed (29 slow), 24 skipped`, `RC=0`. Two later commits each took their own floor: `.floor/2026-10-03T01-02-40Z` at `e935b3093` (`6394 passed (28 slow), 24 skipped`, `RC=0`) and the final stamp above.

## What this stone left for later

5c-ii (stdlib), 5c-iii (corpus), 5c-iv (embedded wat), and 255.86 (G1 and R-a) are unstarted. Main `wat/` was not converted. The clone floor was not run, and `/tmp/wat-5c/.floor/2026-10-02T23-03-54Z` was not re-run. `.census/` and `.floor/` are not in the commits.
