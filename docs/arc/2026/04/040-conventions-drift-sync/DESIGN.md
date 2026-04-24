# Arc 040 — wat-rs/docs/CONVENTIONS.md drift sync

**Opened:** 2026-04-24.
**Status:** notes on disk; slices ready.
**Scope:** wat-rs/docs/CONVENTIONS.md only. One file, one arc.

## Why this arc exists

`wat-rs/docs/CONVENTIONS.md` is the rule-book — naming
conventions, namespace claims, crate layouts, install discipline.
Last touched at arc 036 (`47ea30a`, 2026-04-23) for the wat-lru
namespace promotion, but earlier arcs that affect rules and
templates are partially or wholly unreflected.

**Drift, not corruption.** The file is readable; just stale. Same
shape as arc 039.

## What's broken

Surveyed via grep + spot-read:

- **`wat::test_suite!` references** — arc 018 renamed the macro
  to `wat::test!`. 7 occurrences in CONVENTIONS still cite
  `test_suite!` in template paragraphs and example code.
- **`tests/wat_suite.rs` filename** — arc 018's rename also
  shifted the conventional test entry filename to `tests/test.rs`
  (matches `wat::test!`). 2 templates in CONVENTIONS still show
  `tests/wat_suite.rs`.
- **`set-dims!` example** — arc 037 retired `:wat::config::set-dims!`
  in favor of `set-dim-router!`. CONVENTIONS' §Sandbox Config
  inheritance has one example still showing `(set-dims! 1024)`.
- **§Sandbox Config inheritance prose** — describes commits as
  "capacity-mode + dims" — both `dims` and the framing are
  pre-arc-037.
- **§Namespace table** — `:wat::config::*` description names
  "noise floor, dimensions" as committed values; under arc 037,
  `dim-router` (function) replaces scalar `dims`; sigma functions
  joined the surface (arc 024). `:wat::std::*` description names
  `LocalCache, stream::*, program::Console, program::Cache` —
  but `LocalCache` moved to wat-lru via arc 013 + 036
  (`:wat::lru::*`), and `program::*` was renamed `service::*`
  (BOOK chapter 21).
- **§Binary vs library `(:wat::load-file! (path argument) "...")`**
  — odd phrasing from a pre-arc-028 transition; the actual form
  is just `(:wat::load-file! "...")`.

## Out of scope

- Other audit-set docs (`wat-tests/README.md`, `INVENTORY.md`,
  `ZERO-MUTEX.md`, lab `CLAUDE.md`). Each gets its own arc.
- Re-deriving claims. INSCRIPTIONs are source of truth.
- Restructuring sections. The structure is sound.
