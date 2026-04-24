# Arc 045 — capacity-mode rename `:abort` → `:panic` + demo cleanup

**Opened:** 2026-04-24.
**Scope:** the `:abort` variant rename + demo sweep across
user-facing docs and examples. Substrate change.

## Why this arc exists

Builder's directive (verbatim):

> if we see the form (set-capacity-mode! :error) explicitly - it
> can now be omitted. if we see the form (set-capacity-mode! :abort)
> it should be kept and :abort swapped to :panic
>
> the default mode is :error - we don't need to state the default
> anymore

Two coupled changes:

1. **Substrate rename `:abort` → `:panic`.** Survey shows zero
   active callers outside test fixtures (in `wat/`, `wat-tests/`,
   `examples/`, `crates/`, lab repo proper). The `:panic` name
   matches the actual behavior — the variant triggers Rust's
   `panic!()` macro, which unwinds. `:abort` connotes
   `std::process::abort` (no unwinding) which would mislead a
   fresh reader. /gaze-discipline read says the identifier should
   match the thing.

2. **Demo cleanup `:error` line removal.** Since arc 037 made
   `:error` the default capacity-mode, demos showing
   `(set-capacity-mode! :error)` redundantly state the default.
   Per "the most-honest demo shows the user the minimum that
   works," those lines come out.

## What's broken

### `:abort` references (substrate rename targets)

`src/config.rs` (5 active sites):
- Line 446: `":abort" => Ok(CapacityMode::Abort)` — parse arm.
- Line 450: error message text "expected :error / :abort".
- Line 457: `expected: "keyword (:error / :abort)"`.
- Line 506: unit test `(set-capacity-mode! :abort)`.
- Lines 655, 667, 694: more unit-test fixtures.

`src/runtime.rs:4980`: panic message text mentioning `:abort`.

`tests/wat_bundle_capacity.rs:67, 155`: 2 test fixtures.

User-facing docs mentioning `:abort` by name (~6 sites across
README, USER-GUIDE, CONVENTIONS, docs/README, wat-tests/README).

### `:error` redundant demo lines

USER-GUIDE.md, README.md, CONVENTIONS.md, wat-tests/README.md,
wat/std/test.wat, examples/with-loader/wat-tests/* — every
demo showing `(:wat::config::set-capacity-mode! :error)` as the
preamble. Approximate count: ~10 sites.

## The discipline

- Targeted edits per file. No mechanical sweeps.
- `:abort` → `:panic` is a **rename**, not a deletion. Substrate
  test fixtures keep the same shape, just the keyword changes.
- `(set-capacity-mode! :error)` is a **deletion**. The line goes
  away; the surrounding scheme block stays.
- Verify after each slice: `cargo test --release` after slice 1;
  grep audit after slice 2.

## What stays unchanged

- **Arc INSCRIPTIONs / DESIGN / BACKLOG** (frozen historical
  records). Even the ones that mention `:abort` — they describe
  what the substrate was at slice-close. arc 045's INSCRIPTION
  records the rename; predecessors stay frozen.
- **Arc 005 INVENTORY.md** (preserved per builder's earlier call).
- **`scripts/arc_037_slice_5_sweep.py`** — frozen tool, ran once.
- **Test fixtures verifying that `:silent` / `:warn` are
  rejected** (arc 037 retired those; this arc doesn't touch
  them).

## Out of scope

- Lab repo (`holon-lab-trading/`). Survey showed zero current
  consumers there (the only `:abort` references are in archived/,
  BOOK chapters, FOUNDATION-CHANGELOG, arc records — all frozen
  historical or BOOK narrative). Lab will pick up the new name
  naturally if/when a future lab arc commits a capacity mode.

## What this arc proves

The `:panic` name is the *current* moment for this rename — zero
active downstream callers means zero migration cost beyond the
substrate edit + tests + docs. Renaming any later means more
callers to drag through. The discipline says rename when the
cost is cheapest; that's now.

The `:error` deletion is its own pedagogical move: demos should
show the minimum that works, not the redundant default-statement.
Arc 018 made the case for opinionated defaults; arc 045 finishes
applying it to the capacity-mode setter specifically.
