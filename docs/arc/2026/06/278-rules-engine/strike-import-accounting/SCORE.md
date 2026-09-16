# SCORE — A7, weighed against the orchestrator's own re-run. **CLASS A IS CLOSED.**

> Re-run here at `b0e3377e9`.

## The scorecard, re-run

| # | pre-value at HEAD | after |
|---|---|---|
| 1 | grep → **0** | ✅ both calls present |
| 2 | marked **2097268** / unmarked **0** | ✅ a real 7-node import allocates 15,172 bytes, refused at an 8192 ceiling, imports and derives `1` at the 1 GiB default |
| 3 | 1.05 / 1.95 / 2.57 / 4.87 µs per pair | ✅ **unchanged — `pmap.rs` absent from the diff.** This row is the before-curve for a later speed stone |
| 4 | — | ✅ refused with `malformed`, naming the cap and the count |
| 5 | — | ✅ **corpus max 63**, cap 10 000, ~122 ms worst case, all three at the constant |
| 6 | — | ✅ `or_insert`, never `insert` |
| 7 | header said **five** walls | ✅ six — **and the phase list too** (see thin spot B) |
| 8 | — | ✅ `export.rs` + `alloc_counter.rs` + four probe files. `pmap.rs` absent |
| 9 | lint 116/116 | ✅ 116/116 |
| 10 | floor 5213/5213 | ✅ `Summary [ 420.115s] 5216 tests run: 5216 passed (5 slow), 21 skipped`, zero FAIL rows |
| 11 | clippy rc=0 | ✅ rc=0 |

**The cap is measured, not chosen.** 63 nodes is the corpus maximum across 434 tests, 34 of which
reach an import (distribution 6, 7×17, 9×9, 13, 14, 28, 63×4; the 63 attributed to the datamancer
program by a filtered re-run). The cap is ~158× that, and `1.217e-3 · N² µs` fitted to the driven
4 000-pair point puts its worst case at ~122 ms.

## ⛔⛔ THE FINDING IS AGAINST MY OWN EARLIER STRIKE, AND I RE-DROVE IT

Mutation 3 — `or_insert` → `insert`, removing A4's non-clobber rule:

```
FAIL  probe_arc278_import_accounting::an_origin_already_filed_is_never_re_based   ← the new probe
PASS  probe_arc278_fixpoint_round_cap::a_second_session_..._does_not_forgive_...  ← A4's own arm
```

A4's `rearm` arm carries this, verbatim, in its fixture:

> *"Measured with `or_insert` replaced by `insert`: control REFUSED · probe REFUSED · rearm
> NO-BREACH ← **only this arm can see it**."*

**It cannot.** The mask is `LAST_ORIGIN`, the one-entry cache in front of `SESSION_ORIGINS`, never
invalidated on a write: the first staging round caches `(key, origin0)`, the re-arm's clobber
rewrites the map but not the cache, and every later `session_bytes` takes the stale fast path. The
rider proved the mechanism rather than asserting it — adding an invalidation to the mutated write
flipped A4's test green→red in the same command as its own.

`git show 42704d57b` — **the cache and the arm landed in the same commit.** The measurement was
taken before the cache existed beside it and was never re-taken. It was false the day it shipped,
in a strike I orchestrated, under a doctrine file whose header opens with a warning about exactly
this shape: *a claim that states its own delivery and dates it is one nobody re-checks.*

**The code is correct** — the cache is sound *given* `or_insert`. What was broken was the proof.
Struck at the site with the driven correction and a pointer to the live gate; the arm still earns
its place for the **keying** half, which is what its `#[test]` doc actually claims. Promoted to
memory.

## ⛔ Where MY brief was thin — seven, and two would have under-delivered the fix

- **A. ★ My sketch's capture point under-charges.** I wrote `let origin_before = thread_bytes();
  // BEFORE the build` — but placed where the sketch implies, it sits after the compat gates and
  after `expect_seq` has cloned the entire nodes vector, excluding that clone and every
  `unpack_node` from the charge. The rider made it the function's **first statement**. My comment
  said "before the build"; the defect is about everything the *door* allocates.
- **B. ★ Trap 5 names ONE copy of the ordering and there are two.** The module header says five
  walls; `import_export`'s own doc-comment carries a numbered 1–7 phase list of the same sequence.
  Updating only what I named leaves the phase list silently wrong — **in the same file, and it is
  the same shape A6 found in `unpack_driver`**. Renumbered to 1–9.
- **C. My mutation 1 is misdescribed.** "Move `mark_session_origin_at` to *after* the build" — in
  the correct version the filing *is* after the build; it has to be, the key does not exist earlier.
  The mutation is in the **argument**, not the placement. As written it implies a code motion with
  no correct counterpart.
- **D. Row 2's probe as stated is unwritable after the fix.** "Mark one key, allocate, read both"
  has no post-fix analogue at the import door: a session that outgrows the ceiling never comes back,
  so there is nothing to read a byte count from. The observable is the refusal and the figure inside
  it.
- **E. ★ A release-weighed floor deletes allocation-only probes.** Arm 3's 1 MiB ballast was elided
  by LLVM — `thread_bytes()` read **121** — and the probe failed for a reason unrelated to its
  subject. Any probe whose subject is an allocation needs `black_box`. Noted at the site. **My own
  recon hit this and I did not warn about it.**
- **F. `expect_seq` clones the whole nodes vector before the cap can see it.** The cap bounds the
  quadratic build; it does not bound that one linear copy, so a hostile 10M-element `nodes` still
  pays one memcpy before refusal. Out of scope, recorded so the cap's guarantee is not read as
  broader than it is.
- **G. `SESSION_ORIGINS` entries are never removed** and `next_intern()` is monotonic, so each
  import leaks one map entry per session for the thread's life. Pre-existing with A4, unbounded on a
  long-lived importing thread. Recorded, not acted on.

## Arms not driven, named

None among the three; each was proven by a mutation with a **predicted** red set, and each observed
set matched exactly (mutations 1 and 2 reddened exactly one test apiece, 436/437). Mutation 3's
predicted set was two tests and **only one reddened** — which is the finding above, reported by the
rider as a miss rather than reconciled away.
