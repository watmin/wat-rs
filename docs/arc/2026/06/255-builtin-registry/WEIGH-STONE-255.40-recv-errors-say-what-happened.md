# WEIGH — STONE 255.40: the Rust recv errors say what happened — ACCEPTED

**Executor: grok via pulsare, commit `d88103eb5`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T21-17-45Z` | `6132 tests run: 6132 passed (10 slow), 22 skipped` |
| clippy, fresh | `touch src/lib.rs; cargo clippy --release --all-targets -- -D warnings` | rc=0 |
| the IDE's `RejectsWire` has no `Debug`, and the unused `IntoRawFd` | read HEAD | **stale**: `#[derive(Debug)] struct RejectsWire` (`src/comms/process.rs:2154`); the new tests import no `IntoRawFd` |

Taken from the SCORE: census `no STOP-8` with 0 rc flips; delta NEW 2 / RECOVERY 0; ledger 198.

## What landed

- **C3:** `RecvError::Malformed(String)` is split from `Failed`. **Each builder is told apart by the value
  it already holds** (a `Utf8Error`, a `WireError`, `FrameScan::Malformed`), never by reading a string.
  `Failed` is io only. Every consumer keeps building the wat variant it built before, so the split is
  invisible to wat until the recv vocabulary stone uses it.
- **C4:** `classify_peer_error`'s end-of-file arm now **calls** `classify_peer_death`: one classification,
  and they cannot drift apart again. A crash-channel io failure is `Lost`, **no longer a clean `Closed`**.

## ⚠ The orchestrator's brief contradicted itself

It asked C4 to stop reporting a crash-channel io failure as `Closed`, **and** set STOP-2 on "any
wat-visible outcome changes". C4 cannot hold without one wat value moving: that case now builds
`RecvOutcome.Lost`/`ServiceEvent.Lost` where it built `Closed`. **The executor did C4 and disclosed the
change plainly** rather than stopping on a contradiction the brief created. The variant set did not
grow, and the census shows 0 flips. **Accepted: the brief was wrong, not the strike.**
