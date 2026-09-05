# EXPECTATIONS — STONE: a type application is atomic

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **the ruling** | `[xs <- (:wat::core::Vector :- [:wat::core::i64])]` on **ONE LINE** |
| 3 | ★★ a NESTED type is also one line | `(:wat::core::HashMap :- [(:wat::core::Vector :- [:wat::core::i64]) :wat::core::String])` unbroken |
| 4 | ★★★ **a generic `fn` is NOT collapsed** | the `wat/core.wat:1349` shape — `(fn :- [T] [params] -> R body)` — still lays out normally, ret-spec on its own line |
| 5 | ★ the recognised count is PRINTED | a probe prints how many type applications were treated as atomic. **A green over zero proves nothing** |
| 6 | every fixture idempotent | `IDEMPOTENT=true` across all |
| 7 | ruled shapes hold | `defn-multi`, `defn-empty`, `let-two`, `half-broken`, `all-four`, `claim-demo`, `assoc-ride`, `foldl-bare`, `let-complex`, `unruled-*` |
| 8 | the ret-spec still one line | `foldl-bare.wat` — last stone's ruling must not regress |
| 9 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0; `col` 0 in every rule file |
| 10 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 11 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 12 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 13 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 25-45 min. One predicate and one descent decision.

## Trap-doors named in advance

- **Row 4 is the census's counterexample promoted to a gate.** The naive predicate passes rows 2 and
  3 and FAILS row 4. A strike green on 2-3 and silent on 4 has shipped the wrong predicate.
- **Row 5 guards the silent-empty case**, which is this stone's version of last stone's signature
  failure: if the predicate never matches, rows 6-9 all pass and nothing is fixed.
- **Row 8 guards the previous stone.** Making a type atomic must not disturb the `Slot` mechanism.
- **`Named` covers StringLit** — the string `":-"` is indistinguishable from the symbol without the
  kind check.
- **The vacuous green:** row 10 prints the comment COUNT.
