# SCORE — Stone 243.7a — Box `RuntimeError` large variants (`result_large_err`)

## Scope decision

**Boxing only.** The BRIEF named two banked obligations:

- **RuntimeError → Pattern A** (error/signal separation) — BANKED. Signals (`TryPropagate`, `OptionPropagate`, `TailCall`, `UserMainMissing`, `EvalVerificationFailed`) do not carry `span`, so the span-at-outer-struct Pattern A form does not apply until the conflation is resolved in a future rolling-audit stone. RuntimeError remains flat; no vigilia REMARKABLE this stone.
- **Signal-split** — BANKED with Pattern A above.

This stone's obligation: box the large variant payloads so `clippy::result_large_err` clears and the 10 open-deferral runes close.

## Variants boxed

### Primary (from BRIEF — drove the lint)
- `NotCallable.got: ValueSnapshot` → `Box<ValueSnapshot>`
- `TypeMismatch.got: ValueSnapshot` → `Box<ValueSnapshot>`
- `BadCondition.got: ValueSnapshot` → `Box<ValueSnapshot>`
- `TryPropagate(Value)` → `TryPropagate(Box<Value>)`

### Iterate-added (clippy named after primary boxing)
- `PostconditionFailed.returned_value: ValueSnapshot` → `Box<ValueSnapshot>` — clippy named this variant (at least 200 bytes) once the ValueSnapshot variants shrank.
- `HarnessError::Runtime(RuntimeError)` → `Box<RuntimeError>` — clippy flagged `HarnessError` in `src/harness.rs` (`compose.rs` callers).
- `StartupError::Runtime(RuntimeError)` → `Box<RuntimeError>` — clippy flagged `StartupError` in `src/freeze.rs`.

## Cascade size

**21 files modified:**

`src/runtime.rs` (enum + 330 construction sites), `src/runtime_error_edn.rs` (Deref-transparent — 0 changes needed), `src/harness.rs`, `src/compose.rs`, `src/freeze.rs`, `src/function/eval.rs`, `src/function/mod.rs` (stamp only), `src/function/parse.rs`, `src/assertion.rs`, `src/edn_shim.rs`, `src/fork.rs`, `src/io.rs`, `src/rust_deps/custodia.rs`, `src/rust_deps/marshal.rs`, `src/rust_deps/mod.rs` (stamp only), `src/spawn.rs`, `src/spawn_process.rs`, `src/string_ops.rs`, `src/thread_io.rs`, `src/time.rs`, plus 2 test files `tests/probe_arc237_stone4_rich_errors.rs`, `tests/probe_stone_233_3_runtime_error_edn.rs`.

Match/access sites across `src/`: Deref coercions (`Box<T>` → `T`) handled all read-only uses transparently. The only non-transparent match was `TryPropagate(e) => { Ok(Value::Result(Arc::new(Err(e)))) }` where `e` needed `*e` to deref the `Box<Value>` to owned `Value`. Handled explicitly.

## Verify outputs (verbatim)

```
$ cargo clippy --release -p wat 2>&1 | grep -c result_large_err
0

$ cargo test --release --lib -p wat 2>&1 | tail -3
test result: ok. 895 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.18s

$ cargo build --release --tests -p wat 2>&1 | grep "^error" | head -5
(no output)

$ grep -rn "OPEN-DEFERRAL → 243.7a" src/ && echo "FOUND" || echo "CLEAN"
CLEAN

$ grep -rn "result_large_err" src/ && echo "FOUND" || echo "CLEAN"
CLEAN
```

## Runes / allows removed (10 pairs)

All 10 `rune:excusare(OPEN-DEFERRAL → 243.7a)` + `#[allow(clippy::result_large_err)]` pairs struck:

| File | Count |
|---|---|
| `src/function/eval.rs` | 1 |
| `src/function/parse.rs` | 1 |
| `src/rust_deps/marshal.rs` | 3 |
| `src/rust_deps/custodia.rs` | 4 |
| `src/runtime.rs` | 1 |
| **Total** | **10** |

## Stamp drifts closed

- `src/function/mod.rs:1` — dropped `(clippy clean-or-runed: result_large_err → excusare OPEN-DEFERRAL 243.7a)` qualifier. Stamp now reads: `vigilatum: 2026-06-01T04:45:47Z — vigilia 8-spell L1+L2=0`
- `src/rust_deps/mod.rs:1` — dropped same qualifier. Stamp now reads: `vigilatum: 2026-06-01T04:45:47Z — vigilia 7-spell L1+L2=0`

Both homes are now fully clippy-clean with no outstanding runes. The L1+L2=0 claim is no longer qualified.

## Notes

- `runtime_error_edn.rs` needed zero source changes: all match arms accessing boxed fields (`got`, `returned_value`, `TryPropagate(value)`) resolved transparently via Rust's Deref coercions (`&Box<T>` → `&T` for `&`-access, `&T` for display/function-call arguments).
- No behavior change: boxing is heap-indirection only; identical runtime behavior.
- No `#[allow]` re-added at any site. Every cleared lint = structural fix.
