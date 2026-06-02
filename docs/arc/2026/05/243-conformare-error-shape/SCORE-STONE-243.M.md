# SCORE — Stone 243.M — meaningful spans at the parser/eval boundary

## Result

All 66 `ArityMismatch` + `Span::unknown()` sites eliminated. Zero remaining. Build green, 895/0/1 lib parity, clippy `result_large_err` = 0.

---

## Sites threaded — direct in-place substitution (Rule 1)

Functions that already had `list_span: &Span` in scope; `Span::unknown()` → `list_span.clone()`:

| File | Count | Functions |
|------|-------|-----------|
| `src/runtime.rs` | 54 | All eval_* and helper fns with `list_span: &Span` (bulk replace) |
| `src/time.rs` | 3 | `eval_time_at`, `eval_time_at_millis`, `eval_time_at_nanos` |
| `src/string_ops.rs` | 1 | `eval_string_concat` |
| `src/assertion.rs` | 1 | `eval_kernel_assertion_failed` |
| `src/fork.rs` | 2 | `eval_kernel_fork_program_ast`, `eval_kernel_fork_program` |
| `src/spawn_process.rs` | 1 | `eval_kernel_spawn_process` |

**Total direct substitutions: 62**

Additionally, 4 sites that used `Span::unknown()` in the empty-args fallback of ternary span expressions (Uuid helpers, `eval_char_of`, `eval_string_join`) were changed to `list_span.clone()` — same Rule 1 class.

---

## Helpers broadened — signature + caller threading (Rule 2)

### `src/io.rs` — `fn arity(...)`
- **Change:** Added `list_span: &Span` to `fn arity(op, args, n, list_span)`.
- **Callers updated:** 22 call sites across `eval_ioreader_*` and `eval_iowriter_*` functions. All callers had `list_span` in scope.

### `src/edn_shim.rs` — `fn require_one_arg(...)`
- **Change:** Added `list_span: &crate::span::Span` to `fn require_one_arg(op, args, env, sym, list_span)`.
- **Callers updated:** 6 call sites (`eval_edn_write`, `eval_edn_write_pretty`, `eval_edn_write_json`, `eval_edn_write_notag`, `eval_edn_write_json_natural`, `eval_edn_read`).
- **Bonus:** `eval_edn_read` had two additional `Span::unknown()` TypeMismatch + MalformedForm errors that also gained `list_span`.

### `src/thread_io.rs` — `fn require_one_arg(...)`
- **Change:** Added `list_span: &Span` to `fn require_one_arg(op, args, env, sym, list_span)`.
- **Callers updated:** 2 call sites (`eval_kernel_println`, `eval_kernel_eprintln`).
- **Bonus:** `eval_kernel_readln` MalformedForm for wrong arg count also got `list_span`.

### `src/time.rs` — `fn require_i64`, `fn require_string`, `fn require_instant`, `fn require_duration`
- **Change:** Added `list_span: &Span` to all four helper signatures.
- **Callers updated:** 16 call sites total across `eval_time_at`, `eval_time_at_millis`, `eval_time_at_nanos`, `eval_time_to_iso8601`, `eval_time_epoch_seconds`, `eval_time_epoch_millis`, `eval_time_epoch_nanos`, `eval_time_sub`, `eval_time_add`, `eval_time_ago`, `eval_time_from_now`, `unit_constructor`, `unit_ago`, `unit_from_now`.
- **Bonus:** All chrono range-error TypeMismatch sites in these functions also gained `list_span` (previously `Span::unknown()` with arc 138 deferral comment).

### `src/string_ops.rs` — `fn one_string`, `fn two_strings`
- **Change:** Added `list_span: &Span` to both helper signatures.
- **Callers updated:** 7 call sites (`eval_string_contains`, `eval_string_starts_with`, `eval_string_ends_with`, `eval_string_length`, `eval_string_trim`, `eval_string_split`, `eval_regex_matches`).

### `src/fork.rs` — `eval_kernel_wait_child`
- **Change:** Added `list_span: &Span` to `pub fn eval_kernel_wait_child`. This function is retired (arc 112 comment; no callers in runtime.rs dispatch). Signature broadened for completeness.

---

## `Span::unknown()` remaining — genuinely synthetic

All remaining `Span::unknown()` constructions in eval/parse code are at sites where no originating source node exists:

### Synthetic AST construction (not error paths)
- `src/runtime.rs:17896–17992` — `holon_ast_to_watast()` constructs WatAST nodes from HolonAST values with no source provenance; unknown span is structurally correct.
- `src/runtime.rs:25376–25633` — macro expansion helpers (`eval_if_ast`, `eval_let_ast`, `eval_do_ast`, etc.) synthesize WatAST node sequences; unknown span is structurally correct.
- `src/check.rs:7442–7484`, `src/check.rs:2689`, `src/check.rs:7406` — check-layer synthetic AST nodes for let-scope lowering and canonical nil forms.
- `src/runtime.rs:2949–2953` — `params_to_type_ast()` builds a synthetic list keyword; no source.
- `src/form_match.rs:253–259`, `src/form_match.rs:276–312` — test helpers constructing synthetic AST nodes.
- `src/sigma.rs:81` — synthetic call span for an internal sigma dispatch.

### OS/channel errors with no WatAST context
- `src/fork.rs:324` — `waitpid(2)` OS error from `eval_kernel_wait_child`; occurs after all args are evaluated.
- `src/fork.rs:355` — `pipe2(2)` OS error from `make_pipe`; called from deep inside fork with no AST context.
- `src/fork.rs:672`, `src/fork.rs:1076`, `src/fork.rs:1178` — MalformedForm for parse/scope errors inside `fork_program_from_source`; no WatAST context available at that level.
- `src/thread_io.rs:164` — `ServiceNotRunning` from `with_thread_io`; fires when the thread-local cell is empty, no AST context.
- `src/thread_io.rs:205–647` — `ChannelDisconnected` errors from channel `.send()` / `.recv()` within `with_thread_io` closures; OS-level disconnect, no AST.
- `src/thread_io.rs:426–455`, `src/thread_io.rs:858–866` — TypeMismatch/MalformedForm on internal conversion helpers with no WatAST context.
- `src/thread_io.rs:331–336` — EDN parse of a runtime-received string; no originating AST.
- `src/runtime.rs:12673` — sentinel form emission in `eval_lookup_define` `SpecialForm` arm; synthetic.
- `src/runtime.rs:19165`, `src/runtime.rs:32504` — test helpers.
- `src/runtime.rs:19267–26069` — `EvalVerificationFailed` wrapping Rust-level evaluation errors; synthetic.
- `src/spawn_process.rs:219` — MalformedForm for an internal arg-assembly error; no originating AST.
- `src/edn_shim.rs:341–2103` — `EdnReadError` constructions inside EDN deserializer; no WatAST context; these are conversion-layer errors, not eval errors.
- `src/sandbox.rs:45` — ScopedLoader construction error; path is a String, no AST.
- `src/lower.rs:178` — LowerError; structural synthetic.
- `src/rust_deps/custodia.rs:89` — internal ownership check; no eval context.
- `src/runtime.rs:1569` — binding_span for internal VM insertions.
- `src/io.rs:969–1354` — MalformedForm errors at writer-snapshot level and file/loader errors; Value-only context or OS-level.
- `src/check.rs:9759` — returns `Span::unknown()` as a fallback default span; synthetic.

---

## Verify results (verbatim)

```
cargo build --release -p wat
  Finished `release` profile [optimized] target(s)

cargo build --release --tests
  Finished `release` profile [optimized] target(s)

cargo test --release --lib -p wat
  test result: ok. 895 passed; 0 failed; 1 ignored

cargo clippy --release -p wat
  (result_large_err: 0 occurrences)
  Finished `release` profile [optimized] target(s)

grep -rn "ArityMismatch" src/ | grep "Span::unknown"
  (no output — 0 remaining)

ls tools
  ls: cannot access 'tools': No such file or directory
```

---

## Content integrity

No Rust tool was written. All edits were direct in-place substitutions and helper-signature broadenings. No content-integrity check required.

## Commit

None — tree left dirty per brief.
