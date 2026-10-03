# SCORE — STONE 255.88: the first slash is the only partition

**Accepted.** Local `main`, not pushed. The probe prints `0`, `7`, `42`, `1`, `99`. `wat.core//` is namespace `wat.core`, name `/`. STOP-1 did not fire. STOP-2 did not fire.

## Commits

| commit | what |
|---|---|
| `f789723de` | the probe, committed red: startup `UnresolvedReference` of `:u::a/b` and `:u::pathological/name//foo` |
| `ab9bc3f8b` | the first `/` is the only partition, and the doc-link judge leaves nextest |
| `d916e49c4` | `core.wat` keeps its line count so the span goldens stay put |

Author `watmin <john@shields.wtf>`. No `Co-Authored-By`.

## What changed

The reader already split on the first `/` (`Identifier::bare`, `flat.find('/')`). Two other doors did not.

`wat/core.wat`'s `defn` name block took the first segment and the last segment of a `/` split, so `u/a/b` registered as `:u::b` while the call looked up `:u::a/b`. A name that contains `/` now goes through `canonical-identity`. A slash-less name still gets the colon prefix, because `canonical-identity` returns that spelling unchanged. `wat/core.wat` is 2331 lines, the same length as the parent of this stone.

A binder is `Identifier::into_bound`: namespace `$bound`, name the whole spelling, `flat` and scopes unchanged. Normalize applies it by position (`let`, `fn` / `lambda` parameters, a two-element `match` binder, a hash-destructure binder, variant field patterns). A variant head stays a reference. A body reference whose `env_key` is in scope is bound the same way and is not rewritten to a keyword. `parse_triple` does the same for a parameter captured at register. Stored function bodies are normalized with the function's params and rest param already in scope.

`resolve_namespaced_symbol`, and the two eval arms that join a reference symbol, skip `reconstruct_call_path` and `other_join_spelling` when the local name contains `/`. They use `ns_to_wat_path`. `wat.core.Option/expect` still reconstructs: its local name is `expect`. Keyword spellings never enter `resolve_namespaced_symbol`.

`rekey_type_member_functions` already skips a method that contains `/`. The 255.86 wall still chooses `::` versus `/` only when the name after the first slash does not itself contain `/`. A symbol name that contains `/` is not a member join.

The field doc at `crates/wat-reader/src/identifier.rs` now says the first slash. The `normalize.rs` header no longer says the last slash.

## `flat`

Not removed.

Call expressions, every receiver: `.as_str()` is 598 in `src/`, 84 in `crates/`, 153 in `tests/`. `Identifier::leaf` and `Identifier::path` each have one call site, the unit test in `identifier.rs`. The free functions `identifier::leaf` (20) and `identifier::path` (12) take `&str` and do not read `flat`.

One `startup_from_file` plus `invoke_user_main` of the probe, counter on the three methods, then removed: `as_str` 504923 calls, 4316957 bytes that a reconstruct of `flat` would have allocated; `leaf` 0; `path` 0.

## Doc-link judge

The judge is `src/bin/doc_link_ledger.rs`. The ledger and the comparison are `src/doc_link.rs`. The two fast tests stay in nextest. The `#[ignore]` test is gone. `scripts/floor.sh` runs the bin against the captured log. The 300s hang bound is unchanged. No private `CARGO_TARGET_DIR`.

Planted proof, not a floor: `cargo doc --release --no-deps --workspace` exited 0 and named `warning: unresolved link to amend88_this_link_does_not_resolve` at `src/lib.rs:3:7`. `WAT_DOC_LINK_LOG` through `doc_link_ledger` exited 1 and reported `src/lib.rs: [amend88_this_link_does_not_resolve] × 1`. The link was removed before either commit. It is not in the tree.

## The red floor, then the green one

`.floor/2026-10-03T12-30-40Z`, exit 100. Do not re-run.

```
Summary [ 392.089s] 6412 tests run: 6407 passed (28 slow), 5 failed, 24 skipped
```

All five fired `assertion left == right` on an EDN span in `wat/core.wat`. Actual lines were one below the golden: `1462` against `1463` (`probe_arc249_threading::witness_thread_first_empty_step_panics_at_expansion`), `1514` against `1515` (both cond tests), `2011`–`2015` against `2012`–`2016` (`format_strict_missing_kwarg_is_macro_error`), `2039`–`2043` against `2040`–`2044` (`format_strict_unused_kwarg_is_macro_error`). `wat/core.wat` was 2330 lines; its parent was 2331. `d916e49c4` put the line back. The same five tests then passed in 0.176s (`/tmp/g1-88-arms.log`).

Acceptance floor `.floor/2026-10-03T12-41-26Z`, exit 0, tree clean:

```
Summary [ 392.358s] 6412 tests run: 6412 passed (28 slow), 24 skipped
```

Against 255.87's final green floor (6410 passed, 25 skipped): two new tests (`the_first_slash_is_the_only_partition`, `into_bound_keeps_the_whole_spelling_as_the_name`) and one fewer skip (the judge is not a test). Doctests exit 0. Doc-link exit 0. `cargo doc` finished in 11.43s. Fuzz `deftest_wat_tests_rete_fuzz_test_native_matches_oracle` PASS [71.586s]. `retirement_table_is_fully_reachable` PASS [168.014s]. Reachability shards PASS 23.000 / 22.630 / 21.215 / 20.989 / 20.946 / 20.795s. Time limits were not raised.

`cargo clippy --release --all-targets -- -D warnings` exited 0 in 12.59s on this tree.

## Census

Pre: `.census/2026-10-03T11-38-03Z.txt` (2292 files, `0:2082, 1:208, 101:2`).

Post: `.census/2026-10-03T12-40-09Z.txt`, 2293 files, `0:2083, 1:208, 101:2`. `--diff` against the pre file printed `census-diff: no STOP-8`. No path changed rc. The one new file is `tests/resolve/probe_arc255_88_first_slash.wat` at rc 0.

## STOPs

STOP-1: no `defn`, `defmacro`, or `def` name in `wat/` has a `/` after the first. `wat.core//` is a `defclause`. The name after the first slash is `/`, and `ns_to_wat_path("wat.core", "/")` is `:wat::core::/`, which is what `canonical-identity` already returns.

STOP-2: no existing `let` or `fn` binder in `wat/` is a slashed symbol. Those forms were rejected as a keyword in binder position. Variant heads are not rebound, so a symbol-spelled variant arm stays a variant.

## Amend — no padding to fit a golden

`d916e49c4` is withdrawn. Its one change was a bare `;;` in `wat/core.wat` so five span goldens would keep the line numbers they had before this stone. That line is gone in `b56f70ba6`. The five goldens were recaptured in the same commit. `wat/core.wat` is 2330 lines.

Isolated before the recapture, `/tmp/g1-88a-arms-red.log`, RC=100:

```
Summary [   0.180s] 5 tests run: 0 passed, 5 failed, 6431 skipped
```

Each failure is `assertion left == right`, an EDN data mismatch. The actual diagnostic and the golden share message, reason, file, and columns. The `wat/core.wat` line is one lower.

| test | assertion | golden line | measured line | columns |
|---|---|---|---|---|
| `probe_arc249_threading::witness_thread_first_empty_step_panics_at_expansion` | empty `->` step must match macro-expansion failure golden | 1463, both spans | 1462, both spans | 30–34 |
| `probe_arc258_stone2b_macro_error::contract_02_non_exhaustive_cond_names_else` | non-exhaustive cond must match diagnostic golden | 1515 | 1514 | 5–79 |
| `wat_core_cond::cond_refuses_missing_else` | expected missing-`:else` diagnostic | 1515 | 1514 | 5–79 |
| `probe_arc279_format::format_strict_missing_kwarg_is_macro_error` | missing kwarg must match macro-error diagnostic golden | 2012–2016 | 2011–2015 | 40–74 |
| `probe_arc279_format::format_strict_unused_kwarg_is_macro_error` | unused kwarg must match macro-error diagnostic golden | 2040–2044 | 2039–2043 | 24–75 |

`git diff` of the five `.edn` files is those line numbers and nothing else. Reasons unchanged: `:wat::core::first: WatAST List has 0 child(ren); no child at index 0`, `cond: non-exhaustive — needs a terminal :else arm`, `format: placeholder {y} has no matching kwarg`, `format: kwarg :y is unused — no {y} in template`. After the recapture the same five passed, `/tmp/g1-88a-arms-green.log`, RC=0, `Summary [   0.185s] 5 tests run: 5 passed, 6431 skipped`.

### What each span is there to prove

The line number is not the property. Each golden's `wat/core.wat` span names the stdlib form the diagnostic blamed. The call-site span, in the probe file, did not move.

- `witness_thread_first_empty_step_panics_at_expansion` — an empty `->` step. Line 1462 of `defmacro wat.core/->` is `` `(~(wat.core/first step) ~a ~@(wat.core/rest step)) ``. Columns 30–34 of that line are the text `step)`. The reason names `:wat::core::first` of a list with no child. The span proves the failure is that `first` applied to the empty step.
- `contract_02_non_exhaustive_cond_names_else` and `cond_refuses_missing_else` — a `cond` with no `:else`. The span is `(wat.core/macro-error "cond: non-exhaustive — needs a terminal :else arm")` (line 1514, col 5–79). The kind guard already requires that reason. The span proves the refusal is that `macro-error` form.
- `format_strict_missing_kwarg_is_macro_error` — placeholder `{y}` with no `:y`. The span is the `macro-error` that builds `format: placeholder {…} has no matching kwarg` (lines 2011–2015, col 40–74).
- `format_strict_unused_kwarg_is_macro_error` — `:y` given and unused. The span is the `macro-error` that builds `format: kwarg :… is unused — no {…} in template` (lines 2039–2043, col 24–75).

### The fragility, not cured

These goldens pin a line number inside the stdlib. An edit to `wat/core.wat` above them moves the line and fails tests that are not about that edit. Twice today: 255.87 moved them +3 (kwargs-lower +2, `kwargs-type-slot-name` +1), and this stone moved them −1 (the `defn` name block was one line shorter than the first/last split it replaced). A later stone can decide whether they should assert the span's file and the form, rather than a line number in `wat/core.wat`. That decision is not made here.

### Floor

`.floor/2026-10-03T12-58-25Z`, exit 0, tree clean at `b56f70ba6`. `.floor/2026-10-03T12-30-40Z` was not re-run.

```
Summary [ 392.919s] 6412 tests run: 6412 passed (28 slow), 24 skipped
```

Same count as this stone's acceptance floor (6412 passed, 24 skipped). Doctests exit 0. Doc-link exit 0. `cargo doc` finished in 11.40s. Fuzz `deftest_wat_tests_rete_fuzz_test_native_matches_oracle` PASS [71.877s]. `retirement_table_is_fully_reachable` PASS [168.339s]. Reachability shards PASS 22.943 / 22.343 / 20.677 / 21.019 / 21.553 / 21.091s. Time limits were not raised.

`cargo clippy --release --all-targets -- -D warnings` exited 0 in 12.60s.

### Census

Post: `.census/2026-10-03T13-06-27Z.txt`, 2293 files, `0:2083, 1:208, 101:2`. `--diff` against `.census/2026-10-03T12-40-09Z.txt` printed `census-diff: no STOP-8`. No path changed rc. No path was added.
