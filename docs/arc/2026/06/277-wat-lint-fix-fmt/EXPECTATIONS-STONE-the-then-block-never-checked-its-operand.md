# EXPECTATIONS — STONE: the `:then` block never checked its operand

⚠ **This stone's blast radius is UNKNOWN by construction.** Row 5 is a REPORT, not a number — a row
that demanded "N sites fixed" would be a number I made up. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

| # | what | command | expected — EXACT |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★★★ **the i64-into-a-String `:then` is REFUSED** | the ARM-A probe | a rete validation error naming the rule, the field, the declared type and the actual type. **`BOXES=1` must become a refusal.** |
| 3 | ★★★ **and so is enum-into-a-DIFFERENT-enum** | the Alpha/Beta probe | REFUSED. **This is the row a segment-based check passes.** `resolve_operand_type` returns `"enum"` for both; the check must compare DECLARED types. |
| 4 | ★★★ **every LEGAL `:then` still compiles** | the whole corpus | no rule refused that was legal before. **A check that rejects correct work is worse than the hole.** |
| 5 | ★★★ **THE SCREAMS ARE REPORTED IN FULL** | impose the check, run the floor | every rejected site listed **verbatim** — file, rule, field, declared type, actual type. **Not a count. Not a summary.** This is the stone's first deliverable and the input to every later decision. |
| 6 | ★★★ **`g7_rule.wat:26` is among them** | the known scream | it stores a `NodeKind` into `Capture.value <- :wat::core::String`. **If the check does NOT flag it, the check does not work** — this site is the reason the stone exists. |
| 7 | ★★ **and g7 is resolved without relaxing the fence** | `wat_grep::g7_end_to_end_prints_expected_match` | green. The capture is a `String`. **A user conversion fn in the `:then` is REFUSED (measured) — if the resolution needs one, STOP.** |
| 8 | ★★★ **the `accepted` list names the call form** | trigger `RhsUnresolvableOperand` with a bare `:Block` | the message lists the **call form / constructor** alongside `?var` and the four scalar literals. **The old text is what taught `Break.kind` to be a String.** |
| 9 | ★★ the message is still accurate about what is REFUSED | same probe | a bare keyword is still refused, and the reason (a keyword is a FIELD REFERENCE in a RHS) is still findable |
| 10 | ★★★ **negative controls, both defects, committed** | `tests/rete/…` | a test that goes RED if either check is removed. **Not a probe in a scratchpad — a test in the floor.** |
| 11 | ★★ the batching contract holds | a rule with TWO bad operands | **both** reported, not the first |
| 12 | ★★ nothing else moved | the fmt digests | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` — unchanged |
| 13 | ★★ the enum gates still hold | `ast_kind_nodekind_sync` · `every_wat_scripts_file_loads` | `PASS` · `1 passed` |
| 14 | floor (ORCHESTRATOR) | `scripts/floor.sh` | `5180+` run, **0 FAILED** |
| 15 | clippy (ORCHESTRATOR) | `-D warnings --all-targets` | `0` |

**Runtime prediction:** 60-120 min, and the spread is honest — the check is small, the screams are
unknown, and row 5 is what turns the second number into the first.

## Trap-doors named in advance

- **Row 5 is the deliverable, not a formality.** 319 `:then` sites exist and nobody knows how many
  lie. A SCORE that says "fixed the mismatches" without listing them has not done the work.
- **Row 3 is the row the obvious fix fails.** `resolve_operand_type` is right there and returns a
  SEGMENT; two different enums both segment to `"enum"`, and the hole admits exactly that today
  (measured). Reusing it wholesale ships a check that cannot see the case that surfaced the bug.
- **Row 4 is the counterweight to rows 2/3.** Imposing a check is easy; imposing one that only
  rejects the wrong things is the work. A wave of false rejections is a failure, not progress.
- **Row 6 is the self-test.** The stone exists because of that site. A check that misses it is
  measuring something else.
- **Row 7's obvious resolution is REFUSED and it was measured before this brief:**
  `':user::k-name' is not a rete primitive; a then admits only :wat::rete:: ops`. And **no rete op
  renders an enum as a string** — the vocabulary has only `core::enum::=` / `::not=`. Minting one is
  OUT OF SCOPE and gated on what row 5 reports.
- **Row 8 is the CLASS.** Row 2 fixes the hole; row 8 fixes the message that made a String look like
  the ceiling. Shipping row 2 alone leaves the next author making the same choice.
- ⚠ **`wat/*.wat` is FROZEN into the release binary** — `cargo build --release` between an edit and a
  measurement.
