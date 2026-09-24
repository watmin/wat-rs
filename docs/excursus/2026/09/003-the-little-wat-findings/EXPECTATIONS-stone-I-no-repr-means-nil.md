# EXPECTATIONS — STONE I

| # | what | expected |
|---|---|---|
| 1 | HandlePool / forced Stream / empty Stream over a wire | refused at the sender, stone G's message |
| 2 | a stream materialized to a vector | crosses whole |
| 3 | `:wat::edn::write` of each, no wire | tagged `nil` |
| 4 | consumers of the removed bodies | enumerated; none behavioural (else STOP-1) |
| 5 | mutation: empty-Stream `()` restored | the empty case arrives as a List again |
| 6 | floor | 0 failed |
| 7 | clippy | clean |
| 8 | `Value::Vector`'s arm | UNTOUCHED — its own stone |
