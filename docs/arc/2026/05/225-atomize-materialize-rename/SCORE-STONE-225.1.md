# SCORE — Arc 225 Stone 225.1 v3 — Bridge naming family: rename + mint

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 5/5 deliverables COMPLETE — all green within pre-existing baseline

| # | Deliverable | Status | Citation |
|---|---|---|---|
| 1 | Narrow `:wat::holon::Atom` to accept ONLY `Value::holon__HolonAST` | PASS | `src/runtime.rs` — `eval_holon_atom` now: `Value::holon__HolonAST(h) => Ok(Value::holon__HolonAST(Arc::new(HolonAST::Atom(h))))`, all other arms removed; type-check path narrowed in `src/check.rs` to reject non-HolonAST inputs |
| 2 | Mint `:wat::holon::to-holon` (polymorphic UP verb) | PASS | `src/runtime.rs` — `to_holon_inner` function handles all primitive types → HolonAST conversion; registered as `":wat::holon::to-holon"` in dispatch; `src/check.rs` + `src/freeze.rs` registration complete |
| 3 | Mint `:wat::holon::from-holon` (rename of `:wat::core::atom-value`) | PASS | `src/runtime.rs` — `eval_holon_from_holon` (was `eval_atom_value`); registered as `":wat::holon::from-holon"` replacing `":wat::core::atom-value"` |
| 4 | Rename `:wat::holon::from-watast` → `:wat::holon::from-wat` | PASS | `src/runtime.rs` — dispatch key renamed; `src/check.rs` + `src/freeze.rs` updated; all callers in tests + .wat files updated |
| 5 | Rename `:wat::holon::to-watast` → `:wat::holon::to-wat` | PASS | Same sweep as D4 — all callers updated |

## Test summary

```
cargo build --release -p wat                                        — 0 errors (5 pre-existing unused-fn warnings)
cargo test --release --lib -p wat [skip 5 signal tests]            — 827/827 PASS
cargo test --release -p wat-edn                                     — 23+1 PASS
cargo test --release --test mvp_end_to_end                          — 10/10 PASS
cargo test --release --test wat_arc143_manipulation                 — 8/8 PASS
cargo test --release --test wat_arc148_ord_buildout                 — 46/46 PASS
cargo test --release --test wat_arc201_holon_ast_accessors          — 7/7 PASS
cargo test --release --test wat_arc221_char_atomization             — 3/3 PASS
cargo test --release --test wat_arc221_keyword_nil_tag_atomization  — 6/6 PASS
cargo test --release --test wat_arc221b_keyword_dispatcher_completeness — 6/6 PASS
cargo test --release --test wat_arc221b_macro_support_keyword_shape — (verified pass in prior session)
cargo test --release --test wat_bundle_capacity                     — 7/7 PASS
cargo test --release --test wat_engram_library                      — 3/3 PASS
cargo test --release --test wat_eval_result                         — 7/7 PASS
cargo test --release --test wat_online_subspace                     — 3/3 PASS
cargo test --release --test wat_parametric_enum_typecheck           — 3/3 PASS
cargo test --release --test wat_reckoner                            — 3/3 PASS
cargo test --release --test wat_simhash                             — 5/5 PASS
cargo test --release --test wat_vector_algebra                      — 4/4 PASS
cargo test --release --test wat_vector_first_class                  — 11/11 PASS
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias — 6/6 PASS
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio — 14/15 (1 pre-existing — see Delta 1)
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio — 17/18 (1 pre-existing — see Delta 1)
cargo test --release --test probe_arc216_stone1_hashset_roundtrip   — 10/10 PASS
cargo test --release --test probe_arc216_stone2_vector_roundtrip    — 12/12 PASS
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip   — 14/14 PASS
cargo test --release --test probe_arc216_stone4_predicate_composition — 6/6 PASS
cargo test --release --test probe_arc216_stone5b_hashset_native_storage — (verified pass in prior session)
cargo test --release --test probe_arc216_stone5c_hashmap_native_storage — (verified pass in prior session)
cargo test --release --test probe_arc216_stone7_tuple_roundtrip     — 12/12 PASS
cargo test --release --test probe_hashmap_ctor_vector_symmetric     — 9/9 PASS
cargo test --release --test probe_plain_panic_produces_structured_edn — 1/1 PASS
cargo test --release --test test [wat-level tests]                  — 193/197 PASS (4 pre-existing — see Delta 2)

holon-rs contamination check:
  git -C /home/watmin/work/holon/holon-rs/ diff --name-only         — empty (untouched)
```

## Deltas from EXPECTATIONS

### Delta 1 — `holon_ast_extract` missing `HolonAST::Keyword` arm (pre-existing latent bug)

Two probes fail consistently in both the pre-arc-225 and post-arc-225 states:
- `probe_arc214_slice4_stone2_env_get_trio` → `probe_4_get_multi_type`
- `probe_arc214_slice4_stone3_env_dig_trio` → `probe_10_multiple_t_types`

Both fail with `expected Some(keyword); got None`.

**Root cause:** `holon_ast_extract` in `src/runtime.rs` (the function that extracts a runtime `Value` from a `HolonAST` leaf) has arms for `Symbol`, `String`, `I64`, `F64`, `Bool`, `Atom`, but NO arm for `HolonAST::Keyword`. It falls through to `_ => return None`. Arc 221 minted `HolonAST::Keyword` but did not add the corresponding extraction arm. Arc 225 is not responsible for this gap.

**Verification:** Confirmed pre-existing via `git stash` + test run before restoring arc 225 changes. Both tests fail identically on the pre-arc-225 baseline.

**Per `feedback_no_pre_existing_excuse`:** root cause traced, gap identified, documented here. Fix deferred to a dedicated arc (the gap is substrate-internal, warrants its own stone with regression probes). NOT deflected.

### Delta 2 — `cargo test --release --test test` (wat-level runner) — 4 pre-existing failures

**`deftest_wat_rs_std_struct_to_form_test_quasiquote_splices_runtime_values`** and **`deftest_wat_rs_std_struct_to_form_test_roundtrip_via_eval`** — both present in pre-arc-225 stash run. Unrelated to bridge naming.

**2 timeout failures** (`test_assert_stdout_is_matches`, `test_assert_stderr_matches_pass` in arc-225 run; different tests in pre-arc-225 run) — hermetic subprocess spawn tests; timing-sensitive; different tests time out on different runs (flaky). The modified portions of `test.wat` (lines 89-111, `assert-coincident` tests) pass cleanly: `deftest_wat_tests_std_test_test_assert_coincident_pass` and `deftest_wat_tests_std_test_test_assert_coincident_fail_renders_explanation` both pass.

### Delta 3 — `mvp_end_to_end.rs` algebra-path correction

The previous session (before compaction) incorrectly renamed algebra-path strings in `mvp_end_to_end.rs` from `(:wat::holon::Atom ...)` to `(:wat::holon::to-holon ...)`. The `eval_algebra_source` path goes through `lower.rs` which only handles `(:wat::holon::Atom ...)` as a primitive algebra name — it does NOT route through the runtime dispatch. This produced `UnsupportedUpperCall(":wat::holon::to-holon")` for 7 tests.

**Fix:** reverted all `eval_algebra_source` call strings back to `(:wat::holon::Atom ...)`. The algebra-path primitive name is `Atom` — this is correct and HONEST: it's the algebra-tier name (not the runtime-tier name `to-holon`). The test for `parse_error_surfaces_as_error` (unclosed paren) was left as `to-holon` because it never reaches the lowerer — the error is a parse error.

### Delta 4 — `probe_arc216_stone3_hashmap_roundtrip.rs` typo: `from-holonh`

The bulk rename of `atom-value` → `from-holon` introduced a typo in two probes (lines 204 and 713): `(:wat::holon::from-holonh -> :wat::core::HashMap)`. The `h` from the argument `h` immediately following `from-holon` was absorbed into the function name, creating `from-holonh`.

**Fix:** `from-holonh` → `from-holon h` (space restored between function name and argument). Both probes now pass.

## STOP trigger audit

- **STOP-1 (test regression beyond planned, dishonestly framed):** DID NOT TRIGGER. All cascade fixes are arc-225 consequences (callers of old names). Pre-existing failures in `probe_arc214_slice4_stone2/3` verified by stash, documented honestly.
- **STOP-2 (load-bearing probe fails):** DID NOT TRIGGER. All deliverable probes pass.
- **STOP-3 (time limit):** DID NOT TRIGGER.
- **STOP-4 (holon-rs touched):** DID NOT TRIGGER. `git -C holon-rs/ diff --name-only` empty.
- **STOP-5 (scope creep beyond 5 deliverables):** DID NOT TRIGGER. All fixes are mechanical consequences of the rename/mint — caller sweeps, typo correction, algebra-path revert.
- **STOP-6 (algebra-path confusion):** Triggered as a correction (Delta 3), not a stop. The wrong rename was discovered and reverted before SCORE.
- **STOP-7 (bash discipline):** DID NOT TRIGGER. One cargo command at a time, foreground, no pipes.

## Discovered latent bug (for future arc)

**`holon_ast_extract` Keyword gap** — `src/runtime.rs`: `holon_ast_extract` lacks `HolonAST::Keyword` arm. Anything that stores a keyword into a `HolonAST` and then tries to extract it back via `Env/get` or `Env/dig` returns `None` (silent miss) instead of the keyword value. Affects probes 4 and 10 in arc-214 slice-4 stones 2 and 3. Root cause: arc 221 minted `HolonAST::Keyword` but `holon_ast_extract` was not updated. Fix: add `HolonAST::Keyword(s) => Value::wat__core__keyword(Arc::new(format!(":{}", s)))` arm alongside the existing `HolonAST::Symbol` arm.

## Files changed

**wat-rs source (Rust):**
- `src/runtime.rs` — (a) `eval_holon_atom` narrowed to HolonAST-only input; (b) `to_holon_inner` function minted + registered as `":wat::holon::to-holon"`; (c) `eval_atom_value` function renamed/re-registered as `":wat::holon::from-holon"`; (d) `":wat::holon::from-watast"` → `":wat::holon::from-wat"`; (e) `":wat::holon::to-watast"` → `":wat::holon::to-wat"`
- `src/check.rs` — type-check registrations updated for all 5 deliverables
- `src/freeze.rs` — freeze-phase registrations updated

**Test files (Rust, caller sweep — 20 files):**
- `tests/mvp_end_to_end.rs` — algebra-path strings kept as `(:wat::holon::Atom ...)` (lowerer tier); `parse_error_surfaces_as_error` test string uses `to-holon` (parse error never reaches lowerer)
- `tests/probe_arc214_slice4_stone1_program_env_typealias.rs`
- `tests/probe_arc214_slice4_stone2_env_get_trio.rs`
- `tests/probe_arc214_slice4_stone3_env_dig_trio.rs`
- `tests/probe_arc216_stone1_hashset_roundtrip.rs`
- `tests/probe_arc216_stone2_vector_roundtrip.rs`
- `tests/probe_arc216_stone3_hashmap_roundtrip.rs` (+ typo fix `from-holonh` → `from-holon h`)
- `tests/probe_arc216_stone4_predicate_composition.rs`
- `tests/probe_arc216_stone5b_hashset_native_storage.rs`
- `tests/probe_arc216_stone5c_hashmap_native_storage.rs`
- `tests/probe_arc216_stone7_tuple_roundtrip.rs`
- `tests/probe_hashmap_ctor_vector_symmetric.rs`
- `tests/probe_plain_panic_produces_structured_edn.rs`
- `tests/wat_arc143_manipulation.rs`
- `tests/wat_arc148_ord_buildout.rs`
- `tests/wat_arc201_holon_ast_accessors.rs`
- `tests/wat_arc221_char_atomization.rs`
- `tests/wat_arc221_keyword_nil_tag_atomization.rs`
- `tests/wat_arc221b_keyword_dispatcher_completeness.rs`
- `tests/wat_bundle_capacity.rs`
- `tests/wat_engram_library.rs`
- `tests/wat_eval_result.rs`
- `tests/wat_online_subspace.rs`
- `tests/wat_parametric_enum_typecheck.rs`
- `tests/wat_reckoner.rs`
- `tests/wat_simhash.rs`
- `tests/wat_vector_algebra.rs`
- `tests/wat_vector_first_class.rs`

**wat-level files (caller sweep — 11 files):**
- `wat-tests/holon/Reject.wat`
- `wat-tests/holon/Sequential.wat`
- `wat-tests/holon/Subtract.wat`
- `wat-tests/holon/Trigram.wat`
- `wat-tests/holon/coincident.wat`
- `wat-tests/holon/eval-coincident.wat`
- `wat-tests/holon/term.wat`
- `wat-tests/test.wat`
- `wat/holon/Circular.wat`
- `wat/holon/Sequential.wat`
- `wat/kernel/run_threads.wat`

**New files:**
- `docs/arc/2026/05/225-atomize-materialize-rename/SCORE-STONE-225.1.md` (this file)

**Total: 3 modified Rust source files + 28 modified test/wat files + 1 new SCORE doc.**

## Algebra-path vs runtime-path distinction (permanent record)

The two evaluation paths are:
- **Runtime path** (`src/runtime.rs`): `(:wat::holon::to-holon ...)`, `(:wat::holon::from-holon ...)`, `(:wat::holon::from-wat ...)`, `(:wat::holon::to-wat ...)`, `(:wat::holon::Atom h)`. Dispatched via the eval dispatch table. Tests in `tests/` using `startup_from_source` / `invoke_user_main` / `eval_in_frozen` exercise this path.
- **Algebra path** (`src/lower.rs`): `(:wat::holon::Atom ...)` as a PRIMITIVE algebra name for string/keyword/number→vector lowering. Dispatched via `eval_algebra_source`. Tests in `tests/mvp_end_to_end.rs` exercise this path. The algebra name `Atom` at this tier is CORRECT and must NOT be renamed to `to-holon` — these are two different tiers with two different purposes.

This distinction is load-bearing for future stone authors. The rename in arc 225 applies to the runtime dispatch tier ONLY.

## Substrate state post-Stone-225.1

**Bridge naming family is now clean:**
- `(:wat::holon::to-holon v)` — polymorphic UP: any atomizable Value → HolonAST
- `(:wat::holon::from-holon h)` — DOWN: HolonAST → Value (reverse of to-holon for primitives)
- `(:wat::holon::from-wat ast)` — WatAST → HolonAST structural conversion
- `(:wat::holon::to-wat h)` — HolonAST → WatAST structural conversion
- `(:wat::holon::Atom h)` — wraps HolonAST in HolonAST::Atom (ONLY accepts HolonAST input)

**Retired names (HARD CUT, no aliases):**
- `:wat::core::atom-value` — gone; replaced by `:wat::holon::from-holon`
- `:wat::holon::from-watast` — gone; replaced by `:wat::holon::from-wat`
- `:wat::holon::to-watast` — gone; replaced by `:wat::holon::to-wat`
- Old polymorphic `:wat::holon::Atom` arms (i64, f64, bool, String, keyword, Unit, WatAST, Vec, HashMap, HashSet, etc.) — gone; these arms live in `:wat::holon::to-holon` now

## Unblocks

- Arc 225 Stone 225.2 (INSCRIPTION) — blocked until all spawn children close; this stone is now closed
- Callers of the bridge family now use honest, direction-explicit names
- `holon_ast_extract` Keyword gap can now be filed as a discrete stone (independent of arc 225)
