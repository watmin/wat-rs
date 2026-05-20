# EXPECTATIONS — Arc 214 Parser-Pivot P2 — `{...}` map literal

Mode A target: 20/20 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Parser LBrace dispatch | content-shape branch on first child: empty/Keyword/Symbol/other; lands in `src/parser.rs` near line 200-230 |
| 2 | Empty `{}` semantics | parses to `(:wat::core::HashMap :wat::core::Keyword :wat::holon::HolonAST)` (length-0 HashMap) — arc 169 degeneracy check moved into struct-pattern branch ONLY |
| 3 | Keyword-headed dispatch | first child is Keyword → enter map-literal validation + desugar |
| 4 | Symbol-headed dispatch | first child is bare Symbol → preserve arc 169 StructPattern path; helper function rename to `parse_struct_destructure_body` |
| 5 | Map-literal helper named | `parse_map_literal_body` exists (or equivalently the map-literal validation + desugar function); name matches task #404 |
| 6 | Non-Keyword, non-Symbol first child | rejected with `MalformedBraceLiteral` naming actual variant kind |
| 7 | Auto-wrap values | every odd-indexed child wrapped in `(:wat::holon::Atom v)` at parse time; wraps unconditionally (no smartness) |
| 8 | Even-count rule | odd body length rejected with `MalformedBraceLiteral`; reason names `"got {n} forms"` |
| 9 | Keyword-key rule | non-Keyword in key position rejected with `MalformedBraceLiteral`; reason names actual key-position variant kind |
| 10 | `MalformedBraceLiteral` ParseError variant | added to ParseError enum; Display impl follows existing pattern |
| 11 | Probe 1 — empty `{}` | length-0 HashMap; arc 169 degeneracy retirement proven |
| 12 | Probe 2 — single pair | length 1; get :foo → Some(Atom 42); auto-wrap proven |
| 13 | Probe 3 — multi pair | length 3; get :b → Some(Atom 2); alternation proven |
| 14 | Probe 4 — nested in expression | `(:wat::core::length {:a 1 :b 2})` → 2; expression-position composability proven |
| 15 | Probe 5 — map-literal-of-map-literal | inner `{:inner 42}` evaluates; outer auto-wraps the HashMap value; behavior LIMITATION-commented honestly (probe captures actual outcome, not aspiration) |
| 16 | Probe 6 — non-keyword key | `{42 :v}` rejected at parse with `MalformedBraceLiteral` naming integer literal in key position |
| 17 | Probe 7 — odd count | `{:foo}` rejected at parse with `MalformedBraceLiteral` naming alternation requirement + actual count |
| 18 | Probe 8 — struct-pattern preserved | arc 169 bare-symbol `{outcome residue}` shape still parses to StructPattern; non-empty bare-symbol rule still enforced |
| 19 | Probe 9 — keyword in binding position | `(:wat::core::let [{:foo bar} ...] ...)` rejected by downstream layer (let validation/check/lower); error message captured + LIMITATION-commented for actual rejection layer |
| 20 | WAT-CHEATSHEET § 8 | `{...}` literal row added; position-discipline sub-paragraph cites arc 214 P2 + arc 169; verb-call vs literal relationship made explicit |

## Out-of-scope rows (deliberately absent)

- Match-arm `{...}` pattern matching (task #402)
- Generic `HashMap<K,V>` literal (pinned shape only)
- Macro layer
- Auto-wrap smartness / Atom polymorphism extension
- WARD-PASS (out-of-zone)
- INTERSTITIAL entry (orchestrator-direct)

## Independent prediction (calibration record)

Recorded before sonnet completes, after sonnet spawn. Per recovery doc § 7 time-boxing.

**Target runtime:** 20-35 min Mode A
**Upper bound:** 50 min
**2× upper-bound cap:** 100 min (would clamp to 60 min via ScheduleWakeup runtime ceiling)
**Confidence:** medium-high

**Rationale:**
- P1 calibration record — predicted 30-50 min, actual ~20 min (orchestrator tends to overestimate)
- P2 surface is narrower than P1's: parser refactor lives in one file region (`src/parser.rs:200-230` + new helper), new ParseError variant + Display, one new probe file (9 probes mirroring P1's probe shape), small doc updates
- P1 touched 4+ production files + 4 test migration files; P2 touches ~1 production file + 1 new test file + 3 docs
- Risk factors that could push toward upper bound: STOP-1 (arc 169 `{}` semantics ripple), STOP-2 (Probe 5 Atom polymorphism surprise), STOP-3 (any pre-existing `{...}` → StructPattern assertion)
- Risk factors that could push under target: zero substrate gaps (P1 shipped the verb-call sonnet desugars to); content-shape dispatch is mechanical; probes mirror P1's matrix shape

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]
- If overrun: where did time go? [TBD]
- If under: what made it cheaper than predicted? [TBD]

## Honesty deltas accepted

Sonnet may surface deltas if encountered:
- Pre-existing test asserts on `{}` → degenerate semantics (STOP-1 territory; if minor adjustment, fix in-stone with note)
- arc 169 helper function rename ripples wider than anticipated (sweep additional files; note in SCORE)
- Probe 5 behavior surprises (honest LIMITATION; do not aspire)
- Probe 9 rejection layer is not the expected one (capture actual, LIMITATION-comment)
- Any new pre-scope find: log under "Honest deltas" section in SCORE; do NOT silently fix beyond stone scope.
