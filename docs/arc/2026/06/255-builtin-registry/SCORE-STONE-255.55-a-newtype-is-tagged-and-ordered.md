# SCORE — STONE 255.55: a newtype is tagged, and ordered by its inner value

Struck against draw `52a2ea7ce` (brief drawn at `11769ba51`). Written fresh
against `main`. The names are `AggregateValue.is_newtype` and
`AggregateValue::newtype`. Nothing was cherry-picked.

## F-030 on this tree

It reproduces. Before the tag, a newtype was `AggregateValue::struct_` with
the field name `"0"`. The EDN writer passed that name to `Keyword::new`,
which panics: `invalid keyword name "0": first character must be non-numeric`.
Measured by `edn::render::tests::f030_measure_newtype_shaped_struct_panics_on_write`
against the writer as it stood, rc 0 (the panic was the expected result).

## The marker

`is_newtype` is stamped only by `AggregateValue::newtype`. `eval_struct_new`'s
`TypeDef::Newtype` arm calls that constructor. `struct_` still leaves the
flag false. Identity, `PartialEq`, `Hash`, and `Debug` do not read the flag.
A struct whose only field is named `"0"` still dies in `Keyword::new` with
that same sentence (`the_marker_not_the_shape_decides`).

## Tagged EDN

A newtype writes `#ns/Name <inner>`. `(:u::T 7)` writes `"#u/T 7"`. Reading
that string back is `=` to `(:u::T 7)`. The untyped reader resolves a newtype
tag before the map-shaped body switch, because the body is the inner value.
The typed reader requires the tag and rebuilds the newtype. A bare inner
value is not accepted there.

`println` in this harness stops at `ServiceNotRunning` before it renders.
The panic site is the writer `println` calls. The print row is that writer:
`(:wat::edn::write (:u::T 5))` is `"#u/T 5"`.

## Ordering

`values_compare` has one new arm: two newtypes of the same class compare by
their inner values, recursively. Different classes return `None`. The test
calls `wat::runtime::values_compare` on the values the fixture builds.
`(:u::T 1)` against `(:u::T 2)` is `Ordering::Less`. `(:u::T 1)` against
`(:u::U 1)` is `None`.

`<` still uses `is_type_orderable`. That predicate does not admit a newtype,
so the wat operator is not the path that reaches the arm.

`values_equal` already compares an aggregate by nature, class, and fields
(`src/runtime.rs` aggregate arm). No new equality arm. The round-trip row
is that comparison: read-back `=` is true.

## Proof

Floor `.floor/2026-09-27T05-44-51Z` was red and was not re-run.
`Summary [ 362.173s] 6186 tests run: 6183 passed (16 slow), 3 failed, 23 skipped`.
The three arms are the test-text lints, all on this stone's own rows:

- `tests/lint/no_inlined_edn.rs:805` — `probe_arc255_55_newtype.rs:14` and `:33`, the `"#u/T …"` literals.
- `tests/lint/no_inlined_wat_in_tests.rs:440` — that same file, the wat forms inside the panic strings.
- `tests/lint/no_loose_string_assert.rs:135` — `src/edn/render.rs:5266`, `contains` on the keyword-panic text.

The expected strings moved into the wat fixture and are compared with `=`.
The panic text is `assert_eq!` of the whole sentence. A new floor followed.

Floor `.floor/2026-09-27T05-53-04Z`: 6186 passed, 23 skipped, rc 0. That is
6179 at `0746b9307` plus these 7 rows (5 in `probe_arc255_55_newtype`, 2
writer unit tests). STOP-1 did not fire: no record, enum, or scalar golden
went red. STOP-2 did not fire: the names fit the construction funnel that
is already on `main`.

Clippy `--release --all-targets -- -D warnings`: rc 0.

Pre-census `.census/2026-09-27T05-38-03Z.txt` (2293, the unmodified draw).
Post `.census/2026-09-27T06-00-11Z.txt`: `census-diff: no STOP-8`. 0 rc flips.
215 nonzero of 2294. One new file, rc 0:
`tests/types/probe_arc255_55_newtype.wat`.

Delta `.delta/2026-09-27T06-01-06Z`: NEW 2 / RECOVERY 0. The two files are
`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.
