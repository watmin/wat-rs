# EXPECTATIONS — boot names where its time goes

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **A per-PHASE breakdown of one boot** | the phases the loader ACTUALLY has, named from `src/load/`, not from the DESIGN's guess. Quote the table. |
| 2 | ⭑⭑ **The phases reconcile with the wall clock** | they sum to the externally measured boot (0.425–0.438 s today) ± stated noise. A large unexplained remainder IS a finding — report it as one. |
| 3 | ⭑⭑ **A per-MANIFEST-ENTRY curve, all 55** | which files are expensive, in order. Name the top 5 and their share. |
| 4 | ⭑⭑ **Does expansion dominate — or not?** | the DESIGN predicts it from marginal cost. If parsing or type-checking wins instead, say so LOUDLY; it redirects the attack. |
| 5 | ⭑ **`defsurface`/`defservice`-bearing files vs the rest** | 17 such forms exist. Is their per-line cost above average, and by how much? |
| 6 | ⭑⭑ **Off by default, proven** | env var unset ⇒ boot indistinguishable from the 0.425–0.438 s band. Quote three runs. |
| 7 | ⭑ **The instrument's own overhead** | measured and stated; say which numbers it distorts, if any. |
| 8 | ⭑⭑ **Floor** | Summary verbatim, `.floor/<stamp>/`, `ARM.txt` or not. |
| 9 | clippy + `--no-run` | `--release --workspace --all-targets`. |
| 10 | ⛔ **Nothing was optimised and no timeout was touched** | `git diff` shows instrumentation only; `.config/nextest.toml` unchanged. |
| 11 | ⭑ **What the fix should target, as a RECOMMENDATION** | one paragraph, explicitly not a decision, naming what the profile says is worth attacking and what it says is not. |
