# Arc 170 slice 3 Gap D — SCORE (top-level `let` splice for `def`/`defn`)

**Scored:** 2026-05-11. Sonnet one-spawn.

## Scorecard (6 rows)

| Row | What | Result |
|-----|------|--------|
| A | `register_defines` extended with `let` arm | PASS — grep confirms `":wat::core::let"` arm in `register_defines` (runtime.rs:1510) calling `preregister_fn_defs_in_let` |
| B | `register_stdlib_defines` extended with `let` arm | PASS — grep confirms mirror arm (runtime.rs:1558) |
| C | All three probes in `tests/probe_let_splice_def.rs` pass | PASS |
| D | Workspace at 0 failed (non-pre-existing) | PASS — 0 new failures; pre-existing `wat-holon-lru` + `wat-sqlite` flaky failures unchanged |
| E | `cargo check --release` green | PASS — clean, 0 errors |
| F | SCORE documents impl + `let*` gap status | PASS — see below |

**All 6 rows PASS.**

## Workspace delta

- Baseline (excluding pre-existing flaky failures): 2176 passed / 0 failed
- Post Gap D: 2179 passed / 0 failed (+3 probes, all new, all passing)

(Pre-existing flaky failures in `wat-holon-lru` and `wat-sqlite` unchanged — confirmed by stashing changes and rerunning both packages to verify identical failure patterns.)

## Files changed

| File | Change |
|------|--------|
| `src/runtime.rs` | `register_defines` + `register_stdlib_defines` extended with `let` arm; `preregister_fn_defs_in_let` helper added; `register_runtime_defs_form` `def` arm extended to also update `sym.functions` when value is a fn (closure sync) |
| `tests/probe_let_splice_def.rs` | Three regression probes added (new file) |

## Implementation details

### Helper choice: parallel `preregister_fn_defs_in_let` (not generalization)

A separate `preregister_fn_defs_in_let` helper was added rather than generalizing `preregister_fn_defs_in_do`. The structural difference between the two forms requires different body offsets:

- `do` body starts at `items[1..]` (after the head keyword only)
- `let` body starts at `items[2..]` (after the head keyword AND the bindings vector, per arc 168 multi-form body)

Unifying them would require passing an offset parameter, which buys no clarity and costs readability. Two clearly-named helpers are simpler.

The `preregister_fn_defs_in_let` helper (runtime.rs:2281) recurses into nested `let` forms in the body — the same nested-recursion discipline as `preregister_fn_defs_in_do`.

### Closure sync fix: `register_runtime_defs_form` updates `sym.functions`

**Unexpected complication discovered and fixed in this slice.**

Pre-registering fn-shape `def`s from a `let` body into `sym.functions` (with `closed_env: None`) created a dispatch-priority bug:

- `eval_tail` checks `sym.functions.contains_key(other)` and dispatches through `sym.get` if found. A `def`-bound fn pre-registered as a stub (no closure) wins — even when `runtime_def_values` holds the correctly-closed version evaluated from inside the `let` scope.

**Fix:** `register_runtime_defs_form`'s `def` arm (runtime.rs:2059) now additionally writes to `sym.functions` when the evaluated value is a fn:

```rust
if let Value::wat__core__fn(ref func) = value {
    sym.functions.insert(name.clone(), func.clone());
}
```

This overwrites the `preregister_fn_defs_in_let` stub with the properly-closed fn. `sym.functions` becomes authoritative for both resolve-time validation AND call dispatch — no change to dispatch ordering needed, no performance regression.

Test that would have regressed without this fix: `def_runtime_let_splice_closure_capture` in `tests/wat_arc157_def.rs`. A `let`-bound `config = 42` was captured by a `def`-bound fn. The pre-registered stub (no closure) was winning in `eval_tail` and could not find `config` at call time. With the sync fix, `sym.functions` holds the closed fn before any call dispatch.

This fix is a correctness requirement. It is sound for Gap C's `do` case as well (fns in `do` don't capture let-local bindings, so `runtime_def_values` and `sym.functions` already agree — no change in behavior there).

### Registration passes extended

Same analysis as Gap C V2. The `let` arm was added to:
- `register_defines` (user source, reserved-prefix check enforced)
- `register_stdlib_defines` (stdlib source, privileged — no reserved-prefix check)

Both were needed for symmetry (no stdlib sources currently exercise this path, but the mirror is correct by construction).

## `let*` parallel gap status

**No gap exists.** `:wat::core::let*` was retired into `:wat::core::let` by arc 154. Any source using `:wat::core::let*` is flagged with a `RetiredKeyword` diagnostic at the check pass. The `collect_splice_defs_ctx` function (check.rs:6853) has no `let*` arm because retired keywords are rejected before the splice-eligibility check runs. `register_defines` correspondingly needs no `let*` arm.

## Honest deltas

1. **Closure sync was the unexpected complication.** `preregister_fn_defs_in_let` inserts a stub (no closure) into `sym.functions` for resolve-time validation. `eval_tail` dispatches through `sym.functions` first — so the stub won over the properly-closed fn in `runtime_def_values`. The fix: `register_runtime_defs_form`'s `def` arm now writes the evaluated fn BACK into `sym.functions`, overwriting the stub. `sym.functions` becomes authoritative for both validation and call dispatch. No dispatch ordering change needed; no performance regression.

2. **The fix is minimal and non-disruptive.** One `if let Value::wat__core__fn` block in `register_runtime_defs_form`. No changes to `eval_tail`, `eval`, or any dispatch path. `sym.functions` becomes the single source of truth for fn-shape def calls, consistently closed at freeze time.

3. **`let*` gap is non-existent.** `:wat::core::let*` is retired — no arm needed anywhere. Clean.

4. **Pre-existing flaky failures not introduced.** `wat-holon-lru` and `wat-sqlite` show the same flaky failure patterns before and after my changes (confirmed by package-isolated test runs with stashed changes).

5. **Helper is a clean parallel, not a generalization.** The body-offset difference (`items[1..]` vs `items[2..]`) means the two helpers are structurally symmetric but cannot share an implementation without an opaque offset parameter. Separate helpers with clear names are the honest shape.

## Cross-references

- Gap C V2 SCORE (the precedent): [`SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md`](./SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md)
- Arc 154 (let* retirement): `src/check.rs:251` and throughout
- Arc 157 doctrine (def-in-top-level-let legal): `src/check.rs:715`
- Arc 168 multi-form body (items[2..] for let): established body-offset convention
- Phase E V3 (next): `deftest` macro — now unblocked across `do` AND `let` registrations
