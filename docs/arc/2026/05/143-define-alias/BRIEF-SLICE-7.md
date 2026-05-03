# Arc 143 Slice 7 — Sonnet Brief — Apply :wat::list::reduce alias

**Drafted 2026-05-02 (late evening)** in parallel with slice 6 sweep.
Trivial slice; one application + two substrate call-site updates.

**The architectural framing:** slice 6 shipped
`:wat::runtime::define-alias` as a defmacro. This slice INVOKES it
to create the actual `:wat::list::reduce` alias for
`:wat::core::foldl`, fixing the arc 130 RELAND v1 stepping stone
that's been failing with "unknown function: :wat::core::reduce" for
days.

**Goal:** ship the alias + transition the arc 130 LRU stepping stone
from FAILED to PASSING (or at minimum, the failure mode CHANGES — the
substrate's CacheService.wat:213 + HologramCacheService.wat:251 call
sites now resolve `:wat::list::reduce`).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/143-define-alias/DESIGN.md`** — slice 7 plan.
2. **`docs/arc/2026/05/143-define-alias/SCORE-SLICE-6.md`** (read when
   shipped) — verify slice 6 shipped Mode A; the macro is registered
   and works.
3. **`wat/runtime.wat`** (slice 6 output) — confirm the
   `:wat::runtime::define-alias` macro is present.
4. **`crates/wat-lru/wat/lru/CacheService.wat:213`** — current
   substrate call site using `:wat::core::reduce`.
5. **`crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:251`**
   — same shape; mirrors the LRU.

## What to ship

### Piece 1 — Create `wat/list.wat` (NEW top-level file)

Header (mirror `wat/test.wat`'s style):

```scheme
;; wat/list.wat — :wat::list::* — list operations.
;;
;; Forward-looking namespace per arc 109's wind-down direction
;; ("we need to move things to :wat::list::* then we can mirror
;; that stuff for lazy seqs"). Currently houses one alias —
;; :wat::list::reduce → :wat::core::foldl — using arc 143's
;; :wat::runtime::define-alias macro.
;;
;; Future :wat::core::foldl → :wat::list::foldl rename in a
;; follow-on arc; this alias's TARGET updates without touching
;; the alias's NAME.

(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)
```

Total: ~15 LOC including header.

### Piece 2 — Register `wat/list.wat` in `src/stdlib.rs`

Add a registration entry alongside the existing entries (mirror
slice 6's `wat/runtime.wat` registration). The `list.wat` file must
load AFTER `wat/runtime.wat` (because `list.wat` USES
`:wat::runtime::define-alias` from `runtime.wat`).

### Piece 3 — Update arc 130 substrate call sites

Two files modify (one keyword each):

**`crates/wat-lru/wat/lru/CacheService.wat:213`:**
```scheme
;; OLD:
(:wat::core::reduce results 0
;; NEW:
(:wat::list::reduce results 0
```

**`crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:251`:**
```scheme
;; OLD:
(:wat::core::reduce results 0
;; NEW:
(:wat::list::reduce results 0
```

(Same change in both files; the surrounding context is identical
hit-counting logic.)

## Tests

NO new test file. The existing failing test
(`deftest_wat_lru_test_lru_raw_send_no_recv`) is the verification:

- BEFORE this slice: failing with "unknown function:
  `:wat::core::reduce`".
- AFTER this slice (Mode A): the test PASSES — the macro-emitted
  define resolves, CacheService Get path runs to completion, the
  test transitions to green.
- AFTER this slice (Mode B): the test fails with a DIFFERENT error
  message (no longer "unknown function: reduce"; some other reason
  surfaces). Still considered useful — surfaces the next gap in arc
  130's chain (probably reply-tx-disconnected or similar).

EITHER outcome is meaningful. The KEY metric: the "unknown function"
failure is GONE.

## Constraints

- **Files modified:** 1 NEW (`wat/list.wat`) + 1 modified
  (`src/stdlib.rs`) + 2 substrate call-site updates (CacheService.wat
  + HologramCacheService.wat).
- **No new tests.** The existing arc 130 stepping stone IS the test.
- **Workspace state:** before this slice runs, slice 6 must have
  shipped (Mode A). Verify by reading `wat/runtime.wat`'s presence
  and the slice 6 SCORE.
- **No commits, no pushes.**

## What success looks like

**Mode A:**
- `wat/list.wat` exists with the alias application.
- `src/stdlib.rs` registers it (in the right load order).
- Both CacheService.wat call sites updated to `:wat::list::reduce`.
- `cargo test --release --workspace` shows the
  `deftest_wat_lru_test_lru_raw_send_no_recv` test PASSES (or
  transitions to a different failure mode — see Mode B).

**Mode B (different failure):**
- The "unknown function" failure is GONE; macro resolution works.
- The test fails for a DIFFERENT reason (e.g., reply-tx-disconnected,
  another arc 130 stepping stone issue). Surface; still successful
  — the arc-143 chain held; arc 130 has its own continuing chain.

**Mode C (macro registration order issue):**
- `wat/list.wat` loads BEFORE `wat/runtime.wat`; the alias macro
  isn't registered yet when the application form runs. STOP +
  report; orchestrator re-orders the stdlib registration.

## Reporting back

Target ~150 words (slice is small):

1. **`wat/list.wat` content** verbatim.
2. **`src/stdlib.rs` change** — line numbers + the new entry +
   confirmation that load order is `runtime.wat` THEN `list.wat`.
3. **The two substrate call-site updates** — confirm both
   one-keyword changes applied.
4. **Test transition** — what does
   `deftest_wat_lru_test_lru_raw_send_no_recv` report now? Verbatim
   error message (or "ok" if Mode A).
5. **Test totals** — `cargo test --release --workspace` totals.
6. **Honest deltas** — anything you needed to invent or adapt.

## Sequencing

1. Read DESIGN.md + SCORE-SLICE-6.md + verify slice 6 shipped.
2. Read `wat/test.wat:1-20` for header style.
3. Create `wat/list.wat` with the alias application.
4. Update `src/stdlib.rs` with the registration (after
   `wat/runtime.wat`).
5. Update CacheService.wat:213 (`:wat::core::reduce` →
   `:wat::list::reduce`).
6. Update HologramCacheService.wat:251 (same).
7. Run `cargo test --release --workspace`.
8. Report.

Then DO NOT commit. Working tree stays modified for orchestrator
to score.

## Why this slice matters

This is the END of arc 143's substantive work. After slice 7:
- Arc 130's stepping stone unblocks.
- Arc 109 v1 closure depends on this chain completing.
- The reflection foundation is in active use.

Slice 8 is closure (INSCRIPTION + 058 row + cross-references).
Trivial paperwork after slice 7 ships.
