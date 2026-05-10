# Arc 170 slice 1f-0a — SCORE

**Result:** Mode B — partial. Structural fix correct + non-
regressive; predicted fail-count drop didn't materialize because
the 855-failure baseline has multiple independent root causes.
The macro signature change shipped clean and lays correct
foundation; the remaining rot needs sibling slices.

**Runtime:** ~18 min sonnet (within 15-30 predicted band).
**Files:** 1 modified (`wat/test.wat`, 2 insertions / 10 deletions).

## Calibration

- **Predicted runtime band:** 15-30 min sonnet (60 min hard cap)
- **Actual:** ~18 min — within band
- **Pattern fit:** verbatim edit per BRIEF; no judgment required
  on the edit itself
- **Diagnostic value:** sonnet identified three independent
  rot sources for the 855-failure baseline (which the slice
  cannot all fix; they each warrant their own slice)

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A — `:wat::test::deftest` emits `(:user::main -> :wat::core::nil)` | ✓ grep confirms line 315 + old 3-param shape gone |
| B — `:wat::test::deftest-hermetic` emits same | ✓ grep confirms line 342 |
| C — `cargo check --release` green | ✓ |
| D — Workspace fail count drops dramatically | ✗ failed: 854 (delta: -1 from 855). Predicted ≤ 50; actual near-baseline. **Honest delta #1 — rot has multiple independent causes; macro fix alone insufficient.** |
| E — Pass count rises | ✗ 1328 (delta: +1 from 1327). Predicted +800. **Same root cause as D.** |
| F — No previously-passing tests regress | ✓ stash round-trip confirmed: 1327 baseline → 1328 after edit; +1 net, 0 regressions |
| G — slice 1f-α tests still green | ✓ `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 10/10 |
| H — Zero new dependencies | ✓ Cargo.toml unchanged |
| I — Only `wat/test.wat` modified | ✓ git diff shows 1 file |
| J — Honest deltas surfaced | ✓ three categories surfaced (see below) |

**8/10 rows pass. 2/10 fail (D + E — both same root cause).
Mode B partial.**

## Honest deltas surfaced (the diagnostic value of this slice)

### Delta 1 — The 855-failure baseline has MULTIPLE independent root causes

The macro signature edit is structurally correct but the
855-failure baseline has at least three independent rot sources:

**1a — `unknown function: :wat::kernel::run-sandboxed-ast`** —
A significant subset of failures are caused by this. Per the
orchestrator's post-slice grep: the primitive IS registered in
`src/check.rs:2027` + `:2121` + `:2939` + `:3308`, so the
sonnet-reported "unknown function" might be a different
failure-class symptom (perhaps the runtime arm or a specific
caller path); needs deeper investigation. Per
`src/stdlib.rs:95-120` (arc 170 slice 3 retirement notes):
`run-sandboxed-hermetic-ast` was retired; non-hermetic
`run-sandboxed-ast` may have a similar status.

**1b — `wat-tests/test.wat` manual `user::main` definitions
NOT going through the macro** — Lines 48, 78, 124, 163, 185,
205, 215, 267 contain inline `run-ast` + `program` calls with
the legacy 4-arg `user::main` shape. These are test-framework
self-tests; the macro fix doesn't reach them.

**1c — Other `wat-tests/*.wat` files with manual `user::main`**
— `wat-tests/core/struct-to-form.wat`,
`wat-tests/core/option-expect.wat`,
`wat-tests/core/result-expect.wat`, `wat-tests/console.wat`
all contain manual `user::main` defs inside quasiquoted program
forms.

**1d — Rust test files with inline legacy `user::main` literals**
— `tests/wat_arc144_hardcoded_primitives.rs:23` +
`tests/wat_arc144_special_forms.rs:34` contain inline wat
strings with the 4-arg shape. Outside the wat-source scope of
this slice; warrants its own Rust-side migration slice.

### Delta 2 — The +1 pass / -1 fail delta is real but tiny

One test shifted from failing to passing. Likely a test that
exercised the macro expansion but failed for a secondary
reason the signature fix unblocked. The overall plateau remains
because the dominant failure causes (above) are still present.

### Delta 3 — No new failures introduced

Confirmed via stash round-trip. The structural correctness of
the macro edit is established.

## Implementation notes

The edit is verbatim per BRIEF before/after:
- `wat/test.wat:315` (`:wat::test::deftest` body): replaced
  the 5-line `(:user::main (stdin ...) (stdout ...) (stderr ...) -> :wat::core::nil)`
  with single-line `(:user::main -> :wat::core::nil)`
- `wat/test.wat:336+` (`:wat::test::deftest-hermetic` body):
  same replacement

The alias factories (`:wat::test::make-deftest` +
`:wat::test::make-deftest-hermetic`) inherit the new shape via
their inner expansion — no edits needed.

## Calibration row

- **Actual runtime:** ~18 min (within band)
- **Workspace post-1f-0a:** 1328 passed / 854 failed
- **Fail-count delta:** -1 (predicted: -800 to -855; actual:
  -1 — Mode B partial)
- **Pass-count delta:** +1 (predicted: +800; actual: +1)
- **Honest deltas surfaced:** 3 categories (rot has multiple
  independent causes; +1 delta real but tiny; no regressions)
- **Mode:** B partial — structural fix correct; diagnostic
  output is the genuine deliverable

## Lessons captured

1. **The BRIEF's prediction was wrong**, but the WORK is
   correct. Slice scope was "fix the deftest macros' signature."
   Sonnet executed that exactly. The 855-failure prediction
   assumed the macro WAS the sole cause; in reality it's one
   of multiple independent causes. The slice fulfilled its
   stated scope.

2. **Mode B is the honest classification.** The slice's
   primary deliverable (the macro fix) shipped clean and
   non-regressive. Its secondary prediction (fail-count drop)
   didn't materialize. Future slices fill the remaining gaps.

3. **The diagnostic value of the slice is HIGHER than the
   pass-count delta suggests.** Sonnet's report identifies the
   three other root causes; this becomes the spec for the next
   foundation slices (1f-0a-ii, 1f-0a-iii, etc.) that complete
   the test-harness migration.

4. **Per `feedback_no_known_defect_left_unfixed.md`:** the
   diagnosis names the remaining rot precisely. Subsequent
   slices have a clear scope (no further investigation
   needed to start them).

## What's next (orchestrator-side)

1. **Commit slice 1f-0a atomically** (this turn) — bundle the
   one `wat/test.wat` edit + this SCORE doc
2. **Author slice 1f-0a-ii BRIEF + EXPECTATIONS** — investigate
   the `unknown function: :wat::kernel::run-sandboxed-ast`
   diagnostic; understand whether the primitive is genuinely
   missing or whether the failure is a different class. Then
   fix.
3. **Author slice 1f-0a-iii BRIEF + EXPECTATIONS** —
   `wat-tests/test.wat` + `wat-tests/core/*.wat` +
   `wat-tests/console.wat` manual `user::main` migration
   (consumer sweep within wat-tests/).
4. **Author slice 1f-0a-iv BRIEF + EXPECTATIONS** — Rust test
   files with inline 4-arg `user::main` literals
   (`tests/wat_arc144_*.rs`). Sonnet.
5. **THEN slice 1f-0b** — the original Event-reshape work
   (now sequenced after the foundation crack is fully closed).

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-0A.md`](./BRIEF-SLICE-1F-0A.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1F-0A.md`](./EXPECTATIONS-SLICE-1F-0A.md)
- BUILD-PLAN ref: §3 slice 1f-0a (the spec sonnet fulfilled)
- REALIZATIONS pass 18 (the umbrella diagnostic this slice
  contributes to)
- Sister slices (pending): 1f-0a-ii (substrate-primitive
  investigation), 1f-0a-iii (wat-tests sweep), 1f-0a-iv (Rust
  inline literals)
- Predecessor: pass 18 commit `7709d0f`
- Substrate references: `src/check.rs:2027-3308` (the
  `run-sandboxed-ast` registration sites; investigation
  source for slice 1f-0a-ii)
