# EXPECTATIONS — Arc 214 Slice 4 Stone 4.3 — `/dig` trio

Mode A target: 21/21 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `/dig` verb registered | runtime.rs eval function + check.rs infer; composes on Stone 4.2 helpers |
| 2 | `/expect-dig` verb registered | Same pattern; panic on None with KeyError flavor + path context |
| 3 | `/dig-default` verb registered | Same pattern; return default on miss/wrong-type/non-traversable |
| 4 | Probe 1 — single-step equivalent to /get | `(dig env [:foo] -> :T)` and `(get env :foo -> :T)` return identical results |
| 5 | Probe 2 — single-step miss | `(dig env [:nope] -> :T)` returns None |
| 6 | Probe 3 — two-step nested HashMap | env = `{:outer (Atom {:inner (Atom 42)})}`; dig with `[:outer :inner]` returns Some(42) (or design-equivalent if atomizable issue) |
| 7 | Probe 4 — three-step deep | Deeper nesting works (or design-equivalent per STOP-1 resolution) |
| 8 | Probe 5 — missing intermediate | Path step missing midway → None |
| 9 | Probe 6 — missing final | Path step missing at end → None |
| 10 | Probe 7 — non-HashMap intermediate | Path traversing into primitive intermediate → None (terminated early) |
| 11 | Probe 8 — type extraction success | Found + correct T returns Some(T) |
| 12 | Probe 9 — type extraction wrong T | Found + wrong T returns None |
| 13 | Probe 10 — multiple T types | i64, String, bool, keyword extraction all work |
| 14 | Probe 11 — `/expect-dig` found | Returns T directly |
| 15 | Probe 12 — `/expect-dig` not found | Panics with diagnostic naming the path |
| 16 | Probe 13 — `/expect-dig` wrong type | Panics with type-mismatch diagnostic |
| 17 | Probe 14 — `/dig-default` found | Returns found value |
| 18 | Probe 15 — `/dig-default` not found | Returns default |
| 19 | Probe 16 — `/dig-default` wrong type / non-traversable | Returns default |
| 20 | Probe 17 — empty path | Behavior per design call; documented in probe + SCORE |
| 21 | Probe 18 — non-keyword path step | Rejected at check OR handled at runtime; sonnet picks; documented |

## Independent prediction (calibration record)

**Target runtime:** 60-75 min Mode A
**Upper bound:** 90 min
**Confidence:** medium

**Rationale:**
- Stone 4.2 shipped in ~20 min; 4.3 is comparable shape (3 verbs sharing 80%+ impl) + the walk loop
- Walk loop is the new complexity — composes on Stone 4.2 helpers (single-step lookup + atom-value extract); should be a clean loop
- Risk: nested HashMap traversal via Atom-wrapped values may surface arc 215's atomizable-set limitation (STOP-1). If that's a real blocker, multi-step gets deferred and Stone 4.3 ships single-step only.
- Risk: empty path and non-keyword step semantics may surface unexpected substrate behavior; sonnet picks; documented honestly

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Path with non-keyword steps for vector indexing — future stone
- HolonAST::Bundle traversal (explicit holon values) — future stone
- Polymorphic `:wat::core::dig` dispatch entry — future stone

## Honesty deltas accepted

- Nested HashMap traversal mechanics may require substrate adjustment — flag honestly; reduce scope if needed
- Empty path semantics — sonnet's call; document
- Non-keyword path step — sonnet's call (reject at check vs handle at runtime); document
- Walk loop topology — pure recursion vs iterative; either fine; document the choice
