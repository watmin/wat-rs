# Arc 036 — wat-lru namespace promotion — BACKLOG

**Shape:** two slices. Zero substrate changes expected (the
bypass mechanism is already in place per freeze.rs:362-368).

---

## Slice 1 — wat-lru source rename

**Status: ready.**

Rename across the wat-lru crate:

- `crates/wat-lru/wat/LocalCache.wat` — define paths
- `crates/wat-lru/wat/CacheService.wat` — define paths + the
  service-loop macro reference
- `crates/wat-lru/wat-tests/LocalCache.wat` — test body calls
- `crates/wat-lru/wat-tests/CacheService.wat` — test body calls
- `crates/wat-lru/src/lib.rs` — any doc-comments or re-exports
  referencing the old keyword paths
- `crates/wat-lru/Cargo.toml` — description/keywords if any
  reference the path

Current-state consumer touch points:

- `examples/with-lru/wat/main.wat` — 5 occurrences in the
  example body + comments
- `src/test_runner.rs` — 1 doc comment

**Sub-fogs:**
- **1a — CacheService macro expansion.** `CacheService` is a
  defmacro that expands into a loop body referring to
  `LocalCache` methods. Macro body needs full-qualified path
  rewrite alongside the declaration.
- **1b — test import patterns.** wat-tests use
  `(:wat::load-file! "wat/LocalCache.wat")` relative to crate
  root — the load path is filesystem, unaffected by keyword
  namespace. No action.

## Slice 2 — doc sweep + CONVENTIONS amendment

**Status: obvious in shape** (once slice 1 lands).

- `docs/CONVENTIONS.md` — the `:wat::*` row in the namespace
  table gains workspace-member crates alongside baked stdlib;
  the `:user::*` row updates to "user program code +
  third-party community crates (non-workspace)." Add a
  sub-section or note explaining the rule.
- `README.md` — 2 occurrences in the shipped Caches section.
- `docs/README.md` — 1 occurrence in the arc-013 entry's
  forward-reference.
- Historical arc directories (013 / 015) stay untouched per
  the INSCRIPTION-is-frozen-history discipline.

Tests:
- `cargo test --workspace` — every wat-lru test + the
  with-lru example's smoke test re-runs with new paths.
- `cargo clippy --workspace --all-targets -- -D warnings` —
  stays green (arc 035's recovery holds).

## Slice 3 — INSCRIPTION + 058 CHANGELOG row

**Status: obvious in shape** (once slices 1 + 2 land).

- `docs/arc/2026/04/036-wat-lru-namespace-promotion/INSCRIPTION.md`
- 058 FOUNDATION-CHANGELOG row (lab repo) for the
  cross-reference — this changes the namespace contract
  consumers compose against.

---

## Working notes

- Cave-quested 2026-04-23, straight after arc 035.
- This is the first arc where a workspace-member crate other
  than the wat-rs root registers under `:wat::*`. Sets the
  precedent for future `:wat::sqlite::*`, `:wat::redis::*`
  crates if/when the workspace grows.
- Narrow-first, broad-after sweep pattern per arc 004's
  lesson: one file at a time via targeted Edit calls; no
  perl with `|` alternation, no sed, no regex sweeps
  (Chapter 32's poison pattern stays avoided).
