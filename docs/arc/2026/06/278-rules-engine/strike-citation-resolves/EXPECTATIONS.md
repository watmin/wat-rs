# EXPECTATIONS — a name in prose either resolves or declares

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT, AND MY POPULATION IS A FLOOR, NOT A TARGET

**The floor must be ≥ 5,267 plus every arm you drive.** My classifier says **33 unresolved of 732**;
it is a throwaway regex. **Yours is the instrument — report its number, and if it disagrees with
mine, say why.**

## The scorecard, with pre-values measured at HEAD `de8f3f6a0`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | ★ prose cannot vouch for prose | a comments-included universe reports **0 of 732** — self-vouching | a name existing only in a comment comes back **unresolved**, driven |
| 2 | backticked identifiers | **33 unresolved**, incl. all 7 the row predicted | resolved, reworded, or declared |
| 3 | the bare filename | *"Tests are `tests.rs`"* — **file does not exist** (driven) | corrected; the gate catches its class |
| 4 | the existing gate's hole | `no_stale_path_in_doc.rs:47` requires `/` | covered — by extension or a sibling, **stated which** |
| 5 | my two rotted citations | `check_field_at`, `keyword_constant_segment` (driven) | fixed, and **named in the report as strike-caused** |
| 6 | noise is declared | — | clippy lints / memory slugs / `_`-prefixed each a **named vocabulary or rune** |
| 7 | non-vacuity | — | declared with a real floor; the vacuity gate green |
| 8 | radius | — | `tests/lint/` + comment sites. **No `src/` behaviour change** |
| 9 | lints | **153/153** (measured) | green |
| 10 | floor | **5267/5267** (measured) | ≥ 5,267, zero FAIL rows |
| 11 | clippy | **rc=0** (measured) | silent |

## The mutation proofs

1. **A backticked name that exists only in a comment** → RED, naming file and token. *This is trap 1;
   if it passes, the universe is self-vouching and every green is unearned.*
2. **A backticked name that exists only in `tests/`** → **GREEN**. Trap 2's other side: the universe
   must not manufacture findings out of test-only names.
3. **A bare `*.rs` filename that does not exist** → RED. And a real one → green.
4. **Blind the code universe** (resolve against nothing) → the non-vacuity floor REDs, not a silent
   pass.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

70–90 minutes. The classifier and its four mutations are most of it; the citation fixes are
mechanical once the population is settled.

## What would make this strike a failure even if every test passes

**Widening the universe until the findings disappear.** Every one of the 33 can be made to resolve by
searching more text — including the comments they live in. STOP-1 exists for this, and mutation 1 is
what keeps it honest.

The second: **hand-filtering the noise.** Three clippy lint names and a memory slug can be dropped
with a `if name == …` and nobody would notice. Row 6 requires a named vocabulary with a reason —
an unexplained exclusion is precisely the defect this class removes.
