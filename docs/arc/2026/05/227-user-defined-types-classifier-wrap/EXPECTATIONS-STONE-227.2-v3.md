# EXPECTATIONS — Arc 227 Stone 227.2 v3 — defrecord with N≥0 fields shipping canonical instance shape

Mode A target: 16/16 PASS. **Every row binds to a specific test fn that proves it.** No row may be marked PASS without naming the test.

## v3 supersedes v2

v2 EXPECTATIONS row 3 said "N-field N-arg constructor" but tests only covered N=0 + N=1; macro panics at expand for N≥2. v3 demands N≥2 actually working with canonical instance shape per typed-entities doctrine.

| # | Row | Binding test fn | Expectation |
|---|---|---|---|
| 1 | defrecord macro head is 2-arg ALWAYS | `probe_two_arg_form_only_one_arg_errors` | Single-arg `(defrecord :fqdn)` errors at expand time with ArityMismatch (HARD CUT preserved from v2) |
| 2 | N=0 empty field-list mints zero-arg constructor | `probe_zero_field_tag_construct_and_predicate` | `(defrecord :ns::Tag [])` → `(:ns::Tag)` returns `Bind(Atom("ns::Tag"), Bundle())`; predicate works |
| 3 | N=0 canonical instance shape uses Bundle | `probe_zero_field_instance_uses_empty_bundle` | Inner slot is `Bundle()` (zero children), NOT `Atom(nil)` (the retired v2 workaround). Verify via `Bundle/children-length` or equivalent traversal |
| 4 | N=1 single-field constructor | `probe_one_field_construct_with_typed_arg` | `(defrecord :ns::W [v <- :i64])` → `(:ns::W 42)` returns canonical shape |
| 5 | N=1 canonical instance shape uses Bundle | `probe_one_field_instance_uses_bundle_with_one_bind` | Inner slot is `Bundle(Bind(Atom("v"), Atom(42)))`, NOT `Bind(Atom("v"), Atom(42))` (the retired v2 workaround) |
| 6 | **N=2 multi-field constructor takes 2 typed args** | `probe_two_field_construct_with_typed_args` | `(defrecord :ns::P [a <- :i64, b <- :String])` → `(:ns::P 5 "hi")` returns canonical shape |
| 7 | **N=2 canonical instance shape uses Bundle with 2 children** | `probe_two_field_instance_bundle_has_two_binds` | Inner Bundle has exactly 2 field-Binds with correct field-name + field-value via traversal |
| 8 | **N=3 multi-field constructor takes 3 typed args** | `probe_three_field_construct_with_typed_args` | `(defrecord :ns::T [a <- :i64, b <- :String, c <- :bool])` → 3-arg constructor; inner Bundle has 3 field-Binds |
| 9 | Predicate works for all N | `probe_predicate_works_for_n0_n1_n2_n3` | `:ns::is-Tag?`, `:ns::is-W?`, `:ns::is-P?`, `:ns::is-T?` all return true for matching instances; false for mismatched classifier |
| 10 | Cross-namespace independence with N≥2 | `probe_cross_namespace_distinct_classifiers_n2` | `(defrecord :appA::Point [x <- :i64, y <- :i64])` + `(defrecord :appB::Point [x <- :i64, y <- :i64])` produce distinct classifiers ("appA::Point" vs "appB::Point"); predicates discriminate |
| 11 | Constructor type-checks each field | `probe_constructor_rejects_wrong_typed_field` | `(:ns::P "wrong" "hi")` (i64 expected, String given) errors at check time citing field `a` |
| 12 | All existing Stone 227.2 v2 N≤1 probes still pass | `probe_arc227_stone2_defrecord::*` (all 25 existing) | Migrated tests for v3's canonical-Bundle shape; no regression |
| 13 | Diagnostic probes (the design substrate) still pass | `probe_diagnostic_macro_splice_from_let::*` + `probe_diagnostic_bundle_result_compose::*` | The composition v3 uses is provably the composition shown in these probes |
| 14 | macro file doc-comment updated | grep `wat/holon/defrecord.wat` for v2's "STOP-5b finding" text → ZERO matches | Header reflects v3 canonical composition; cites probe commits as design substrate |
| 15 | SCORE-STONE-227.2.md gets v3 supersession addendum | grep SCORE-STONE-227.2.md for "v3 supersedes" → present | APPENDED per `feedback_inscription_immutable`; body unchanged |
| 16 | All test suites green + holon-rs untouched | `cargo build --release -p wat` 0 errors; full verification cascade per BRIEF Phase 4; clippy clean; `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty | Workspace impeccable post-stone |

## Independent prediction (calibration record)

**Target runtime:** 120-240 min Mode A
**Upper bound:** 360 min
**Confidence:** high (composition is empirically proven; sonnet mirrors)

**Rationale:**
- Both probes already passing — sonnet doesn't need to discover; they need to ADAPT the proven composition to the defrecord macro structure
- N≥2 case is the load-bearing addition; sonnet's previous stop was discovery failure, not substrate gap
- Test additions are mechanical: 4 new test fns (N=2, N=3, cross-namespace-N2, type-check) + verification of canonical shape via traversal

**Risks:**
- Bundle traversal for N-child verification may need helpers (Bundle/first + recursion OR new helper). If a clean composition exists, sonnet uses it. If not, STOP-5 fires.
- Migration of v2's 25 existing tests to the canonical-Bundle shape may surface assertion-rewrite work
- The macro structure has nested quasiquote layers; sonnet must keep depth tracking honest

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Methods bundled in defrecord (per Pattern 3)
- Inheritance (Stone 227.3 RETIRED)
- Setters (future)
- Invariants (future)
- defprotocol (arc 232)
- typed-Tuple from-holon return (future)
- holon-rs / wat-edn / aliases

## Honesty deltas accepted

- Test naming may differ from sketch; sonnet picks consistent shape with arc 227 conventions
- The `name-of pair` / `var-of pair` decomposition for the field-list parsing may need substrate primitives; if those primitives don't exist, sonnet writes them as wat-level helpers in the same file (NOT new substrate Rust)
- Bundle traversal for assertion may use existing primitives (Bundle/first + recursion) or chained accessors

## Honesty deltas NOT accepted (STOP triggers fire)

- **N≥2 case still panics at expand time** — STOP-5; ship the composition or STOP, not both
- **Flat-Bind workaround retained for any N** — STOP-6; canonical Bundle is non-negotiable
- **Substitute composition** (anything other than the proven probe-mirrored pattern) — STOP-8
- **"STOP-5b deferred" language anywhere in SCORE** — REJECT; v3 explicitly retires that framing
- **SCORE row marked PASS without binding test fn** — REJECT; each row binds 1:1 to a test
- **Test exercises only N≤1** — REJECT; row 6/7/8 require N=2 + N=3 tests
- **Historical artifact rewritten** — STOP-9
- **Touching holon-rs** — STOP-4
- **Adding aliases** — HARD CUT

## STOP triggers (cross-ref from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** test failure beyond migrated probes
- **STOP-3:** 360 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** N≥2 still panics — composition proven by probes; iterate
- **STOP-6:** canonical instance shape not produced
- **STOP-7:** bash discipline
- **STOP-8:** substitute composition (not probe-mirrored)
- **STOP-9:** historical artifact rewritten instead of append-only
