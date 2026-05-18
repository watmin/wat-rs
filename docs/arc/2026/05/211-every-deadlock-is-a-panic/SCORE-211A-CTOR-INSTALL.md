# Arc 211a — SCORE: `#[ctor]` auto-install of `panic_hook`

**Mode:** A (ships per scope; all scorecard PASS; surprises bounded within honest-delta-watch)

**Completed:** 2026-05-18

---

## Scorecard

| # | Criterion | Verification | Result |
|---|---|---|---|
| 1 | `ctor` dep added to `Cargo.toml` | `grep -n "^ctor\|\"ctor\"" Cargo.toml` → line 69: `ctor = "1"` | PASS |
| 2 | `INSTALLED: AtomicBool` static in `panic_hook.rs` | `grep -n "INSTALLED" src/panic_hook.rs` → line 64: `static INSTALLED: AtomicBool = AtomicBool::new(false);` | PASS |
| 3 | `install()` short-circuits when already installed | Guard at line 102: `if INSTALLED.swap(true, Ordering::SeqCst) { return; }` — swap returns old value; true → already installed → early return | PASS |
| 4 | `#[ctor::ctor(unsafe)] fn auto_install()` at module scope | `grep -n "ctor::ctor" src/panic_hook.rs` → line 74: `#[ctor::ctor(unsafe)]` + line 75: `fn auto_install()` | PASS |
| 5 | `pub fn is_installed() -> bool` exists | `grep -n "fn is_installed" src/panic_hook.rs` → line 121: `pub fn is_installed() -> bool` | PASS |
| 6 | Probe test file exists | `ls tests/probe_panic_hook_auto_installed.rs` → confirmed | PASS |
| 7 | Probe test passes | `cargo test --release --test probe_panic_hook_auto_installed` → `test panic_hook_auto_installed_via_ctor ... ok` | PASS |
| 8 | Existing `panic_hook` lib tests still pass | `cargo test --release --lib panic_hook` → 4 passed; 0 failed | PASS |
| 9 | Workspace failure count not increased vs baseline | Pre-flight: 11 targets failed. Post-ship: 11 targets failed (same set). See summaries below. | PASS |

---

## Pre-flight workspace summary (raw `tail -20`)

```
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

---

## Post-ship workspace summary (raw `tail -25`, final stable run)

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

**Delta: 0.** Same 11 targets; identical set.

---

## Honest deltas vs EXPECTATIONS

### 1. ctor attribute spelling: `#[ctor::ctor(unsafe)]` not `#[ctor::ctor]`

EXPECTATIONS item 4 explicitly called out "`#[ctor::ctor]` vs `ctor::ctor!`" as a predicted surprise. The actual surprise was a third form: ctor 1.x (the latest stable, 1.0.6) requires `#[ctor(unsafe)]` — the compiler error message reads:

```
error: Missing unsafe keyword in #[ctor] annotation. Use #[ctor(unsafe)].
```

This is NOT a STOP trigger (STOP trigger #1 is "API changed such that `#[ctor::ctor]` ISN'T the attribute spelling" — but it IS the attribute spelling, just with a required `unsafe` annotation). The crate is working; the ctor mechanism is intact; the `unsafe` requirement is ctor 1.x's safety-conscious API change for library constructors that run before full Rust runtime initialization. The fix was a single word addition. Attribute spelling used: `#[ctor::ctor(unsafe)]`.

### 2. ctor version: `1` (1.0.6) not `0.2.x` or `0.3.x`

EXPECTATIONS predicted "0.2.x or 0.3.x." The latest stable is 1.0.6. Used `ctor = "1"` (semver-compatible with 1.0.6). No dep conflict in Cargo.lock.

### 3. Ordering: `SeqCst` as predicted

EXPECTATIONS predicted `SeqCst` as the conservative default. Used as specified. No deviation.

### 4. Probe shape: exact match to BRIEF spec

Probe test is exactly the shape specified in BRIEF § "Probe test shape." No deviation.

### 5. Intermediate workspace run showed `-p wat --lib` in failure set

During step 7 (post-ship workspace test), one intermediate run showed `-p wat --lib` in the failure set (with `probe_lifeline_pipe_proof` absent). This was non-deterministic: `probe_lifeline_pipe_proof` is a hang-prone test (pre-existing); when it hangs and times out during the parallel workspace run, it can hold resources that cause other tests to fail. The FINAL stable run (run 3 of 3) recovered to the identical 11-target pre-flight set. No new test failure introduced.

---

## Mode classification

**Mode A.** Ships per scope. All 9 scorecard rows PASS. Surprises bounded within EXPECTATIONS honest-delta-watch (ctor version + attribute spelling). Workspace failure count: 0 delta. LOC delta: ~30 (matches EXPECTATIONS prediction of ~30 LOC). Probe test passes first time after attribute spelling fix.

---

## Files touched

- `/home/watmin/work/holon/wat-rs/Cargo.toml` — added `ctor = "1"` dep
- `/home/watmin/work/holon/wat-rs/src/panic_hook.rs` — added `INSTALLED` static, `auto_install()` ctor fn, idempotency guard in `install()`, `is_installed()` accessor; updated module doc comment
- `/home/watmin/work/holon/wat-rs/tests/probe_panic_hook_auto_installed.rs` — NEW probe test

## Files NOT touched (as required)

- The 5 existing explicit `panic_hook::install()` call sites remain untouched:
  - `src/test_runner.rs:161`
  - `src/test_runner.rs:431`
  - `src/compose.rs:170`
  - `src/runtime.rs:21174`
  - `crates/wat-cli/src/lib.rs:253`

---

## Cross-references

- BRIEF-211A-CTOR-INSTALL.md — work definition
- EXPECTATIONS-211A-CTOR-INSTALL.md — independent prediction
- DESIGN.md § "Scope corrected 2026-05-18 (later)" — four-sub-arc locked scope
- Next: 211b (panic-as-EDN) — `AssertionPayload` gains EDN serializer
