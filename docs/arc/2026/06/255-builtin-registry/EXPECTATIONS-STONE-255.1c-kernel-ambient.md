# EXPECTATIONS — STONE 255.1c-kernel-ambient · written BEFORE the strike

Independent scorecard. Fixed now so the result cannot move the goalposts.

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | seven arms leave `runtime.rs` | `grep -c '":wat::kernel::\(stopped?\|sigusr[12]?\|sighup?\|reset-sig\)' src/runtime.rs` (dispatch block) | 0 remaining literal arms |
| 2 | the new home exists and is annotated | `grep -c '@Category      Ambient' src/intrinsic/kernel_ambient.rs` | 7 |
| 3 | the build is green | `cargo build --release` | exit 0 — proves every row carries a `@Category` (`MissingCategory` ⇒ `compile_error!`) |
| 4 | ★ **the purity collision fires** | `cargo nextest run --release -E 'test(/intrinsic::tests::/)'` | **`pure_declared_matches_is_effectful_op` RED on the four readers** |
| 5 | nothing else in that module regresses | same run | the other four tests green |
| 6 | `@ret` agrees with the checker | same run | `doc_arg_ret_types_match_checker_scheme` green |
| 7 | blast radius held | `git status --short` | exactly 4 paths: the new file, `mod.rs`, `runtime.rs`, and nothing else |
| 8 | routing unchanged | **orchestrator**, post-strike | `wat-tests/process/signal-*.wat` + `service-signal-observer.wat` unchanged in behaviour |
| 9 | floor · clippy · ignores | **orchestrator**, own invocation, quiescent tree | 4819/4819 · 0 · 13 |
| 10 | registered production names | **orchestrator** | 53 → 60 |

## Runtime prediction

**35–55 minutes.** Two release builds dominate. Time-box at 110 minutes (2× upper bound).

## ★ Row 4 is the stone. Read this before scoring it.

Row 4 expects a **RED**, and that inverts the usual reading of the scorecard — so it needs its
non-vacuity guard stated up front, or a red that fires for the *wrong reason* scores as success.

**The red must name the four readers and only the four readers.** `stopped?`, `sigusr1?`,
`sigusr2?`, `sighup?` — each with the message *"declares purity=Pure but is_effectful_op says
effectful=true"*. If it fires on a `reset-*!`, or on a verb outside this stone, or with a different
assertion text, that is a **different defect** and row 4 has NOT been demonstrated.

**A green on row 4 is a finding, not a pass.** It would mean the biconditional does not hold the way
the design reads it, and the design's central claim is wrong. That outcome gets reported as loudly as
the red, and this stone's conclusion changes.

## The trap-doors

- **The gate could be edited into agreement.** The single most likely bad outcome is a rider (or me,
  scoring) treating row 4's red as a chore. Home #3's design pinned it: *"making one copy the other
  destroys it."* Both `@Purity` values and `is_effectful_op` must be **untouched** in the diff —
  `git diff src/runtime.rs` showing `is_effectful_op` unmodified is the proof, exactly as home #3
  proved it.
- **`env`/`sym` unused in seven nullary bodies** may draw a clippy warning the rider cannot see (I
  own clippy). Predicted, mine to resolve, not a rider failure.
- **`stopped?` is the newcomer.** It is the one verb the kernel-decomposition table filed under
  *misc*, and the one most likely to be dropped by a rider working from that table instead of from
  `:Ambient`'s prose. Row 2 expecting **7**, not 6, is the guard.
- **A scoped filter can be blind to its own subject.** `test(/intrinsic::tests::/)` is a regex over
  the module path; all five gate fns live in `src/intrinsic/mod.rs`'s `mod tests`, so the filter
  provably reaches the load-bearing one. Row 5 naming the other four is what makes a silently-empty
  run detectable. *(This is last session's mistake, gated.)*

## What this stone does NOT get to claim

That the collision is **resolved**. It is measured. The ruling — whether `is_effectful_op` narrows,
or the gate becomes an implication rather than a biconditional, or the readers are genuinely
Effectful and the `time` precedent is what is wrong — is the builder's, and it is a separate stone.
