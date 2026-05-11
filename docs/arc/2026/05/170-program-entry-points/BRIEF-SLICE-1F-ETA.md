# Arc 170 slice 1f-η — BRIEF (Console retirement)

**Opus.** Retire the wat-side Console subsystem and migrate all consumers to the ambient stdio services shipped in slice 1f-γ. Console was the pre-orchestrator stdio gateway; the trio (StdIn/StdOut/StdErr) + orchestrator now own that contract per TIERS.md doctrine.

**Direction context:** This slice was triggered by slice 1f-ζ's sonnet flailing on Harness::run() stdio capture issues — those issues are downstream of Console still being plumbed-in. Console must retire before the `:user::main` migration continuation (1f-ζ) can resume cleanly.

## Slice surface

> *"Retire wat/console.wat; migrate 8 consumer files to ambient stdio."*

Per user direction 2026-05-10 — arc 170's slices grow as necessary; this is the Console retirement that was always going to happen.

## Scope

### Edit 1 — migrate consumers (~8 files)

Identified consumer files:
- `wat-tests/console.wat` — tests Console itself; **delete** (the tested behavior is now substrate-direct via kernel::println)
- `tests/wat_tco.rs` — uses Console somewhere; migrate
- `examples/console-demo/wat/main.wat` — example demonstrating Console; migrate or delete
- `crates/wat-telemetry/wat/telemetry/ConsoleLogger.wat` — telemetry crate's Console-backed logger
- `crates/wat-telemetry/wat-tests/telemetry/Console.wat` — telemetry tests
- `crates/wat-telemetry/wat/telemetry/Console.wat` — duplicated path?
- `crates/wat-telemetry/src/lib.rs` — Rust-side telemetry Console refs

Migration target — replace Console-mediated stdio with ambient `:wat::kernel::*`:
- `(:wat::console::Console/out console "foo")` → `(:wat::kernel::println "foo")`
- `(:wat::console::Console/err console "bar")` → `(:wat::kernel::eprintln "bar")`
- Drop Console handle plumbing (typealiases, channels, driver spawn)

For files that test Console SPECIFICALLY (the wat-tests/console.wat suite) — **delete** rather than migrate. The Console-specific behaviors (mini-TCP discipline, ack channels, driver loop semantics) are no longer the contract; the trio + orchestrator carry that doctrine now.

### Edit 2 — retire `wat/console.wat` from stdlib

Remove the entry at `src/stdlib.rs:178-179`. Drop `wat/console.wat` from the project (or move to archive per user-direction; pick the cleaner option).

### Edit 3 — retire substrate Rust dispatch arms (if any)

Search for `:wat::console::*` keyword paths in `src/runtime.rs` + `src/check.rs`:
```
grep -n '":wat::console::' src/runtime.rs src/check.rs
```

If any exist as eval arms or type-check registrations, retire them (the wat-side definitions go away; substrate arms become unreachable).

## Pre-flight verification

- Trio of stdio services: shipped at slices 1f-β-i/ii/iii (`e898c7a` / `fe9b9e9` / `52319ba`) ✓
- Runtime orchestrator: shipped at slice 1f-γ (`1c083d0`) ✓
- Ambient `:wat::kernel::println` / `eprintln` / `readln`: shipped at slice 1f-α via `src/thread_io.rs` ✓
- `invoke_user_main_orchestrated` spawns services automatically: shipped at slice 1f-γ ✓

All migration targets are in place. The retirement can proceed without substrate gaps.

## What to NOT do

- Don't touch the `:user::main` 3-arg migration continuation (slice 1f-ζ — separate slice)
- Don't restore retired `spawn-program` / `fork-program-ast` (separate sibling slice)
- Don't fix heterogeneous-tail test assertion bugs (separate triage)
- Don't fix fork-process waitpid leak (separate)
- Don't commit yourself — orchestrator atomic-commits with SCORE

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `wat/console.wat` removed from `src/stdlib.rs` | grep |
| B | 0 references to `:wat::console::` in wat-tests/, tests/, examples/, crates/ (excluding archive) | grep |
| C | `cargo check --release` green | clean |
| D | Workspace failure count does not regress (461 floor) | cargo test count |
| E | Some Console-test-binaries (e.g., `wat-tests/console.wat` deftests) may shrink in test count if files are deleted — pass count may DECREASE by the deleted-test count, but failure count should NOT rise | cargo test count |
| F | Substrate dispatch arms for `:wat::console::*` retired (if any) | grep |
| G | Honest deltas surfaced | per FM 5 |
| H | Files deleted are documented in SCORE (per "what is inscribed is inscribed" — deletions are forward progress, not erasure) | inline |

**8 rows.**

## Predicted runtime

**90-180 min opus.** Migration is substantive (each Console caller needs body rewrite); telemetry crate is the biggest chunk. Mechanical replacement pattern but spans Rust + wat boundaries.

**Hard cap:** 360 min.

## Honest-delta categories (anticipated)

1. **Telemetry crate scope** — `crates/wat-telemetry/` may have deeper Console integration than the file count suggests. If substantive enough, surface and we narrow this slice + open a sibling.
2. **`wat-tests/console.wat` test deletions** — pass count may decrease by the number of Console-specific deftests removed. This is expected (tests for retired subsystem). Document count.
3. **Example file** — `examples/console-demo/wat/main.wat` may demonstrate Console specifically. Migrate to demonstrate the new ambient API instead, OR delete + replace with a new ambient-stdio demo. Pick the cleaner option; surface choice.
4. **Substrate dispatch arms** — if `:wat::console::*` has eval arms (verb routing in runtime.rs), retiring them may affect tests we haven't anticipated. Surface.

## Reference

- Slice 1f-γ (`1c083d0`) — runtime orchestrator that replaced Console's role
- Slices 1f-β-i/ii/iii — trio of substrate services that Console fronted
- TIERS.md § OS-boundary handling — locked architecture; Console doesn't appear in the canonical model
- `wat/console.wat` — current Console.wat to retire

## Path forward post-slice-1f-η

1. Orchestrator scores; atomic-commits deliverable + SCORE
2. **Resume slice 1f-ζ** — `:user::main` migration continuation (now without Console-backed plumbing complications)
3. **Sibling slice — restore retired `spawn-program` / `fork-program-ast`** (bridge pattern)
4. **Fork waitpid follow-up**
5. **Heterogeneous-tail triage**
6. **Arc 170 INSCRIPTION** — once baseline stabilizes
