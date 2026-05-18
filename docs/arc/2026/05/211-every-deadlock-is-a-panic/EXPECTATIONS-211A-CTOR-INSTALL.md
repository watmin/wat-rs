# Arc 211a — EXPECTATIONS (orchestrator's independent prediction)

## Independent prediction

- **Runtime band:** 15–25 min Mode A. Small slice; well-scoped; minimal new code.
- **Lines changed:** ~30 LOC total
  - `Cargo.toml`: 2 lines (workspace + wat-lib dep entries)
  - `src/panic_hook.rs`: ~10 lines added (static + guard + ctor fn + is_installed)
  - `tests/probe_panic_hook_auto_installed.rs`: ~12 lines (one test)
- **New files:** 1 (`tests/probe_panic_hook_auto_installed.rs`)
- **Workspace failure delta:** 0 (or slightly fewer; this slice doesn't fix logic, only formats panic output)
- **Surprises expected:** 0–2 (small slice; contained surface area)

## Scorecard (mirror BRIEF Success Criteria)

| # | Criterion | Expected result |
|---|---|---|
| 1 | `ctor` dep added | YES; one of `ctor = "0.2"` or `ctor = "0.3"` |
| 2 | `INSTALLED: AtomicBool` static | YES; one static at module scope |
| 3 | `install()` short-circuits | YES; `if INSTALLED.swap(true, SeqCst) { return; }` at top of fn |
| 4 | `#[ctor::ctor] fn auto_install()` | YES; calls `install()` |
| 5 | `pub fn is_installed() -> bool` | YES; reads `INSTALLED` |
| 6 | Probe test file exists | YES |
| 7 | Probe test passes | YES |
| 8 | Existing lib tests pass | YES; idempotency guard doesn't break the test that exercises `install()` twice |
| 9 | Workspace failure count not increased | YES; sonnet captures pre + post summaries |

## Honest-delta watch (predicted surprises)

1. **`ctor` version pin** — orchestrator guessed 0.2.x or 0.3.x. Sonnet picks whichever resolves cleanly. If neither does, surface as STOP.

2. **Probe test shape** — orchestrator proposed `is_installed()` accessor + single assertion. Sonnet may shape differently if a cleaner proof exists (e.g., reading `take_hook` state directly). Either is fine if it verifies the load-bearing claim.

3. **Ordering on the AtomicBool** — orchestrator proposed `SeqCst`. Sonnet may pick `Acquire`/`Release` if reasoned. SeqCst is the conservative default; weaker is fine if argued.

4. **`#[ctor::ctor]` vs `ctor::ctor!`** — the crate provides both attribute and macro spellings depending on version. Sonnet uses whichever the version supports.

5. **Existing lib test interaction** — `src/panic_hook.rs:184-276` has 4 tests that call `write_assertion_failure` directly (not `install`). They should be unaffected. If any test calls `install()` and depends on stacking behavior, surface as STOP.

## Mode classification

- **Mode A:** ships per scope; all scorecard YES; no surprises beyond delta-watch.
- **Mode B:** ships with honest deltas (different ctor version, different probe shape, different Ordering) but workspace clean + all scorecard YES.
- **Mode B-time-violation:** ran >50 min (2× upper-bound). Investigate; the surface area shouldn't justify this.
- **Mode C:** substrate gap surfaced; sonnet stops + reports per STOP triggers in BRIEF.

## Calibration metadata

- Orchestrator's confidence: HIGH that this slice is correctly scoped + sized; ctor crate is mature; panic_hook surface is well-understood.
- Risk factors: low. Worst case is a transitive dep conflict from adding ctor.
- Why this is the first 211 sub-arc: it gives every subsequent sub-arc (211b/c/d) WORKING panic output to read; without it, 211c's audit is reading `Box<dyn Any>` placeholders.

## Post-completion orchestrator actions

1. Read sonnet's SCORE; verify each scorecard row independently
2. Re-run probe test locally: `cargo test --release --test probe_panic_hook_auto_installed`
3. Re-run workspace test: capture before/after summary lines
4. Commit atomically: BRIEF + EXPECTATIONS + sonnet's changes + SCORE
5. Push
6. Open BRIEF for 211b (panic-as-EDN) — the next stepping stone

## Cross-references

- BRIEF-211A-CTOR-INSTALL.md — the work definition
- Arc 211 DESIGN § "Scope corrected 2026-05-18 (later)" — the four-sub-arc scope
- INTERSTITIAL § 2026-05-18 (later) "Panic-as-EDN doctrine + ctor-install discipline" — origin story
