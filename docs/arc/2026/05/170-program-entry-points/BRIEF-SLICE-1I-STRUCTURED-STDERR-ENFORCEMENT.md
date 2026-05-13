# Arc 170 slice 1i BRIEF — substrate-wide structured-exit protocol

**Sonnet.** Substrate-level fix. Enforces the existing `structured-stderr-only` doctrine ([`TIERS.md`](./TIERS.md):75) at every wat-process child exit path. No wat-cli-only patch — the discipline lives in the substrate so EVERY way to spawn a wat-process inherits it.

User direction 2026-05-15 (rescope):
> *"i think we articulated that making wat-cli do this was wrong... we needed it deeper in the such that everything in wat does this, not something you can forget if you aren't going via wat-cli.. wat-cli is a benefactor of this, not an implementor."*

## The doctrine (already exists)

[`docs/arc/2026/05/170-program-entry-points/TIERS.md`](./TIERS.md) line 75:

> **Doctrine: structured-stderr-only.** Inside wat-land, fd 2 ONLY ever carries panic-cascade EDN. wat-cli has zero direct stderr writes. Pretty-printing is downstream (shell user pipes through formatter if they want).

The doctrine is named. The substrate violates it. This slice closes the gap.

## The violations (empirically confirmed)

The probe `tests/probe_runtime_err_stderr_visibility.rs` (committed `507ae5c`) shows what currently lands on fd 2 for an AssertionPayload panic:

```
[0] (empty)
[1] thread 'probe...' panicked at src/assertion.rs:151:5:    ← Rust default panic handler
[2] Box<dyn Any>                                              ← Rust default panic handler
[3] note: run with `RUST_BACKTRACE=1` ...                     ← Rust default panic handler
[4] #wat.kernel/ProcessPanics [...]                           ← THE STRUCTURED LINE (good)
[5] panic: spawn-process body panicked                        ← substrate plain-text trailer
```

Of 6 lines on stderr, ONLY line 4 is the structured EDN the doctrine requires. The other 5 lines violate.

### Three classes of violation

**Class A — runtime-error path** (`spawn_process.rs:392-394` and `fork.rs:676-679`):
```rust
Ok(Err(runtime_err)) => {
    write_direct_to_stderr(&format!("runtime: {:?}\n", runtime_err));
    unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
}
```
Plain text. No structured EDN. The 5 svc-test failures observed post-Gap-K all hit this path.

**Class B — plain-panic path** (`spawn_process.rs:396-411` and `fork.rs:680-695`):
```rust
Err(panic_payload) => {
    if let Some(payload) = panic_payload.downcast_ref::<AssertionPayload>() {
        emit_panics_to_stderr(&world, payload);   // ← ONLY this case emits structured
    }
    write_direct_to_stderr("panic: spawn-process body panicked\n");
    unsafe { libc::_exit(EXIT_PANIC) };
}
```
Only `AssertionPayload` panics emit structured EDN. Plain panics (bare `String`, `&str`, raised types other than AssertionPayload) skip the structured emit.

**Class C — startup / entry-form / Rust-default-handler** (multiple sites in both files):
- `write_direct_to_stderr(&format!("startup: {}\n", e))` — plain text
- `write_direct_to_stderr(&format!("entry_form eval: {}\n", e))` — plain text
- Rust's default panic handler runs BEFORE substrate's `catch_unwind` and writes 3-4 lines of its own plain-text output to fd 2 (the noise in probe lines [0]-[3])

## What the harness does with the violations

`wat/test.wat:530-540` `run-hermetic-driver`:

```scheme
failure
 (match joined-result
   ((Ok _)   None)
   ((Err chain)
    (Some (failure-from-process-died
            (match stderr-chain
              ((Some sc) sc)
              (None      chain))))))     ; ← (None chain) DROPS the stderr-lines
```

When the child violates the doctrine, `extract-panics` returns `None`. The harness's `(None chain)` fallback discards `stderr-lines` and uses ONLY join-result's exit-code-summary chain. The user sees "forked program exited N" with no actual error content. The actual diagnostic is in `RunResult.stderr` but `failure_to_diagnostic` (`src/test_runner.rs:640`) only reads `Failure.message` — never the stderr field.

This is the test infrastructure failing its users. The diagnostic is in hand; it's thrown away.

## The fix (failure engineering, level-2)

Eliminate the violation class structurally. Every child exit path emits the SAME structured envelope. The harness has ONE parse path. No fallback because `extract-panics` always finds the envelope.

### Substrate-side changes

**1. Custom panic hook in spawn-process / fork children.**

Before the child's `catch_unwind` block, install a panic hook (`std::panic::set_hook`) that suppresses Rust's default panic-output-to-stderr. The substrate's own structured emit becomes the SOLE source of stderr content per panic.

```rust
fn install_structured_panic_hook() {
    std::panic::set_hook(Box::new(|_info| {
        // Suppressed: substrate's catch_unwind + emit_structured handles
        // panic propagation to stderr. Rust's default handler must not
        // leak plain text on fd 2 in wat-process children.
    }));
}
```

Install this in BOTH `spawn_process_child_branch` (after `setpgid`/`dup2` setup) AND `fork.rs::child_branch_from_source` (or equivalent).

**2. Unified structured emit for ALL exit paths.**

Extend `emit_panics_to_stderr` to accept a discriminator for the source kind:

```rust
fn emit_structured_exit(
    world: &FrozenWorld,
    kind: ProcessDiedKind,   // RuntimeError, Panic, StartupError, EntryFormFailure, MainSignature, BadReturn
    message: String,
    upstream: Option<AssertionPayload>,
) {
    // Build #wat.kernel.ProcessDiedError/<kind> value with message + optional upstream chain
    // Wrap in #wat.kernel/ProcessPanics [...]
    // Write the EDN line via write_direct_to_stderr
}
```

Then every child exit path calls it before `libc::_exit`:

| Path | Kind | Message source |
|------|------|----------------|
| `Ok(Err(runtime_err))` | `RuntimeError` | `format!("{}", runtime_err)` (use Display, not Debug — cleaner) |
| Panic-with-AssertionPayload | `Panic` | existing chain from payload |
| Panic-plain (no payload) | `Panic` | downcast to `String` or `&str`; else fallback "<unknown panic payload>" |
| startup error | `StartupError` | `format!("{}", e)` |
| entry_form eval err | `EntryFormFailure` | `format!("{}", e)` |
| non-fn entry value | `EntryFormFailure` | "entry_form did not evaluate to fn" |
| non-nil return | `BadReturn` | `format!("non-nil return: {}", value.type_name())` |
| main signature mismatch (fork.rs) | `MainSignature` | existing message |

Substrate's `ProcessDiedError` enum (`src/runtime.rs`) likely needs these variants if not already present — verify.

**3. Retire `write_direct_to_stderr` for plain-text exits.**

After (2), `write_direct_to_stderr` should ONLY be called from the structured emit helper. Direct calls from exit paths get replaced. The helper stays (for future debug-tracing if needed, behind a feature flag), but it's no longer the substrate's user-facing stderr surface.

### Wat-side harness changes

**4. Retire the `(None chain)` fallback in `run-hermetic-driver` (and 3 siblings).**

After the substrate enforces structured emission, `extract-panics` returns `Some(...)` on every child error. The `(None chain)` arm becomes unreachable. Replace with a panic that names the contract violation:

```scheme
((:wat::core::None)
 (:wat::kernel::assertion-failed!
   "structured-stderr-only contract violation: child error but no structured EDN found on stderr"
   :wat::core::None :wat::core::None))
```

This is the harness teaching: if a child exits with error but stderr-chain is None, that's a SUBSTRATE BUG — not a runtime variation to silently work around. Sites: `wat/test.wat:530`, `wat/test.wat:715` (run-hermetic-with-io-driver), `wat/kernel/hermetic.wat:120`, `wat/kernel/sandbox.wat:120`.

### Tests / probes

**5. Extend the existing probe + add coverage for the 3 violation classes.**

`tests/probe_runtime_err_stderr_visibility.rs` (committed `507ae5c`) currently exercises the AssertionPayload path. Add:

```rust
#[test]
fn probe_runtime_error_produces_structured_edn() {
    // body that hits Ok(Err(runtime_err)) — e.g., option::expect on None
    // OR call to an undefined symbol via try-resolve
    // assert: RunResult.failure.message != "forked program exited N"
    // assert: RunResult.failure.message is the actual runtime error text
    // assert: stderr-chain extracted properly
}

#[test]
fn probe_plain_panic_produces_structured_edn() {
    // body that raises a plain panic (raise! with a non-AssertionPayload value)
    // assert: same as above
}

#[test]
fn probe_no_default_rust_panic_noise_on_stderr() {
    // body that panics with AssertionPayload
    // assert: stderr_lines does NOT contain "thread '...' panicked at" or "note: run with RUST_BACKTRACE=1"
    // proves the custom panic hook suppressed Rust's default handler
}
```

These probes are the substrate-as-teacher integ tests for the new contract.

### Substrate-as-teacher cross-link

This slice is the canonical Pattern 1 application of the existing structured-stderr-only doctrine. The doctrine names the rule; this slice enforces it structurally. After landing, future arcs that mint new child-exit paths inherit the discipline by calling `emit_structured_exit` instead of `write_direct_to_stderr`. Adding a new "forgetting" path requires deliberate work — it's not the path of least resistance anymore.

## Required reading IN ORDER

1. `docs/arc/2026/05/170-program-entry-points/TIERS.md` § structured-stderr-only doctrine (line 75)
2. `docs/SUBSTRATE-AS-TEACHER.md` § Pattern 1 / Pattern 3 — the discipline shape
3. `src/spawn_process.rs:247-456` (entire child branch + helpers) — the violation site
4. `src/fork.rs:564-700` (child_branch_from_source) — the parallel violation site
5. `src/fork.rs:53-160` (older fork-program child branch, simpler form) — check for similar paths
6. `src/runtime.rs` — `ProcessDiedError` enum definition (grep for `ProcessDiedError::`); verify variants for RuntimeError/Panic/StartupError exist; mint if missing
7. `src/assertion.rs` — `AssertionPayload` shape; `emit_panics_to_stderr` flow
8. `wat/test.wat:506-542` (`run-hermetic-driver`) — harness's match against extract-panics result
9. `tests/probe_runtime_err_stderr_visibility.rs` — existing probe; extend with the 3 new probes
10. `src/check.rs::ProcessJoinBeforeOutputDrain` (committed `8ef69f4`) — recent precedent for adding a substrate-level check; not directly relevant but pattern reference

## Scope (what's IN)

- Custom panic hook installation in BOTH spawn-process and fork child branches
- `emit_structured_exit` helper (or extended `emit_panics_to_stderr`) covering ALL 7+ child exit paths
- Substrate `ProcessDiedError` enum variants added if missing (RuntimeError, StartupError, EntryFormFailure, BadReturn, MainSignature — confirm via grep first)
- 3 new probes covering runtime-error path, plain-panic path, and Rust-default-handler-suppression
- Existing probe (`probe_runtime_err_stderr_visibility.rs`) verified still passing — its assertion expectations may need tightening now that Rust noise is gone
- Wat-side harness retirement of `(None chain)` fallback in `run-hermetic-driver` + 3 siblings — replaced with structural-contract-violation panic
- Update USER-GUIDE — note that wat-cli is a BENEFACTOR not implementor; the structured-stderr-only contract lives in the substrate

## Scope (what's OUT)

- The 5 svc-test failures themselves (separate concern — Gap K BRIEF Row F said they'd surface real errors; this slice surfaces them; the real fixes follow)
- Pattern A typealias / Pattern C exit-3 categorization beyond the harness fix
- Any change to `src/check.rs` (deadlock detection — independent)
- `wat-cli`'s own exit paths — `wat-cli` inherits the discipline by going through these child-branch paths; no separate cli-side fix needed
- Console / StdErrService wat-side services (slice 1F-* territory; separate)
- Anything under `docs/arc/` (FM 11)
- Memory under `~/.claude/`

## Hard constraints

- DO NOT modify `src/check.rs`
- DO NOT add wall-clock timeouts
- DO NOT touch deftest macro (V5 retry shape stays)
- DO NOT touch `docs/arc/` or `~/.claude/`
- DO NOT use `cd <subdir> && ...` (FM 7) — use absolute paths or `git -C`
- DO NOT commit / push / git add — orchestrator atomic-commits after scoring
- DO NOT use `timeout 600` or any > 120s wrapper
- DO use `git -C /home/watmin/work/holon/wat-rs` for any git operations
- DO use `pkill -9 -f "target/release/deps/test-"` if orphans appear; report in SCORE
- DO NOT name a probe file in a way that doesn't match what its bodies test (Row G discipline carries over)
- If the workspace doesn't pass post-fix, STOP and report — the substrate-as-teacher discipline says diagnostics are the brief

## Verification (substrate-as-teacher style)

The Diagnostic stream itself is the verifier. Before the fix, the 5 svc-test failures all say "forked program exited 3" — no diagnostic. After the fix:

```bash
cd /home/watmin/work/holon/wat-rs
timeout -k 5 90 cargo test --release -p wat --test test 2>&1 | grep "forked program exited"
# Before fix: 5+ matches (the svc-test failures' Failure.message text)
# After fix: 0 matches — every Failure.message now carries the actual error
```

The probes verify the contract structurally. The workspace test count + the human-readable diagnostic content verify the user-facing impact.

## Ship criteria (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Custom panic hook installed in spawn_process_child_branch + fork.rs::child_branch_from_source; suppresses Rust default panic output | grep + read; probe_no_default_rust_panic_noise_on_stderr passes |
| B | `emit_structured_exit` (or extended emit_panics_to_stderr) emits structured `#wat.kernel/ProcessPanics` for ALL exit paths: runtime error, plain panic, AssertionPayload panic, startup error, entry-form failure, bad-return, main-signature | grep src/ for `write_direct_to_stderr` — only the helper itself remains; no direct callers from exit paths |
| C | `ProcessDiedError` enum has variants for all kinds emitted (Panic / RuntimeError / StartupError / EntryFormFailure / BadReturn / MainSignature); mint missing variants if needed | grep + read |
| D | `probe_runtime_error_produces_structured_edn` PASSES — runtime-error path produces structured EDN; failure.message is actual runtime error text, not "forked program exited N" | cargo test |
| E | `probe_plain_panic_produces_structured_edn` PASSES — plain panic path produces structured EDN | cargo test |
| F | `probe_no_default_rust_panic_noise_on_stderr` PASSES — Rust's default panic handler output absent from RunResult.stderr | cargo test |
| G | Existing probe `probe_runtime_err_stderr_visibility` still PASSES (may need updated expectations since Rust noise is gone) | cargo test |
| H | Wat-side harness `(None chain)` fallback retired in `run-hermetic-driver` + 3 siblings; replaced with structural-contract-violation panic; workspace remaining failures (Pattern A/C surfaced from svc-tests) now show ACTUAL error messages, not "exited 3" | grep + read + workspace test output |

**8 rows. All must PASS.**

## Predicted runtime

**60-90 min sonnet.** Substantive Rust + small wat-side change. The structured emit + custom panic hook are mechanical. The harness retirement is a 4-site grep+replace. The probes are 3 new test bodies. Workspace verification is fast.

**Hard cap:** 180 min (2×). ScheduleWakeup at T+10800s (substrate slices warrant more headroom).

## Honest deltas (anticipated)

1. **ProcessDiedError variants** — `src/runtime.rs` may not have all the kinds we need. Mint missing variants; document the addition.
2. **emit_structured_exit ergonomics** — taking `&FrozenWorld` for the EDN encoding context. For startup errors, the FrozenWorld may not exist yet (startup itself failed). Surface this — likely emit without TypeEnv context (best-effort EDN).
3. **Rust panic hook ordering** — the hook MUST install before any code that might panic in the child branch. Setpgid + dup2 happen BEFORE the hook is installed today. Either move hook install up, or audit those early calls for panic-safety.
4. **Probe path-honesty (Row G from Gap K)** — each new probe file must exercise the path its filename names. Don't switch paths to make tests pass.
5. **Workspace test count delta** — after the harness change, the 5 svc-test failures probably STILL fail (real underlying defect not fixed by this slice) but Failure.message becomes useful. Verify the message text actually surfaces the real error.

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 1 (type-shape change) discipline
- `docs/arc/2026/05/170-program-entry-points/TIERS.md`:75 — the doctrine being enforced
- `docs/arc/2026/05/170-program-entry-points/SPAWN-MIGRATION-BACKLOG.md` — this slice IS a foundational discovery from Step 2 verification (the harness analysis surfaced it)
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 17 — pre-action sweep applies throughout
- `tests/probe_runtime_err_stderr_visibility.rs` (committed `507ae5c`) — the empirical proof of the gap

## Deliverable

After implementing + verifying, write `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1I-STRUCTURED-STDERR-ENFORCEMENT.md` with:

- 8-row scorecard (PASS/FAIL per row)
- Before/after of each modified exit path in spawn_process.rs and fork.rs
- The 3 new probe filenames + brief description of what each tests + Row G alignment confirmation
- Workspace state after fix (pass/fail counts; remaining failures' actual error messages now visible)
- Honest deltas (≥ 3)
- USER-GUIDE update noting wat-cli as benefactor not implementor

Then STOP. Report what shipped + path to SCORE doc + scorecard summary.

User direction load-bearing here:
> *"making wat-cli do this was wrong... we needed it deeper in the such that everything in wat does this, not something you can forget if you aren't going via wat-cli."*

The discipline lives in the substrate. wat-cli is a benefactor, not an implementor. The slice ships when that's structurally true.
