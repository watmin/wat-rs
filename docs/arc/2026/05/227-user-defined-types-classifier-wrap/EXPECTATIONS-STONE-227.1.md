# EXPECTATIONS — Arc 227 Stone 227.1 — User-defined types via `defclass` macro

Mode A target: 10/10 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | NEW `:wat::core::defclass` defmacro authored | Wat-level defmacro in appropriate stdlib path (likely `wat/core/defclass.wat` or similar); single-arg form `(defclass Name)`; expands to constructor + predicate defns using arc 226/228 primitives |
| 2 | defmacro registered via stdlib defmacros | Auto-loaded via `register_stdlib_defmacros` path; no manual user import needed; verified by tests being able to call `(:wat::core::defclass ...)` directly |
| 3 | Constructor auto-generated correctly | `(defclass MyType)` produces a callable `(:user::MyType <data>)` (or similar namespace) that returns `Bind(Atom("MyType"), Atom(<data>))` |
| 4 | Predicate auto-generated correctly | `(defclass MyType)` produces a callable `(:user::is-MyType? x)` (or similar) that calls `:wat::holon::is?` with "MyType" classifier; returns bool |
| 5 | Multiple user types independent | `(defclass A)` + `(defclass B)` — instances of A are NOT B and vice-versa; predicates discriminate correctly |
| 6 | User types distinct from built-in types | `(defclass MyMap)` produces instances queryable as MyMap but NOT as built-in Map (different classifier strings; no collision) |
| 7 | Polymorphic `is?` works on user-defined classes | `(:wat::holon::is? user-instance "MyType")` returns true; same machinery as built-in types |
| 8 | Constructor errors on non-atomizable input | Calling `(MyType <fn>)` (non-atomizable) errors at check time per arc 225's narrow `:wat::holon::Atom` constructor; honest error message |
| 9 | New test file `probe_arc227_stone1_defclass.rs` | 8+ tests: simple defclass; multiple classes; cross-discrimination; user-vs-builtin; polymorphic is?; edge cases; all PASS |
| 10 | All test suites green + holon-rs untouched | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5]` PASS; new probe + arc 226 + arc 216 + arc 221 + arc 143 + mvp + wat-edn PASS; clippy clean; holon-rs untouched |

## Independent prediction (calibration record)

**Target runtime:** 60-120 min Mode A
**Upper bound:** 180 min
**Confidence:** high

**Rationale:**
- Pure macro expansion using existing primitives; no substrate-primitive minting; no encoding cascade
- Pattern locked from defservice / defn-restricted / etc.; sonnet has clear precedent
- Tests are mechanical: defclass + verify + cross-discriminate

**Risks:**
- Macro template construction — quasiquote/unquote nesting can be tricky; may need iteration
- Constructor name generation — building `:user::MyType` from `MyType` requires symbol manipulation; sonnet may need to find the right substrate helper
- defmacro auto-load path — sonnet may need to add the new wat file to the stdlib path list if not already scanned

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

- defmacro syntax may evolve as sonnet finds the cleanest template form
- Constructor namespace (`:user::MyType` vs `:wat::user::MyType` vs other) — sonnet picks consistent with existing convention
- Re-declaration behavior — error OR idempotent; sonnet picks honest behavior + documents
- Test count may exceed 8 if sonnet finds more edge cases worth covering

## Honesty deltas NOT accepted

- Adding new substrate primitives instead of using defmacro — STOP-5; the doctrine says user-defined types live at the wat-surface
- Inheritance in v1 — STOP-6; Stone 227.2 territory
- VSA similarity scoring — different arc (226.2)
- "Pre-existing failure" framing for tests broken by this stone — STOP per Stone 221.3 Delta 1a
- Touching holon-rs — STOP-4
- Aliases for any existing macro name — HARD CUT

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors
- **STOP-2:** test failure beyond new probe
- **STOP-3:** 180 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** substrate-primitive route instead of wat-defmacro
- **STOP-6:** inheritance scope creep (deferred to 227.2)
- **STOP-7:** bash discipline — cargo hang from pipes
