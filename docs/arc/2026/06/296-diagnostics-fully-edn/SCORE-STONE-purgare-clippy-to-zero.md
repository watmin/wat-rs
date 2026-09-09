# SCORE — STONE purgare: clippy to ZERO

## Result

`cargo clippy --release --all-targets -- -D warnings` — **EXIT 0.**

Baseline (before this stone): exit 101, exactly the 5 `dead_code` errors named in the brief, no others.

## Cascade

6 sites total, 2 build-delete rounds. Full detail: `TABLE-STONE-purgare-the-cascade.md`.

- Round 1: the 5 named items (`pattern_coverage`, `Coverage::Wildcard`, `try_match_pattern_ast`,
  `substitute_many`, `MatchArm::Binding::ident_span`) deleted verbatim per the brief's diagnosis.
- Round 1's build surfaced exactly one new problem: an `E0599` compile error (not a fresh `dead_code`
  warning) at `src/check.rs:6453` — the `Some(Coverage::Wildcard) => {...}` match arm in `infer_match`,
  orphaned by the `Wildcard` variant's deletion. Confirmed unreachable before deletion too: the only
  function feeding that match (`cover_variant_arm`) never constructs `Wildcard`.
- Round 2: deleted that arm. Build succeeded clean; clippy exited 0 on the first pass after — no
  further `dead_code` (or any other) warnings appeared.

Well under the ~20-item STOP-2 "campaign" threshold.

## STOP triggers — none fired

- **STOP-1** (behavioural change required) — did not fire. The one cascade edit (the `Wildcard` match
  arm) removed an already-unreachable branch; `wildcard_seen` is still set from three other live arms
  untouched by this stone. No live code's output, control flow, or observable behaviour changed.
- **STOP-2** (cascade > ~20 items) — did not fire. 6 sites, 2 rounds.
- **STOP-3** (`#[allow(dead_code)]` would suffice) — did not fire; not used anywhere. Every item was
  deleted, not suppressed.
- **STOP-4** (reachable via macro/`#[cfg]`/test-only/reflection) — did not fire. Checked each of the
  5 items and the cascade item with `grep -rn` across `--include="*.rs"` (not just `src/`) for every
  name before and after deletion; remaining hits were prose comments only, plus one unrelated
  same-named local variable in `src/macros/expand.rs` (different struct, different function). No
  test-only (`#[cfg(test)]`), macro-expansion, or reflective caller existed for any deleted item.

## Verification run

```
cargo clippy --release --all-targets -- -D warnings         EXIT 0    (confirmed twice)
cargo nextest run --release -E 'test(a2_a_variant)'          15 passed, 0 failed, 5298 skipped
cargo nextest run --release -E 'test(p1_annotation)'         10 passed, 0 failed, 5303 skipped
cargo nextest run --release -E 'test(p1b_a_parametric)'       4 passed, 0 failed, 5309 skipped
cargo nextest run --release -E 'test(p2prereq)'               4 passed, 0 failed, 5309 skipped
cargo nextest run --release -E 'test(p3_one_question)'        5 passed, 0 failed, 5308 skipped
cargo nextest run --release -E 'test(a1_one_rule)'            4 passed, 0 failed, 5309 skipped
```

The five stone filters (`p1_annotation`, `p1b_a_parametric`, `p2prereq`, `p3_one_question`,
`a1_one_rule`) match the brief's expected counts exactly: 10 / 4 / 4 / 5 / 4.

## Honest delta: the `a2_a_variant` acceptance row

The brief's acceptance table states `cargo nextest run --release -E 'test(a2_a_variant)'` should show
**16 passed, 0 skipped**. The live run — both after this stone's edits AND on the unmodified tree
(verified via `git stash` / re-run / `git stash pop`) — shows **15 passed**, not 16. This is a
pre-existing discrepancy in the brief's acceptance number, not a regression introduced by this stone:
the filtered count was already 15 before any file in this stone was touched. Reporting it rather than
silently rounding it to match, or silently fixing the row.

("0 skipped" in the brief likely meant "no test inside the filtered group itself was skipped," not
literally 0 in nextest's report — nextest reports every test outside an `-E` filter as "skipped" by
design, which is why every row above shows several thousand skipped.)

## Files changed

- `src/check.rs` — deleted `pattern_coverage` (incl. doc comment), the `Coverage::Wildcard` variant,
  and the orphaned match arm that consumed it. Net: −452 / +0 lines (`git diff --numstat`).
- `src/runtime.rs` — deleted `try_match_pattern_ast` and `substitute_many` (incl. their doc comments).
  Net: −147 / +0 lines.
- `src/match_arm.rs` — removed the unread `ident_span` field from `MatchArm::Binding` and its
  construction site. Net: −6 / +1 lines (one construction site collapsed to a single line).

No signature of any *live* function changed. No `#[allow(dead_code)]` added anywhere. Not committed —
per instruction, left for the orchestrator.
