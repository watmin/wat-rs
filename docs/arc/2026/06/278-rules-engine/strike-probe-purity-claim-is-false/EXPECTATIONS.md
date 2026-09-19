# EXPECTATIONS — striking the false purity claim

| what | command | expected |
|---|---|---|
| the evidence re-runs | `cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat` | `COMPILE: Compiled` |
| comment-only | `git diff -U0 -- tests/rete/probe_arc278_then_user_forms_userfn.wat` | every `+`/`-` line begins `;;` |
| the probe still passes | `cargo nextest run --release -E 'test(then_user_forms_userfn)'` | unchanged, green |
| the false sentence is gone | `grep -c 'is refused today' tests/rete/probe_arc278_then_user_forms_userfn.wat` | 0 |
| both drivings cited | `grep -c '2026-08-28\|2026-09-07' <that file>` | ≥2 |
| floor | `scripts/floor.sh` | 5480, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

15–25 min, nearly all floor.

## Trap doors

- **Deleting the claim instead of striking it.** A reader who acted on that paragraph needs to see
  it withdrawn. Silent deletion leaves them believing it and unable to find it.
- **Asserting a new fact about `purity.rs`.** The paragraph died of exactly that. State only what
  `stratify.rs` records and what the scratch fixture drives.
- **Citing a line number that moves.** Prefer naming the function or the gate over a bare
  `file:line` — line citations in this arc have gone stale twice in one day, and this file is
  gated by nothing that would catch it.
