# SCORE — STONE 255.86: STOP-1 on `:wat::vector::concat`

Branch `main` @ the brief draw `9711dfbe4` (parent `279edda38`). **Not pushed.** No name moved. No registry row changed. Main `wat/` is untouched.

## The pair

`:wat::vector::concat` and `:wat::core::concat` do not return the same value on the same inputs.

Programs under `/tmp/g1-diff/`, each a `:user::main` that shows the call and then `assertion-failed!` so the shown text is the message. Constructors are the live spelling `wat.type/…`. Binary `./target/release/wat`.

| program | call | result |
|---|---|---|
| `pv-vec-old.wat` | `(:wat::vector::concat (wat.type/PersistentVector :- [wat.type/i64] 1) (wat.type/Vector :- [wat.type/i64] 2))` | RC=2. Message `#pv[1, 2]` |
| `pv-vec-core.wat` | the same two arguments under `:wat::core::concat` | RC=3. Check error: `:wat::core::concat: parameter #2 expects (wat.type/PersistentVector :- [wat.type/i64]); got (wat.type/Vector :- [wat.type/i64])` |
| `pv-pv-old.wat` / `pv-pv-core.wat` | both PersistentVectors, `1` then `2` | both RC=2, message `#pv[1, 2]` |
| `vv-old.wat` (`:wat::vec::concat`) / `vv-core.wat` | both Vectors, `1` then `2` | both RC=2, message `[1, 2]` |
| `hm-old.wat` (`:wat::hashmap::contains-key?`) / `hm-core.wat` (`:wat::core::contains?`) | `(wat.type/HashMap :- [wat.type/String wat.type/i64] "a" 1)` and `"a"` | both RC=2, message `true` |

The mixed pair is the stop. `:wat::vector::concat` produced a PersistentVector `#pv[1, 2]`. `:wat::core::concat` produced no value. `wat/core.wat:44` aliases `:wat::core::concat` to `:wat::vec::concat`. `infer_concat` (`src/collection/infer.rs`) refuses a different container kind. `persistentvector_concat_inner` (`src/collection/eval.rs`) accepts a Vector as its second argument and returns a PersistentVector. Replacing the name would change that result.

`:wat::vector::concat` stays registered. A search outside `docs/` and `target/` found 42 occurrences in 20 files. They were not rewritten.

## What was not done

No mapping table was committed, because the collection mapping that includes this name is not uniform. No codemod ran. No retirement row was added. Census, clippy, and the release floor were not run: the tree is the draw commit, and a floor of unchanged code would not be a measurement of this stone. R-a was not started. The `contains-key?` / `contains?` pair above matched on the one map that was run; that match does not license moving the rest of the table past this stop.

## Amend: `into` covers the pairs `:wat::vector::concat` accepts

Continues at `6c48ae7d8`. Pre-edit census `.census/2026-10-03T02-19-03Z.txt`, files=2284, RC=0.

`:wat::vector::concat` accepts a PersistentVector `to` and a `from` that is a Vector or a PersistentVector. A List `from` is a check error (`parameter #2 expects (Vector :- [T]) or (PersistentVector :- [T])`). Before this amend, `(:wat::core::into …)` on PersistentVector×PersistentVector was `NoMatchingClauseAtCallSite` (five clauses, none of them that pair). PersistentVector×Vector already matched and showed the same value as `:wat::vector::concat`.

`wat/seq.wat` gained one clause, body `(:wat::vector::concat to from)`:

```
([to <- (wat.type/PersistentVector :- [T]) from <- (wat.type/PersistentVector :- [T])] -> (wat.type/PersistentVector :- [T])
  (:wat::vector::concat to from))
```

Release rebuild RC=0. Shown values after that clause, same programs as the section above (`assertion-failed!` message is `:wat::core::show` of the call), binary `./target/release/wat`:

| pair | `:wat::vector::concat` | `:wat::core::into` |
|---|---|---|
| PV `[1]` × Vector `[2]` | `#pv[1, 2]` RC=2 | `#pv[1, 2]` RC=2 |
| PV `[1]` × PV `[2]` | `#pv[1, 2]` RC=2 | `#pv[1, 2]` RC=2 |
| empty PV × PV `[2]` | `#pv[2]` RC=2 | `#pv[2]` RC=2 |
| PV `[1]` × empty PV | `#pv[1]` RC=2 | `#pv[1]` RC=2 |
| PV `[1]` × empty Vector | `#pv[1]` RC=2 | `#pv[1]` RC=2 |
| PV `[1 2]` × Vector `[3 4]` | `#pv[1, 2, 3, 4]` RC=2 | `#pv[1, 2, 3, 4]` RC=2 |
| PV `[1 2]` × PV `[3 4]` | `#pv[1, 2, 3, 4]` RC=2 | `#pv[1, 2, 3, 4]` RC=2 |
| PV `["a"]` × Vector `["b"]` | `#pv["a", "b"]` RC=2 | `#pv["a", "b"]` RC=2 |

`:wat::vector::concat` maps to `:wat::core::into` on every pair it accepts that was run. The name was not retired: `into`'s clause still calls it, and the rest of the table is not uniform.

## STOP-1 on `:wat::map::dissoc`

Sixty-nine programs under `/tmp/g1-diff2/`, each binding the old call and the proposed replacement and returning nil when `(:wat::core::= a b)`. RC=0 on 63. The six RC=3 are all `:wat::map::*` on `(wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1)`.

`:wat::map::assoc` of that map with `"b" 2`, and the overwrite of `"a"` to `9`, both type-check under `:wat::core::assoc`. `=` cannot compare them (`parameter #1 expects :wat::core::Equatable; got (wat.type/PersistentMap :- [wat.type/String wat.type/i64])`). Shown text matches: `#pm{"a": 1, "b": 2}` and `#pm{"a": 9}`.

`:wat::map::dissoc` does not.

| program | call | result |
|---|---|---|
| `/tmp/g1-pm/dissoc-old.wat` | `(:wat::map::dissoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1) "a")` | RC=2. Message `#pm{}` |
| `/tmp/g1-pm/dissoc-miss-old.wat` | the same map, key `"z"` | RC=2. Message `#pm{"a": 1}` |
| `/tmp/g1-pm/dissoc-core.wat` / `dissoc-miss-core.wat` | the same arguments under `:wat::core::dissoc` | RC=3. `:wat::core::dissoc: parameter #1 expects (wat.type/HashMap :- [_ _]); got (wat.type/PersistentMap :- [wat.type/String wat.type/i64])` |

`wat/core.wat:41` aliases `:wat::core::dissoc` to `:wat::hashmap::dissoc`. Replacing the name would change that result. `:wat::map::dissoc` stays registered.

The same check refusal, measured in the same pass, hits the next two map verbs. `:wat::map::keys` of that map shows `["a"]` (RC=2); `:wat::core::keys` is RC=3, `parameter #1 expects (wat.type/HashMap :- [_ _])`. `:wat::map::values` shows `[1]` (RC=2); `:wat::core::values` is the same HashMap expectation (RC=3). Those aliases are `wat/core.wat:42` and `:43`. They are the same stop, not a license to keep going.

`:wat::map::{length,empty?,contains-key?,get}` on that map compared equal to `:wat::core::{length,empty?,contains?,get}` (hit and miss, empty and non-empty) under `=`. `:wat::hashmap::*` (all eight), `:wat::vec::*` including `extend` against `into` on Vector×Vector and Vector×PersistentVector, `:wat::vector::*` including `concat` against `into`, `:wat::hashset::*`, and `:wat::linkedlist::*` compared equal on the values in `/tmp/g1-diff2/results.txt`. Those matches do not license a codemod past this stop.

No mapping EDN was committed. No call site moved. No retirement row was added. R-a was not started. Clippy and the release floor were not run.

## Amend 2: the coverage gap is an arm

Continues at `7aafe62b3`. The STOP-1 on `:wat::map::dissoc`, `:wat::map::keys`, and `:wat::map::values` is overruled. Those verbs accept a PersistentMap and the core verbs did not. That is a coverage gap: the core verb gained the arm, backed by the same implementation. `:wat::vector::concat` stays mapped to `:wat::core::into`. No later pair both accepted the same input and returned different values, so the stone did not stop again.

### Arms

PersistentMap on `:wat::core::dissoc`, `:wat::core::keys`, and `:wat::core::values`. `/tmp/g1-arm/out.txt` (re-read, not re-run): PersistentMap dissoc of key `"a"` shows `#pm{}` on both the old verb and the core verb (RC=2); a missing key shows `#pm{"a": 1}` on both (RC=2). HashMap dissoc shows `{}` on both (RC=2). `hm-keys`, `hm-values`, `pm-keys`, `pm-keys-two`, and `pm-values` are RC=0 (the program returns nil when `=` holds). Record assoc shows `<probe::ArmRec{#0: 9}>` on both `:wat::core::Record/assoc` and `:wat::core::assoc` (RC=2).

The checker typed a bare `wat.type/PersistentMap` or `wat.type/PersistentVector` as `TypeExpr::Path`, and the custom arms match only `TypeExpr::Parametric`. `instantiate_bare_family` (`src/collection/infer.rs`) turns a bare Path of Vector, PersistentVector, List, or HashSet into a one-argument parametric, and HashMap or PersistentMap into a two-argument parametric, then the existing arms run. It is called from `infer_contains`, `infer_get`, `infer_assoc`, `infer_dissoc`, and `infer_map_projection` (keys and values). A copy of the corpus codemod was RC=3 with 93 `TypeMismatch`s before that function and RC=0 after the rebuild (`/tmp/g1-fixsrc-run.log`, `/tmp/g1-fixsrc-run2.log`). That was a checker gap. The values did not disagree.

`infer_assoc` parameter #3 uses `assignable` (a trial subst, committed only on success). Parameter #2 stays exact `unify`. Before that change, census `.census/2026-10-03T03-40-32Z.txt` flipped eight files from rc 0 to rc 1, all `:wat::core::assoc` parameter #3: `tests/rete/probe_arc278_1a_data_model.wat`, `probe_arc278_match_arm_body_ok.wat`, `probe_arc278_match_arm_then_core_bare.wat`, `probe_arc278_match_arm_then_rete_bare.wat`, `probe_arc278_match_arm_then_wrapped.wat`, `wat-scripts/perf/grid/where-query-compat.wat`, `where-query-params.wat`, and `wat-tests/rete/differential-fuzz-rules.wat`. After the rebuild each `--check` was RC=0. That stamp is the failing census. The gate is the later one.

`:wat::core::keys` and `:wat::core::values` fingerprint returns are `(:wat::type::Vector :- [K])` and `(:wat::type::Vector :- [V])`. Call-site types stay `infer_keys` and `infer_values`. Both are `@Determinism Nondeterministic`, so the examples are `@example-norun`.

Six `RETE_OPS` rows keep their `rete_name` and point `core_name` at the polymorphic verb (`:wat::rete::vector::length`, `:wat::rete::vector::contains?`, `:wat::rete::map::contains-key?`, `:wat::rete::vector::get`, `:wat::rete::vec::get`, `:wat::rete::linkedlist::get`). `OpExec::of_row` dispatches those six by `rete_name`. `NAMING_RULE_EXCEPTIONS` length is 25. Three `@alias` attributes in `src/intrinsic/special/rete_alias.rs` (lines 400, 412, 427) name `:wat::core::length` or `:wat::core::contains?`. The prose comments above those attributes still name the old targets. The attribute is the dispatch.

`wat.list/reduce` is in the brief's namespace census and is not a G1 verb, so it has no mapping. A search of `*.wat`, `*.rs`, and `*.edn` finds the arc143 test that renames foldl's head to `:wat::list::reduce` (`tests/wat_lang/wat_arc143_manipulation.wat`, golden `wat_arc143_manipulation__reduce_head.edn`), a comment in `src/reflect/verbs.rs`, and a comment in `wat-scripts/fixes/rename-list-to-seq.wat`. No live call.

### Mapping, codemod, retirement

`wat-scripts/fixes/one-name-per-operation.edn` holds 64 pairs (counted on this tree). `:wat::vector::concat` → `:wat::core::into`. `:wat::vec::concat` → `:wat::core::concat`. `:wat::vec::extend` → `:wat::core::into`. Both `contains-key?` names → `:wat::core::contains?`. The recorded codemod is `wat-scripts/fixes/one-name-per-operation.wat` (`rename-keyword-exact` on keyword leaves). `cargo test --release --test cli every_recorded_migration_replays` after the fixture edit: `18 passed; 0 failed`, 9.55s, RC=0 (`/tmp/g1-cure.log`).

`wat-fix-rust --dry-run` over 1301 tracked `.rs` files: `[wat-fix-rust] 1301 file(s) scanned, 0 changed, 0 edit(s) found, 0 refused, 0 codemod-failed`, RC=0 (`/tmp/g1-fix-rust-dry.log`). The same log prints two lexer panics before that summary, `end byte index 5 is not a char boundary` inside `∅` and `end byte index 14 is not a char boundary` inside `≠`. The driver still reported 0 failed. That dry-run is the codemod; the later cure does not add collection-verb keyword leaves.

An idempotent re-run of 611 files (`/tmp/g1-idem-run.log`, RC=0, byte diff 0) was before the three Pascal pairs left the table. After they left, the replay fixture keeps `/` on those three names and the harness above passed. The corpus call sites of those three names were restored by hand in the cure commit.

Retirement adds 43 rows at `src/remedy/retirement.rs` lines 436–478. Older colon rows still name the per-type verbs this stone retires. A second row with the same retired string would be shadowed by the first, so that chain stays. The slash forms retire to the core verb or to `:wat::bytes::` / `:wat::record::`.

### R-a

Lowercase parents in the mapping were respelt from `/` to `::`. `:myapp::Formattable/format`, `:wat-tests::holon::Reject/bundle-or-fail`, and `:wat-tests::holon::Reject/project-bundle-or-fail` stay on `/`. The member-join wall reads a Pascal segment followed by a lowercase member as Type/member. Whether those two parents are rows in the type registry was not queried. Restoring `/` is the spelling that wall requires, and the rest of the cutover stayed.

### Census

Baseline before any amend-2 edit: `.census/2026-10-03T02-28-00Z.txt`, 2284 files, `0:2074, 1:208, 101:2`. After `assignable`: `.census/2026-10-03T03-44-18Z.txt`, 2284 files, same distribution, 0 rc flips against the baseline (recomputed from the two stamps). After the cure, on the tree that became `54e2d948d`: `.census/2026-10-03T03-58-35Z.txt`, 2285 files, `0:2075, 1:208, 101:2`. The extra file is `wat-scripts/fixes/one-name-per-operation.wat` at rc 0. 0 rc flips against the baseline. `.census/` is not committed.

### Floors

Do not re-run either.

The cutover commit `4f8f551ea` is red. `.floor/2026-10-03T03-46-12Z`, log `/tmp/g1-floor-25586.log`:

```
Summary [ 405.188s] 6394 tests run: 6388 passed (29 slow), 6 failed, 24 skipped
RC=100
```

The six arms, each cured in `54e2d948d`:

1. `no_stale_path_in_doc::every_location_named_in_a_doc_comment_exists` — `src/rete/purity.rs` cited `src/intrinsic/hashmap.rs:206`, and that file is 6 lines. The cure cites `eval_keys` / `eval_values` and drops the line number.
2. `one_member_join::no_colon_joined_type_member_in_tracked_wat` — six call heads, quoted from the log:

```
colon-joined Type::member still in tracked .wat (6):
tests/types/probe_diagnostic_defprotocol_dispatch_p1.wat:25: :myapp::Formattable::format
tests/types/probe_diagnostic_defprotocol_dispatch_p1.wat:26: :myapp::Formattable::format
tests/types/probe_diagnostic_defprotocol_dispatch_p2.wat:17: :myapp::Formattable::format
tests/types/probe_diagnostic_defprotocol_dispatch_p3.wat:13: :myapp::Formattable::format
wat-tests/holon/Reject.wat:45: :wat-tests::holon::Reject::bundle-or-fail
wat-tests/holon/Reject.wat:65: :wat-tests::holon::Reject::project-bundle-or-fail
```

3. `intrinsic::tests::doc_arg_ret_types_match_checker_scheme` — doc ret for `:wat::core::values` is `(:wat::type::Vector :- [V])`, scheme was `:T`. Panic at `src/intrinsic/mod.rs:2611`.
4. `intrinsic::tests::probe_can_doc_types_reconstruct_the_checker_scheme` — `round-trip EXACTLY 419`, `failed 2`. keys ret doc `(:wat::type::Vector :- [K])` parsed as `Parametric { head: "wat::type::Vector", args: [Path(":K")] }`, scheme `Path(":T")`; values the same with `:V`. Panic at `src/intrinsic/mod.rs:2545`.
5. `intrinsic::tests::purity_mandated_examples` — `non-pure-det intrinsic :wat::core::values has no @example-norun`. Panic at `src/intrinsic/mod.rs:2857`.
6. `test::deftest_wat_tests_reflect_render_doc_of_bytes_to_hex` — `assert-contains` expected `Bytes/to-hex`. The actual text starts `:wat::bytes::to-hex`. The two texts match apart from that home name.

The cure commit `54e2d948d` is green. `.floor/2026-10-03T03-59-44Z`, log `/tmp/g1-floor-25586b.log`:

```
Summary [ 403.773s] 6394 tests run: 6394 passed (29 slow), 24 skipped
RC=0
```

That is the same 6394 the brief names. Clippy of the cure tree, `cargo clippy --release --all-targets -- -D warnings`: `Finished release profile [optimized] target(s) in 12.51s`, RC=0 (`/tmp/g1-clippy-cure.log`). This score commit is the document. It does not change code, and it was not floored.

`4f8f551ea` is the cutover. `54e2d948d` is the cure. Local `main`, not pushed. 5c-ii, 5c-iii, and 5c-iv are unstarted.

## Amend 3: the proofs are tests

Continues at `e0daeb4d4`. The pins and the respell are `9162b9f40`. The floor cure is `d3dc12bbc`.

### Pinned values

`tests/cli/probe_stone_25586_one_name.wat` returns `:probe::pins`, a vector of `label` plus `(:wat::core::str value)`. The rust test `one_name_replacements_return_the_pinned_values` compares that vector to 68 strings. Those strings were measured by `./target/release/wat /tmp/g1-pins.wat` twice; the two outputs were identical (`diff` RC=0). HashMap and HashSet renders of more than one entry follow hash order, so those pins are `length` and `get` / `contains?`. One-entry renders are pinned whole.

Measured, among the 68: PersistentMap dissoc of `"a"` is `#wat.core/PersistentMap {}`; keys of that one-entry map are `["a"]`; values are `[1]`. `into` of Vector `[1]` and Vector `[2]` is `[1 2]`; Vector `[1]` and PersistentVector `[2]` is `[1 2]`; PersistentVector `[1]` and Vector `[2]` is `#wat.core/PersistentVector [1 2]`; two PersistentVectors `[1]` and `[2]` are `#wat.core/PersistentVector [1 2]`. `contains?` of Vector `[1 2]` on `1` is `true` and on `9` is `false`. `contains?` of HashSet `{1}` is the same pair. `contains?` of HashMap `{"a" 1}` on the key `"a"` is `true` and on `"z"` is `false`. `get` hit is `#wat.core/Option.Some {:value 1}`; miss is `#wat.core/Option.None {}`. List conj of `1` onto `(2)` is `(1 2)`.

The same vector pins the new homes. `wat.bytes/to-hex` of bytes 255, 0, 16 is `ff0010`. `wat.bytes/from-hex` of `"ff0010"` is `#wat.core/Option.Some {:value [255 0 16]}`. `wat.record/field-at` of `(:probe::PinRec :sk 9)` at 0 is `9`. `wat.record/same-data?` of two records with `:sk 9` is `true`, and against `:sk 8` is `false`. `wat.core/assoc` of `(:probe::PinRec :sk 1)` at `:sk` to `9` is `#probe/PinRec {:sk 9}`. `(wat.type/List :- [wat.type/i64] 1 2)` is `(1 2)`. `(wat.type/char "a")` is `\a`.

The R-a service renames (`cache-svc`, `hologram-svc`, `pcache`, `mal`, `barebox`, `t::svc`, `t::worker`) are not container operations. They are not in this vector.

### Retirement

`wat::retirement_table_pairs_for_gate` walks `RETIREMENT_TABLE`. `stone_rows_are_refused_naming_their_replacement` keeps rows whose retired name starts with `:wat::hashmap::`, `:wat::map::`, `:wat::vec::`, `:wat::vector::`, `:wat::hashset::`, `:wat::linkedlist::`, `:wat::core::Bytes/`, `:wat::core::Record/field-at`, `:wat::core::Record/same-data?`, or `:wat::core::Record/assoc`. The filter saw at least 43. Each probe is a real `wat` process calling the retired name, and the output contains `is retired` and the table's replacement string.

### R-a

`startup_from_file` of `tests/types/probe_diagnostic_defprotocol_dispatch_p1.wat` does not contain `:myapp::Formattable`. `startup_from_file` of `wat-tests/holon/Reject.wat` does not contain `:wat-tests::holon::Reject`. Both loads succeeded. Those three names respell `/` to `::` in the probes, in `Reject.wat`, and in the mapping. The table is now 67 pairs.

The member-join wall still uses the Pascal-and-lowercase shape as the candidate filter. The decision is `TypeEnv::contains` on the parent, from `startup_from_file` of the file that holds the call. `:wat::core::Option::expect` is a member join against the stdlib registry. `:myapp::Formattable::format` is not. A candidate file that will not load fails the test, because the registry was not asked.

### Census, clippy, floors

Do not re-run the red floor.

Census after the pins were tracked: `.census/2026-10-03T04-29-41Z.txt`, 2286 files, `0:2076, 1:208, 101:2`. Against `.census/2026-10-03T03-58-35Z.txt`: no rc flips, one new file `tests/cli/probe_stone_25586_one_name.wat` at rc 0. `census-diff: no STOP-8`, DIFF_RC=0. The cure commit changes no `.wat`.

Clippy of the cure tree, `cargo clippy --release --all-targets -- -D warnings`: `Finished release profile [optimized] target(s) in 12.64s`, RC=0 (`/tmp/g1-clippy-amend3c.log`).

`9162b9f40` is red. `.floor/2026-10-03T04-30-42Z`, log `/tmp/g1-floor-amend3.log`:

```
Summary [ 400.551s] 6397 tests run: 6395 passed (29 slow), 1 failed, 1 timed out, 24 skipped
RC=100
```

Two arms, both cured in `d3dc12bbc`:

1. `no_loose_string_assert::tests_carry_no_loose_string_assert` — `tests/lint/one_member_join.rs:89` and `:98`. The panic is `LOOSE STRING ASSERTIONS — 2 site(s)`. Those lines called `TypeEnv::contains` inside `assert!`. The asks now sit on their own statements. The lint re-run is `ok. 1 passed`, RC=0 (`/tmp/g1-loose2.log`).
2. `probe_stone_25586_retirement::stone_rows_are_refused_naming_their_replacement` — `TIMEOUT [  30.024s]`. Isolated the same test is `finished in 16.28s` (`/tmp/g1-retire-iso.log`). The default kill is 30s. The override is warn 60s / kill 120s.

`d3dc12bbc` is green. `.floor/2026-10-03T04-41-21Z`, log `/tmp/g1-floor-amend3b.log`:

```
Summary [ 404.574s] 6397 tests run: 6397 passed (29 slow), 24 skipped
RC=0
```

6394 at the previous green floor, 6397 here. The three new tests are the pin vector, the registry query, and the retirement-row walk. This score commit is the document. It was not floored. Not pushed. 5c-ii, 5c-iii, and 5c-iv are unstarted.
