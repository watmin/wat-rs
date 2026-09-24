# EXPECTATIONS — STONE F

| # | what | expected |
|---|---|---|
| 1 | annotated `(Box :- [Lru])` with a handle | refused, check time |
| 2 | inferred `(Box {:x h})` with a handle | refused, check time |
| 3 | generic-fn-built `Box<T>`, T = Lru at the call | refused, runtime, wat error with the user's span |
| 4 | generic `:Pure` enum holding a handle | refused |
| 5 | every row above with a PURE payload | accepted — no over-refusal |
| 6 | EDN write of a pure record | never emits a handle as `nil` |
| 7 | mutation per layer | each layer's rows red when it is disabled |
| 8 | floor | 0 failed |
| 9 | clippy | clean |

Runtime: 90-150 min — the choke-point search is the unknown. Floor ~17 min.
