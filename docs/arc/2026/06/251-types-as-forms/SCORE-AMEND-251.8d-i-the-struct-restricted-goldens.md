# SCORE — AMEND 251.8d-i: three stale goldens + two stale doc comments

Folded into the stone. **No follow-up commit.** Stone is now `a83eae411` (was `22118e145`).
AMEND brief remains `66f9864d9` on top. **Not pushed.** Floor / workspace clippy / census:
orchestrator's row.

## The red

Three `wat::types struct_restricted` goldens, all the same diagnostic sentence. Structural
fields (`:callee`, `:enclosing-fn`, `:prefixes`, `:location`) were already right. Did **not**
revert `src/check/error.rs:721`.

## Regen, not a hand-typed sentence

Path: `UPDATE_EDN=1 cargo test --release -p wat --test types struct_restricted`
(`assert_edn_matches_file!` in `src/lib.rs` — `UPDATE_EDN` writes pretty EDN after parse).
Three files, one line each. Re-ran without `UPDATE_EDN`: **9 passed**.

## The two comments

`tests/types/struct_restricted.rs` and `tests/kernel/wat_arc198_def_restricted.rs` now say
the `/` discriminator. The kernel file's tests were already green; only a reader would have
noticed.

## Walls I ran (not the floor)

- `cargo test --release --offline -p wat --test types struct_restricted` — 9 passed.
- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — exit 0.

Floor + workspace clippy + `census.sh --diff`: **not run** (brief: orchestrator, uncontended).
Do not push. Do not start 8d-ii.
