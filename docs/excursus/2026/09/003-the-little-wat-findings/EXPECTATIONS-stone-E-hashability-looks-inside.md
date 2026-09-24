# EXPECTATIONS — STONE E

| # | what | expected |
|---|---|---|
| 1 | nested handle as a key, every container × every verb | no `panicked at`; the verb's normal refusal |
| 2 | deep pure-data key | inserts and is found |
| 3 | `value_is_hashable` | exhaustive `match`, no `_ =>`; shallow part derived from `key_eligibility()` |
| 4 | `List`/`PersistentMap`/`PersistentVector` keys | still accepted (they hash today) |
| 5 | mutation (a): shallow again | nested cases RED |
| 6 | the key_eligibility gates | green |
| 7 | callers | unchanged |
| 8 | floor | `Summary`: 0 failed |
| 9 | clippy | clean |

Runtime: 60-90 min; floor ~17 min.
