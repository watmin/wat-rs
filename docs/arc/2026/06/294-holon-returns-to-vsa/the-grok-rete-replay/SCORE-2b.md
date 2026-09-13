# SCORE 2b — main's record teaches what main refuses

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Binary: `./target/release/wat` (boots). Floor + clippy run.

```
floor  scripts/floor.sh   .floor/2026-09-13T04-45-24Z
       Summary [ 219.334s] 5411 tests run: 5411 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

First floor `.floor/2026-09-13T04-40-00Z` was RED on `no_loose_string_assert` (our new test's `assert!(msg.contains("…"))`). Captured, not re-run. Fixed by splitting the contains onto its own statement. Second floor green. No STOP.

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **7/7 PASS.** `cargo nextest run --release -E 'test(refusals_teach_the_dot_separator)'`. One test per D1 site. |
| E2 | **exactly `match_arm_not_namespaced_teaches_dot` RED.** Reverted `match_arm.rs:108` to `<enum>::<Variant>`, re-ran E1, restored. Panic names the site: `match_arm.rs:108 must name the dot form`. Other six stayed PASS. |
| E3 | **no remaining CODE string teaches `::` as the variant separator.** Census of `src/` `crates/` `wat/` (code before `//` / `;;`). D1's seven format strings all say `.`. Comment survivors (not code): `src/reflect/verbs.rs:1585` doc, `src/check.rs:13761` doc, `src/rete/expr_ir.rs:58,602` comments. None are live refusal text. No new runes. |
| E4 | **both describe `.`.** `compose_variant` writes `.`; `decompose_variant` reads `.`; lint help `compose_variant(…) -> {enum}.{variant}`. `only_identifier_rs_spells_the_variant_separator` PASS. |
| E5 | **RED, verbatim below.** `contract_03` now invokes `(:test::variadic-wrap 1 2 3)` via `:test::three`. Today's template `` `(:wat::core::Vector ~@items) `` dies at check. |
| E6 | **GREEN.** Template is `` `(:wat::core::Vector :- [:wat::core::i64] ~@items) ``. `contract_03` observes a 3-element `Value::Vec` of `1 2 3`. |
| E7 | **each prints `echo:z`.** `./target/release/wat wat-scripts/probes/arc-170/probe-m1-ann-erase.wat` and `…erase2.wat`. Output is the string, not the RecvOutcome tag. rc 0. |
| E8 | **blast radius only.** `git diff --stat`: identifier.rs, check.rs, match_arm.rs, one_variant_separator.rs, the c03 `{.rs,.wat}`, the two probes, plus the new diagnostics test + fixtures. No predicate change. |
| E9 | **5411 passed, 0 failed. Clippy 0.** |

## E5 — the red, verbatim

```
thread 'probe_arc241_stone17_defmacro_canonical::contract_03_defmacro_canonical_rest_binder_works' panicked at tests/macros/probe_arc241_stone17_defmacro_canonical.rs:51:29:
c03 startup (rest-binder wrap of 1 2 3): #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::core::vec form: first argument must be a `(Head :- [T])` type param-spec" :location #wat.core/Span {:file "tests/macros/probe_arc241_stone17_defmacro_canonical_c03.wat" :line 7 :col 25 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 7 :col 26}}} :causes [] :head ":wat::core::vec" :reason "first argument must be a `(Head :- [T])` type param-spec" :remedies []}]}
```

That is the param-spec wall. The rest binder was never the defect; the template was.

## D1 — seven sites, each driven

| site | input that fires it | remedy the predicate accepts |
|---|---|---|
| `match_arm.rs:108` | `[:u::E::A {:x x} x]` (`decompose_variant` is None → not namespaced) | `[:u::E.A {:x x} x]` |
| `match_arm.rs:187` | unreachable from wat (callers only pass the five bares, which already name `:wat::core::Option.Some`). Pin is the format string `Type.Variant FQDN`. | `(:wat::core::Option.Some {:value 1})` |
| `check.rs:2877` | `(:wat::runtime::variant-parent-of)` arity 0 | one arg `:u::E.A` |
| `check.rs:6955` | unreachable from wat (`is_namespaced_variant` already required). Pin is `variant `{path}` must be `<enum>.<Variant>``. | same as 108's remedy |
| `check.rs:7253` | unreachable from wat (same guard). Nested `[:u::E::A {:x x}]` hits "vector sub-patterns are not supported", not 7253. Pin is `{path}` form. | nested `[:u::E.A {:x x}]` |
| `check.rs:7508` | keyword sub-pattern `:B` | `:u::E.B`. Dropped retired `:None` from the message (the five are intercepted above this arm). |
| `check.rs:7755` | list ctor `(:u::E::A x)` | `(:u::E.A x)` |

Not a text sweep: 108 and 7755 refuse `::` because `.` is the split; 2877 is arity; 7508 is a bare keyword. Two predicates, four live wat inputs, three format-string pins for dead arms.

## D3 — after the `::` fix, two more walls

The composition finding named `::` and the leaked `{:deps}`/`{:pair}` / `:probe::CMsg`. After those, both children still died:

1. `recv` / `Echo/echo` return `RecvOutcome` on today's binary. Wrapped like `probe-m1-surface-ast.wat` / `probe-m1-phantom-d.wat`. Send gained the `Stopped` arm.
2. erase's child `CMsg` vs parent's `PMsg` — EDN tag `#probe/PMsg.Setup` will not decode as `CMsg`. erase2 had already unified on `PoolMsg`. Child enum renamed to `PMsg` (same name, child's concrete Address type still differs). Then both print `echo:z`.

## First floor (captured, not re-run)

`.floor/2026-09-13T04-40-00Z/` Summary [222.301s] 5411 tests run: 5410 passed, 1 failed, 22 skipped.

Arm: `no_loose_string_assert::tests_carry_no_loose_string_assert` at `tests/lint/no_loose_string_assert.rs:112` — `has_loose_string_match` on `assert!(msg.contains("…"))` in the new diagnostics file, lines 41, 69, 110, 125.

## Blast radius

Message strings in `src/match_arm.rs` and `src/check.rs` · door doc in `crates/wat-reader/src/identifier.rs` · lint help in `tests/lint/one_variant_separator.rs` · `tests/diagnostics/refusals_teach_the_dot_separator*` · c03 `{.rs,.wat}` · the two `probe-m1-ann-erase*.wat`. No predicate change. Not pushed.
