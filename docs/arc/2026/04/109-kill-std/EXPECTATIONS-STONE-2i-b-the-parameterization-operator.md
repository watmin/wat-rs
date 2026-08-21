# EXPECTATIONS — arc 109 Stone ②-i-b: `:-`, the parameterization operator

Written BEFORE the strike, against `c557e34b5`. Scored after the orchestrator's own independent
re-run; the rider's report does not move a row.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the RED baseline turns green | `--check` a defn with `(wat.type/HashMap :- [wat.type/String wat.type/i64])` in a param slot | EXIT 0 (RED at HEAD: *"function-type bracket needs a `:->` arrow"*) |
| 2 | the renderer emits the operator | `./target/release/wat ./wat-scripts/scratch-pad/arc109-tuple-arm-faults.wat` | the six rows below, exactly |
| 3 | the reader takes what the writer emits | `--check ./wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat` (rider adds `:-` rows) | EXIT 0 |
| 4 | ★ the unmarked form still reads | the same probe's existing bare-bracket rows | EXIT 0 — **dual-read; this is the load-bearing preservation row** |
| 5 | ★ the angle form still checks | a `Vector<i64>` annotation | checks — ② adds spellings, removes none |
| 6 | ★ the positional form still builds | `(:wat::core::Vector :i64 1 2 3)` | `[1 2 3]` |
| 7 | the constructor takes the operator | `(:wat::core::Tuple :- [:wat::core::keyword :wat::core::keyword] :some :keyword)` | a built 2-tuple of keywords |
| 8 | the verb's contract suite | `cargo nextest run --release -E 'test(contract_0)'` | 9 pass, 0 fail |
| 9 | the floor | `scripts/floor.sh` | **0 FAIL** after the goldens the floor itself named are updated |
| 10 | ★ a literal in a TYPE slot is a NAMED error | `--check` `[p :- (wat.type/Tuple :- [wat.type/i64] 42)]` | a diagnostic naming *a literal in a type position* — NOT "function-type bracket needs a `:->` arrow" |
| 11 | ★ the same form in a VALUE slot is legal | `(:wat::core::Tuple :- [:wat::core::i64 :wat::core::keyword] 42 :some-keyword)` | a built 2-tuple |
| 12 | ★ the EMPTY tuple LITERAL is writable | `(:wat::core::Tuple :- [])` in value position | an empty tuple — **not** `[[]]`, which is what `(:wat::core::Tuple [])` gives today |
| 13 | a 2-tuple of keyword VALUES is writable | `(:wat::core::Tuple :- [:wat::core::keyword :wat::core::keyword] :a :b)` | a built 2-tuple — today `(Tuple [:a :b])` dies on `expected 2, got 0` |
| 14 | clippy | `cargo clippy --all-targets -- -D warnings` | 0 |

Row 2's expected output, predicted exactly:

```
1 nil bare       : :wat::core::nil
2 nil nested     : (:wat::core::Result :- [:wat::core::nil :wat::core::String])
3 tuple 3-ary    : (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
4 tuple 1-ary    : (:wat::core::Tuple :- [:wat::core::i64])
5 tuple empty    : (:wat::core::Tuple :- [])
6 control parm   : (:wat::core::Vector :- [:wat::core::i64])
```

Row 6 is no longer a pure control — the `Parametric` arm moves too. **The control is now rows 4–6
of the scorecard**: every old spelling still works. If any of those three breaks, the stone became
a hard-cut and that is ③'s job, not this one.

Row 7 matters because it is the shape the builder wrote down, and because ①b's rider proved the
runtime has its OWN constructor arms — a green check with a failing build is exactly the divergence
that stone found. **Demand a built value, not a green check.**

## Independent prediction

**25–40 minutes.** Larger than the previous draft: two shared helpers collapse into one door, twelve
call sites re-point across two files, plus the renderer, the parse production, and a golden pass
whose size is unknown until the floor runs. Mechanical throughout — every room is exact and the
one-door helper means the twelve sites are a re-point, not twelve judgments.

## Trap-doors, named in advance

1. ★ **The runtime's own constructor arms.** `runtime.rs:6257/6479/6494` are the twins of
   `check.rs:14062/14165/14330`. ①b's brief missed this room entirely and the symptom was
   *check-says-yes / runtime-says-no*. Row 7 exists to catch a repeat.
2. **The sniff running when `:-` is present.** If the `:-` path still consults
   `is_type_bracket_candidate`, everything goes green and the stone delivers nothing — the guess
   survives behind a marker that was supposed to retire it. STOP-4 covers it; verify by reading the
   helper, not by reading the floor.
3. **A golden rewritten from observed output.** Any golden that gets `cargo insta`-style blessed
   from what the binary printed cannot fail. Each red golden must match the row-2 table's shape.
4. **`reject_any` dropped** from the copied preserving entry point — silently opens an `:Any` hole
   on the renderer path, and the floor would not necessarily catch it.
5. **The empty rung.** `(:wat::core::Tuple :- [])` is a first-class rung, not a defensive branch.
   A special-case `if args.is_empty()` anywhere in the arm is a defect even if it prints correctly.
6. ★ **The sniff's `!items.is_empty()` guard leaking into the `:-` arm.** Rows 12 and 13 both die
   if it does, and row 12 dies SILENTLY — `(Tuple :- [])` would keep meaning `[[]]` while every
   other row goes green. The two arms cannot share one rule; verify by reading
   `split_type_param_bracket`, not by reading the floor.

## What makes this a Mode B

Any of: the unmarked bracket stops parsing (rows 4–6) · the angle form stops checking · `Fn` or
`Path` arms touched · `src/types.rs:4728` edited · `src/collection/eval.rs` touched · a golden
blessed from output · the sniff still consulted on the `:-` path · cargo run by the rider.
