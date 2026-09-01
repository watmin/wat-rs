# EXPECTATIONS — a surplus argument is not a value to place, it is a question to refuse

> Written **before** the strike. Scored against the orchestrator's own re-run, never the
> executor's report.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,195 plus every arm you drive. Exceeding it is a PASS.** Report the final
number; do not tune the work to hit one. (Two strikes ago a pinned count silently capped a rider's
coverage — it reported that a fourth arm *"would have falsified your own scorecard row before you
ran it"*.)

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | control green before | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all green |
| 2 | the untampered fixture answers 1 | a probe calling `:user::import-and-hits` on `cool-export` | **1** — the mouth works and the fixture is live |
| 3 | arm 1 RED before | its own probe | **FAIL by a WRONG ANSWER** — import accepted, hits = **0** |
| 4 | arm 2 RED before | its own probe | **FAIL by a DROPPED ARG** — import accepted, hits = **2** |
| 5 | arm 3 RED before | its own probe | **FAIL** — the refusal is `UnboundSymbol: "slot 1"`, not an `ArityMismatch` |
| 6 | all three GREEN after | the three probes | refused with `ArityMismatch`, **counts named** |
| 7 | the surplus branch is gone | `grep -n 'else if i < inner.len()' src/rete/expr_ir/eval.rs` | **no hit** — deleted, not bounds-checked |
| 8 | no second copy of the invariant | `git diff --stat` | `export.rs` is **NOT** in the diff. A sixth import wall is out of scope by the ★ decision |
| 9 | blast radius | `git diff --stat` | `expr_ir/eval.rs` + `probe_arc278_export.rs`. Nothing else |
| 10 | the floor | `./scripts/floor.sh`, Summary from the captured log | **≥ 5,195 + every new arm**, zero FAIL rows, exit 0 |
| 11 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof — one per arm, and the arms are named

Rows 3/4/5 → row 6 must prove three *different* pre-fix behaviours: a wrong answer, a silently
dropped argument, and a diagnostic naming an internal slot. They are one missing check with three
faces, and a single probe proves one face.

Per arm, state: **proven** (driven, red→green), **reachable but not driven**, or **not reachable,
and why**. An unreached arm named as unreached is a pass; an unreached arm not mentioned is a fail.

**Row 5 is the trap.** Arm 3 already errors before the fix, so a probe asserting merely "this
errors" passes in both states and proves nothing — the shape this arc has caught twice. It must
assert the error is an `ArityMismatch` carrying both counts.

One further mutation, cheap: **restore the `else if` branch while keeping the length check.** All
three probes must stay green (the branch is now unreachable). If any goes red, the check is not
where the design says it is.

## Runtime prediction

30–45 minutes; smaller than the last two. Three or four release builds at ~1–3m, one floor at
~400s. The change itself is perhaps 12 lines; the probes are the work.

## What would make this strike a failure even if every test passes

**A bounds-check instead of a deletion.** Making the surplus write safe — clamping it, guarding
it, skipping it — leaves an argument with no parameter *meaning* something, and the next reader
finds a branch that looks deliberate. The whole finding is that the branch should not exist.

The second failure shape: **the check added at the import door instead of at `exec_program_on`.**
That would refuse the tampered exports and turn all three probes green while the executor still
holds no arity invariant — a green wall with the native and `foldl` doors still assuming. Row 8
exists to catch exactly that, because rows 3–6 cannot.
