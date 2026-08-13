# EXPECTATIONS — STONE 251.8a-ii

Written **before** the strike.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | `$bound/x` is refused | run `./target/release/wat` on a file containing `($bound/x 1)` | non-zero exit, a **located** parse error naming the spelling |
| 2 | **the refusal is not vacuous** | the RED probe, mutated then restored | probe RED with the check removed; GREEN restored. **Mutation result reported explicitly** |
| 3 | **positive control — `$` still works** | the same probe: a `$x` binder and a `:foo$impl`-style keyword | both parse clean |
| 4 | the constant is used, not re-spelled | `git diff` | the check reads `BOUND_NAMESPACE`; the literal `"$bound"` appears nowhere new |
| 5 | one site | `git diff --stat` | `parser.rs` + at most one file for the error variant. **No lexer change.** |
| 6 | the corpus is untouched | `git diff --stat` | zero `.wat` / `.wat.bad` files |
| 7 | build | `cargo build --release` | exit 0 |
| 8 | lint | `cargo clippy --release --all-targets` | zero warnings |
| 9 | the message teaches | read the error text | says the namespace is substrate-minted, and that a local is written **bare** (`x`, not `$bound/x`) |
| 10 | **floor** | orchestrator's own `scripts/floor.sh`, Summary read by hand | zero new failures against 4393/4393. A changed count in either direction is a finding |

**Row 3 is the one that matters most and it is not the obvious one.** Without it, the probe cannot
distinguish "refused `$bound/`" from "refused `$`" — and refusing `$` is a thing the orchestrator
already wrongly reached for once in this stone's own history. Row 3 is the guard against repeating it.

## Runtime prediction

**10–20 minutes.** One match arm, one error variant, one probe. Predicted overrun cause: STOP-1 —
the namespace split not being cleanly available at the parser without duplicating logic.

Time-box: 40 minutes.

## Trap doors — named in advance

- **The `$`-vs-`$bound` conflation.** The whole stone is one namespace. A check that fires on a
  leading `$`, or on any `$` in a namespace segment, is WRONG and row 3 is what catches it. This is
  not hypothetical: the orchestrator measured "is banning `$` free?" earlier in this stone's history
  and reported it as if it answered the builder — it did not, and the builder's correction is why
  this row exists.
- **A second hand-rolled `rfind('/')`.** 251.8a existed to collapse four of those into one door. If
  the parser needs the namespace segment and cannot reach `BOUND_NAMESPACE`'s door cleanly, that is
  STOP-1 — report it, do not quietly add a fifth.
- **A refusal without a span.** The parser has `ParseError { span, kind }`. An error that reaches the
  user unlocated is the weaker half of the win; row 1 says *located* for that reason.
- **`nil`'s arm is the precedent, and it is a bare-spelling check.** `$bound/x` is a *namespace*
  check, not a whole-token equality. Copying `nil`'s shape too literally gives a check that only
  fires on the exact token `$bound` with no name after it.

## What this stone does NOT claim

It does not make the namespace *stored* — that is 251.8b. It does not touch `$impl`, `$x`, or any
other `$` use. It does not close #95. It closes exactly one hole: user source can no longer forge a
symbol in the binder namespace.

It also does not resolve #99's second reading — whether a genuinely-unbound local should report
earlier. That question survives this stone and stays open.
