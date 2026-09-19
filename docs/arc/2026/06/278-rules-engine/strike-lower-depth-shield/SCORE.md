# SCORE — STOP-3. LowerCx refuses the quote door; 2 MiB abort is the parser

A shared depth budget on `LowerCx` binds `EXPANSION_DEPTH_LIMIT` and refuses
`:wat::rete::lower` of a quoted tree by kind. On an 8 MB stack a 3000-deep quoted
fixture is a clean refusal at depth 513. On a 2 MiB stack the same fixture is still
`rc=134` / `fatal runtime error: stack overflow` — **before `lower` runs**. STOP-3.

## STOP-3

EXPECTATIONS: `ulimit -s 2048; wat <3000-deep quoted fixture>` must be a clean
refusal naming the depth, **not** rc=134.

```
ulimit -s 2048; ./target/release/wat /tmp/arc278-lower-depth-3000.wat
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
rc=134
```

Same file with the deep quote in an **unused** defn (main only prints `1`) also
aborts under 2 MiB. Expansion does not walk quote children (`AllData`). `lower`
is never reached. **The parser (or freeze's walk of the parse tree) recurses
first.** The 1520/1539 bisect on the quote+lower fixture was this stack, not
`lower`'s.

On the default 8 MB stack the 3000-deep lower fixture is:

```
malformed :wat::rete::lower form: lowering nesting depth 513 exceeds EXPANSION_DEPTH_LIMIT 512
rc=1
```

So the budget does close the door `lower` itself sees (50,000-deep accepted on
8 MB, gone). It cannot close an abort that happens in parse.

Mutation (raise `LowerCx` bound to 100_000, expect abort under 2 MiB) was not
run: parse dies first, so that mutation would not isolate `lower`.

## What landed anyway

- `LowerCx.depth` + `deeper`, bound `EXPANSION_DEPTH_LIMIT` (not a restated 512).
- Shared across `lower_list` / `lower_hof_callee` / `lower_pat`. `lower_expr` does
  **not** increment: it dispatches into `lower_list` on the **same** List node;
  counting both would double-count and refuse source expansion already accepts
  (that would have been STOP-2).
- `LowerErrorKind::DepthExceeded` — a refusal, not a panic.
- Header records all three doors.
- Probe `tests/rete/probe_arc278_lower_depth_shield.rs`: quote accepts 8, quote
  `LIMIT+1` names `lowering nesting depth` + `EXPANSION_DEPTH_LIMIT`; source
  `LIMIT - HARNESS_FORMS` still compiles (`HARNESS_FORMS = 4`; `LIMIT - 3` was
  already the expander); source past that is `ExpansionDepthExceeded`, not
  lowering.

No existing test reddened (STOP-2 of v2 did not fire). No `EXPANSION_DEPTH_LIMIT`
or `export.rs` change.

## Scorecard

| # | result |
|---|---|
| 1 shared budget | **HOLD** on the three composite entries. |
| 2 bound not restated | **HOLD.** `grep -c 512 src/rete/expr_ir/mod.rs` → 0. |
| 3 2 MiB abort gone | **FAIL. STOP-3.** Parser, unused-quote fixture, rc=134. |
| 4 8 MB 3000-deep | **HOLD.** Clean refusal at depth 513. |
| 5 compile path | **HOLD.** Source `LIMIT - 4` compiles; `LIMIT - 3` is the expander. |
| 6 probe | **HOLD.** 4 tests. No `509`/`510` in the file. |
| 7 mutation | **not run** — parse dies first. |
| 8 floor | **HOLD.** `.floor/2026-09-09T00-27-34Z/`: `Summary [ 488.091s] 5484 tests run: 5484 passed (3 slow), 19 skipped`. +4. First floor RED `no_loose_string_assert` line 120; rune added; this floor green. |
| 9 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

## What this did not do

No parser depth budget. No `EXPANSION_DEPTH_LIMIT` change. No `export.rs`. The
2 MiB abort remains a parse-stack problem; a bound in `LowerCx` cannot catch it.
