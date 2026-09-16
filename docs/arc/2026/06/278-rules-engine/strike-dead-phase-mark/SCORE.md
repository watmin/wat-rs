# SCORE — C3, weighed against the orchestrator's own re-run

> Re-run at `eb3e260f6` + the rider's tree. **STOP-1 FIRED AND THE RIDER WAS RIGHT: five dead names,
> not one. My measurement was wrong, and I found the bug in my own instrument.**

## The scorecard

| # | required | result, MY re-run |
|---|---|---|
| 1 | ★ the lint exists | ✅ `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs`, 780 lines, 4 arms |
| 2 | ★ RED at HEAD on exactly one name | ⛔ **RED on FIVE** — see A. STOP-1 fired, correctly |
| 3 | ★ green after the fix | ✅ |
| 4 | the dead read gone | ✅ no reader of `setup:seen:insert` anywhere |
| 5 | the false rows gone | ✅ `insert` and `in-fire insert − S` both absent |
| 6 | no row negative from an absent mark | ✅ `in-fire seen − S` is negative **by construction** and now says so in the table |
| 7 | the premise restated | ✅ table header and the doc at `:1528` both state coextensivity and where insert lives |
| 8 | `REQUIRED_PHASES` untouched | ✅ |
| 9 | engine untouched | ✅ `git diff --stat -- src/rete/kernel/fire/` → empty |
| 10 | radius | ✅ `accum_cost.rs` (+44 −11) + one new lint file |
| 11 | lints | ✅ **210/210** (was 196; +14) |
| 12 | clippy | ⛔ **RED, then fixed by me** — see C |

## ⛔⛔ A — MY "EXACTLY ONE" WAS WRONG, AND THE BUG WAS IN MY SWEEP

The rider's HEAD RED named five, four of them `ALPHA_KIDS` entries in `accum_cost.rs`. I verified
independently — three of the four have **zero** emitters anywhere under non-test `src/`; the one
`alpha:match` hit is a prose line in `compiled_cond.rs:16`.

**Why my instrument missed them.** My sweep used a naive `"((?:[^"\\]|\\.)*)"` regex with no notion
of comments or char literals. One unbalanced quote earlier in the file **inverts the parity**, and
from there the regex matches the *gaps between* literals instead of the literals. Dumped at HEAD, my
matches around `ALPHA_KIDS` read:

```
match at 19858: ',\n    ];\n    const ALPHA_KIDS: [&str; 4] = [\n        '
match at 19933: ',\n        '
```

It was reading the **complement** of the string set. `setup:seen:insert` survived only because my
*other*, anchored regex (`of\(\s*"..."\)`) found it independently — and the four `ALPHA_KIDS` names
are read through a loop variable, so only the broken unanchored rule could ever have seen them.

The rider's lint has an extractor with 11 unit tests written against exactly this hazard class
(char literals `'"'`, raw strings) — the thing my throwaway lacked. **The DESIGN's "measured:
exactly one" claim is false and is corrected in place.**

## ⭐ B — AND THE RIDER WAS RIGHT NOT TO DELETE THE OTHER FOUR

I checked its reasoning against the code rather than accepting it. `accum_leftover_split` reads each
kid as `(ns, pairs)` and computes `kids_retired = kid_pairs.iter().all(|k| *k == 0)` (`:613`). Every
derived quantity — `remainder_alpha`, `tax_alpha`, `honest_alpha` — branches on that flag and does
**no arithmetic on the zero nanoseconds**. The four rows still print, but as `0.00 raw 0.00 net
**0x**`: the pair count carries its own evidence of non-firing.

That is a **handled absence**, not C3's defect. C3's row printed `0.00 ms` with nothing to say it was
never taken, and then subtracted it. The rider justified the four in place with a per-name
`rune:lint(census-name-retired)` — this tree's existing idiom — and said the disposition is
reversible in four lines. Correct call.

## ⛔ C — CLIPPY WENT RED AND THE FLOOR COULD NOT SEE IT

Floor: **`5327 tests run: 5327 passed, 21 skipped`**, exit=0. Clippy on the same tree:

```
error: redundant closure
   --> tests/lint/census_name_read_by_a_cost_test_is_emitted.rs:355:30
355 |         if after.is_some_and(|c| is_ident_char(c)) {
    = note: `-D clippy::redundant-closure` implied by `-D clippy::all`
```

Third time in this arc that clippy caught what nextest structurally could not. Fixed by me
(`is_some_and(is_ident_char)`, semantically identical); clippy rc=0 and `binary_id(wat::lint)`
**210/210** after.

## ⛔ D — I PRESENTED A HAND-INDENTED TABLE AS DRIVEN OUTPUT

DESIGN.md quotes the HEAD table with `alloc` and `insert` indented under `setup:seen`. **The code
does not render that.** My own captured run, raw bytes, has them flush-left — I added the indent
while transcribing and presented the block as driven. The rider nearly "fixed" an indentation
regression that never existed.

This is **C11 concealing itself in my prose**: I had already recorded that a `\`-newline
continuation eats the leading whitespace, then unconsciously corrected for it when quoting. The
rider renamed the row to `setup:seen:alloc` so it names its own mark instead of depending on an
indent that was never there — better than what I asked for.

## ⭐ E — MUTATION 2 DRIVEN BY ME, IN A DIFFERENT FILE

The row I said I would read hardest, because it is the only one separating a name-resolver from a
check hard-coded to one string. I typed an existing mark in `node_share_cost.rs` — a **different
file** and a **different reader closure** (`ns_of`, not `of`) than the rider used:

```
1 census name(s) are READ by a cost test and EMITTED by nothing under src/.
  src/rete/kernel/tests/node_share_cost.rs:454  "filte"
Summary [0.660s] 14 tests run: 13 passed, 1 failed, 196 skipped
```

Caught at the right line, through a reader name my own brief had not enumerated. It resolves names
generally.

## Other brief defects the rider named, all upheld

- **`of("...")`-style undersells the corpus** — three reader closures exist (`of`, `ns_of`, `get`);
  a literal `of("` grep misses `get(` entirely. Confirmed by my own mutation landing on `ns_of`.
- **`2.55 ms` is box-specific** — the rider saw `S` range 1.79–4.28; my re-run gave 3.67. It
  deliberately pinned the *identity* (`in-fire insert − S` was always `−S`) rather than the number.
  Correct, and the discipline C4's score demanded.
- **A false-positive class I never mentioned** — `delta.rs:737,749` emit via `census_count(ebucket(n))`,
  so 16 real names appear as no literal. A literal-only universe would false-RED the day a cost test
  read one. The rider built a second universe half; its mutation 5 proves it.

## Per-arm status

| arm | status |
|---|---|
| unresolved-name arm | **proven** — rider's mutations 1–2, plus mine in a third file |
| rune-reason-length arm | **proven** (rider mutation 3) |
| rune-rot arm (rune on a live name) | **proven** (rider mutation 4) |
| computed-name universe half | **proven** (rider mutation 5) |
| 11 extractor unit tests | **proven** — each written against a hazard present in this corpus |
| the four non-vacuity count asserts | **reachable, not driven** — rider's disclosed choice |
