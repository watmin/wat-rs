# SCORE — STONE 255.56

The first strike, kept below, is STOP-2. The amend resumed the stone. That
resume switched the gates and is STOP-1. The floor is red and was not re-run.

## Resume — STOP-1. `assert-eq` refuses `:()` and `:test::Wrapper`

Struck on top of `5a9e9bd1d` (the amend) and `0197212c8` (the STOP-2 score).
Draw `56faa2bc7`, brief `e8646678e`. The amend's three rulings are in the
tree: `wat/doctest.wat` compares through `:wat::doctest::matches?`, whose
parameters are `:wat::core::Equatable`; `wat/rete/acc.wat` is unchanged;
the three contract goldens are unchanged.

`<` and `=` ask the classes. After widening, equality unifies (and commits
the substitution only when that succeeds) or takes the numeric cross
(`is_numeric_check_path`), then `require_class` refuses a remaining
`TypeExpr::Var` and asks `:wat::core::Equatable`. Ordering keeps its unify,
and both the numeric-cross path and the successful unify ask
`:wat::core::Orderable`. `is_type_orderable` and `is_type_equatable` are
deleted. `grep` of `src` finds those names only in comments in
`src/runtime.rs`.

Z1: an empty `:..` slot list is a membership miss, so `:()` is not a member.
`(:wat::core::extend-type :wat::core::nil :wat::core::Equatable)` is in
`wat/class.wat`. `require_class` sees a path's declared edge before alias
expansion, which is how the `nil` path is Equatable. The empty tuple is not
that path.

`assert-eq`, `dedupe-walk`, and `dedupe` carry
`[T :< :wat::core::Equatable]`. So does
`wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat`. A bound on `T`
is not applied when the expected slot is still a variable, so `dedupe-walk`
keeps `(Stream :- [T])`. The six `ord_result_*` wat files pin both `Result`
parameters with a helper whose return type is the result (or the tuple).
`Pt`/`HPt` equality moved to
`tests/types/probe_arc237_sC3_macro_split_cross_flavor.wat.bad`; the Rust
test asserts `TypeMismatch` on `:wat::core::=` parameter `#2`, expected
`:my::Pt`, got `:my::HPt`. `values_compare` is `pub(crate)` again. The
same-class newtype row goes through `<`. Different classes return `None`
from `src/runtime.rs`'s `different_newtype_classes_do_not_compare`. The
agreement test is deleted. Ledger `LEDGER_TOTAL` is 195 (the
`is_type_equatable` and `is_type_orderable` rows left the frozen table).

### The floor

`.floor/2026-09-27T07-53-24Z` was not re-run.

```
Summary [ 361.259s] 6192 tests run: 6179 passed (14 slow), 13 failed, 23 skipped
```

`[floor] ⛔ RED — exit=100`. Arm: `.floor/2026-09-27T07-53-24Z/ARM.txt`.

Against 6186 passed at `55fe36769`: 6192 run is 6186 plus 7 operator tests
plus `different_newtype_classes_do_not_compare`, minus the agreement test,
minus the 255.55 different-class integration test. 6179 passed is that set
minus the 13 failures below. Skipped stays 23.

The 13 failures are a startup check error. The test runner panics at
`src/host/test_runner.rs:467:13`.

Five tests, one file, one arm. `BoundNotSatisfied`. `:wat::test::assert-eq`
parameter `T` is bounded by `:wat::core::Equatable`; got `:()`. Lines 40 and
65 of `wat-tests/bracket.wat`, column 4–25. `each` / `each-worker` return
`:wat::core::nil` and the call compares that with `nil`. The checker reports
the empty tuple `:()`. Z1 says that tuple is not a member of the one-or-more
`:..` edge, and the `nil` path edge is a different type. These calls are not
a step-5 site. `:()` was not made Equatable.

```
thread 'wat-test:::wat-tests::bracket::each-worker-returns-nil' (2628854) panicked at src/host/test_runner.rs:467:13:
test-runner: /home/john/work/holon/wat-rs/wat-tests/bracket.wat: startup: #wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/BoundNotSatisfied {:message ":wat::test::assert-eq: type parameter T is bounded by :wat::core::Equatable; got :()" :location #wat.core/Span {:file "wat-tests/bracket.wat" :line 40 :col 4 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 40 :col 25}}} :causes [] :function ":wat::test::assert-eq" :param "T" :bound ":wat::core::Equatable" :got ":()"} #wat.check/BoundNotSatisfied {:message ":wat::test::assert-eq: type parameter T is bounded by :wat::core::Equatable; got :()" :location #wat.core/Span {:file "wat-tests/bracket.wat" :line 65 :col 4 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 65 :col 25}}} :causes [] :function ":wat::test::assert-eq" :param "T" :bound ":wat::core::Equatable" :got ":()"}]}
```

The same panic, same two spans, is the arm of:

- `deftest_wat_tests_bracket_each_worker_returns_nil`
- `deftest_wat_tests_bracket_map_preserves_order_50`
- `deftest_wat_tests_bracket_each_returns_nil`
- `deftest_wat_tests_bracket_map_doubles_in_order`
- `deftest_wat_tests_bracket_map_worker_ignoring_wid_equals_map`

Seven tests, one file, one arm. `BoundNotSatisfied`. The same parameter,
got `(:test::Wrapper :- [:wat::core::i64])` at `wat-tests/edn/roundtrip.wat:75`
and `(:test::Wrapper :- [:test::Event.Sell])` at line 86. `:test::Wrapper`
is `(:wat::core::defstruct :test::Wrapper :- [E] …)`. It is not Vector, List,
Option, or Result. No Equatable edge was added for it.

```
thread 'wat-test:::wat-tests::edn::roundtrip-enum-variant' (2631871) panicked at src/host/test_runner.rs:467:13:
test-runner: /home/john/work/holon/wat-rs/wat-tests/edn/roundtrip.wat: startup: #wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/BoundNotSatisfied {:message ":wat::test::assert-eq: type parameter T is bounded by :wat::core::Equatable; got (:test::Wrapper :- [:wat::core::i64])" :location #wat.core/Span {:file "wat-tests/edn/roundtrip.wat" :line 75 :col 6 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 75 :col 27}}} :causes [] :function ":wat::test::assert-eq" :param "T" :bound ":wat::core::Equatable" :got "(:test::Wrapper :- [:wat::core::i64])"} #wat.check/BoundNotSatisfied {:message ":wat::test::assert-eq: type parameter T is bounded by :wat::core::Equatable; got (:test::Wrapper :- [:test::Event.Sell])" :location #wat.core/Span {:file "wat-tests/edn/roundtrip.wat" :line 86 :col 6 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 86 :col 27}}} :causes [] :function ":wat::test::assert-eq" :param "T" :bound ":wat::core::Equatable" :got "(:test::Wrapper :- [:test::Event.Sell])"}]}
```

The same panic is the arm of `deftest_wat_tests_edn_roundtrip_{enum_variant,bool,nested,i64,vec,string,struct}`.

The thirteenth failure is
`wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`
(361.252s). Four of 790 files do not load:

```
thread 'wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime' (2572880) panicked at /home/john/work/holon/wat-rs/tests/lint/wat_scripts_fixes_load.rs:64:5:
4 of 790 wat-scripts/ files do not load on the current runtime (rotted):
  wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat
      #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::core::=: parameter #1 expects a resolved type; the operand is unresolved; got _" :location #wat.core/Span {:file "wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat" :line 25 :col 42 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 25 :col 45}}} :causes [] :callee ":wat::core::=" :param "#1" :expected "a resolved type; the operand is unresolved" :got "_" :remedies []}]}
  wat-scripts/scratch-pad/255-home-12-ast-verify-examples.wat
      #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::core::=: parameter #1 expects a resolved type; the operand is unresolved; got _" :location #wat.core/Span {:file "wat-scripts/scratch-pad/255-home-12-ast-verify-examples.wat" :line 52 :col 70 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 52 :col 73}}} :causes [] :callee ":wat::core::=" :param "#1" :expected "a resolved type; the operand is unresolved" :got "_" :remedies []}]}
  wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat
      #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::core::=: parameter #1 expects a resolved type; the operand is unresolved; got _" :location #wat.core/Span {:file "wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat" :line 57 :col 51 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 57 :col 53}}} :causes [] :callee ":wat::core::=" :param "#1" :expected "a resolved type; the operand is unresolved" :got "_" :remedies []}]}
  wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat
      #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/BoundNotSatisfied {:message ":user::eq-generic: type parameter T is bounded by :wat::core::Equatable; got [:wat::core::i64 :-> :wat::core::i64]" :location #wat.core/Span {:file "wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat" :line 37 :col 8 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 37 :col 25}}} :causes [] :function ":user::eq-generic" :param "T" :bound ":wat::core::Equatable" :got "[:wat::core::i64 :-> :wat::core::i64]"}]}
```

The first two compare `eval-ast!` Ok payloads. Each call mints its own `T`,
the same shape the amend moved out of `wat/doctest.wat` and into `matches?`.
These two scratches are `_` rows in
`tests/types/probe_arc255_54_class_hits.txt` and were not pinned. The third
compares `t1` with `t2` at
`255-struct-field-is-a-constant-projection.wat:57`; that operand is `_`
(255.46: `struct-field` has no clause). It is the same hits-file `_` list
and was not pinned. Pinning them does not make `:()` or `:test::Wrapper` a
member, so they were left. The fourth is the bound step 5 asked for:
`:user::eq-generic` is called with two `[:wat::core::i64 :-> :wat::core::i64]`
values, and a function is not Equatable.

`bracket.wat` and `roundtrip.wat` were not edited. That is the stop.

### Census

Pre `.census/2026-09-27T06-55-26Z.txt`, post `.census/2026-09-27T08-01-30Z.txt`.
Both 2294 files. Nonzero 215 → 224. Nine flips, all 0→1.
`scripts/replay/census.sh --diff` printed STOP-8 for each and exited 8.

Expected by the amend, and left byte-identical:

- `tests/resolve/probe_arc251_fix_source_local_rules__contract-06a-less-than.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-06b-less-equal.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-07-greater-than.wat`

The other six are the floor's files: the three unpinned `_` scratches,
`probe-eq-generic-instantiation.wat`, `wat-tests/bracket.wat`,
`wat-tests/edn/roundtrip.wat`.

### The rest of the proof

Clippy's first pass, in the same shell as the floor, failed
`empty_line_after_doc_comments` on the old `///` block above `require_class`.
That block is now a `//` comment. After that,
`cargo clippy --release --all-targets -- -D warnings` finished in 12.32s,
rc 0. The floor was not re-run on that comment.

Delta `.delta/2026-09-27T08-07-00Z`: ORIG-CLEAN 160/179, CONV-CLEAN 158/179,
NEW 2, RECOVERY 0, exit 0. The two NEW files are
`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Operator rows that the floor did not fail: `a_newtype_orders_through_less_than`,
`nil_is_equatable_and_not_orderable`, `a_struct_is_not_equatable`,
`an_impure_enum_is_not_equatable`, `an_unresolved_operand_is_refused`,
`numeric_cross_is_admitted`, `an_enum_compares_with_its_variant`.

## First strike — STOP-2. Two stdlib operands have no type to pin

Struck against draw `56faa2bc7` (brief drawn at `e8646678e`). The gates were
not switched. `<` and `=` still use the predicates. Nothing in `src/` or
`wat/` changed.

## Why this stops

Step 5 says to pin each unresolved stdlib operand to the type the value
actually has, and not to cast it to a class. STOP-2 fires when that type
is not one type.

`wat/doctest.wat:118` is `(:wat::core/= got want)`. `got` is the `:Ok` payload
of `(:wat::eval-ast! (:wat::intrinsic::Example/expr ex))` and `want` is the
`:Ok` payload of a second call on `expected-ast`. `:wat::eval-ast!` returns
`(Result :- [T :wat::core::EvalError])` and `T` is the caller's expected
value (`src/check.rs:20540-20562`). Each call mints its own `T`. An example
evaluates to whatever that example returns. There is no one type to write
on either call. `:wat::core::Value` is the holder the 255.54 finding already
refused to call `Equatable`, and `:wat::WatAST` is the input of `eval-ast!`,
not its result.

`wat/rete/acc.wat:68` and `:87` compare `v` with `cur`. `cur` is the `:value`
of an `(:wat::core::Option :- [:wat::core::i64])` accumulator, so that side
is `i64`. `v` is `(:wat::core::Option/expect (:wat::map::get bindings var) …)`.
`:wat::rete::Element.bindings` is the bare `:wat::core::PersistentMap`
(`wat/rete.wat:40`), and `wat/rete/oracle/accum-pass.wat:26` says that bare
map accepts any accum result. `:wat::map::get` returns `(Option :- [V])`
with `V` fresh (`src/check.rs:23119-23127`). A `let` binding has no type
slot (`src/check.rs:8403`, a flat `[name expr]` vector), so there is no
place to write "this binding is an i64" without claiming the map is
`(:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])`.
That claim is false for the other accum results that share the map.

Both are the STOP-2 shape: the value is heterogeneous, and the type the
checker sees is a fresh variable because the producer was declared that way.
No gate was added on top of that.

## What a switch would do to the other named sites

Read from the schemes, not from a floor. The work list's order is: widen,
then `unify` or the numeric cross, then refuse an operand that is still a
`TypeExpr::Var`, then the class. A variable unified with a concrete operand
is no longer unresolved.

These stay a concrete type after that `unify`, so they are not STOP-2:

| site | other side |
|---|---|
| `tests/collection/list.wat:46` | the literal `20` (`i64`) |
| `tests/types/probe_arc234_7a_base_record_roundtrip.wat:16` | the record `p` |
| `tests/types/probe_arc234_7b_holon_record_roundtrip.wat:16` | the record `h` |
| `tests/types/uuid_edn_roundtrip_typed.wat:7` | the uuid `u` |
| `tests/value/wat_arc220_char.wat:79` | the char `orig` |

`:wat::edn::read` returns a fresh `T` (`src/check.rs:21610-21618`). The
round-trip's other operand is the value that was written.

These do not unify to a concrete type:

| site | both sides |
|---|---|
| `wat/test.wat:62`, `wat/seq.wat:560`, `wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat:32` | rigid `:T`. Step 5's `[T :< Equatable]` is the fix, and it was not applied |
| `wat-scripts/scratch-pad/255-probe-metadata-of-one-shape.wat:63` | two `:wat::core::Value`s. Step 5 says stop comparing them. Not applied |
| `tests/resolve/probe_arc251_fix_source_local_rules__contract-06a-less-than.wat` and `06b`, `07` | the whole file is `(wat.core/< a b)` (and `<=`, `>`). `a` and `b` have no type. Census `.census/2026-09-27T06-55-26Z.txt` has each at rc 0. After Refuse they would not check. They are not step-5 sites. That flip would be STOP-1, so they were not edited |

## Proof

Pre-census `.census/2026-09-27T06-55-26Z.txt` (2294, the unmodified draw).
No post-census, no floor, no clippy: the tree is the draw.
