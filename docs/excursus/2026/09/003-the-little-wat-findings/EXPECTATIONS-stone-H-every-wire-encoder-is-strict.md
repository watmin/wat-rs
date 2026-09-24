# EXPECTATIONS — STONE H

| # | what | expected |
|---|---|---|
| 0 | the three "before" measurements (a)(b)(c) | reported verbatim |
| 1 | process-tier `after` with a handle | refused at the sender, user span, stone G's message shape |
| 2 | process-tier `after` with pure data | arrives whole |
| 3 | `Value::to_wire` | proven unreachable with a stated invariant, OR strict |
| 4 | thread-tier `try-send` of an in-locus value | `Sent` — no spurious refusal |
| 5 | process-tier `try-send` of a handle | still refuses (stone G unchanged) |
| 6 | `HandlePool` / `Value::Vector` over a wire | measured; any lie REPORTED, not patched |
| 7 | the invariant | a gate, or a stated reason the material does not allow one |
| 8 | floor | 0 failed |
| 9 | clippy | clean |
