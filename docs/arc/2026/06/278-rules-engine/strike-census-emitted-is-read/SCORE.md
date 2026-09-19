# SCORE — EMITTED ⇒ READ for census counters; the seven all got readers

The mirror of `census_name_read_by_a_cost_test_is_emitted` is live. Every `census_count` /
`census_count_n` name the engine emits must be an argument of a sibling-READ row reader, or
carry `rune:lint(census-emitted-unread)`. The DESIGN seven all took disposition 1 (nonzero
reader). Zero deleted. Zero runed. That is not a ratchet.

## Scorecard

| # | result |
|---|---|
| 1 ★ the gate exists and passes | **HOLD.** `every_emitted_census_count_is_read_or_declared` green on the floor. |
| 2 ★ mutation-proved | **HOLD.** `census_count("probe:never-read")` in `matcher.rs` (engine, not a test). RED named that counter. Then removed. Quoted below. |
| 3 ★ the sets are the sibling's | **HOLD.** No second scanner. Sibling `emitted()` / `reads()` are `pub(crate)`; this gate filters `count_names` and uses READ unchanged. Same loop, plus a `COUNT_EMITTERS` split in that loop. Scope cuts untouched. |
| 4 ★ `phase_end` untouched | **HOLD.** `git diff` hits `phase_end` only in sibling comments / `EMITTERS` vis. None of the 14 timing marks moved. |
| 5 ★ readers assert NONZERO | **HOLD on the seven.** Every DESIGN name is `of("…") > 0` (or `of_interp` / `of_miss`). No `assert_eq!(x, 0)`. Histogram extras: see split. |
| 6 ★ the split is reported | **HOLD.** Per name below. 7/7 readers. 0 deleted. 0 runed. |
| 7 ★ floor | **HOLD.** `.floor/2026-09-07T13-17-34Z/`: `Summary [ 465.247s] 5480 tests run: 5480 passed (2 slow), 22 skipped`. +5 tests (1 gate + 4 readers). |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is the proof the gate can fail.** Row 3 is the STOP-1 that did not fire.

## The seven — all disposition 1

| name | disposition | where / why |
|---|---|---|
| `filter:test-env-builds` | **reader** | `census_counter_readers::filter_test_env_counters_are_nonzero`. Direct `build_test_env`. A real fire no longer takes this path (`exec_where` over BindSpan). |
| `filter:test-key-alloc` | **reader** | same test; one string key in the seed. |
| `match:clause` | **reader** | `alpha_discrimination::compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`. Interpreter walk of the `[50 100]` corpus. Whole-fire census reads zero (compiled step 1). |
| `match:bind-insert` | **reader** | same. |
| `match:head-miss` | **reader** | same test; deliberate `zz::NoSuchType` head mismatch. |
| `prod:class-alloc` | **reader** | `census_counter_readers::prod_class_alloc_is_nonzero_on_interpreter_insert`. `build_insert_fact` of `(:ccr::T 1)`. Compiled RHS does not take this path. |
| `rematch:compiled` | **reader** | `census_counter_readers::rematch_compiled_is_nonzero`. Direct `exec_compiled_under`. |

STOP-2 did not fire: none of the seven is `#[cfg]` or feature-gated.
STOP-3 did not fire: readers call existing functions; engine counts did not move.

## DESIGN 7 vs sibling READ 21

DESIGN swept `census_count` *literals* against *any string literal* under `kernel/tests/` and
got 7 unread. Sibling READ is stricter (two scope cuts, each against a measured false RED) and
its EMITTED includes the computed `ebucket`/`tbucket` families. First gate run was 21 undeclared:
the seven, plus `accum:index-builds` / `accum:index-elements` (mentioned via `==` and a
`const [&str; N]` — both cuts), plus 16 histogram buckets (computed names; tests walk them via
`format!("{pfx}{suf}")`, which is not a reader-argument literal).

Widening READ is REJECTED. Those names went through the existing door: `let of = |name: &str|`.

| name | disposition | where / why |
|---|---|---|
| `accum:index-builds` | **reader** | `gather_probe_cost::gather_index_is_built_once_per_alpha_and_keyset`. Already asserted `builds > 0`; `== "…"` swapped for `of("…")` so sibling READ sees it. |
| `accum:index-elements` | **reader** | same; added `elements > 0`. |
| `elem-card:0` | **reader** | `census_counter_readers::binding_card_histogram_buckets_are_read`. Nonzero on accum 60/60 (the matcher-op census already lists it present). |
| `elem-card:{1,2,3,4,5,6-7,8+}` and `tok-card:{0,1,2,3,4,5,6-7,8+}` | **reader (family)** | same test; each name is an `of("…")` argument. Family sum `> 0`. A per-bucket nonzero pin would freeze a distribution the diagnostic only prints. `#[cfg(test)]` end-of-fire histogram in `delta.rs` — outside the seven, so STOP-2 did not apply. |

The 15 family-of()'d buckets can still read 0 on this axis. That 0 now means *measured zero*, not
*absent*: sibling READ⇒EMITTED already proves the name emits. The new gate's job is the other
direction, and they are in READ.

## Mutation

Added `census_count("probe:never-read")` next to `match:calls` in `src/rete/matcher.rs`.
`cargo nextest run --release -E 'test(every_emitted_census_count_is_read_or_declared)'`:

```
1 census_count name(s) are EMITTED and neither READ by a cost test nor declared.
An unread counter is where a name rots: the A–M census was "the counter's name says X, the quantity is Y".

  probe:never-read
```

Then removed. `matcher.rs` is clean.

## What this did not do

Did not touch `phase_end`. Did not re-litigate the sibling's two scope cuts. Did not touch A–M.
Did not rune the seven. Did not change what the engine counts. No `wat/`.

## Final floor

`.floor/2026-09-07T13-17-34Z/`: `Summary [ 465.247s] 5480 tests run: 5480 passed (2 slow), 22 skipped`.
