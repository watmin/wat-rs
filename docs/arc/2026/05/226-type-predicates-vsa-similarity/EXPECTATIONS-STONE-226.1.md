# EXPECTATIONS — Arc 226 Stone 226.1 — Type predicates for classifier-wrapped entities

Mode A target: 12/12 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | NEW `:wat::holon::is?` polymorphic predicate verb minted | Rust fn `eval_holon_is_predicate`; 2-arg form `(is? value class-name)`; dispatches via `extract_classifier`; returns `Value::bool`; TypeScheme + check.rs registration |
| 2 | NEW `:wat::holon::is-Map?` predicate verb | 1-arg `(is-Map? value)`; calls `extract_classifier == Some("Map")`; bool return |
| 3 | NEW `:wat::holon::is-Set?` predicate verb | Same pattern; "Set" classifier |
| 4 | NEW `:wat::holon::is-Vector?` predicate verb | Same pattern; "Vector" classifier |
| 5 | NEW `:wat::holon::is-List?` predicate verb | Same pattern; "List" classifier |
| 6 | NEW `:wat::holon::is-Tuple?` predicate verb | Same pattern; "Tuple" classifier — DISTINCT from is-Vector? per arc 228 substrate distinction |
| 7 | NEW `:wat::holon::is-Symbol?` predicate verb | "Symbol" classifier (post-arc-230 classifier-wrap encoding) |
| 8 | NEW `:wat::holon::is-Keyword?` predicate verb | "Keyword" classifier (post-arc-230) |
| 9 | NEW `:wat::holon::is-Tag?` predicate verb | "Tag" classifier (post-arc-230) |
| 10 | NEW `:wat::holon::is-Nil?` predicate verb | Special case: `extract_classifier == Some("Symbol") AND inner == "nil"`; use `HolonAST::is_nil()` accessor if available |
| 11 | New test file `probe_arc226_stone1_type_predicates.rs` | Positive + negative case for each of the 10 predicates; edge cases (bare primitive, nested classifier, non-Bind top-level); 100% PASS |
| 12 | All test suites green + holon-rs untouched | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5]` PASS; new probe + arc 216 probes (1/2/3/4/7) + arc 221 + arc 143 + mvp PASS; `cargo test -p wat-edn` PASS; clippy clean; `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty |

## Independent prediction (calibration record)

**Target runtime:** 90-180 min Mode A
**Upper bound:** 240 min
**Confidence:** high

**Rationale:**
- Pattern locked from Stone 228.1 (5 new verbs) + Stone 230.1 (variant retirement); calibration trend favors faster-than-target
- Stone 228.1 already minted `extract_classifier`; this stone just wires it into predicate verbs
- No encoding cascade; no consumer sweep (predicates are NEW verbs; no callers to update)
- Test file is mechanical: positive + negative per predicate; pattern-locked

**Risks:**
- `is-Nil?` special case — sonnet should use `HolonAST::is_nil()` accessor (added by arc 230) rather than reimplementing the check
- Polymorphic `is?` 2-arg form — the class-name arg could be String OR keyword; sonnet picks; tests should cover both
- Doctrine framing: this is the FIRST type-checking-as-VSA-algebra primitive; the in-code doc comments should set the foundation for arc 226 future stones (226.2+ VSA similarity)

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Variant-based predicates for substrate primitives (is-I64? / is-Bundle? / etc.) — Stone 226.2 scope
- VSA similarity with threshold-tunable answers — Stone 226.3+ scope
- Polymorphic dispatch integration with arc 146/147 multimethod machinery — arc 226 closure
- User-defined type predicates (arc 227)
- INSCRIPTION (Stone 226.4)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Honesty deltas accepted

- Polymorphic `is?` may accept class-name as String OR keyword (sonnet picks; tests cover the chosen form)
- New test file naming may vary (e.g., `probe_arc226_type_predicates.rs`); sonnet picks consistent with arc-22N convention
- `is-Nil?` implementation may use `HolonAST::is_nil()` directly OR re-extract via `extract_classifier`; both honest
- Doc comments on each new verb may vary; load-bearing point is "describes intent + cites arc 226 doctrine"

## Honesty deltas NOT accepted

- VSA similarity scoring in this stone — STOP-6; deferred to 226.2+
- Extending to variant-based predicates (is-I64?, etc.) — STOP-5; deferred to 226.2 sub-stone
- "Pre-existing failure" framing for tests broken by this stone — STOP per Stone 221.3 Delta 1a
- Touching holon-rs — STOP per STOP-4
- Aliases for any existing predicate name — HARD CUT

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors
- **STOP-2:** test failure beyond new probe
- **STOP-3:** 240 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** scope creep beyond 10 predicates
- **STOP-6:** VSA similarity rabbit hole (v1 is structural exact-match)
- **STOP-7:** bash discipline — cargo hang from pipes
