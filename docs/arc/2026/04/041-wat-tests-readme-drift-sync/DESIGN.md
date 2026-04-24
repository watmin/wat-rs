# Arc 041 — wat-rs/wat-tests/README.md drift sync

**Opened:** 2026-04-24.
**Status:** notes on disk; one implementation slice + INSCRIPTION.
**Scope:** wat-rs/wat-tests/README.md only. One file, one arc.

## Why this arc exists

`wat-rs/wat-tests/README.md` is the test-tree onboarding doc for
wat-rs's own self-hosted tests. 70 lines. Last touched at commit
`5b5fad8` (the same arc 028 doc-sweep that corrupted USER-GUIDE.md;
this file was confirmed not poisoned, just stale).

**Drift only.** Same shape as arcs 039 + 040 — drift forward via
targeted edits per topic.

## What's broken

Surveyed via grep + full read (the file is small enough):

- **§Layout** describes wat-rs's own `wat-tests/` tree (arc 022
  layout — `wat/holon/*` ↔ `wat-tests/holon/*`). Doesn't mention
  that consumer crates (including the in-workspace `crates/wat-lru/`)
  ship their own `wat-tests/` trees and run via per-crate
  `cargo test -p <crate>` (arcs 013 + 015 + 036). A reader who
  only sees wat-rs's wat-tests/ thinks that's the only place wat
  tests live.
- **§In-process vs hermetic** describes the substrate primitives
  (`:wat::test::run`, `:wat::test::run-hermetic-ast`) but doesn't
  mention the user-facing macros (`deftest`, `deftest-hermetic`,
  `make-deftest`) that arc 029 + 031 made canonical. Most readers
  reach for `deftest` first and only encounter the substrate
  primitives if they need fork-based isolation.
- **No retired-form occurrences** in the standard audit set
  (`set-dims!`, `:wat::algebra::*`, `:wat::core::load!`,
  `wat::test_suite!`, etc.). Arc 022 + arc 018 + arc 037 surface
  changes are already reflected.

The gap is "what's missing," not "what's wrong" — small additions,
not corrections.

## Out of scope

- Other audit-set docs (`INVENTORY.md`, `ZERO-MUTEX.md`, lab
  `CLAUDE.md`).
- Rewriting sections wholesale. The shape is sound; we extend.

## Why this is honest now

A reader new to wat-rs lands on `wat-tests/README.md` to learn
how the language tests itself. The current text correctly
documents wat-rs's internal pattern but stops there. Naming the
external-crate variant explicitly tells the reader that the
pattern they're seeing is a portable one — `crates/wat-lru/`
demonstrates the same shape against a third-party stdlib
addition. That generality is what arc 013 + 015 + 036 shipped;
this doc should reflect it.
