# Arc 039 — wat-rs/README.md drift sync

**Opened:** 2026-04-24.
**Status:** notes on disk; slices ready in obvious-in-shape order.
**Scope:** wat-rs/README.md only. One file, one arc.

## Why this arc exists

`wat-rs/README.md` is the crate-level entry doc — the first surface
a new reader sees. Its last touch was arc 036 (`47ea30a`,
2026-04-23 19:18) which only modified the wat-lru namespace
references. Many earlier arcs (022, 023, 024, 025, 026, 027, 028,
029, 030, 031, 032, 033, 034, 035, 037) shipped surface that the
README still describes wrong or doesn't describe at all.

**This is drift, not corruption.** Builder confirmed via GitHub
browse that the README is readable; just stale. No `5b5fad8`-class
recovery needed — standard targeted-edit forward sync per arc.

## What's broken

Surveyed via grep against the audit set:

- **`:wat::algebra::*` retired in arc 022** in favor of
  `:wat::holon::*`. README still uses the old namespace in 12
  places (`:wat::algebra::Atom`, `Bind`, `Bundle`, `presence?`,
  `cosine`, `CapacityExceeded`, etc.).
- **`set-dims!` retired in arc 037** in favor of `set-dim-router!`.
  README has 2 `(:wat::config::set-dims! 1024)` examples in §wat
  binary and §self-hosted testing.
- **`:silent` / `:warn` capacity modes retired in arc 037** — only
  `:error` / `:abort` remain. README's §Capacity guard documents
  all four modes as if current.
- **Test count** (731 Rust + 31 wat) is pre-arc-029. Has likely
  drifted; verify against `cargo test` output before pasting new
  numbers.
- **§Status section** lists 10 arcs as the most-recent landings
  (mid-March through 2026-04). Arcs 022-037 are unreflected as
  status updates; some need 1-line callouts in §Status, some need
  surface mentions elsewhere.

Plus narrower drift:
- Arc 028 root-hoist of load/eval forms (also affects example code
  blocks if any).
- Arc 029 `make-deftest` factory (affects testing examples).
- Arc 031 `deftest` signature drop (affects testing examples).
- Arc 032/033 typealiases (`BundleResult`, `Holons`).
- Arc 034 `ReciprocalLog` (stdlib mention).
- Arc 036 wat-lru namespace promotion (already partially reflected
  via 47ea30a; double-check no straggler `:user::wat::std::lru::*`
  paths).

## The discipline

Same as arc 038's recovery shape:

- **No mechanical sweeps.** `:wat::algebra::*` → `:wat::holon::*`
  is the biggest fix (12 sites); doing it as 12 individual `Edit`
  calls keeps the audit trail clean and avoids re-introducing the
  poison-class pattern.
- **One slice per section group.** Edits within a slice touch one
  cohesive region of the doc. A slice's commit can be reverted
  without dragging unrelated work.
- **Verify after each slice** — `wc -l`, header grep, spot-read.

## Out of scope

- Other docs in the audit set (`CONVENTIONS.md`, `wat-tests/README.md`,
  `INVENTORY.md`, `ZERO-MUTEX.md`, lab `CLAUDE.md`). Each gets its
  own arc per the builder's "checkpoint diligence" framing.
- Test count refresh — touched only if I can verify the actual
  number from `cargo test`; otherwise leave a note pointing at
  current counts in arc 037's INSCRIPTION + flag for a future
  audit when test counts naturally update.
- Restructuring §sections. The structure is sound; we extend.
- Re-deriving any arc's claims. INSCRIPTIONs are source of truth.
