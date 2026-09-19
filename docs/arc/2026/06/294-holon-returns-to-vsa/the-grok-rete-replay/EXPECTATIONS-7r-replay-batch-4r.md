# EXPECTATIONS 7r — replay batch 4r, grok-rete #481 → #500 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed against HEAD `0038bbf83` and ALL PASS:** E10 (origin an ancestor; `refs/original/` empty), E11
(0 replace refs), tree clean, 480 REPLAY steps, E2 (8 docs-only), E2b (12 code), E5 (zero hazard rows; no
`wat-scripts/fixes/` edit; both M-status-absent paths created earlier in the range), the `#[test]`/
`#[ignore]` deltas (**+5 / −3**, the −3 all at #498), and the #498 measurement: grok's
`benches/binding_repr.rs` is `harness = false`, keeps only a faithfulness `assert_eq!` and the
non-vacuity `assert!`, and states its timing-ordering assertions are gone.

**Predictions, not verified rows:** E3–E4, E6–E9, E12–E18.

⛔ **Two things I could NOT measure and did not pretend to.** (1) **Our** emitted-counter set against
#496's new EMITTED ⇒ READ gate — grok's "all 25 got readers" is grok's number, and this tree ran the
whole census campaign with its own counts. (2) Whether the slim floor test required by the #498 ruling
can carry the surviving assertion without dragging the benchmark harness back in. **E4 and E3 score
measuring both and reporting what they say.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 0038bbf83 HEAD 481 500` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | **trailers COPIED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **8** (#483 #485 #489 #491 #493 #495 #497 #499) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 12 code steps | each carries ≥1 non-docs file |
| E3 | ⛔ **THE #498 RULING LANDED** | `git show` #498; `-E` on the kept test | the bench move landed in full (`benches/binding_repr.rs`, `[[bench]]`, `matcher.rs`); **the small-end GET ordering assertion and its non-vacuity companion still run ON THE FLOOR**, green, N > 0; the record above them names the 4i strike, #472 and the margin |
| E4 | ⛔ **#496's gate is GREEN and OUR set was measured** | `-E 'test(census_emitted_name_is_read_or_declared)'` | green, N > 0; the SCORE states **our** emitted-counter count, not grok's 25, and disposes of any unread counter as a finding — never by deleting a live counter |
| E5 | ZERO hazard paths | `git diff --name-only` over the range | none; `wat/` touched only at #486/#490, one file each |
| E6 | **#486's `wat/` edit really is comments-only** | `git show` #486 on `wat/` | every changed line under `wat/` is a comment line; a code line is reported |
| E7 | **the deltas landed, not the blobs** (finding 36) | the SCORE | delta-vs-delta for every conflicted step |
| E8 | finding 33's class swept per code step | the SCORE | explicit per step; every new `.wat` cured via `convert.sh`, never hand-edited |
| E9 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row**, and `--all-targets` is what compiles the new bench |
| E10 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E11 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E12 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | **+5 from #482/#488/#496, −3 at #498 minus the one the ruling KEEPS = −2**, so **5867 run, 21 skipped**: the three ignores leave with the bench move, and the kept test is not ignored. ⚠ This encodes the ruling; if the executor's own count disagrees it must say so |
| E13 | **the SCORE discloses what BOUGHT each green** | read the SCORE | the #498 divergence, the kept assertion, every conversion |
| E14 | **every `census:` line is TRUE, and on ONE line** | `census.sh --diff`; grep each verdict | `no STOP-8`; no wrapped verdict |
| E15 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E16 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E17 | **NO COUNTERPART ACTIVITY**; no unfiltered run; no `cargo bench` | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E18 | **a red was never re-run into green** | the SCORE | any red carries its verbatim block and arm, and the batch STOPPED |
| E19 | no knowingly-red commit | every step green at its own landing | none |
| E20 | **commit messages survived their heredoc** | `git log --format=%B` per step | no eaten backtick spans |

## ⛔ E3 and E4 are the rows that catch a lazy 4r

#498 applies cleanly if you let it — grok's hunk deletes the function and the floor stays green, because
nothing fails when a gate stops existing. **E3 fails unless the kept assertion is still running.** And
#496 goes green trivially if our counter set happens to match grok's — **E4 fails unless the SCORE shows
our own measured number.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~110–160 min. The weight is #498 (the bench move plus the kept test), #496 (a new
gate over our own census set), and the four grid steps.

**What would make me reject the batch:** the kept assertion gone from the floor; grok's 25 transcribed as
ours; a live counter deleted to green the new gate; a hand-edited `.wat`; a wrapped verdict line; a
paraphrased subject or typed trailer; any `refs/original/` entry; any `pulsare_yield`.
