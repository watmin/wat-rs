# EXPECTATIONS — Arc 214 Slice 4 Stone 4.2 — `/get` trio

Mode A target: 18/18 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `/get` verb registered | `src/runtime.rs` — dispatch arm + eval function near eval_hashmap_get (~7813); check.rs entry with `-> :T` extraction |
| 2 | `/expect-get` verb registered | Same pattern; panic on None with KeyError flavor (mirror option::expect's arc 107 pattern) |
| 3 | `/get-default` verb registered | Same pattern; return default arg on None; default type unifies with T at check |
| 4 | Probe 1 — `/get` found + correct type | `(get env :foo -> :wat::core::String)` → `Some("bar")` when env has `:foo` → `"bar"` |
| 5 | Probe 2 — `/get` not found | Returns `None` for missing key |
| 6 | Probe 3 — `/get` wrong type | Returns `None` when stored HolonAST variant doesn't match requested T |
| 7 | Probe 4 — `/get` multi-type | Works for T ∈ {i64, String, bool, keyword} |
| 8 | Probe 5 — `/expect-get` found | Returns T directly |
| 9 | Probe 6 — `/expect-get` not found | Panics with diagnostic naming the key |
| 10 | Probe 7 — `/expect-get` wrong type | Panics with diagnostic naming the type mismatch |
| 11 | Probe 8 — `/get-default` found | Returns found value; default ignored |
| 12 | Probe 9 — `/get-default` not found | Returns supplied default |
| 13 | Probe 10 — `/get-default` wrong type | Returns supplied default |
| 14 | Probe 11 — `/get-default` default type unification | Default arg's type must unify with T; mismatch fails at check |
| 15 | Probe 12 — All three on same env | Cross-verb consistency: get returns Some(x); expect-get returns x; get-default returns x |
| 16 | Probe 13 — Empty env behavior | get → None; expect-get → panic; get-default → default |
| 17 | Probe 14 — HolonAST::Atom unwrap | Stored `HolonAST::Atom(primitive)` extracts cleanly to T |
| 18 | Probe 15 — Nested holon as wrong type | Stored `HolonAST::Bundle` treated as wrong-type for primitive T (returns None / panics / default) |

## Independent prediction (calibration record)

**Target runtime:** 45-75 min Mode A
**Upper bound:** 90 min
**Confidence:** medium

**Rationale:**
- Stone 4.1 shipped in ~7 min (small mechanical typealias); 4.2 is larger (3 verbs + 15 probes + check.rs + runtime.rs work)
- Pattern templates exist (eval_hashmap_get, eval_atom_value, option::expect) — should compose cleanly
- Risk: HolonAST→T extraction may need a new helper function or composition; arc 107 typed-expect pattern needs adaptation for the (key lookup + extract) two-step
- Risk: `-> :T` return-type annotation flow through check.rs for these specific verbs
- The trio shares 80%+ of implementation (lookup + extract is common; only the None-handler differs); writing one verb makes the others trivial

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- `/dig` trio — Stone 4.3
- Polymorphic `:wat::core::get` dispatch entry — future stone
- spawn-program' — Stone 4.4
- Kernel verbs — Stone 4.5
- Integration tests — Stone 4.6

## Honesty deltas accepted

- HolonAST extraction mechanism reuse vs new helper — sonnet picks honest path; flag in SCORE
- Probe matrix may surface edge cases worth additional coverage (e.g., HolonAST::Bundle as the V; HolonAST::Atom on non-Atom variants) — document if added
- Check.rs integration for `-> :T` annotation flow — may need brief new helper or just pattern match arc 107
- Polymorphic dispatch entry deliberately deferred; if sonnet finds it trivial-to-add-while-here, document but DO NOT add (out of scope)
