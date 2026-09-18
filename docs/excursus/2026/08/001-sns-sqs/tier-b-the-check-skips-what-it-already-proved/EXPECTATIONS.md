# EXPECTATIONS — Tier B

⛔ **Row 1 gates everything. If the closure cannot be proven, STOP — do not elide.**

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **The closure, PROVEN or REFUTED — structurally, not by census** | every name a stdlib body can resolve is `:wat::` (reserved) or a local binding. Read the seven doors and the reservation guard. A refutation is a perfectly good outcome and stops the stone. |
| 2 | ⭑⭑ **Each known route closed, individually** | user `defn`/`def` · `defclause` (incl. **eval-time/REPL**, which bypasses `check_program`) · macros · types/`extend-type` · acronyms/companions · `installed_dep_sources()`. Say how each is closed, or that it is not. |
| 3 | ⭑⭑ **The sweeps are PURE** | immutable borrows, no interior mutability, no side effects. Re-confirm rather than inherit. |
| 4 | ⭑⭑ **USER errors are still caught — driven** | a user program with a type error must still fail, with the cache warm. This is the control that matters; a Tier B that hides a user error is worse than a slow boot. |
| 5 | ⭑⭑ **Boot drops** | before/after with `WAT_BOOT_CENSUS=phases`, cold and warm. Target shape: 191.3 → ~65 ms accounted. Report what you got. |
| 6 | ⭑⭑ **Stale / absent / corrupt snapshot ⇒ derive AND check** | all three driven, as Tier A did. |
| 7 | ⭑ **Eval-time/REPL `defclause` on a stdlib name** | driven: refused, or reaches. Either answer is publishable; a guess is not. |
| 8 | ⭑⭑ **Floor** | Summary verbatim, `.floor/<stamp>/`, `ARM.txt` or not. |
| 9 | clippy + `--no-run` | **read the number from the output**, do not assume it. |
| 10 | ⭑ **Circuit byte-identical** | `distinct=8000;dup=0` and `timeout=yes;…;retry-on=fresh`. |
| 11 | ⛔ **Open items untouched** | `check.rs:6068`, `env.rs:456`, `runtime.rs:2112`, `.config/nextest.toml`. |
| 12 | ⭑ **What got slower or riskier** | the bill, as Tier A reported it. |
