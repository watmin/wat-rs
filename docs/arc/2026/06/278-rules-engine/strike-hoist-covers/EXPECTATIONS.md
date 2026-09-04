# EXPECTATIONS — hoist `covers`

> ⚠ **This is a fire-time change. Fire is 0.18% of wall on this axis.** A report claiming a
> wall-clock or user-visible win is wrong.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,408 plus every arm you drive.**

## The scorecard — pre-values driven at HEAD `5f0b2f1b1`

| # | what | state AT HEAD | required after |
|---|---|---|---|
| 1 | ★ semantics unchanged | differential GREEN over 115 fixtures / 34,368 pairs / 9,576 facts | **still GREEN** |
| 2 | ★ the gate still bites on the NEW code | — | mutation 1: breaking the hoisted loop REDs with **dropped** facts |
| 3 | ★ the hoist is the value actually read | — | mutation 2 (inverted `covered`) REDs |
| 4 | the index is not a coincidence | — | mutation 3 (fixed index) REDs — **or an honest "no mixed-coverage fixture exists"** |
| 5 | measured before/after | `J−I` ≈ **290 µs** of a ~**414 µs** phase | **six samples per side, with spread** |
| 6 | no rung pinned | the arms' small rungs are below resolution | no assertion on a rung value |
| 7 | blast radius | — | `dispatch_where_tests` only |
| 8 | floor / lints / clippy | **`5408 tests run: 5408 passed, 21 skipped`** (440.0 s, 0 FAIL), lints **258**, clippy rc=0 | ≥ 5408 + arms, 0 FAIL, lints ≥ 258, rc=0 |

## Runtime prediction

**40–70 minutes.** The edit is six lines; the six-sample before/after and the three mutations are the
work. **Budget four release rebuilds** — the last three strikes each overran on exactly this.

## Trap doors named in advance

- **⛔ RE-PROVE THE GATE ON THE NEW CODE.** A differential written against the old loop is not
  automatically a differential over the new one. Row 2.
- **Six samples or no number.** One reading of a perf delta is worth nothing, and this arc has three
  separate incidents behind that rule.
- **Do not pin a rung.** C12's own instrument found its small rungs below resolution on the sixth
  consecutive drive.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.

## What would make this strike a failure even if every test passes

**Landing it without re-proving the differential.** The whole argument for doing this now — rather
than a week ago — is that a proof exists. Shipping under a gate nobody re-earned discards the only
reason the sequencing made sense.

**And quoting a wall-clock win.** Fire is 0.18% of wall here. The honest claim is fire-time and the
Clara ratio; anything larger is the defect C8 was opened for.
