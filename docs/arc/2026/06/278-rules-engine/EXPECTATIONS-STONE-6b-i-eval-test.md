# EXPECTATIONS — Stone 6b-i: `eval-test`

Independent scorecard, fixed before the strike. Weighed against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe greens | `cargo test --release -p wat --test probe_arc278_6b_eval_test` | **7 passed; 0 failed** |
| 2 | lib floor holds | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | 941 passed / 36 failed (unchanged) |
| 3 | 6a probe still green (no regression) | `cargo test --release -p wat --test probe_arc278_6a_purity \| grep "test result"` | 19 / 0 |
| 4 | deftest floor | `cargo test --release --test test \| grep "test result"` | 264 / 1 |
| 5 | build clean | `cargo build --release` | builds; warnings ≤ 25 (no NEW from matcher.rs) |

## Load-bearing assertions (weighed by eye, against the disk)

- **#4 computed operand** `(> (- ?hi ?lo) 10)` → the proof eval-test is a real evaluator, not a 2-operand
  comparison (it reaches arithmetic over bindings).
- **#6 user-fn predicate** `(:test::big? ?x)` → THE point: a rule filters with the user's own fn. Proves
  `eval_inner` reaches user defns with `?vars` bound from the token. (If STOP-1 fires — `?`-symbols don't
  resolve from the child env — this is where it shows.)
- **#7 non-bool → error** — a `where` is a predicate; `(+ ?a ?b)` must `TypeMismatch`, not silently coerce.
  (At HEAD this passed for the WRONG reason — UnknownFunction; after 6b-i it must pass because eval-test
  ran and rejected the non-bool. Confirm the error KIND is TypeMismatch on `eval-test`, not UnknownFunction.)

## Diff integrity

- `git diff --stat` shows ONLY `src/rete/matcher.rs`, `src/runtime.rs`, `src/check.rs` (+ the probe,
  already committed). Read `runtime.rs`/`check.rs` diffs by eye — one arm, one scheme; nothing else moved.

## Runtime prediction

5–12 min. Trap-doors: (a) STOP-1 — `?`-symbol resolution from a child env is the assumption the whole
approach rests on (grounded plausible: `eval_inner` resolves `Symbol` via `env.lookup`, runtime.rs:3616);
(b) the `TrackedValue`-from-`Value` constructor; (c) PersistentMap iteration variant/shape.
