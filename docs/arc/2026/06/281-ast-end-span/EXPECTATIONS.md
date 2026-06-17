# EXPECTATIONS — Arc 281 (weigh on the orchestrator's own build)

The blast radius touches `Span` — every AST node, every error. The load-bearing weigh is that the
START positions (`ast-span`) are byte-identical; only the END is new.

| what | command | expected |
|---|---|---|
| end-span gate | `cargo test --release -p wat --test probe_arc281_ast_end_span` | 1 passed / 0 failed (`(a b c)` end :col == 8) |
| lib floor | `cargo test --release -p wat --lib -- --test-threads=1 2>&1 \| grep "test result"` | **929 passed / 36 failed — UNCHANGED** |
| nursery floor | `cargo test --release -p wat --test nursery -- --test-threads=1 2>&1 \| grep "test result"` | **893 passed / 4 failed — UNCHANGED** |
| deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | **259 passed / 1 failed — UNCHANGED** |
| deporder gate | `cargo test --release --test test_stdlib_load_order 2>&1 \| grep result` | 1 passed / 0 failed |

Runtime prediction: 25–40 min (wide but mechanical; the risk is the span ripple, not algorithmic depth).

## Trap-doors named

- **START regression (the big one)** — any change to a node's start line/col will move dozens of
  span-asserting tests. The lib/nursery/deftest counts are the tripwire: they must NOT change. If a
  passed-count drops by even 1, a span regressed (STOP-1).
- **Off-by-one on the end** — end is ONE PAST the last char (exclusive), so `(a b c)` (cols 1-7) ends at
  col 8, and `foo` (cols 1-3) ends at col 4. `old-len = end-offset - start-offset` must equal the form's
  char length. Verify with the probe (col 8) + an atom case.
- **Multi-line forms** — a form spanning lines: `end_line > line`, `end_col` is the col on `end_line`.
  `fix-text-offset-of` already handles arbitrary line/col, so this composes.
- **`Span::new` default** — must set `end = start` so the ~15 existing call-sites compile AND so error
  spans (which never set a real end) render unchanged.
- **Clone/Debug/PartialEq on Span** — adding fields: ensure derives still hold and no test compares
  whole-Span equality in a way the new fields break (if so, that's a real find — report it).

## Definition of done

The end-span gate is green; ALL four floors are byte-identical to baseline (no count moved);
`ast-end-span` mirrors `ast-span` (impl + dispatch + scheme + allow-list); `ast-span` start positions
unchanged. Pure substrate — no wat-source edits. The auto-fixes that consume this (277.1b, concat→format)
are the NEXT stones, named, not built here (exigere: no deferral-prose in the code).
