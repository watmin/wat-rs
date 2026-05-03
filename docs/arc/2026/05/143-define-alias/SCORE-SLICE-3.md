# Arc 143 Slice 3 — SCORE

**Sweep:** sonnet, agent `abe31c0f7661c10b5`
**Wall clock:** ~7.5 minutes
**Output verified:** orchestrator re-ran `cargo test --release --test
wat_arc143_manipulation` + `cargo test --release --workspace`.

**Verdict:** **MODE A — clean ship.** 10/10 hard rows PASS; 4/4 soft
rows PASS. The substrate-informed brief discipline held end-to-end —
sonnet executed mechanically against a verified brief in well under
the predicted runtime band.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ 2 Rust files modified (`src/runtime.rs` + `src/check.rs`) + 1 NEW test file (`tests/wat_arc143_manipulation.rs`). NO wat files. |
| 2 | `eval_rename_callable_name` present | ✅ runtime.rs:6314-6432 (~118 LOC). Validates 3 args; returns `Value::holon__HolonAST`. |
| 3 | `eval_extract_arg_names` present | ✅ runtime.rs:6434-6487 (~53 LOC). Validates 1 arg; returns `Value::Vec<keyword>`. |
| 4 | Rust helpers added | ✅ `require_bundle` (Bundle destructuring, runtime.rs:6270-6291) + `split_type_params` (string surgery, runtime.rs:6292-6298). |
| 5 | Dispatch arms in runtime.rs | ✅ Lines 2415-2416, immediately adjacent to slice 1's arms at 2411-2413. |
| 6 | Scheme registrations in check.rs | ✅ Lines 11081-11097, adjacent to slice 1's registrations at 10997-11019. |
| 7 | Type-checker special-case extended | ✅ Lines 3161-3197, immediately after slice 1's special-case block at 3126-3160. Both new primitives bypass arg-type unification appropriately. |
| 8 | `cargo test --release --workspace` | ✅ Same baseline + 8 new tests pass; 1 pre-existing arc 130 LRU failure unchanged; ZERO new regressions. |
| 9 | New tests cover all cases | ✅ 8 tests: rename happy-path with type-params + without type-params + error from-mismatch; extract foldl-three-names + zero-args + stops-before-return-type + error-non-bundle + composing-rename-then-extract. ALL pass. |
| 10 | Honest report | ✅ ~250-word report covers file:line refs, helpers, dispatch, scheme registrations, verbatim AST output, test totals, honest deltas (arc-009 names-are-values; TypeMismatch.expected is &'static str). |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (150-300) | ✅ runtime.rs +221, check.rs +76, new test file +476. Test file is heavier than predicted but tests are individually small + focused; honest. |
| 12 | Style consistency | ✅ Helpers placed adjacent to slice 1's helpers; pattern-match shape on `HolonAST::Bundle` matches existing usage; arg-validation pattern mirrors slice 1's eval functions. |
| 13 | Test coverage breadth | ✅ Both happy path AND error path covered for both primitives. Composing-with-slice-1 test (`rename_then_extract_preserves_arg_names`) verifies the slices integrate. |
| 14 | Compose with slice 1 | ✅ Test 4 (`rename_then_extract_preserves_arg_names`) chains slice 1's `signature-of` → slice 3's `rename-callable-name` → slice 3's `extract-arg-names`. |

## Honest deltas (calibration record)

### Delta 1 — Arc 009 names-are-values hit `from`/`to` keyword args too

The slice 1 SCORE established that function-named keywords like
`:user::my-add` evaluate at runtime to `Value::wat__core__lambda` AND
infer at check-time as `:fn(...)`. The runtime helper
`name_from_keyword_or_lambda` (slice 1, runtime.rs:6078) handles both.

Sonnet's slice 3 work surfaced that this issue applies to ALL keyword
args of the manipulation primitives — not just `head`. The `from` and
`to` arguments to `rename-callable-name` are also keyword inputs and
hit the same runtime resolution path. Sonnet correctly reused the
slice 1 helper for both. Honest delta surfaced + handled.

### Delta 2 — TypeMismatch.expected is &'static str

For the "from name does not match head's base name" error,
`RuntimeError::TypeMismatch` was a poor fit because its `expected`
field is `&'static str` (can't carry a formatted message). Sonnet
substituted `RuntimeError::MalformedForm` which carries a `String`
reason. Honest delta; the right call.

## Calibration record

- **Predicted Mode A (~70%)**: ACTUAL Mode A. Calibration accurate.
- **Predicted runtime (12-18 min)**: ACTUAL ~7.5 min. Faster than
  predicted — pattern was now well-trodden after slices 1+2; sonnet
  composed the third slice fluently.
- **Predicted LOC (150-300)**: ACTUAL ~300 (runtime+check Rust) + 476
  (test file). Test file weight surprised but is honest — 8 tests
  with individual setup; not pollution.
- **Predicted soft drift (~15%)**: NOT HIT — tests passed cleanly;
  no test-construction drift.
- **Predicted helper-placement surprise (~8%)**: NOT HIT — helpers
  placed adjacent to slice 1's per the brief.

## What this slice delivered

- **Two HolonAST manipulation primitives.** Substrate now has the
  surgery tools userland macros need to compose generated define
  forms.
- **Re-usable Rust helpers** (`require_bundle`, `split_type_params`)
  that future HolonAST-manipulation work can reuse.
- **The substrate side of arc 143's macro layer is COMPLETE.**
  Slices 1 (point lookups) + 2 (computed unquote) + 3 (manipulation)
  are all shipped. Slice 6 (define-alias defmacro) is now pure-wat
  composition.

## What this slice surfaces

The verbatim AST output preserves the bare-name type formatting:
`:wat::list::reduce<T,Acc>` with `:Vec<T>`, `:Acc`,
`:fn(Acc,T)->Acc` as inner types. The orchestrator's pre-spawn
crawl established that the substrate's type registry uses bare-name
heads as canonical; the parser accepts bare names. This means slice
6's macro-emitted define SHOULD parse + type-check correctly with
this rendering — but slice 6 will VERIFY this empirically (Mode A
or Mode B FQDN gap surfaces clean diagnostic).

## Discipline lessons

The substrate-informed brief discipline produced a 7.5-minute clean
ship. The brief was tight; sonnet executed mechanically; honest
deltas surfaced exactly the substrate quirks (arc-009 + struct-
constraint) that previous slices established as known patterns.

This is the cadence the project had pre-compaction. Restored.
