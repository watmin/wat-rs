# EXPECTATIONS — STONE G

| # | what | expected |
|---|---|---|
| 0 | the sender's `SendOutcome` today, handle case | measured and reported (the "before") |
| 1 | generic fn, T = Lru, wire send | the SENDER raises a wat error at the user's span, naming the type |
| 2 | generic fn, T = i64 | received whole: `#t/Box {:x 42}` |
| 3 | Wire `Address` over a wire | still crosses (encode_capability) |
| 4 | `:wat::edn::write` of a handle, no wire | still `nil` — arc 294 unchanged |
| 5 | mutation: wire path non-strict | row 1 regresses to a receiver-side `Lost` |
| 6 | every wire `send`/`try-send` arm | listed, and all use the strict encode |
| 7 | floor | 0 failed |
| 8 | clippy | clean |

Runtime: 90-120 min; floor ~17 min.
