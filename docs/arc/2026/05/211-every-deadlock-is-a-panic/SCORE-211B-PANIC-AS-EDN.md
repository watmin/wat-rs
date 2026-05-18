# Arc 211b — SCORE: panic-as-EDN

**Ship date:** 2026-05-18
**Agent:** Claude Sonnet 4.6
**Mode:** A

---

## Scorecard

| # | Criterion | Result | Verification evidence |
|---|---|---|---|
| 1 | `write_assertion_failure` emits `#wat.kernel/AssertionFailure` tag prefix | PASS | `grep -n "AssertionFailure" src/panic_hook.rs` → line 128: `format!("#wat.kernel/AssertionFailure {}\n", wat_edn::write(&edn_value))` |
| 2 | `payload_to_edn(payload) -> OwnedValue` helper exists | PASS | `grep -n "fn payload_to_edn" src/panic_hook.rs` → line 137: `pub(crate) fn payload_to_edn(payload: &AssertionPayload) -> OwnedValue` |
| 3 | EDN envelope contains all 7 documented fields | PASS | Lines 187–193: `:thread :message :location :actual :expected :frames :upstream-chain` all constructed in the `OwnedValue::Map` vec |
| 4 | `:location` is a map when present, `nil` when absent | PASS | Test #2 (`renders_message_only_when_location_missing`) asserts `loc == &OwnedValue::Nil`; Test #1 (`renders_location_and_values_when_present`) asserts `loc.as_map()` and inspects `:file :line :col` — both pass |
| 5 | `:frames` is always a vector (possibly empty) | PASS | Line 164: `OwnedValue::Vector(payload.frames.iter().map(frame_to_map).collect())` — unconditional; no guard |
| 6 | `:upstream-chain` properly serializes Value via `edn_shim::value_to_edn_with(&v, None)` | PASS | Line 179: `.map(\|v\| crate::edn_shim::value_to_edn_with(v, None))` — called for each Vec element |
| 7 | `backtrace_enabled` / `RUST_BACKTRACE_ENABLED` removed | PASS | `grep -n "RUST_BACKTRACE\|backtrace_enabled" src/panic_hook.rs` → only one doc-comment hit (`RUST_BACKTRACE is no longer`); zero production code |
| 8 | All 4 updated lib tests pass | PASS | `cargo test --release --lib panic_hook::` → 4 passed, 0 failed |
| 9 | Probe test still passes | PASS | `cargo test --release --test probe_panic_hook_auto_installed` → 1 passed |
| 10 | Workspace failure count not increased | PASS | Pre-flight: 11 targets failed; Post-ship: 11 targets failed; identical target set |

---

## Pre-flight workspace summary (raw `tail -25`)

```
   Doc-tests wat_telemetry

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests wat_telemetry_sqlite

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 11 targets failed:
    `-p wat --test probe_lifeline_pipe_proof`
    `-p wat --test probe_no_default_rust_panic_noise_on_stderr`
    `-p wat --test probe_plain_panic_produces_structured_edn`
    `-p wat --test probe_run_hermetic_no_deadlock`
    `-p wat --test probe_runtime_err_stderr_visibility`
    `-p wat --test probe_runtime_error_produces_structured_edn`
    `-p wat --test test`
    `-p wat --test wat_arc113_cross_fork_cascade`
    `-p wat --test wat_arc170_program_contracts`
    `-p wat --test wat_run_sandboxed`
    `-p wat-cli --test wat_cli`
```

## Post-ship workspace summary (raw `tail -25`)

```
   Doc-tests wat_telemetry

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests wat_telemetry_sqlite

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 11 targets failed:
    `-p wat --test probe_lifeline_pipe_proof`
    `-p wat --test probe_no_default_rust_panic_noise_on_stderr`
    `-p wat --test probe_plain_panic_produces_structured_edn`
    `-p wat --test probe_run_hermetic_no_deadlock`
    `-p wat --test probe_runtime_err_stderr_visibility`
    `-p wat --test probe_runtime_error_produces_structured_edn`
    `-p wat --test test`
    `-p wat --test wat_arc113_cross_fork_cascade`
    `-p wat --test wat_arc170_program_contracts`
    `-p wat --test wat_run_sandboxed`
    `-p wat-cli --test wat_cli`
```

Failure sets are identical. Delta = 0.

---

## Sample rendered EDN output

For the representative `AssertionPayload` from `renders_location_and_values_when_present` (thread named, location present, frames=[one frame], upstream_chain=None):

```edn
#wat.kernel/AssertionFailure {:thread "wat-test::my-deftest" :message "assert-eq failed" :location {:file "wat-tests/foo.wat" :line 12 :col 5} :actual "-1" :expected "42" :frames [{:callee :my.app/foo :at {:file "wat-tests/foo.wat" :line 12 :col 5}}] :upstream-chain nil}
```

Key shape observations:
- Top-level: `#wat.kernel/AssertionFailure {...}` — tagged map, single line (no pretty-print; `wat_edn::write` is compact)
- `:thread` — string value
- `:location` — inline map `{:file "..." :line 12 :col 5}`
- `:frames` — vector of maps; each map has `:callee` (keyword) and `:at` (location map)
- `:callee` for `":my::app::foo"` renders as `:my.app/foo` — `::` is translated: last `::` splits ns from name; intervening `::` become `.`
- `:upstream-chain` — `nil` when `payload.upstream_chain` is `None`
- `:thread` — `nil` when `payload.thread_name` is `None`

For the minimal payload (no thread, no location, no frames, no upstream):
```edn
#wat.kernel/AssertionFailure {:thread nil :message "plain panic" :location nil :actual nil :expected nil :frames [] :upstream-chain nil}
```

---

## Honest deltas vs EXPECTATIONS

### Delta 1: `OwnedValue = Value<'static>` — orchestrator's hypothesis confirmed with nuance

Orchestrator hypothesized `OwnedValue::Map(Vec<(OwnedValue, OwnedValue)>)`. Actual: `OwnedValue` is a type alias `type OwnedValue = Value<'static>`, and `Value::Map(Vec<(Value<'a>, Value<'a>)>)`. In practice this means constructing `OwnedValue::Map(vec![(OwnedValue::Keyword(...), OwnedValue::Nil), ...])` which matches the hypothesis exactly — just spelled differently. `Cow::Owned(string)` is required for string values since `'static` lifetime is needed.

### Delta 2: `keyword_from_wat_path` not reused — inlined instead

`edn_shim::keyword_from_wat_path` is a private `fn`; not accessible from `panic_hook`. Sonnet inlined the same logic as `keyword_from_callee_path` in `panic_hook.rs`. Identical behavior: strip leading `:`, rfind `"::"`, split on last `::`, ns = prefix with `::` → `.` translation, name = suffix. Fallback to `String` on validation failure.

### Delta 3: `:upstream-chain` for empty Vec

Orchestrator asked: empty Vec → `nil` or `[]`? Sonnet picked `nil` for both `None` and `Some(vec![])` to match Option semantics and keep the nil-vs-present distinction clean. Empty upstream chain carries no information different from no chain.

### Delta 4: Test assertion strategy — parse_owned round-trip

Orchestrator suggested either parse round-trip or substring assertions. Sonnet used `wat_edn::parse_owned` successfully — the EDN round-trips cleanly. Tests parse the envelope, assert the tag string, and inspect map fields via `get_field` helper. More robust than substring matching.

### Delta 5: `Tag` import removed — not needed directly

The envelope is emitted via `format!("#wat.kernel/AssertionFailure {}\n", wat_edn::write(&edn_value))` — the tag string is literal, not constructed via `Tag::ns`. `Tag` import was removed. This is intentional: `payload_to_edn` returns a plain `OwnedValue::Map`; the caller wraps it in the formatted tag prefix. This mirrors `emit_structured_exit` which uses `format!("#wat.kernel/ProcessPanics {}\n", ...)`.

### Delta 6: No STOP triggers hit

All four STOP triggers were checked:
- `OwnedValue::Map` construction: works with `Vec<(OwnedValue, OwnedValue)>` via `Cow::Owned` strings — no lifetime obstruction.
- `value_to_edn_with` signature: `pub fn value_to_edn_with(v: &Value, types: Option<&TypeEnv>) -> OwnedValue` — `None` works as TypeEnv; no SymbolTable reference needed.
- Workspace failure count: stayed at 11, same set.
- `frame.callee_path` keyword construction: `keyword_from_callee_path` handles all path shapes; falls back to String on validation failure.

---

## Mode classification

**Mode A** — ships per scope; all 10 scorecard rows PASS; surprises bounded within EXPECTATIONS delta-watch.

Reasoning:
- All 4 lib tests updated and passing
- Probe test unaffected (211a's load-bearing claim preserved)
- Workspace delta = 0 (same 11 failing targets, same set)
- `RUST_BACKTRACE_ENABLED` static and `backtrace_enabled()` fn removed
- `payload_to_edn` helper implemented and exported `pub(crate)`
- 7 envelope fields present: `:thread :message :location :actual :expected :frames :upstream-chain`
- EDN format mirrors `#wat.kernel/ProcessPanics{...}` from arc 170 slice 1i
- No new files, no new dependencies, no Cargo.toml changes
- Only `src/panic_hook.rs` touched

---

## Files modified

- `/home/watmin/work/holon/wat-rs/src/panic_hook.rs` — sole change target

## Files created (new)

- `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/211-every-deadlock-is-a-panic/SCORE-211B-PANIC-AS-EDN.md` — this file
