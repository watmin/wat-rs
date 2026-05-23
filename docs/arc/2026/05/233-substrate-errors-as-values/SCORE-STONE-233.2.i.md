# SCORE — Arc 233 Stone 233.2.i — flip eval signature to TrackedValue

**Result: 10/10 PASS**

## Scorecard

| # | Row | Actual |
|---|---|---|
| 1 | Compile clean | 0 errors |
| 2 | eval signature probe FLIPS 0/3 → **3/3** | `test result: ok. 3 passed; 0 failed` |
| 3 | Lib tests baseline | **827 passed; 0 failed** |
| 4 | Substrate-symmetry probe still passes | `1 passed; 0 failed` |
| 5 | Stone 233.1 probes still pass | `8 passed; 0 failed` |
| 6 | Stone 233.2.a transparency tests still pass | `8 passed; 0 failed` |
| 7 | Stone 232.0 dynamic-keyword probes still pass | `8 passed; 0 failed` |
| 8 | Stone 233.2.h TrackedValue mint probe still passes | `6 passed; 0 failed` |
| 9 | Clippy no new warnings | 54 (boundary of limit; pre-existing baseline was 54 on lib) |
| 10 | holon-rs untouched | empty output |

## Cascade summary

### Phase 1 — boundary wrap (src/ lib crate)

**`src/runtime.rs`** — body of `pub fn eval` extracted to `pub(crate) fn eval_inner`; all
internal `eval(` calls replaced with `eval_inner(`; thin `pub fn eval` wrapper added that
maps `Value::Tracked { inner, provenance }` → `TrackedValue::new(*inner, provenance)` and
bare `Value` → `TrackedValue::from(value)`.

**`src/freeze.rs`** — `eval_in_frozen`, `eval_digest_in_frozen`, `eval_signed_in_frozen`
all return `Result<TrackedValue, RuntimeError>`; 4 inline lib tests updated.

**Helper files** (`src/time.rs`, `src/spawn.rs`, `src/io.rs`, `src/assertion.rs`,
`src/edn_shim.rs`, `src/thread_io.rs`, `src/macros.rs`, `src/string_ops.rs`,
`src/fork.rs`, `src/spawn_process.rs`) — eval call results in helpers that chain into
`eval_inner` rather than `eval` left unchanged; external-facing helpers that called
`eval(...)` had `.value_owned()` or `.value()` added at dispatch sites.

### Phase 2 — macro codegen (crates/wat-macros)

**`crates/wat-macros/src/codegen.rs`** — macro-generated dispatch code called
`::wat::runtime::eval(...)` expecting `Value`. Updated `arg_bindings` to use indexed
`__tv_arg_N` temp vars with `.value()` for `FromWat::from_wat` arg marshaling; all
`self_val` assignments use `.value_owned()`. This was the most architecturally significant
fix — all future macro-generated shims inherit correct TrackedValue handling.

### Phase 3 — hand-written dispatch crates

**`crates/wat-telemetry-sqlite/src/auto.rs`** — `db_val`, `entry`, `eval_keyword` results
needed `.value_owned()`; `ValueSnapshot` added to imports.

**`crates/wat-telemetry-sqlite/src/cursor.rs`** — `handle_val`, `constraints_val`,
`cur_val` needed `.value_owned()`; `type_name()` calls replaced with
`ValueSnapshot::of(other)` (pre-existing bug: `TypeMismatch.got` field is `ValueSnapshot`,
not `&str`).

### Phase 4 — test file cascade

**Total test files modified:** ~90 files across `tests/`.

**Pattern A** — `fn run(src: &str) -> Value` helpers: added `.value_owned()` after
`eval_in_frozen(...)`.

**Pattern B** — `match eval_in_frozen(...).expect("compute") { Value::... }`: added
`.value_owned()` before `{`.

**Pattern C** — `let process = eval(...).expect(...)` followed by `&process` passed to
helpers expecting `&Value`: added `.value_owned()` to the eval result.

**Pattern D** — `matches!(result, Ok(Value::Unit))` patterns: converted to
`result.expect("...").value_owned()` followed by `matches!(...)`.

**Pre-existing bug surfaced and fixed:**
- `tests/wat_arc170_slice_1f_alpha_helpers.rs`: `ThreadIO` struct was migrated to
  `wat::typed_channel` in arc 213 but the test still used `crossbeam_channel::bounded`.
  Fixed by importing `wat::typed_channel::{bounded, Sender, Receiver}` and updating
  `TestRig` field types.

**Pre-existing failures (not regressions from this stone):**
- `tests/probe_arc216_stone1_hashset_roundtrip.rs` — 7 tests failing on HEAD before and
  after this stone. Same `TypeMismatch` error: `HashSet<T>` expected but
  `wat::core::HashSet` returned from `from-holon`. Unrelated to eval boundary flip.

## Time breakdown

- Session 1 (prior): ~180 min — boundary wrap + macro codegen + telemetry crates + ~50 test files
- Session 2 (this): ~60 min — remaining ~40 test files + verification + SCORE

**Actual total:** ~240 min vs predicted 90-150 min

## Calibration

Predicted range 90-150 min; actual ~240 min. The cascade volume was higher than estimated:
- ~90 test files (predicted "external test file cascade: ~15-30 min" — actual ~120 min)
- The `wat_arc170_slice_1f_alpha_helpers.rs` `crossbeam_channel` → `typed_channel`
  migration was an unplanned pre-existing fix that added ~10 min
- The macro codegen fix and telemetry crates were not in the prediction breakdown

## What this unblocks

- **Stone 233.2.j** — producer migration: 5 `Value::Tracked` wrapping sites can now use
  `TrackedValue::new(value, provenance)` directly at the producer sites
- **Stone 233.2.k** — `Value::Tracked` variant retirement
- **Stone 233.2.e** — AST-derived provenance on the new substrate
