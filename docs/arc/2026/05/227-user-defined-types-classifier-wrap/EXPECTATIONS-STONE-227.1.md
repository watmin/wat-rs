# EXPECTATIONS — Arc 227 Stone 227.1 — User-defined types via `:wat::holon::defclass` macro (v3)

Mode A target: 12/12 PASS.

## v3 supersedes v1

The v1 EXPECTATIONS had two violations corrected in v3: (1) defclass moved from `:wat::core::*` to `:wat::holon::*` namespace; (2) macro REQUIRES user-declared FQDN (no `:user::*` insertion).

| # | Row | Expectation |
|---|---|---|
| 1 | NEW `:wat::holon::defclass` defmacro authored | Wat-level defmacro in `wat/holon/defclass.wat` (or similar holon-tier path); single-arg form `(defclass :fqdn::Name)`; expands to constructor + predicate using arc 226/228 primitives |
| 2 | defmacro registered via stdlib defmacros | Auto-loaded via `register_stdlib_defmacros` path; verified by tests being able to call `(:wat::holon::defclass ...)` directly without user import |
| 3 | Constructor auto-generated in USER-DECLARED namespace | `(defclass :myapp::Voltage)` produces `(:myapp::Voltage <data>)` — NOT in `:user::*` or any other auto-inserted namespace; constructor returns `Bind(Atom("myapp::Voltage"), Atom(<data>))` |
| 4 | Predicate auto-generated in USER-DECLARED namespace | `(defclass :myapp::Voltage)` produces `(:myapp::is-Voltage? x)`; is- prefix attaches to basename; namespace preserved; calls `:wat::holon::is?` with FQDN classifier string |
| 5 | Classifier string = FQDN (collision-free) | The Atom that classifies the instance carries the FQDN without leading colon — e.g., `"myapp::Voltage"` not `"Voltage"`. Distinct user types across namespaces produce distinct classifiers |
| 6 | Multiple namespaces independent | `(defclass :appA::Voltage)` + `(defclass :appB::Voltage)` produce distinct classifiers ("appA::Voltage" vs "appB::Voltage"); cross-discrimination via predicates verified |
| 7 | User types distinct from built-in types | `(defclass :test::MyMap)` produces instances queryable as MyMap but NOT as built-in Map (classifier "test::MyMap" ≠ "Map") |
| 8 | Polymorphic `:wat::holon::is?` works on user types | `(:wat::holon::is? user-instance "myapp::Voltage")` returns true; bare "Voltage" returns false |
| 9 | Constructor errors on non-atomizable | Calling `(:test::Voltage <fn>)` errors at check time per arc 225 narrow Atom constructor |
| 10 | New test file `probe_arc227_stone1_defclass.rs` | 6+ tests covering: single defclass, cross-namespace independence, multiple-same-namespace, user-vs-built-in, polymorphic is?, non-atomizable error; all PASS |
| 11 | All test suites green + holon-rs untouched | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5]` PASS; new probe + arc 226 + arc 216 + arc 221 + arc 143 + mvp PASS; `cargo test -p wat-edn` PASS; clippy clean; holon-rs untouched |
| 12 | No `:user::*` insertion anywhere | grep for `:user::` in the generated macro output should ONLY appear if the user explicitly declared `:user::SomeType`. Substrate never auto-inserts |

## Independent prediction (calibration record)

**Target runtime:** 60-120 min Mode A
**Upper bound:** 180 min
**Confidence:** high

**Rationale:**
- Pure macro expansion using existing primitives; no substrate-primitive minting
- defclass body needs to extract FQDN → namespace + basename via keyword-manipulation primitives (sonnet investigates what's available)
- Test pattern is mechanical

**Risks:**
- Keyword manipulation — building "myapp::is-Voltage?" from ":myapp::Voltage" may require finding the right substrate helper. If unavailable, STOP-5b surfaces as finding (orchestrator decides whether to mint helpers OR defer arc 227 v1 to a simpler shape)
- Macro template construction with computed unquote may need iteration
- defmacro auto-load path may need adjustment

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Class inheritance (Stone 227.2)
- Multimethod dispatch integration (Stone 227.3+)
- VSA similarity scoring (Stone 226.2 — different arc)
- USER-GUIDE chapter (Stone 227.4)
- INSCRIPTION (Stone 227.4)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Honesty deltas accepted

- defmacro template syntax may evolve as sonnet finds the cleanest form
- Test count may exceed 6 if sonnet finds more edge cases worth covering
- If keyword-manipulation helpers are missing AND sonnet can mint them as wat-level helpers (not new substrate primitives), acceptable — document as Delta

## Honesty deltas NOT accepted

- Inserting into `:user::*` or any auto-namespace — STOP-8; users declare their own
- Adding new substrate primitives instead of using defmacro — STOP-5
- Inheritance in v1 — STOP-6
- VSA similarity scoring — different arc
- Touching holon-rs — STOP-4
- Aliases — HARD CUT

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors
- **STOP-2:** test failure beyond new probe
- **STOP-3:** 180 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** substrate-primitive route instead of wat-defmacro
- **STOP-5b:** substrate lacks keyword-manipulation helpers — surface as finding
- **STOP-6:** inheritance scope creep
- **STOP-7:** bash discipline — cargo hang from pipes
- **STOP-8:** namespace insertion violation (no `:user::*` auto-insertion)
