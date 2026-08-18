# EXPECTATIONS — STONE 118.B4-i · widen `nth` to Seqable

Written before the strike. The scorecard cannot move.

| # | what | command | expected |
|---|---|---|---|
| 1 | `nth` answers on all four containers | the new deftests | Vector / PersistentVector / List / Stream agree at 0, middle, last |
| 2 | past-the-end raises **by name**, all four | the new deftests | `"nth: index out of range"` |
| 3 | ★ Stream `nth` visits exactly **i+1** cells | force-counting deftest | `i+1` FORCED lines |
| 4 | Vector path unchanged | floor | every pre-existing `nth` caller green, no edits to their files |
| 5 | four arms register | `cargo build --release` + floor | no `UnreachableClause` |
| 6 | floor | `scripts/floor.sh` | **≥4747 run, 0 FAIL, 19 skipped** |
| 7 | clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
| 8 | ignores unchanged | `grep -rn '^\s*#\[ignore' --include=*.rs tests/ src/ crates/ \| wc -l` | 13 |
| 9 | blast radius held | `git status --short` | `wat/core.wat` + new test file only |

**Row 3 is the load-bearing one.** Rows 1 and 2 would pass on an implementation that realizes the
whole stream and indexes it — which would reintroduce exactly the retention B3 deleted. Only the
force count distinguishes a walk from a drain.

## Independent prediction

**25–40 minutes.** One `defn` → `defclause` with four arms plus a helper, in one file, copying a
shape that already exists two files away. The tests are the bulk of it.

## Trap-doors named in advance

- **The three O(1) arms are byte-identical modulo receiver type.** That is expected and tracked (the
  "eager indexable container" gap the 294 seam records for `reduce`). A rider that collapses them by
  inventing a new type has exceeded scope; a rider that reports the duplication has done right.
- **`:wat::WatAST` is `gettable()` but is NOT one of `Seqable`'s four `extend-type`s.** So `nth` on a
  raw `WatAST` reaches no arm. Out of scope — the corpus uses `ast->children`, which returns
  `Vector<WatAST>` and hits arm 1. Do not add a WatAST arm here.
- **A Stream `nth` is O(n) and that is honest**, not a regression: `seq_container.rs:65` already
  states Stream has no O(1) nth. Do not add a cache to make it look faster.
