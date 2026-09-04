# SCORE — the blame is deterministic, and my own scorecard row was the fourth false claim

> **Written after the orchestrator's own re-run.** Rows marked *re-driven* were run on this machine
> at HEAD `c6bfe2fbb` + the strike.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 | ★ the blamed function is stable | ✅ **48/48 one outcome** (`:probe::a` @ line 8), **re-driven by the orchestrator**. Was 12 runs → 6/6 |
| 2 | ★ unrepresentable, not sorted | ✅ `BTreeSet<String>` at **7 sites** — one more than my brief listed. Zero `HashSet` remains for this value; the only mention left is the warning comment `⛔ BTreeSet, NOT HashSet — C20` |
| 3 | ★ the test reads IDENTITY | ✅ mutation 2 (`.iter().rev()`) REDs on the identity assertion in both tests, while stability stays green — `left: (":probe::b", 5) / right: (":probe::a", 8)` |
| 4 | reverting REDs, count justified | ✅ mutation 1 REDs on stability, printing both outputs. **Count derived, not asserted** — see below |
| 5 | the quarantine can tell cured from broken | ⛔ **THE ROW IS FALSE — see the finding** |
| 6 | the other two stay quarantined with evidence | ✅ `QUARANTINE_LEN` 3 → 2, both rows keep evidence, re-drive recorded |
| 7 | no behaviour change | ✅ 0 FAIL. One non-asserted delta, run down to a known class — below |
| 8 | floor / lints / clippy | ✅ **`5403 tests run: 5403 passed, 21 skipped`** (440.9 s), **0 FAIL rows**, lints **254**, clippy rc=0 |

## ⛔⛔ THE FOURTH FALSE CLAIM WAS MY OWN SCORECARD ROW

I told the rider to assume a fourth existed. It was **row 5**, and the file my own brief listed as
read-item 5 refutes it in a section titled **"WHAT THIS GATE DELIBERATELY DOES NOT DO"**:

> *"It does NOT assert that the two quarantined files are STILL nondeterministic. That assertion is
> attractive — it would make the list self-expiring — and it was rejected on purpose: … a genuine, if
> rare, false RED. **This repo does not ship a test that can fail for a reason other than the
> defect.**"*

My row required, unhedged, a property the authors had **considered and rejected with a stated
reason**. A rider working to the scorecard would have reported a false red or built the assertion
they refused.

The rider did neither: it ran both variants — **3a** (row restored, length left at 2) → RED on the
length pin; **3b** (row and length both restored) → **GREEN, 18/18** — and wrote the true account
into `QUARANTINE_LEN`'s doc: the pin cannot tell cured from broken and is not meant to; what protects
a cured file is **its own** deep determinism test. **Row removal is not the evidence C20 is cured —
the 24-process test is.**

## The run count, derived rather than asserted

The rider refused my floor of 12 and derived its own. Measured **p̂ = 0.58** over 224 runs (not the
0.5 I assumed); conservative upper 95% bound **p ≤ 0.65**. A pinned-identity test is a false green
when all N draws land on the pinned side, `p^N`:

| N | false green |
|---:|---|
| 2 | 0.42 |
| 12 | 5.7e-3 — **1 in 176** ← my row 1's floor |
| 24 | 3.3e-5 — **1 in 31,000** |

**My row 1 was satisfiable by a test 176× weaker than it intended.** N=24 also matches C19's
precedent. Placed in `binary_id(wat::rete)` (90 s/180 s budget) rather than `tests/lint/`, where at
that binary's recorded 3.5–4.4x contention band it would have come within a SIGTERM of the default
30 s kill. **Nobody costed the test's home; my blast radius named neither.**

## ⛔ AND MY FALSE-GREEN FORMULA WAS ARITHMETICALLY WRONG, IN FOUR PLACES

`2·0.5^(N−1)` evaluates to **1.0** at N=2 — "wrong with certainty". P(all N draws agree) = 2·0.5^N =
**`0.5^(N−1)`** = 0.5 at N=2. The 50%-at-two-runs headline everyone quoted was right; the formula
under it was not.

**I copied it from `diagnostic_output_is_deterministic.rs`'s own header** into this strike's probe and
EXPECTATIONS; it was already in the C20 work-list row. Four sites, all corrected with the derivation
shown. It had survived since C19 because it was quoted, never evaluated.

## Row 7 — one delta, run to ground

No asserted golden moved. Driving all 30 rete-defn-declaring `.wat`/`.wat.bad` files, HEAD vs fixed,
three changed hash identically: they are stdin-reading codemods that panic on `/dev/null`, and that
panic's `:location` is **`:file "src/freeze.rs" :line 1522`** — a *Rust* source line, emitted by
`crate::rust_caller_span!()`. A 6-line doc note moved it to 1528, exactly +6.

**This is the fifth recorded occurrence of a tracked class** —
`probe_diagnostic_value_snapshot_in_errors.rs:75-98` documents four and calls it *"a gate that tests
its own accident"*. Nothing pins 1522, so nothing is red.

**Orchestrator's ruling: the note stays.** The leak is the defect, not the comment. A rule that
"nobody may add comments above certain Rust lines" is absurd; that a wat user's diagnostic cites the
compiler's own source line is a real defect and is **rowed, not accommodated**.

## Fixed in passing

C18 (`04abe37fc`, the commit immediately before this strike's draw) took the `.wat.bad` corpus
**281 → 268** and left every downstream count stale. The rider corrected four inside
`diagnostic_output_is_deterministic.rs`; the orchestrator corrected the fifth — `.config/nextest.toml`'s
budget derivation, now `268 (266 asserted over, 2 quarantined)` with the re-derivation command
written beside it. The budget itself is unaffected (a smaller corpus is cheaper).

## What the rider did that is worth copying

- **Mutation 1's `git diff --stat` listed only THREE of four changed files** — `runtime.rs` reverted
  to its HEAD content — so a diffstat check would have read a landed mutation as not landing. **The
  md5 manifest is what caught it.** Exactly the trap the brief named, hit for real.
- **Its own corpus sweep silently lost 6 of 30 files** — `wat` inherited the loop's stdin and ate the
  file list. Caught by a **row-count anchor** (30 files, 29 lines), not by the output looking wrong.
  Re-run with `</dev/null`.

## What this strike does NOT close

**C20 shrinks; it does not close.** The two check-phase files are a different root, re-driven on the
final binary: `w2a_kwargs_check_mint_swap` 24 runs → 12/12 two hashes; `c2_mixed_macro_swap` 24 runs
→ 18/6 two hashes. The scoping held.
