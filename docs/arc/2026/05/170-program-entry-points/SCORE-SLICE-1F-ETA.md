# Arc 170 slice 1f-η — SCORE

**Result:** Mode A clean. 8/8 rows pass.
**Runtime:** ~10.5 min opus (well under predicted 90-180 band; well under 360 hard cap).
**Files:** 5 deleted + 7 modified — net **-1336 lines**.

**Workspace: 1752/461 → 1752/451.** Pass count unchanged; failure count dropped by 10 (deleted Console deftests). The trio + orchestrator now exclusively own the stdio contract per TIERS.md doctrine.

## Calibration

- **Predicted runtime band:** 90-180 min opus (substantive cross-substrate work)
- **Actual:** ~10.5 min — 9-17× under
- **Why dramatically faster:** Opus correctly identified that most Console usage was deletable rather than migratable (test files for retired subsystem behavior). The telemetry crate scope was as anticipated. The migration target (`ambient :wat::kernel::println`) was already in place; consumers either delete-or-rewrite cleanly.
- **Calibration lesson:** Retirement slices with full-substitute available + delete-not-migrate option go fast. Future similar slices (e.g., wat_telemetry-Console-equivalent retirements) can predict ≤30 min.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `wat/console.wat` removed from `src/stdlib.rs` | ✓ entry retired with breadcrumb comment |
| B | 0 references to `:wat::console::` in test trees | ✓ grep returns empty |
| C | `cargo check --release` green | ✓ clean (1 pre-existing dead_code warning) |
| D | Workspace failure count doesn't regress (461 floor) | ✓ 451 — 10 BELOW floor |
| E | Pass count may decrease via deleted Console deftests; failure doesn't rise | ✓ pass count unchanged at 1752; failures fell 10 |
| F | Substrate dispatch arms for `:wat::console::*` retired | ✓ `BareLegacyConsolePath` variant + validator + 4 call sites all removed from `src/check.rs` |
| G | Honest deltas surfaced | ✓ 5 categories — including major architectural observation |
| H | Deletions documented | ✓ 5 files; each named with line count |

**8/8 rows pass.** Mode A clean.

## § Architectural observation — ambient stdio is EDN-only (locked)

The Console retirement closed the migration loop on TIERS.md's "trio + orchestrator own stdio" doctrine cleanly. **One downstream consequence surfaced via the example migration:**

The previous Console-mediated surface allowed user code to write **arbitrary `String` lines**:

```
(:wat::console::Console/out console "any string user wants")
```

The ambient surface **always serializes the argument through `wat_edn::write`**:

```
(:wat::kernel::println some-value)  ;; output is wat_edn::write(some-value)
```

This is the right contract for parseable cross-boundary I/O — but it's a real capability change. Apps wanting alternate formats (JSON, custom rendering, mixed structured/unstructured) need to either:

- **(a)** Compose at the value layer — build a tagged struct whose EDN encoding IS the desired format
- **(b)** Write a user-side stdio service driver bypassing ambient ops

The retired `ConsoleLogger`'s format-selection menu (Edn / Json / Pretty / NoTagEdn / NoTagJson) has no current equivalent in the ambient surface — **by design**. Surfacing this so future readers don't interpret it as a regression.

**For the console-demo example** specifically: format-selection-at-call-site (the demo's pedagogical core) is gone. Opus rewrote the demo as an ambient-stdio showcase (structured value emit, automatic EDN encoding, stdout/stderr routing via op choice). Smoke-tested: stdout shows 3 EDN-encoded events; stderr shows 2; split-stream behavior preserved.

## Files deleted (5)

| Path | Lines | Reason |
|---|---|---|
| `wat/console.wat` | 298 | Console driver + handle plumbing |
| `wat-tests/console.wat` | 311 | Console-specific test suite (multi-writer, hermetic stdio round-trip) |
| `crates/wat-telemetry/wat/telemetry/Console.wat` | 98 | dispatcher factory wrapping Console driver |
| `crates/wat-telemetry/wat/telemetry/ConsoleLogger.wat` | 154 | typed structured logger on Console |
| `crates/wat-telemetry/wat-tests/telemetry/Console.wat` | 202 | telemetry dispatcher tests |

**Total: 1063 lines retired.** All deletions are forward progress, not erasure — the behaviors tested in those files are now substrate-direct via the trio + orchestrator.

## Files modified (7)

- `src/stdlib.rs` — retire `wat/console.wat` `WatSource` entry; explanatory breadcrumb
- `src/check.rs` — retire `BareLegacyConsolePath` variant + Display + diagnostic + validator + 4 call sites (~85 lines)
- `crates/wat-telemetry/src/lib.rs` — drop Console.wat + ConsoleLogger.wat from `wat_sources()`
- `examples/console-demo/Cargo.toml` — drop `wat-telemetry` dep; update description
- `examples/console-demo/src/main.rs` — strip `deps: [wat_telemetry]`; update doc
- `examples/console-demo/wat/main.wat` — full rewrite as ambient-stdio demo (see § Architecture observation)
- `tests/wat_tco.rs` — comment-only update (`Console/loop` → `driver-loop` in two places)
- `wat/kernel/services/{stdout,stderr,stdin}.wat` — comment-only update (stale `wat/console.wat:38` cross-references → generic Pattern A description)

## Honest deltas (5 categories)

1. **Telemetry crate scope as anticipated** — Console.wat + ConsoleLogger.wat were the only Console-tied files; non-Console-tied consumers unaffected. WorkUnitLog.wat references ConsoleLogger only in producer-side mirror-pattern comments; those left intact (the pattern is still valid).

2. **Example rewrite, not deletion** — original `console-demo` demonstrated **format-selection at the call site** (Edn/Json/Pretty/NoTagEdn/NoTagJson side-by-side, all Console-handle-mediated). Ambient ops are EDN-only — see § Architecture observation. Rewrote to showcase ambient stdio's actual surface; smoke-tested.

3. **Substrate legacy-console check arms retired** — `BareLegacyConsolePath` was an arc-109-era migration warning that pointed users from `:wat::std::service::Console::*` → `:wat::console::*`. With the canonical target now also retired, the warning would point at a non-existent path. Full retirement.

4. **wat_tco.rs Console/loop comments** — two doc comments referenced `Console/loop` rhetorically (a now-deleted file). Renamed to `driver-loop` to preserve pedagogical intent without dead reference. Test bodies unchanged.

5. **Three new stdio-service files had stale `wat/console.wat:38` cross-references** in typealias-naming-convention comments. Updated to describe Pattern A generically (no file-path breadcrumb). Cosmetic; no contract change.

## Implementation choices (locked)

- **Console.wat retired entirely** (deleted, not bridged) — full-substitute available; no transitional bridge needed
- **Console-test-files deleted** — testing retired-subsystem behavior is correctly deleted, not migrated
- **Example rewrote to ambient** — preserves the demo's role as a pedagogical artifact
- **EDN-only ambient surface accepted** — format-selection capability migration to value-layer composition or user-side driver if a caller surfaces demand

## Lessons captured

1. **Retirement-with-substitute is fast.** Opus 10.5 min vs predicted 90-180 — substantive cross-substrate work, but every consumer either deleted cleanly or migrated to an already-shipped substitute. Future retirement slices with full substitutes can predict aggressively.

2. **Capability consequences surface via examples.** The ambient EDN-only contract was implicit in slice 1f-α's `:wat::kernel::println` signature. The console-demo migration is what made it concrete. Architectural consequences sometimes only surface in the migration's last mile — surfacing them in the SCORE makes the architecture honest.

3. **Architecture observations belong in the SCORE.** When a slice surfaces a non-regression that future readers might misinterpret, the SCORE is the right place. This isn't deferral language (per FM 11) — it's affirmative recognition of a contract choice.

## What's next

1. **Atomic-commit slice 1f-η** (this turn) — 12 files + this SCORE; push to GitHub
2. **Resume slice 1f-ζ** — `:user::main` migration continuation. Now without Console-backed plumbing complications; sonnet's flailing on Harness::run() stdio capture should clear (the test infrastructure no longer routes through Console).
3. **Sibling slice — restore retired `spawn-program` / `fork-program-ast`** (BareLegacy* diagnostics for arc112/103/104 tests)
4. **Fork waitpid follow-up** — close the orphan process leak in `src/fork.rs`
5. **Heterogeneous-tail triage** — substantive test-body issues case-by-case
6. **Bridge-migration slice** — move `run-sandboxed-*` body to Layer 1 (end-state cleanup)
7. **Arc 170 INSCRIPTION** — once baseline stabilizes

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-ETA.md`](./BRIEF-SLICE-1F-ETA.md)
- Predecessors: slices 1f-β-i/ii/iii (trio of substrate services); 1f-γ (orchestrator) — the substrate work that made this retirement possible
- Slice 1e — retired the `:user::main` four-arg signature; this slice closes a parallel retirement
- TIERS.md § OS-boundary handling — locked architecture this slice executes
- `feedback_capability_carrier.md` — pattern the orchestrator uses; same discipline applies to Console retirement (no fallback singleton)
