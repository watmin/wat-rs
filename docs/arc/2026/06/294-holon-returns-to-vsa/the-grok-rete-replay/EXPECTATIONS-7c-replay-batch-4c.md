# EXPECTATIONS 7c — replay batch 4c, grok-rete #212 → #220 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Rows are fixed now so the result
cannot move the goalposts.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 9 replayed steps, in order, correctly subjected | `verify-step-record.sh <start> HEAD 212 220` | exit 0; `step-range: #212..#220 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on #213 #214 #216 #217 #220 | only `docs/`/`.md` |
| E3 | every produced `.wat` checks | `./target/release/wat --check` on #212's, #218's three, #219's one | rc 0, or a deliberate `.bad` |
| E4 | **the arc-130 pair is still `.wat.bad`, and grok's two `historical` runes did NOT land** | `ls docs/arc/2026/05/130-*/complected-2026-05-02/` and `git grep -c 'rune:lint' -- 'docs/arc/2026/05/130-*/complected-2026-05-02/'` — **SCOPED to those files** | `substrate.wat.bad` + `test.wat.bad` present, no `.wat` twins; **0** runes in that directory |

> ⚠ **E4's command was wrong when first written, and measuring the baseline is what caught it.** An
> unscoped `git grep -c 'rune:lint(historical)'` returns **1 today**, before the batch runs — the hit is
> `FINDINGS-composition.md`, whose finding-26 prose quotes the rune syntax. The row would have reported a
> failure on a perfectly correct batch. A census over a tree whose own documentation discusses the string
> being searched must be SCOPED to the artifacts under test
> (`[[feedback_anchor_a_census_to_the_site_never_the_substring]]`, `[[feedback_a_gate_that_reads_its_own_file]]`).
> Baseline measured before the strike: 0 runes in the arc-130 directory, 1 elsewhere in prose.
| E5 | **both walls green together** — the ruling's whole point | `-E 'test(every_tracked_wat_parses)'` and `-E 'test(docs_wat_loads_or_declares_why_not)'` | both pass; the `.wat.bad` pair invisible to each |
| E6 | the docs-wat gate is NOT vacuous | its own non-vacuity guard + `find docs -name '*.wat' \| wc -l` | **exactly 8** — MEASURED at the 4b tip, not predicted (see below) |

> **E6's baseline is now measured, not forecast.** At the close of batch 4b the corpus is **8** `.wat` under
> `docs/` — `probes/{enum-holds-record, red-owner-signals-child, red-send-cause-is-not-matchable,
> surface-field-dispatch}.wat` plus `harness-experiri/{experiri-acc-head, experiri-acc-wrapped,
> experiri-then-match, experiri-when-match}.wat`, the latter four landed by #195 — plus the 2 arc-130
> `.wat.bad`, which are invisible to grok's gate (it filters `extension == "wat"`) and to main's
> `every_tracked_wat_parses` (it globs `*.wat`). That is the ruling working: both walls green, neither
> seeing the preserved pair. So #212's gate walks 8 where grok's walked 10, and the two it will not find
> are exactly the renamed pair.
| E7 | `surface-field-dispatch.wat`'s rot is fixed | `grep -n ':nature\|:holder'` on it; then run it | `:nature`; the file loads and prints 142 |
| E8 | #215's new `.rs` test file meets the hygiene walls | `-E 'test(no_inlined_edn) + test(no_loose_string_assert) + test(no_inlined_wat)'` | green (finding 24) |
| E9 | #219's new docs `.wat` satisfies #212's gate | `-E 'test(docs_wat_loads_or_declares_why_not)'` at #219 | green — it loads, or carries a closed rune whose reason is a sentence |
| E10 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings`, orchestrator-run | green; clippy 0 |
| E11 | test-count delta is ACCOUNTED FOR, not merely green | predict from the diff (`#[test]` added/removed, `#[ignore]` delta) BEFORE running the floor | predicted count == floor count exactly |
| E12 | spot re-run of the walls | orchestrator re-runs lint subset + `kind(lib)` + doctests + stone-3 gate at HEAD, compared to the last code step's verdict lines | identical numbers |
| E13 | no hazard in range | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` | none |
| E14 | no knowingly-red commit | no repair commit after #220 | none |
| E15 | every artifact a body names exists | each `.census/…txt` cited in a commit body | all present on disk |

## ⛔ Every `-E` filter in this table must be confirmed to SELECT A NON-ZERO COUNT

A nextest filterset that matches nothing runs zero tests and **exits 0** — a mis-aimed probe is
indistinguishable from a working gate (`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`).
Before crediting any row above, read the run's own `N tests run` and require N > 0; a row whose filter
selected nothing is UNMET, not passed.

Filter spelling, resolved empirically rather than assumed: nextest matches the **module-qualified** test
path, so a filter naming the FILE/module selects every test in it. Evidence from the #185 checkpoint's own
E7 run — `test(nested_program_starts)` selected **3** and `test(kernel::tests)` selected **87**, both module
names, not function names. So `test(every_tracked_wat_parses)` matches
`every_tracked_wat_parses::every_tracked_wat_file_parses` (note the real fn has `file_` in it; the bare fn
name would NOT have matched had nextest keyed on fn alone). Expect E8's filters to pull the walls' unit
tests too — `no_inlined_edn` carries 19 and `no_inlined_wat_in_tests` 10 beside the corpus test — which is
extra coverage, not a mismatch.

**Runtime prediction:** ~45–75 min (5 docs-only cherry-picks; #212 and #215 are the substantial ones;
#218/#219 add small fixtures). Slower than the file counts suggest, because #212 requires DRIVING a new
gate and judging each file it reds on.

**Trap doors:** **#212** (the two-walls collision — the one step in this batch where the executor must
SUBTRACT content from the replayed commit, which is the shape most likely to be "helpfully" applied in
full); **#215** (446-line new test file → the hygiene walls); **#219** (a `docs`-classified step carrying a
`.wat` that the gate landed seven steps earlier now judges).

**What would make me reject the batch:** grok's two `historical` runes landing on un-renamed `.wat` twins
(E4) — it would mean the executor applied the commit literally and un-did main's wall; or `step-range`
reporting anything but "each present exactly once".
