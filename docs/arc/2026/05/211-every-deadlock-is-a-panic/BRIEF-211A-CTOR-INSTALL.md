# Arc 211a — `#[ctor]` auto-install of `panic_hook`

**Slice scope:** install `wat::panic_hook` automatically at library load via `#[ctor]`. Every binary linking wat-lib gets the hook before `main()` runs — impossible-to-forget by construction.

**Origin:** INTERSTITIAL § 2026-05-18 "Panic-as-EDN doctrine + ctor-install discipline." User direction: *"how do we make everything support this all the time - it is an illegal state to not have this - we can never forgot this - we are in an illegal state."*

**Closes:** the panic-hook install gap. Direct Rust `#[test] fn probe_*()` paths that touch substrate currently get cargo test's default panic formatter (prints `Box<dyn Any>` placeholder); after this slice they get `panic_hook::render_assertion_failure` automatically.

## Locked scope (do NOT expand)

**File touches (exactly these):**

1. **`Cargo.toml`** — add `ctor` crate as workspace dependency + wat-lib dependency. Pick latest stable 0.2.x or 0.3.x; whichever has clean cross-compile story.

2. **`src/panic_hook.rs`** — three additions:
   - `static INSTALLED: AtomicBool = AtomicBool::new(false);` at module scope
   - First line of existing `install()` becomes: `if INSTALLED.swap(true, Ordering::SeqCst) { return; }` — short-circuits when already installed; legacy explicit `install()` calls become idempotent no-ops
   - `#[ctor::ctor] fn auto_install() { install(); }` at module scope — runs at library load time, before `main()`
   - `pub fn is_installed() -> bool { INSTALLED.load(Ordering::SeqCst) }` — read-only accessor for the probe test

3. **`tests/probe_panic_hook_auto_installed.rs`** (NEW) — proves the ctor fires before any explicit install. See "Probe test shape" below.

**NOT in scope (separate slices/arcs):**
- Removing the 5 existing explicit `panic_hook::install()` call sites (they live on as no-ops; cleanup is a separate sweep)
- Changing the panic FORMAT to EDN (that's 211b)
- Fixing the dup-removal regressions (that's 211d)
- Cataloging `panic_any!` sites (that's 211c)
- Modifying any other panic-related code

## Constraints

- Workspace failure count must NOT increase vs baseline.
  - Pre-flight: capture baseline via `cargo test --release --workspace --no-fail-fast 2>&1 | tail -20`.
  - Post-ship: capture same summary; compare; report both in SCORE.
  - Recent baseline (bch3xjrp0): 4 targets failed (`-p wat --test test`, `--test wat_arc170_program_contracts`, `--test wat_run_sandboxed`, `-p wat-cli --test wat_cli`). Re-confirm before changes.
- Do NOT remove existing explicit `panic_hook::install()` calls; they live on as no-ops.
- Do NOT modify any other panic-related code.
- Use `std::sync::atomic::{AtomicBool, Ordering}` — already in std; no new deps beyond `ctor`.

## Implementation protocol

1. Pre-flight baseline: `cargo test --release --workspace --no-fail-fast 2>&1 | tail -20` → capture summary
2. Add `ctor` to `Cargo.toml` (workspace deps + wat-lib dep). Verify version compiles via `cargo build --release`.
3. Edit `src/panic_hook.rs` per Scope file touches above.
4. Create `tests/probe_panic_hook_auto_installed.rs` per "Probe test shape" below.
5. Run probe in isolation: `cargo test --release --test probe_panic_hook_auto_installed`
6. Run lib tests: `cargo test --release --lib panic_hook`
7. Run full workspace: `cargo test --release --workspace --no-fail-fast 2>&1 | tail -20` → capture summary
8. Write `SCORE-211A-CTOR-INSTALL.md` per scorecard below.

## Probe test shape

```rust
//! Proves `#[ctor]` auto-installs `panic_hook` at library load.
//! No explicit `panic_hook::install()` call in this test — if
//! `is_installed()` returns true, the ctor fired before `main()`.

#[test]
fn panic_hook_auto_installed_via_ctor() {
    assert!(
        wat::panic_hook::is_installed(),
        "panic_hook should be auto-installed via #[ctor] at library load \
         (no explicit install() call in this test)"
    );
}
```

That's the entire test. One assertion. Atomic. Verifies the load-bearing claim ("the ctor fires before any test code runs"). If the assertion holds, the gap is closed.

## Success criteria (the SCORE scorecard)

| # | Criterion | Verification |
|---|---|---|
| 1 | `ctor` dep added to `Cargo.toml` | `grep -n "^ctor\|\"ctor\"" Cargo.toml` |
| 2 | `INSTALLED: AtomicBool` static in `panic_hook.rs` | `grep -n "INSTALLED" src/panic_hook.rs` |
| 3 | `install()` short-circuits when already installed | Read the source diff; show the guard |
| 4 | `#[ctor::ctor] fn auto_install()` at module scope | `grep -n "ctor::ctor" src/panic_hook.rs` |
| 5 | `pub fn is_installed() -> bool` exists | `grep -n "fn is_installed" src/panic_hook.rs` |
| 6 | Probe test file exists | `ls tests/probe_panic_hook_auto_installed.rs` |
| 7 | Probe test passes | `cargo test --release --test probe_panic_hook_auto_installed` |
| 8 | Existing `panic_hook` lib tests still pass | `cargo test --release --lib panic_hook` |
| 9 | Workspace failure count not increased vs baseline | Compare before/after `tail -20` summaries |

## Time prediction
15–25 min Mode A. Small slice; well-scoped; minimal surface area.

## STOP triggers
Report and stop (do not work around) if:
- `ctor` crate's published API has changed such that `#[ctor::ctor]` isn't the attribute spelling
- A substrate-internal test depends on multiple stacked `panic_hook` installs (the idempotency guard would break it)
- Adding `ctor` causes a transitive dep conflict in `Cargo.lock`
- Workspace failure count INCREASES vs baseline (a new test fails that didn't before)

## Decay disclosure (orchestrator's hypotheses)
- The 4-target baseline comes from bch3xjrp0 task output (truncated; full details not captured). Sonnet should re-run baseline as step 1; if count differs, that's calibration data, not a 211a problem.
- The `INSTALLED.swap()` idempotency design assumes existing explicit install sites don't NEED to be the OUTER hook in the chain. Sonnet should verify nothing in the codebase relies on multiple-install stacking.
- Ordering choice (`SeqCst` vs `Acquire`/`Release`): `SeqCst` is the conservative default; sonnet may pick weaker if reasoned.

## Cross-references
- Arc 211 DESIGN § "Scope corrected 2026-05-18 (later)" — the locked four-sub-arc scope
- INTERSTITIAL § 2026-05-18 (later) "Panic-as-EDN doctrine + ctor-install discipline" — the full narrative
- `src/panic_hook.rs` — the existing tool this slice extends
- `feedback_substrate_owns_not_callers_match` — the doctrine driving 211a
