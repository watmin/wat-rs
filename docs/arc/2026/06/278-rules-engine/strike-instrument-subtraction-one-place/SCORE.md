# SCORE — instrument subtraction, one place

Executing strike per DESIGN.md / BRIEF.md / EXPECTATIONS.md. Appending as measurements land
(house rule).

## Re-derived count (before touching anything)

Ran the SAME pattern EXPECTATIONS.md's row 3 gate uses, broadened slightly to catch both orders
and both closures:

```
grep -rnE '\- .* as f64 \* cal' src/rete/kernel/tests/*.rs
```

Result — **13**, matching the brief exactly:

| file | line | expression |
|---|---|---|
| `mod.rs` | 547 | `stat(xs).0 - *pairs.get(k).unwrap_or(&0) as f64 * cal_ns_per_pair` (closure `net_of` body) |
| `accum_cost.rs` | 698 | `raw - pairs as f64 * cal` (closure `net` body) |
| `cascade_cost.rs` | 393 | `seen_raw - seen_pairs as f64 * cal` |
| `cascade_cost.rs` | 396 | `arm_raw - arm_pairs as f64 * cal` |
| `fanout_cost.rs` | 425 | `prod_raw - prod_pairs as f64 * cal` |
| `fanout_cost.rs` | 426 | `rhs_raw - rhs_pairs as f64 * cal` |
| `fanout_cost.rs` | 427 | `dedup_raw - dedup_pairs as f64 * cal` |
| `fanout_cost.rs` | 558 | `rhs_raw as f64 - rhs_pairs as f64 * cal` |
| `fanout_cost.rs` | 730 | `rhs_raw - rhs_pairs as f64 * cal` |
| `fanout_cost.rs` | 731 | `dedup_raw - dedup_pairs as f64 * cal` |
| `fanout_cost.rs` | 732 | `probe_raw - probe_pairs as f64 * cal` |
| `fanout_cost.rs` | 828 | `ns as f64 - k as f64 * cal` (inside `.min(...)`) |
| `strat_cost.rs` | 416 | `ns as f64 - k as f64 * cal` (inside `.min(...)`) |

This **matches DESIGN.md's re-derived 13 exactly** (fanout_cost.rs 8, cascade_cost.rs 2, mod.rs 1,
accum_cost.rs 1, strat_cost.rs 1). One small delta from BRIEF.md's prose, noted for the record: the
brief's implementation sketch says "the ten inline sites become calls," but DESIGN.md's own table
sums to **11** inline sites (8 + 2 + 1) plus the 2 closures = 13 — the "ten" is a one-off slip in
the brief's prose, not in its table or its total. Total (13) is unaffected either way.

## Candidate 14th site — found, driven, and NOT converted (out of the stated blast radius)

`src/rete/kernel/tests/rank_and_instrument.rs:1322` also calls `calibrate_mark_ns()` and at
`:1365` computes `let tax = (rhs_k + dedup_k) as f64 * cal;` then at `:1366`
`let honest = fire as f64 - remainder - tax;`. Algebraically this equals
`net_ns(fire as f64 - remainder, rhs_k + dedup_k, cal)`, so it is the SAME arithmetic family.

Did not convert it, for two reasons: (1) BRIEF.md's blast radius names exactly five files —
`mod.rs`, `accum_cost.rs`, `cascade_cost.rs`, `fanout_cost.rs`, `strat_cost.rs` — and
`rank_and_instrument.rs` is not among them; (2) it does not match the mechanical shape the other
13 share (a single `raw - pairs as f64 * cal` expression) — it subtracts `remainder` and `tax` as
two separate terms across two statements, so absorbing it would mean restructuring the caller, not
a 1:1 call swap, which is a different (larger) change than "arithmetic-preserving, mechanical."
Flagging per STOP-3's spirit rather than silently expanding scope. This is a real sibling of the
class DESIGN.md describes, one file over from where DESIGN.md's own re-derivation stopped looking.


## Changes made — all 13 sites, in order

1. `mod.rs` (new): added `pub(super) fn net_ns(raw: f64, pairs: u64, cal_ns_per_pair: f64) -> f64`
   immediately after `calibrate_mark_ns`, doc citing `render_phase_table`'s "two copies is how one
   of them silently stops subtracting."
2. `mod.rs:547` (now :560, `net_of` closure body): `stat(xs).0 - *pairs.get(k).unwrap_or(&0) as f64
   * cal_ns_per_pair` → `net_ns(stat(xs).0, *pairs.get(k).unwrap_or(&0), cal_ns_per_pair)`.
   `net_of`'s own key-lookup shape kept, per the pinned contract.
3. `accum_cost.rs:698`: closure `let net = |raw: f64, pairs: u64| raw - pairs as f64 * cal;`
   DELETED. Its 6 call sites (`:699,700,714,776,779,785` after deletion shifts lines up one) all
   changed from `net(x, y)` to `net_ns(x, y, cal)`.
4. `cascade_cost.rs:393`: `seen_raw - seen_pairs as f64 * cal` → `net_ns(seen_raw, seen_pairs, cal)`.
5. `cascade_cost.rs:396`: `arm_raw - arm_pairs as f64 * cal` → `net_ns(arm_raw, arm_pairs, cal)`.
6. `fanout_cost.rs:425`: `prod_raw - prod_pairs as f64 * cal` → `net_ns(prod_raw, prod_pairs, cal)`.
7. `fanout_cost.rs:426`: `rhs_raw - rhs_pairs as f64 * cal` → `net_ns(rhs_raw, rhs_pairs, cal)`.
8. `fanout_cost.rs:427`: `dedup_raw - dedup_pairs as f64 * cal` → `net_ns(dedup_raw, dedup_pairs, cal)`.
9. `fanout_cost.rs:558`: `rhs_raw as f64 - rhs_pairs as f64 * cal` → `net_ns(rhs_raw as f64, rhs_pairs, cal)`
   (`rhs_raw` is `u64` at this site, unlike the others — cast preserved exactly, moved into the
   call argument, same value).
10. `fanout_cost.rs:730`: `rhs_raw - rhs_pairs as f64 * cal` → `net_ns(rhs_raw, rhs_pairs, cal)`.
11. `fanout_cost.rs:731`: `dedup_raw - dedup_pairs as f64 * cal` → `net_ns(dedup_raw, dedup_pairs, cal)`.
12. `fanout_cost.rs:732`: `probe_raw - probe_pairs as f64 * cal` → `net_ns(probe_raw, probe_pairs, cal)`.
13. `fanout_cost.rs:828`: `e.0.min(ns as f64 - k as f64 * cal)` → `e.0.min(net_ns(ns as f64, k, cal))`
    — fold kept at call site, per the pinned contract (this is one of the two named sites).
14. `strat_cost.rs:416`: `net[i].min(ns as f64 - k as f64 * cal)` → `net[i].min(net_ns(ns as f64, k, cal))`
    — fold kept at call site, per the pinned contract (the other named site).

All 13 DESIGN.md sites converted (counted as 14 numbered steps above because step 1 is the new
`fn` itself, not a conversion).

## Type/cast check (STOP-2 candidate, did not fire)

Read every site's surrounding declarations before editing. `net_ns(raw: f64, pairs: u64,
cal_ns_per_pair: f64)`. Two sites pass a raw value that is `u64` rather than already-`f64`
(`fanout_cost.rs:558`'s `rhs_raw` and the two `.min(...)` sites' `ns`) — at all three, the existing
`as f64` cast was preserved verbatim and simply moved from "cast-then-subtract" to "cast-then-pass
as the first argument"; the cast itself, and the value it produces, is unchanged. No site required
a NEW cast and no cast changed width or truncated. STOP-2 does not fire.

## Post-edit mechanical greps (EXPECTATIONS.md rows 1-3, 5, 7)

```
grep -c 'fn net_ns' src/rete/kernel/tests/mod.rs                                    → 1
grep -c 'let net_of = \|let net = ' src/rete/kernel/tests/{mod,accum_cost}.rs        → mod.rs:2, accum_cost.rs:0
grep -rnE '\-.*as f64 \* cal' src/rete/kernel/tests/*.rs                             → 1 hit: mod.rs:483 (the fn net_ns BODY itself — the one legitimate place)
grep -n 'net_ns(' fanout_cost.rs strat_cost.rs | grep min                            → both .min(net_ns(...)) sites present, fold kept outside
grep -c 'two copies is how one of them silently stops' mod.rs                        → 3 (the original comment + the new fn's doc + net_ns's own doc, see below)
```

Row 2's `mod.rs:2` is EXPECTED, not a miss: line 559 is `net_of`'s own definition line (the pinned
contract keeps `net_of`'s key-lookup shape — only its *body* changed to a call), and line 588 is
`let net = net_of(phase, xs);`, a result binding to net_of's return value, not a re-declaration of
the subtraction. Neither line contains the raw subtraction spelling anymore — confirmed by row 3's
single remaining hit being `net_ns`'s own body.

Row 7's count is 3, comfortably ≥1: `grep -n` shows three distinct lines carrying the phrase —
`mod.rs:307` (the pre-existing `:303-308` note about the min/mean estimator defect, which itself
quotes the same argument), `mod.rs:478` (my new `net_ns` doc's citing sentence), and `mod.rs:491`
(the original, unedited `render_phase_table` doc this strike responds to, shifted down from :476
by my insertion). The row's own condition — the helper's doc points back at the comment that
demanded it — is satisfied by `:478` alone; the other two are pre-existing occurrences, not double
counting of the same site.

## ⛔ Numbers-unchanged check — driven with a control, not assumed (EXPECTATIONS row 6)

Ran the SAME 9 tests (covering all 13 sites, across all 5 touched files) three times: once
BEFORE any edit, once AFTER all 13 edits, and — when a raw `diff` of BEFORE vs AFTER came back
large (641 lines) — a SECOND "before" run (`git stash` the 5 edited files back to HEAD, rebuild,
re-run) to find out how much of that 641 is real wall-clock noise these tests carry BY DESIGN
(`RUNS: usize = 3`, "MINIMUM across runs, not mean" — every number printed is a live
`Instant::now()` measurement, never a fixture).

**Why a raw diff is the wrong instrument here, discovered by driving it:** the first raw `diff`
BEFORE vs AFTER showed phase ROWS in different ORDER between the two runs (e.g. `hash-join` printed
before `root-join` in one run, after in the other) — table rows are pushed in *discovery* order
from `census(a,b)`'s returned `Vec`, and that order is not guaranteed stable run-to-run. A line-by-
line diff would report this reordering as a content change even though the CONTENT is unchanged.

**Control, to separate "my edit" from "this test's own noise":** `git stash` the 5 edited files
(back to `59234af31`, pre-strike), rebuild `--release --tests` (clean, 1m11s), re-ran the same 9
tests a second time on the UNMODIFIED code, `git stash pop` to restore my edits, rebuilt again
(clean). Then compared, all three normalized (nextest run IDs, `[ N.NNNs]` timers, and every
decimal number replaced with `NUM` — but NOT integer pair-counts/multipliers/fact-counts, which
this instrumentation computes exactly and should never move) and **sorted** (order-independent, so
row-reordering does not register as a diff):

| comparison | sorted, normalized diff |
|---|---|
| BEFORE vs. BEFORE-again (same unmodified code, 2 runs) — **the noise floor** | **23 lines** |
| BEFORE vs. AFTER (my 13-site edit) | **23 lines** |

**Identical magnitude, and identical in KIND** — every one of the 23 lines in both diffs is one of
exactly two things, both already documented as expected noise in this very file (`mod.rs:280-313`,
the min/mean note): (1) a printed field's column width shifting by one space because the digit
count of a `{:>7.2}`-formatted number changed run to run (e.g. `0.36` vs `10.36`), or (2) the
`⚠ BELOW ITS OWN INSTRUMENT` flag flipping on a sub-millisecond row whose `net` sits within noise
of zero (`round:preamble`, `filter-after-join`, `accum:snapshot`, `alpha`/`alpha:seed`) — exactly
what `mod.rs:310-313` already warns: *"Sub-millisecond rows in these tables are noise wearing a
number; do not read one as a finding without re-measuring."* None of the 23 lines in the AFTER diff
involves a phase this strike's 13 sites did not touch producing a DIFFERENT kind of change than the
control shows, and none is a missing/extra row, a changed pairs-count, or a changed label.

**Verdict: the printed numbers are unchanged by this strike, to the same precision this
instrumentation can measure anything** — the test's own built-in noise (proven via the control) is
the entire explanation for every line that differs, and that noise is present, unchanged, and of
identical size whether or not my edit is applied. `cargo build --release --tests` was clean (no
warnings) both before and after.

Raw captures kept for the record: `BEFORE.txt`, `AFTER2.txt` (edited code), `BEFORE_A2.txt`
(unmodified-code control) — not committed (perf captures, not source), described here instead.

## Cost tests — full filter (EXPECTATIONS row 8)

```
cargo nextest run --release -E 'test(cost)'
     Summary [  15.163s] 75 tests run: 75 passed, 5429 skipped
```

All 75 green, including every one of the 9 tests used for the before/after comparison above and
every test in the 5 touched files.

## Clippy (EXPECTATIONS row 9)

```
cargo clippy --all-targets --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 14.85s
```

rc=0, no warnings.

## Floor (EXPECTATIONS row 10)

```
./scripts/floor.sh
     Summary [ 453.315s] 5485 tests run: 5485 passed, 19 skipped
exit=0. Log kept at .floor/2026-09-09T02-46-29Z/
```

**5485/0 fail, 19 skipped — matches EXPECTATIONS' predicted 5485 exactly.** Read from
`.floor/latest/clean.log`'s own `Summary` line (never the piped exit code), per house rules. No
red at any point in this strike.

## Scorecard (EXPECTATIONS.md rows)

| # | row | result |
|---|---|---|
| 1 | helper exists, free `fn` (`grep -c 'fn net_ns' mod.rs`) | **HOLD.** 1 |
| 2 | both private closures gone (`grep -c 'let net_of = \|let net = ' {mod,accum_cost}.rs`) | **HOLD, with the expected non-zero on mod.rs** — `net_of`'s definition line and a `net_of(...)`-result binding remain by design (pinned contract: `net_of` keeps its key-lookup shape); neither line still contains the raw subtraction. `accum_cost.rs`: 0, clean deletion |
| 3 | every site converted, 0 hand-rolled remaining (`grep -rnE '\-.*as f64 \* cal' *.rs`) | **HOLD.** 1 hit total, and it is `net_ns`'s own body — the one legitimate place |
| 4 | count re-derived | **HOLD.** 13, matching DESIGN.md exactly; brief prose's "ten inline" vs DESIGN's table-sum "eleven" delta noted (total 13 unaffected). A 14th-shaped sibling found at `rank_and_instrument.rs:1365-1366`, NOT converted (outside blast radius, not a 1:1 call shape) |
| 5 | the two folds keep `min` at the call site | **HOLD.** `fanout_cost.rs:828` and `strat_cost.rs:416` both read `.min(net_ns(...))` |
| 6 | numbers unchanged | **HOLD, via a driven control.** Sorted/normalized diff between BEFORE and AFTER is 23 lines, IDENTICAL in count and kind to the diff between two runs of the SAME unmodified code (the test's own measured noise floor) — see the dedicated section above |
| 7 | helper cites the argument | **HOLD.** `net_ns`'s doc quotes "two copies is how one of them silently stops subtracting" verbatim, pointing back at `render_phase_table`'s doc |
| 8 | cost tests green | **HOLD.** `test(cost)`: 75 tests run, 75 passed |
| 9 | floor 5485/0 fail | **HOLD.** `5485 tests run: 5485 passed, 19 skipped` |
| 10 | clippy | **HOLD.** rc=0, no warnings |

## STOP triggers — none fired against the DESIGN

- **STOP-1** (any cost table's printed numbers change): did not fire, per the driven control above.
- **STOP-2** (operand types don't fit without a value-changing cast): did not fire. Two sites pass
  an already-`u64` raw (`fanout_cost.rs:558`, and the two `.min(...)` sites) — their existing
  `as f64` casts were preserved verbatim, just relocated into the call argument. No new cast, no
  truncation, no widening.
- **STOP-3** (a 14th site): **a sibling was found** at `rank_and_instrument.rs:1322-1366`
  (`fire as f64 - remainder - tax` where `tax = (rhs_k + dedup_k) as f64 * cal`) — algebraically the
  same family (`net_ns(fire as f64 - remainder, rhs_k + dedup_k, cal)`), but NOT converted: it is
  outside BRIEF.md's explicit five-file blast radius, and it does not match the mechanical
  single-expression shape the other 13 share (two terms subtracted across two statements, not one).
  Reported per STOP-3's spirit rather than silently expanding scope or silently ignoring it.
- **STOP-4** (touching `calibrate_mark_ns`/a measured constant/the min-vs-mean estimator question):
  did not fire. Neither was touched; the two `.min` folds keep deciding the estimator at the call
  site, per the pinned contract.

## What this did NOT do

- Did not touch `calibrate_mark_ns`, any measured constant, or the min/mean estimator question
  (`mod.rs:303-308`'s open question stays open, on purpose — DESIGN.md rejects unifying it here).
- Did not convert `rank_and_instrument.rs:1322-1366`'s sibling arithmetic — flagged, not fixed;
  outside the stated blast radius and not a 1:1 call-site swap.
- Did not add any new test. Did not change any test's assertions.
- Did not touch `4T1`'s positional triple or anything outside `mod.rs`, `accum_cost.rs`,
  `cascade_cost.rs`, `fanout_cost.rs`, `strat_cost.rs`.

## Commit

(pending — committing after this SCORE.md is complete, on green, per house rules)
