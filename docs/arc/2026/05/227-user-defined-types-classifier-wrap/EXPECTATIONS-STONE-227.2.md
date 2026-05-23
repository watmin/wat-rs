# EXPECTATIONS — Arc 227 Stone 227.2 v2 — Mandate field-list on defrecord

Mode A target: 14/14 PASS.

## v2 supersedes v1

v1 EXPECTATIONS (committed at `2162d82`) had optional-args defrecord (1-arg + 2-arg). v2 mandates the field-list per user direction + four-questions atomic check. Single-arg form RETIRED (HARD CUT).

| # | Row | Expectation |
|---|---|---|
| 1 | defrecord macro head is 2-arg only | `(defrecord <fqdn> <field-list>)`; field-list is always present; no 1-arg overload |
| 2 | Empty field-list `[]` mints zero-arg constructor | `(defrecord :ns::Tag [])` → `(:ns::Tag)` zero-arg call; instance is `Bind(Atom("ns::Tag"), Bundle())`; no accessors |
| 3 | N-field list mints N-arg constructor | `(defrecord :ns::Foo [a <- :i64, b <- :String])` → `(:ns::Foo a-val b-val)` two-arg call; instance is `Bind(Atom("ns::Foo"), Bundle(Bind(Atom("a"), Atom(av)), Bind(Atom("b"), Atom(bv))))` |
| 4 | Auto-accessors generated per field | `:ns::Foo/a` and `:ns::Foo/b` auto-generated; each accepts `:ns::Foo` instance; returns inner Atom contents as `:wat::holon::HolonAST` |
| 5 | Predicate unchanged shape from 227.1b | `:ns::is-Foo?` generated regardless of field-count; classifier-dispatch via `:wat::holon::is?` |
| 6 | Single-arg form `(defrecord :fqdn)` ERRORS | Macro rejects 1-arg calls with diagnostic citing v2 mandate; HARD CUT — no alias to `(defrecord :fqdn [])` |
| 7 | Cross-namespace independence with multi-field | `(defrecord :appA::Voltage [m <- :f64])` + `(defrecord :appB::Voltage [m <- :f64])` produce distinct classifiers; predicates discriminate |
| 8 | Constructor type-checks each field | Wrong-typed arg → check error citing field name |
| 9 | Existing 18 probes migrated to v2 shape | All 18 tests in `tests/probe_arc227_stone1_defrecord.rs` updated to use explicit field-list (most become `[value <- :Type]` single-field shape); file may rename to `tests/probe_arc227_stone2_defrecord.rs` (sonnet's choice; document in SCORE) |
| 10 | New v2-specific tests added | ~7+ new tests: empty field-list zero-arg, multi-field construct+access, accessor type, field-count edge cases; all PASS |
| 11 | src/stdlib.rs comment updated | Comment on line 74 references Stone 227.2 v2 + the multi-field shape |
| 12 | SCORE-STONE-227.1b.md gets addendum | APPENDED section noting Stone 227.2 v2 supersedes; body unchanged per `feedback_inscription_immutable` |
| 13 | All test suites green + holon-rs untouched | `cargo build --release -p wat` 0 errors; lib tests PASS; migrated probe + arc 226/216/221/143/mvp PASS; wat-edn PASS; clippy clean; holon-rs untouched |
| 14 | SCORE doc written | `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.2.md` mirrors SCORE-STONE-227.1b.md shape |

## Independent prediction (calibration record)

**Target runtime:** 90-180 min Mode A
**Upper bound:** 240 min
**Confidence:** medium

**Rationale:**
- Macro extension (uniform 2-arg head; branching body by field-count) is moderately bigger than 227.1b
- 18 existing probes need migration — bulk sed-able if pattern is consistent
- ~7+ new tests for v2 behavior
- Accessor body needs Bundle-walking — STOP-5b risk

**Risks:**
- Accessor body Bundle-walking ergonomics — if `Bundle/children` iteration + classifier-match isn't clean in pure wat, STOP-5b
- Migration of 18 probes may surface semantic edge cases (probes asserting on opaque-payload specifically)
- Empty field-list edge case in the macro body (Bundle with zero children) — verify cleanly

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Methods bundled (STOP-6; per notes Pattern 3)
- Inheritance (Stone 227.3)
- `:with-<field>` immutable setters (future)
- `:invariants` (future)
- defprotocol / extend-type (arc 232)
- from-holon multi-field support (future)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Honesty deltas accepted

- Probe file may rename or stay — sonnet picks; document in SCORE
- New tests count may exceed 7 if edge cases surface
- Accessor return type stays `:wat::holon::HolonAST` (raw inner contents); typed-primitive return is future ergonomics
- Empty field-list semantic — sonnet picks honest behavior (probably: zero-arg constructor; no accessors; predicate works); document

## Honesty deltas NOT accepted

- Retaining 1-arg form as alias for `(defrecord :fqdn [])` — STOP-8; HARD CUT
- Bundling methods in defrecord — STOP-6
- New substrate primitive minting in this stone — STOP-5; if needed, STOP-5b
- Touching holon-rs — STOP-4
- Rewriting historical artifacts (BRIEF/EXPECTATIONS/SCORE body of 227.1b + STONE-227.2-NOTES.md + arc 232 DESIGN) — STOP-9; append-only per `feedback_inscription_immutable`
- "Pre-existing failure" framing for tests broken by mandate — broken-by-this-stone (Stone 221.3 Delta 1a)

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors
- **STOP-2:** test failure beyond migrated probes
- **STOP-3:** 240 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** substrate-primitive route instead of macro
- **STOP-5b:** substrate lacks ergonomic Bundle-walking — surface as finding
- **STOP-6:** methods bundled (Pattern 3 doctrine violation)
- **STOP-7:** bash discipline — cargo hang from pipes
- **STOP-8:** 1-arg form retained as alias (HARD CUT violation)
- **STOP-9:** historical artifact rewritten instead of append-only
