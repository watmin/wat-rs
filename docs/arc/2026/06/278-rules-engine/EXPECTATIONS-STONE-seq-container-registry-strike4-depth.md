# EXPECTATIONS — seq-container registry strike 4 (depth fix)

Independent scorecard, written before the strike. Graded against my own re-run, not the worker's report.

| # | what | command | expected |
|---|------|---------|----------|
| 1 | builds clean | `cargo build --release` | green, no new warnings |
| 2 | **compile-forcing now holds** | add `ProbeDummy,` to `SeqContainer`, `cargo build` | non-exhaustive error at **all 11 dispatch sites** + the 4 capability methods (was: 4 capability methods only). Then remove → green. |
| 3 | no test regression | `cargo test --release` | matches HEAD baseline (captured this session — see below), zero new failures |
| 4 | parity intact | `cargo test --release --test probe_seq_container_parity` | green (checker ≡ runtime, unchanged) |
| 5 | registry net intact | `cargo test --release --test probe_seq_container_registry` | green |
| 6 | first-bare intact | `cargo test --release --test probe_first_bare_accessors` | green |
| 7 | clippy clean | `cargo clippy --release` | no new warnings |
| 8 | behavior unchanged | the full collection suite | identical pass set to HEAD (this is behavior-preserving) |

**HEAD baseline:** captured this session via `cargo test --release` before spawning (FM 9). Fill the real
numbers into the SCORE; #3 must match them exactly.

**Runtime prediction:** 8–15 min. Mechanical, 11 same-shaped sites, helpers reused.

**Trap-doors (named):**
- `vector_concat_inner` (`eval.rs:761`) has **two** `of_value` calls (left + right) and asserts matching kinds —
  the trickiest site; the `match container` conversion must preserve the left/right-kind logic byte-for-byte.
- The positional accessor (`runtime.rs:10961`) is **inlined** (no helper) → needs `let Value::X = &v else {…}`
  extraction per arm; easy to fumble the `&v` vs `v` borrow.
- `WatAstList` arms carry a nested `match &*ast { WatAST::List(..) => .., _ => unreachable!() }` — that inner `_`
  is over `WatAST` and **stays** (correct shape). Only the outer `match value` catch-all is the target.
- Risk a worker "improves" by deleting the capability gate (`if container.X()`) — DON'T; it's the checker's
  single source of truth (STOP-3). Keep it; the excluded containers get a **named** `unreachable!` arm.

**The one contract decision (from DESIGN):** inner dispatch is `match container` over the closed `SeqContainer`
enum, exhaustive, **no `_`**. That is the whole strike. Everything else is behavior-preserving.
