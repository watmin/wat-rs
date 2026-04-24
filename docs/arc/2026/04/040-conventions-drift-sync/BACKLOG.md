# Arc 040 — BACKLOG

Status markers:
- **ready** — region known, edits obvious.
- **obvious in shape** — clear once a prior slice lands.

Four slices grouped by topic, then INSCRIPTION.

---

## Slice 1 — §Sandbox Config inheritance (arc 037 set-dims! drop)

**Status: ready.**

Lines 352-382. One example with `(:wat::config::set-dims! 1024)`
to drop; prose mentioning "capacity-mode + dims" updated to
current shape (capacity-mode + optional set-dim-router! / sigma-fn
overrides).

## Slice 2 — `wat::test_suite!` → `wat::test!` (arc 018)

**Status: ready.**

7 occurrences across §External wat crates contract paragraph,
§Crate folder layouts (publishable + consumer templates),
§Three varieties of wat crate, §Viewing per-wat-test output, and
§Binary vs library. All `wat::test_suite!` → `wat::test!`. Plus
`tests/wat_suite.rs` → `tests/test.rs` in template paths.

## Slice 3 — Namespace table + load-file! syntax fix

**Status: ready.**

Lines 19-30 (§Namespaces table):
- `:wat::config::*` description: "noise floor, dimensions" →
  capacity-mode, dim-router, sigma functions (arc 024 + 037).
- `:wat::std::*` description: drop `LocalCache` (moved to
  `:wat::lru::*` via arc 036); `program::*` → `service::*`
  (renamed); add `service::Console` accurately.
- Add a `:wat::lru::*` row to the table (arc 036) — currently
  not enumerated.

Line 322: `(:wat::load-file! (path argument) "...")` →
`(:wat::load-file! "...")` (arc 028 simplified the form;
"(path argument)" reads as a leftover iface-keyword reference).

## Slice 4 — INSCRIPTION + cross-references

**Status: obvious in shape.**

- `INSCRIPTION.md` summarizing slices.
- `docs/README.md` arc index extended.
- 058 FOUNDATION-CHANGELOG row in lab repo.

---

## Cross-cutting

- Verification after each slice: grep audit + wc.
- Commit per slice. Push per commit.
