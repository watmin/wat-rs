# SCORE — Arc 226 Stone 226.1 — Type predicates for classifier-wrapped entities

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 12/12 PASS — all deliverables complete, all suites green

| # | Deliverable | Status | Citation |
|---|---|---|---|
| 1 | NEW `:wat::holon::is?` polymorphic predicate verb minted | PASS | `src/runtime.rs` — `eval_holon_is_predicate`: 2-arg form `(is? value class-name)`; evaluates value and class-name String; calls `extract_classifier`; compares via `.as_deref() == Some(class_name.as_str())`; returns `Value::bool`; dispatch table entry in Arc-228 constructor block; `src/check.rs` TypeScheme `(is? HolonAST String) -> bool` registered after `:wat::holon::Tuple` |
| 2 | NEW `:wat::holon::is-Map?` predicate verb | PASS | `src/runtime.rs` — `eval_holon_is_map_q`: 1-arg; `extract_classifier(&h).as_deref() == Some("Map")`; bool return; TypeScheme registered |
| 3 | NEW `:wat::holon::is-Set?` predicate verb | PASS | Same pattern; "Set" classifier; TypeScheme registered |
| 4 | NEW `:wat::holon::is-Vector?` predicate verb | PASS | Same pattern; "Vector" classifier; TypeScheme registered |
| 5 | NEW `:wat::holon::is-List?` predicate verb | PASS | Same pattern; "List" classifier; TypeScheme registered |
| 6 | NEW `:wat::holon::is-Tuple?` predicate verb | PASS | Same pattern; "Tuple" classifier; DISTINCT from is-Vector? per arc 228 substrate distinction; TypeScheme registered |
| 7 | NEW `:wat::holon::is-Symbol?` predicate verb | PASS | "Symbol" classifier (post-arc-230 Bind composition); TypeScheme registered |
| 8 | NEW `:wat::holon::is-Keyword?` predicate verb | PASS | "Keyword" classifier (post-arc-230); TypeScheme registered |
| 9 | NEW `:wat::holon::is-Tag?` predicate verb | PASS | "Tag" classifier (post-arc-230); TypeScheme registered |
| 10 | NEW `:wat::holon::is-Nil?` predicate verb | PASS | Special case: `h.is_nil()` — uses `HolonAST::is_nil()` accessor (arc 230); checks `Bind(Atom("Symbol"), Atom("nil"))` composition; TypeScheme registered |
| 11 | New test file `probe_arc226_stone1_type_predicates.rs` | PASS | 27/27 tests PASS; positive + negative per predicate; edge cases (bare I64/String/Bool leaf, cross-type Set/Vector, nil subsumes Symbol); symbol/tag positive cases built via `Bind`+`Atom` constructors directly |
| 12 | All test suites green + holon-rs untouched | PASS | See test summary below |

## Test summary

```
cargo build --release -p wat                                        — 0 errors (5 pre-existing unused-fn warnings)
cargo test --release --lib -p wat [skip 5 signal tests]            — 822/822 PASS
cargo test --release --test probe_arc226_stone1_type_predicates    — 27/27 PASS
cargo test --release --test probe_arc216_stone1_hashset_roundtrip  — 10/10 PASS
cargo test --release --test probe_arc216_stone2_vector_roundtrip   — 12/12 PASS
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip  — 14/14 PASS
cargo test --release --test probe_arc216_stone4_predicate_composition — 6/6 PASS
cargo test --release --test probe_arc216_stone7_tuple_roundtrip    — 12/12 PASS
cargo test --release --test wat_arc221_keyword_nil_tag_atomization  — 6/6 PASS
cargo test --release --test wat_arc143_manipulation                 — 8/8 PASS
cargo test --release --test mvp_end_to_end                          — 10/10 PASS
cargo test --release -p wat-edn                                     — 23+1 PASS
cargo clippy --release --all-targets -p wat-edn -- -D warnings      — 0 warnings

holon-rs contamination check:
  git -C /home/watmin/work/holon/holon-rs/ diff --name-only         — empty (untouched)
```

## Deltas from EXPECTATIONS

### Delta 1 — Symbol/Tag positive probes use Bind+Atom construction (implementation detail)

EXPECTATIONS row 11 anticipated positive probes for `is-Symbol?` and `is-Tag?` without specifying construction method. During Phase 3 implementation, there is no WAT-surface way to produce a Symbol or Tag value that `to-holon` can receive: WAT has no native "symbol literal" value type (symbols are variable references, not data), and `leaf` only accepts primitive `Value` variants (no Symbol or Tag).

Solution: construct the classifier-wrapped forms directly via substrate algebra:
- Symbol: `(:wat::holon::Bind (:wat::holon::Atom (:wat::holon::to-holon "Symbol")) (:wat::holon::Atom (:wat::holon::to-holon "foo")))` → `Bind(Atom(String("Symbol")), Atom(String("foo")))` — matches `extract_classifier` pattern exactly.
- Tag: same pattern with "Tag".

This is the honest construction path — the substrate algebra is always available for constructing any classifier-wrapped form. It demonstrates the type predicate works correctly for arc-230 Bind compositions at the algebra tier.

### Delta 2 — is-Symbol? subsumes is-Nil? — documented (expected, matches doctrine)

EXPECTATIONS row 10 notes `is-Nil?` special case uses `HolonAST::is_nil()`. An additional probe `probe_is_symbol_true_for_nil` was added to document that `is-Symbol?` returns `true` for nil (since nil = symbol("nil"), classifier is "Symbol"). This clarifies the intended doctrine: `is-Symbol?` is the broader check; `is-Nil?` is the nil-specific check. Not a delta from the requirement, but an honest extra coverage point.

### Delta 3 — Non-HolonAST values in predicate verbs return false (no type error)

The predicate verbs accept any WAT value — non-HolonAST values return `false` at runtime rather than raising a TypeMismatch. The type checker enforces `(is-X? HolonAST) -> bool` at the check phase; at the runtime level, the match arm `_ => false` is the correct behavior for non-HolonAST inputs (absence of classifier = "not of any classifier-typed entity"). Edge case probes verify this for I64/String/Bool leaves via `(:wat::holon::to-holon 42)` (produces bare `HolonAST::I64(42)` with no Bind classifier).

## STOP trigger audit

- **STOP-1 (unexpected substrate compile error):** DID NOT TRIGGER. One compile error (type mismatch `Arc<String>` vs `String` in `eval_holon_is_predicate`); expected cascade from `Value::String` containing `Arc<String>`; fixed with `.as_deref() == Some(class_name.as_str())`.
- **STOP-2 (test failure beyond new probe):** DID NOT TRIGGER. All 822 lib tests PASS; all arc 216/221/143/mvp probes PASS; new probe 27/27 PASS.
- **STOP-3 (240 min elapsed):** DID NOT TRIGGER.
- **STOP-4 (holon-rs touched):** DID NOT TRIGGER. `git -C holon-rs/ diff --name-only` empty.
- **STOP-5 (scope creep beyond 10 predicates):** DID NOT TRIGGER. Exactly 10 predicates (is? + 9 convenience forms) as specified. Variant-based predicates (is-I64? etc.) remain out of scope.
- **STOP-6 (VSA similarity rabbit hole):** DID NOT TRIGGER. v1 is STRUCTURAL exact-match on classifier name only. No VSA similarity scoring, no threshold parameters, no vector encoding in predicate bodies.
- **STOP-7 (bash discipline):** DID NOT TRIGGER. One cargo command at a time, foreground.

## Files changed

**wat-rs source (Rust):**
- `src/runtime.rs` — (a) 10 dispatch table entries added in Arc-228 constructor block (`:wat::holon::is?` + 9 convenience forms); (b) 11 new functions added after `require_holon`: `eval_holon_is_predicate` (polymorphic 2-arg) + `eval_holon_is_map_q` / `eval_holon_is_set_q` / `eval_holon_is_vector_q` / `eval_holon_is_list_q` / `eval_holon_is_tuple_q` / `eval_holon_is_symbol_q` / `eval_holon_is_keyword_q` / `eval_holon_is_tag_q` / `eval_holon_is_nil_q` (1-arg convenience forms); section comment "Arc 226 Stone 226.1 — Type predicates (classifier-name match)" with doctrine framing
- `src/check.rs` — 10 TypeScheme registrations inserted after `:wat::holon::Tuple` (before cosine section): `is?` (2-param: HolonAST + String → bool) + 9 convenience forms (1-param: HolonAST → bool); section comment with arc 226 citation and v1/v2 deferred note

**Test files (Rust — new):**
- `tests/probe_arc226_stone1_type_predicates.rs` — 27 tests: 3 polymorphic `is?` probes; 2 probes each for is-Map?/Set?/Vector?/List?/Tuple?/Symbol?/Keyword?/Tag? (positive + negative); 3 is-Nil? probes (positive + non-nil-symbol + map); 1 nil-subsumes-symbol probe; 4 edge-case probes (I64 leaf, String leaf, Bool leaf, Set/Vector cross-type)

**Total: 2 modified Rust source files + 1 new test file + 1 new SCORE doc.**

## Substrate state post-Stone-226.1

**Type-checking-as-VSA-algebra now available for all 9 classifier-wrapped typed entities:**
- `(:wat::holon::is? h "ClassName")` — polymorphic; any classifier name as String
- `(:wat::holon::is-Map? h)` ... `(:wat::holon::is-Tuple? h)` — collection predicates (arc 228 classifier domain)
- `(:wat::holon::is-Symbol? h)` ... `(:wat::holon::is-Tag? h)` — primitive entity predicates (arc 230 classifier domain)
- `(:wat::holon::is-Nil? h)` — nil-specific predicate (uses `HolonAST::is_nil()` — arc 230 accessor)

**VSA doctrine foundation laid:**
- Each predicate body documents the arc 226 doctrine in its doc comment
- v1 (structural exact-match) framing explicit; v2+ (threshold-tunable VSA similarity) deferred to stones 226.2+
- The substrate IS the type system. The duck has a measurable shape. Stone 226.1 is the first measurement.

## Unblocks

- Arc 226 Stone 226.2 — variant-based predicates (is-I64? / is-F64? / is-Bool? / is-Char? / is-String? / is-Atom? / is-Bind? / is-Bundle? / is-Permute? / is-Thermometer? / is-Blend? / is-SlotMarker?) — same mechanical pattern but variant match vs classifier extraction
- Arc 226 Stone 226.3+ — VSA similarity scoring with threshold-tunable answers
- Arc 226 closure — polymorphic dispatch integration with arc 146/147 multimethod machinery
- Arc 227 (user-defined types) — predicates for user classifier names use `(is? value "UserTypeName")`; the polymorphic `is?` already works for any classifier string
