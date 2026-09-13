# EXPECTATIONS 2b — written BEFORE the strike

| # | what | check | expected |
|---|---|---|---|
| E1 | every refusal names the separator its predicate accepts | `cargo nextest run --release -E 'test(refusals_teach_the_dot_separator)'` | 7 tests, one per site in D1's table, each asserting the refusal text AND that the remedied input is accepted; all pass |
| E2 | E1's tests can fail | revert ONE site's text in the working tree, re-run E1, restore | exactly that site's test goes red, with a message naming the site |
| E3 | no remaining refusal text teaches `::` for a variant | the census in `FINDINGS-composition.md` finding 7, re-run over `src/` `crates/` `wat/` | no code string; any survivor is a `rune:lint(one-variant-separator, …)`-runed non-variant site, each listed with its category |
| E4 | the door's doc and the lint's help describe the code | read `identifier.rs:362-389` and `tests/lint/one_variant_separator.rs` help text | both describe `.`; `only_identifier_rs_spells_the_variant_separator` passes |
| E5 | c03's row sees the defect | `contract_03` against TODAY's template (before the template fix) | RED, verbatim in SCORE |
| E6 | c03's template expands | `contract_03` after the fix | GREEN, observing a 3-element result |
| E7 | the two probes do what their headers say | `./target/release/wat wat-scripts/probes/arc-170/probe-m1-ann-erase.wat` and `…erase2.wat` | each prints `echo:z` |
| E8 | nothing else moved | `git diff --stat` | only the blast-radius files |
| E9 | walls green | `scripts/floor.sh` · clippy | 0 failed · 0 lines |

**Runtime prediction:** 1.5–3 h. Most of it is E1, because each site's remedy is derived from its own
predicate.

**Trap doors, named in advance:**
- **A text sweep.** Replacing `::` with `.` in seven strings without driving each refusal is the
  failure this brief exists to prevent. Two sites might guard different predicates.
- **Trusting `contract_03`'s green.** It was green while the template could not expand. E5 is the
  row that proves the test now sees it.
- **Reading a probe's rc.** Both probes exit 0 while failing; read their output.
