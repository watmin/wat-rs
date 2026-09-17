# EXPECTATIONS — an outcome arm cannot be a wildcard (phase 1, REPORT-ONLY)

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **The diagnostic sees a MACRO-GENERATED site** | flag a wildcard inside one of `wat/service.wat`'s quasiquoted bodies and quote it. If it cannot, the instrument choice failed — report that, do not ship. |
| 2 | ⭑⭑ **The census, per enum** | total sites where the rule would fire, broken down by enum path. Quote the table. |
| 3 | ⭑⭑ **live vs tests/probes split** | `wat/` + `wat-scripts/` services separated from `tests/`, `wat-tests/`, `scratch-pad/`. The live number decides whether a zero-exemption gate is viable. |
| 4 | ⭑ **generated vs literal split** | how many are only visible to the checker. This is the number that justifies not using a text lint. |
| 5 | ⭑ **The known-remaining two are found** | `service.wat:2978` and `:3089` (`RecvOutcome::Message` + `_` in the paging path) must appear. If they do not, the rule is not reaching what item 1 filed. |
| 6 | ⛔ **The three per-surface sites are NOT flagged** | `service.wat:2785`, `:2980`, `:3091` are out of scope by construction (variable variant sets, wildcard is the desync detector). If they are flagged, the scope predicate is wrong. |
| 7 | ⛔ **`Option`/`Result`/domain enums are NOT flagged** | trap-doors 1 and 2. Name what you checked. |
| 8 | ⭑⭑ **It is REPORT-ONLY** | the new test prints and passes; nothing becomes a hard error in this stone. |
| 9 | ⭑⭑ **Floor** | Summary verbatim, `.floor/<stamp>/`, `ARM.txt` or not. |
| 10 | clippy + `--no-run` | `--release --workspace --all-targets`. |
| 11 | ⭑ **Exemption mechanisms REPORTED, not chosen** | the three options in the DESIGN, with a recommendation and its cost. Whether a zero-exemption gate is viable follows from row 3. |
| 12 | ⛔ **Stated plainly: this lint would not have caught the three missing-form failures** | it guards unread variants, not absent ones. Say so in the SCORE so it is not oversold. |
