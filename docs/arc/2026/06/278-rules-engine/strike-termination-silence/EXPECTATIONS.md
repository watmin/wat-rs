# EXPECTATIONS — a verdict that cannot say "I did not look" is not a verdict

> Written **before** the strike. **Every row's command was run against HEAD first and its
> pre-value is recorded below** — three scorecard rows this session could not do their job (one
> bounded the work, one could not see the defect, one could never return the value it demanded),
> and the cure is to run the row, not to phrase it more carefully.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,203 plus every arm you drive. Exceeding it is a PASS.** Report the final
number.

## The scorecard, with pre-values measured at HEAD `09b973d2c`

| # | what | command | pre-value AT HEAD | expected after |
|---|---|---|---|---|
| 1 | the repro | `cargo run --release --bin wat -- wat-scripts/scratch-pad/a5-termination-silence.wat` | **`"Compiled"`** (measured) | still `"Compiled"` — behaviour does not change; the verdict is now *sayable*, not fatal |
| 2 | import never calls the verifier | `grep -c 'refuse_non_terminating\|verify_termination' src/rete/export.rs` | **0** (measured) | still 0 — DESIGN cuts adding a call there |
| 3 | one caller | `grep -rn 'refuse_non_terminating' src/ --include=*.rs \| grep -v 'fn refuse'` | **2 hits, both `arm.rs`** (`:1296` a comment, `:1301` the call) | unchanged |
| 4 | the conflation has no representation | `grep -c 'Ok(())' src/rete/kernel/stratify.rs` | **3** (`:838`, `:894`, `:988`) | the fn returns a verdict; no `Ok(())` carries a termination meaning |
| 5 | `Proven` is not lumped | read the `:894` / `:988` arms | — | both return `Proven`; **371 of 381 corpus rules take `:894`** and must not read as unverified |
| 6 | `NotAnalysable` is reachable | a probe over a MIXED rule set (trap 1) | — | driven, with the count non-zero |
| 7 | the sentence agrees with `stratify.rs:339-342` | read `arm.rs:1294` | *"the one door EVERY rule passes"* — **unqualified** | qualified to locally-compiled rules, naming import as the door that does not call it |
| 8 | `Refused` still reaches the converter | read `arm.rs:1301` | no `?`, by design | unchanged — still no `?` |
| 9 | blast radius | `git diff --stat` | — | `stratify.rs` + `arm.rs` (+ probes). Nothing else |
| 10 | lints | `cargo nextest run --release -E 'binary_id(wat::lint)'` | **114/114** (measured) | green — the rider runs this |
| 11 | floor | `./scripts/floor.sh` | **5203/5203** (measured) | ≥ 5,203 + every new arm, zero FAIL rows |
| 12 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | **rc=0** (measured) | silent |

## The mutation proof

**Collapse the verdict** — make `NotAnalysable` construct `Proven` instead — and the row-6 probe
must go RED. If it stays green, the probe is measuring `:894` and not the `continue` (trap 1), and
the strike has proven nothing about the arm it was drawn for.

Per arm, state: **proven** (driven, red→green), **reachable but not driven**, or **not reachable,
and why**. An unreached arm named as unreached is a pass; an unreached arm not mentioned is a fail.

## Runtime prediction

35–50 minutes. The type split is perhaps 30 lines across two files; the mixed-set probe is the
work, and trap 1 is where the time goes.

## What would make this strike a failure even if every test passes

**`NotAnalysable` becoming fatal.** The finding is that a state cannot be *said*, not that it
should be *refused*. A version that refuses AST-less rule sets would break every legitimate one and
would be a policy change wearing an honesty fix.

The second: **`:894` folded into `NotAnalysable`.** It is a real proof taken by 371 of 381 corpus
rules. Inverting it would make the overwhelming majority of the corpus read as unverified — a green
suite with the finding reversed.
