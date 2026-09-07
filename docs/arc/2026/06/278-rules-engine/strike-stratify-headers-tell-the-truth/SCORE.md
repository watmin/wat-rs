# SCORE — the stratify headers no longer claim lockstep

Three comment edits. No executable line moved. The false `Mirrors stratify-sweep` line is
gone. Edit 3's lockstep sentence is still there — it is true — with a pointer at the `+1`
the sentence does not mention. `produced_type` (`:48`, was `:35`) is untouched.

## Scorecard

| # | result |
|---|---|
| 1 ★ comments only | **HOLD.** `git diff -U0` every `+`/`-` line begins `//`, `///`, or `;;`. |
| 2 ★ false `Mirrors` gone | **HOLD.** `grep -n 'Mirrors .stratify-sweep' src/rete/kernel/stratify.rs` — no hit. |
| 3 ★ six TRUE claims untouched | **HOLD.** `git diff -U0 -- src/rete/kernel/stratify.rs` has 0 hits for `rule-stratum`/`stratify-fix`/`ast-name`. (The EXPECTATIONS command without `-U0` counts 2 from *context* around the header list, which still names those functions and still says they mirror.) |
| 4 ★ gate cited | **HOLD.** `stratify_numbers` appears twice in `stratify.rs`, once in `stratify.wat`. |
| 5 ★ gate still passes | **HOLD.** `native_stratify_numbers_against_the_oracle_scratch` PASS. |
| 6 ★ grid axis still passes | **HOLD.** `wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle` PASS 8.233s. |
| 7 ★ floor | **HOLD.** `Summary [ 485.434s] 5473 tests run: 5473 passed (4 slow), 22 skipped`. `.floor/2026-09-07T10-12-44Z/`. Landed tree (after the clippy blank line). |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. First run was RED on the verbatim docstring; see below. |
| 9 no logic / `:35` untouched | **HOLD.** `produced_type` / `rule_produces` not in the diff. No `+1` moved. |

★ load-bearing.

## Two deviations from verbatim, both named

**Line numbers.** The BRIEF cited `:221-227` and `stratify.wat:233-234` against the *pre-edit*
tree. After the comment inserts those ranges land on the new docstring, not the `+1`. A
reader following them would re-derive the wrong claim — the defect being fixed. Landed
citations are `:247-253` (the `derived` / `+ i64::from(derived)` span) and
`stratify.wat:241-242` (the `NOT +1` comments). Same spans, post-edit addresses.

**Clippy `doc_lazy_continuation`.** The verbatim docstring put `then for each produced p`
immediately after a `*` list. `-D warnings` refused it. Cure is one blank `///` so the
formula is its own paragraph, not a list continuation. `#[allow]` would have been a
non-comment line (STOP-4). First clippy RED is `.floor` sibling, not re-run-to-green:
the arm is `src/rete/kernel/stratify.rs:224` `clippy::doc_lazy_continuation`.

## Edit 3

The sentence *":exists inner and accumulate :from ARE — lockstep with native
`rule_consumes`"* is still in the file, unchanged. The pointer under it names
`rule_bag_consumes` and the `+1`. Reflex-correcting that sentence would have been the
trap.

## Landing

`src/rete/kernel/stratify.rs`, `wat/rete/oracle/stratify.wat`. Comments only. Do not
commit unless asked.
