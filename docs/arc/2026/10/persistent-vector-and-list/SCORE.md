# SCORE — persistent Vector and List

Clone: `/var/tmp/wat-rs-009` (CARGO_TARGET_DIR=/var/tmp/wat-rs-009-target).
Branch: `the-little-wat-persistent` off `origin/the-little-wat` @ 4f6ebcf12.
GitHub remote `github` pushed after each green commit.

## Baseline

- `cargo build --release` on unmodified `the-little-wat-persistent` @ 4f6ebcf12: green, 4m31s.
  Binary saved at `/var/tmp/009-wat-baseline` for benchmark comparison.
- Floor (`NEXTEST_TEST_THREADS=4 cargo nextest run --release`), clean tree:
  **5405 tests run: 5403 passed (4 slow), 2 failed, 22 skipped** (1616s). The 2 failures are
  exactly the known lint reds named in the brief:
  `wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat`,
  `wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert`.
  Matches the brief's recorded baseline exactly. CONFIRMED.
- Gotcha hit and fixed: a first attempt at this baseline nextest run was started while
  `src/value/pvec.rs` / `src/value/value.rs` were already edited for Row V (background wait +
  foreground edits raced). nextest's own build step picked up the modified source mid-run and
  failed with `PVec` compile errors unrelated to a real baseline. Stashed the edits
  (`git stash push -u`, which also stashed this untracked `docs/arc/2026/10/` — recovered with
  `git stash pop` after), reran nextest on a verified-clean tree (`git status --short` empty)
  to get the number above, then popped the stash back. Lesson recorded here for future
  executors on this clone: never run a gate concurrently with edits to the same tree.

## Row V — Vector -> PVec

Status: IN PROGRESS.

## Row L — List -> rpds::ListSync

Status: NOT STARTED.

## Benchmark

Status: NOT STARTED. Will live at `wat-scripts/bench/conj-build.wat`.

## Stops

None yet.
