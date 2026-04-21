# Arc 007 — wat tests wat — Backlog

**Opened:** 2026-04-20 (detour from arc 006).
**Scoped:** 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md).

---

## Tracking

| Slice | Item | Status | Commit |
|---|---|---|---|
| 1 | file-I/O audit — confirm all reads go through Loader | pending | — |
| 1 | `ScopedLoader` impl | pending | — |
| 1 | USER-GUIDE capability-boundary section | pending | — |
| 2 | `:wat::kernel::run-sandboxed` primitive | pending | — |
| 2 | `RunResult` struct registration | pending | — |
| 2 | panic-catch isolation | pending | — |
| 2 | spawned-threads drain-and-join before return | pending | — |
| 3 | `RuntimeError::AssertionFailed` variant + `catch_unwind` surfacing | pending | — |
| 3 | `:wat::test::assert-eq` | pending | — |
| 3 | `:wat::test::assert-contains` | pending | — |
| 3 | `:wat::test::assert-stdout-is` | pending | — |
| 3 | `:wat::test::assert-stderr-matches` | pending | — |
| 3 | `:wat::test::run` / `run-in-scope` wrappers | pending | — |
| 4 | `wat-vm test <path>` subcommand (discovery + runner) | pending | — |
| 4 | cargo-test-style report formatting | pending | — |
| 5 | `wat::Harness` Rust API | pending | — |
| 5 | crate-root re-export from `lib.rs` | pending | — |

---

## Decision log

- **2026-04-21** — Arc scoped. Five slices. Filesystem sandbox
  (slice 1) is the first move because Chapter 16 of BOOK.md
  flagged the capability bypass as future work; this arc closes
  that promise before sandboxed execution ships.
- **2026-04-21** — Main-signature for `run-sandboxed`: STRICT
  three-channel. Rejected "loose / accept any main signature"
  alternative because users who want no-channel main have
  `eval-edn!` already. Two paths, two concerns.
- **2026-04-21** — Assertion mechanism: PANIC-AND-CATCH.
  Rejected Result-return alternative because it taxes every
  assertion with match ceremony. Panic-and-catch requires one
  new `RuntimeError` variant + `catch_unwind` in run-sandboxed.
- **2026-04-21** — `:rust::*` capability allowlist: DEFERRED
  to its own arc. Slice 1 closes filesystem; network + process
  isolation is a bigger design surface. Documented in
  DESIGN.md's out-of-scope section.
- **2026-04-21** — Parallel test execution: DEFERRED. V1 of
  `wat-vm test` runs serial. Parallelism is a follow-up once
  usage patterns expose which tests can safely run
  concurrently.

---

## Relationship to arc 006

Arc 006 paused at `with-state` (slice 4 of arc 006) pending
this detour. Once arc 007 ships slice 3, `with-state` can be
implemented AND TESTED in wat — the first substantive use of
the self-testing harness. That test file is the proof point
for this arc.
