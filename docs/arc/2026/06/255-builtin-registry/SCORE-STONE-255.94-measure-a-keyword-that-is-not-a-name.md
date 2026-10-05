# SCORE — STONE 255.94: measure — a keyword that is not a name

**Drawn against `main` @ `84bc32638`. Scored on `main` @ `a53f1f537` (the weigh and this brief are already that commit).** Executor: grok, solo. Measuring stone. No floor. The scratch runtime counter was removed before this commit. The reader stone was not started.

`git diff 84bc32638 --stat` also shows the already-committed weigh and this brief. Those two docs stay. This commit adds the example, `keyword-not-a-name.tsv`, and this score. Nothing under `src/`, `crates/`, `wat/`, or `tests/` changes.

## The code wins three names in the brief

There is no `Value::Keyword`. The runtime keyword is `Value::wat__core__keyword(Arc<String>)` (`src/value/value.rs:61`). EDN is a different type: `wat_edn::OwnedValue::Keyword`. The parser emits `WatAST::Keyword(String, Span)` and does not classify positions; the census walks the tree.

On this main, `Display for Name` (`crates/wat-reader/src/identifier.rs:117`) writes `{namespace}/{name}`, or the bare name when the namespace is `BOUND_NAMESPACE`. There is no `Name::keyword`. The weigh's `Name::keyword()` that rebuilds `:ns::name` is the rejected branch, not this tree.

`Name::from_keyword` (`identifier.rs:150`) accepts any `::` body, including rendered `(…)` text. `Name::enter` (`:188`) returns `None` on rendered type text. `is_rendered_type_text` (`:219`) refuses `(` and `<`, and refuses `:-` except when those two bytes sit after a double colon. `:wat::core::->` is a name (the `enter` doc at `:187`). `compose_variant` (`:633`) is `format!("{enum_path}.{variant_name}")`.

## Instrument

`examples/keyword_not_a_name.rs`. It parses with the wat reader. Rust string literals go through `is_candidate_wat` (`src/codemod_driver.rs:340`) after `replace_placeholders_preserving_len`. A `.wat` file that fails to parse is a `parse-fail` row, not a regex scan. `parse-fail-count` is 0. The position function can emit `undecided:<role>`. The TSV has no such row. **STOP-1 did not fire.**

Populations: `stdlib` = `wat/`; `corpus` = `wat-scripts` + `wat-tests` + `tests`; `other` = `docs` + `examples` + `crates` + `benches`; `rust` = `.rs` under `src` + `crates` + `tests` whose string passes `is_candidate_wat`.

Surface forms are unexpanded. `defn` stays `defn`; the census does not expand macros. Declaration heads whose child 1 is the declared name: `defn`, `defn-`, `defmacro`, `def`, `defclause`, `defrecord`, `defenum`, `defstruct`, `structtype`, `deftest`, `defservice`, `defsurface`, `defalias`, `extend-type`, `derive`, `declare-acronyms`, `defrule`, `defquery`. A one-element list is not sliced at index 2.

Quote and quasiquote increment a quote depth; unquote and unquote-splicing decrement it. Depth above 0 forces position `quote`. A match arm's last child is the body; earlier children are the pattern. A list whose second child is `:-` is a type form. A binder spec, taken before the call case when quote depth is 0, is a list whose second child is `->`: the binders, the arrow, and the return are walked as `type`, and the rest are arguments. The arrow node itself is walked as `type`, so `:wat::core::->` is `type`. That binder-spec rule is what put `:fn(wat::core::i64)->wat::core::i64` in `reference` and `type`. Keywords inside a declaration's parameter or field vector are `type`. A map key is `fact-field` when the walk is in a pattern or when the parent list's head leaf is PascalCase and the list is not a `:-` form; otherwise `map-key`. Map values are `map-value`. A set element is `set`. A top-level form is `top`. A keyword argument is `rete` when the head text contains `::rete::` or `::rete/`, or the symbol namespace or text contains `rete`. The head is classified before that flag, so a rete call head stays `call-head`.

Spelling verdict (`verdict` in the example), one rule for the whole TSV:

- `enter` returns `None` → **DATA**, even in a name position.
- `enter` succeeds, and the spelling is seen in a name position (`call-head`, `declaration-name`, `type`, `match-variant`, `reference`, `rete`, `fact-field`, `quote`) and in a store position (`map-key`, `map-value`, `set`, `top`) → **MIXED**.
- name position only → **NAME**.
- store only → **DATA**.

Quote of a name is a name position. NAME spellings are counted. DATA and MIXED spellings are listed, one example site each. This is the instrument's rule. The builder rules the MIXED and DATA rows.

Planted position proof is the embedded `PROOF` in the example, not a second file. `cargo run --release --example keyword_not_a_name -- --proof --out /tmp/kw-25594.tsv` printed `proof ok, 39 :: keyword(s)` and `RC=0`. The rendered spelling `:wat::core::fn(T)->R` was `call-head`, `from_keyword=true`, `enter=false`. Wanted rows that passed: `:wat::core::defn` call-head, `:proof::declared` declaration-name and reference, `:wat::type::i64` type, `:wat::core::i64/+` call-head, `:wat::core::Option::Some` match-variant, `:proof::key` map-key, `:proof::val` map-value, `:proof::Person` call-head, `:proof::name` fact-field, `:proof::inside-quote` quote, `:wat::rete::insert` call-head, `:proof::fact-class` rete, `:proof::in-set` set, `:proof::top` top, `:wat::core::->` type. The committed example re-proves these positions. It does not re-prove origins. The origin counter is gone.

## Static census

TSV: `keyword-not-a-name.tsv` beside this score. 260 rows. Denominators are `::` keyword hits.

| population | files | hits | NAME spellings | DATA spellings | MIXED spellings |
|---|---:|---:|---:|---:|---:|
| corpus | 2198 | 116052 | 11113 | 11 | 8 |
| other | 32 | 861 | 238 | 0 | 0 |
| rust | 1297 | 5671 | 848 | 3 | 0 |
| stdlib | 65 | 1359 | 4 | 0 | 0 |

`rust-literals-tried` is 1324. Corpus NAME hits are 116052 − 372 DATA − 548 MIXED = 115132. Spelling sum 11113 + 11 + 8 = 11132.

### Counts by position

Hits, then distinct spellings. A missing row is zero. `fact-field` and `top` are zero in all four populations. The proof still classifies both, so the zero is a corpus result.

| position | corpus | other | rust | stdlib |
|---|---:|---:|---:|---:|
| call-head | 73007 / 5796 | 418 / 106 | 3392 / 425 | 236 / 2 |
| declaration-name | 11753 / 8104 | 141 / 108 | 535 / 353 | 0 |
| type | 16576 / 2603 | 207 / 77 | 741 / 244 | 19 / 2 |
| match-variant | 10206 / 387 | 86 / 25 | 600 / 39 | 0 |
| reference | 2162 / 926 | 8 / 4 | 205 / 52 | 0 |
| quote | 2280 / 626 | 0 | 186 / 55 | 1104 / 3 |
| rete | 53 / 23 | 1 / 1 | 12 / 5 | 0 |
| map-key | 2 / 1 | 0 | 0 | 0 |
| map-value | 12 / 7 | 0 | 0 | 0 |
| set | 1 / 1 | 0 | 0 | 0 |

Position sums equal the denominators (corpus 116052, other 861, rust 5671, stdlib 1359).

### Position verdict

A position is **NAME** when every spelling in it is NAME. It is **MIXED** when a DATA or MIXED spelling also sits there. The hit count of a ruling row is the spelling's total, not a split across its positions.

| position | corpus | other | rust | stdlib |
|---|---|---|---|---|
| call-head | MIXED | NAME | MIXED | NAME |
| declaration-name | MIXED | NAME | NAME | — |
| type | MIXED | NAME | MIXED | NAME |
| match-variant | MIXED | NAME | NAME | — |
| reference | MIXED | NAME | NAME | — |
| quote | MIXED | — | NAME | NAME |
| rete | MIXED | NAME | NAME | — |
| map-key | MIXED | — | — | — |
| map-value | MIXED | — | — | — |
| set | MIXED | — | — | — |
| fact-field | — | — | — | — |
| top | — | — | — | — |

`map-key` is one spelling, `:probe::inc` (2 hits). `set` is the same spelling (1 hit). `map-value` is 7 spellings and 12 hits, and those 7 are exactly the MIXED rows that include `map-value`. Rust `call-head` and `type` are MIXED because the three `<` operator spellings are DATA. Rust `quote`, `reference`, `rete`, `declaration-name`, and `match-variant` have no DATA or MIXED row.

Stdlib's four NAME spellings, all of them:

| spelling | positions | hits |
|---|---|---:|
| `:wat::core::quasiquote` | call-head, quote, type | 251 |
| `:wat::core::quote` | call-head | 5 |
| `:wat::core::unquote` | quote | 983 |
| `:wat::core::unquote-splicing` | quote, type | 120 |

251 + 5 + 983 + 120 = 1359.

### DATA rulings

`enter=false`, `from_keyword=true` on every row. These are not a finding that the operators are opaque tokens. `enter` refuses `<` and `(`. `from_keyword` accepts them. `:wat::core::<` at the cited site is the less-than function: `(:wat::core::< -5 0)` in `src/edn/render.rs:5699`, and `(:wat::i64::< 2 3)` in `src/runtime.rs:15766`.

Corpus, 11 spellings, 372 hits:

| spelling | positions | hits | one site |
|---|---|---:|---|
| `:fn(wat::core::i64)->wat::core::i64` | reference, type | 3 | `tests/function/fn_rename_mixed_legacy.wat:4:20` |
| `:wat::core::<` | call-head, quote, type | 174 | `wat-scripts/scratch-pad/255-19-start-impl-locus-param.wat:20:91` |
| `:wat::core::<=` | call-head, type | 37 | `wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat:65:24` |
| `:wat::f64::<` | call-head | 5 | `wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat:34:29` |
| `:wat::f64::<=` | call-head, type | 3 | same file `:35:29` |
| `:wat::i64::<` | call-head, type | 33 | `wat-scripts/scratch-pad/probe-reland10-journal-then-fire.wat:58:27` |
| `:wat::i64::<=` | call-head | 20 | `wat-scripts/fmt/run-width.wat:59:20` |
| `:wat::rete::f64::<` | call-head, quote, type | 4 | `wat-scripts/scratch-pad/probe-brief-f64-surface-is-a-stub.wat:36:29` |
| `:wat::rete::f64::<=` | call-head, quote | 2 | same file `:35:29` |
| `:wat::rete::i64::<` | call-head, quote, type | 89 | `wat-scripts/fmt/fixtures/doc-example.wat:1:293` |
| `:wat::rete::i64::<=` | quote, type | 2 | `wat-scripts/perf/grid/where-numeric.wat:140:38` |

Rust, 3 spellings, 11 hits:

| spelling | positions | hits | one site |
|---|---|---:|---|
| `:wat::core::<` | call-head | 2 | `src/edn/render.rs:5699:2` |
| `:wat::i64::<` | call-head | 4 | `src/runtime.rs:15766:2` |
| `:wat::rete::i64::<` | call-head, type | 5 | `src/rete/validate/mod.rs:1748:49` |

### MIXED rulings

Corpus only. 8 spellings, 548 hits. `from_keyword=true`, `enter=true`.

| spelling | positions | hits | one site |
|---|---|---:|---|
| `:probe::inc` | declaration-name, map-key, reference, set, type | 16 | `tests/services/probe_mapv_side_effect_once.wat:46:19` |
| `:u::E.B` | map-value, match-variant | 2 | `tests/diagnostics/refusals_teach_the_dot_separator_s6_keyword_subpattern.wat:5:27` |
| `:wat::core::Option.None` | call-head, map-value, match-variant, quote, reference, rete, type | 508 | `wat-scripts/fmt/run-tables.wat:19:6` |
| `:wat::runtime::Category.Transform` | map-value, reference | 4 | `wat-scripts/scratch-pad/probe-255-defn-metadata-map.wat:31:14` |
| `:wat::runtime::Determinism.Deterministic` | map-value, reference | 4 | same file `:28:17` |
| `:wat::runtime::ExpandTime.Legal` | map-value, reference | 4 | same file `:30:17` |
| `:wat::runtime::Purity.Pure` | map-value, reference | 4 | same file `:27:12` |
| `:wat::runtime::Totality.Total` | map-value, match-variant, reference | 6 | same file `:29:14` |

`:probe::inc` is declared and also stored as a map key and a set element (`tests/types/probe_arc255_74_key_must_be_data__*`). The five `:wat::runtime::…` rows are metadata enum variants stored as map values. `:wat::core::Option.None` is a name that is also a map value.

### Top files

Full top-8 is the TSV `top-file` rows. Corpus, heaviest eight per position:

| position | file | hits |
|---|---|---:|
| call-head | `tests/cli/mode_parity__deep_freeze_recursion.wat` | 3005 |
| call-head | `wat-scripts/fixes/positional-ctor-to-map.wat` | 733 |
| call-head | `wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat` | 712 |
| call-head | `wat-tests/gen.wat` | 665 |
| call-head | `wat-scripts/scratch-pad/255-stone-h-1b-atom-wrong-arity.wat` | 542 |
| call-head | `wat-scripts/scratch-pad/255-stone-h-1b-atom-success-calls.wat` | 488 |
| call-head | `wat-scripts/fixes/one-param-spec.wat` | 421 |
| call-head | `wat-scripts/scratch-pad/census-one-param-spec.wat` | 400 |
| declaration-name | `tests/cli/mode_parity__deep_freeze_recursion.wat` | 1003 |
| declaration-name | `wat-tests/gen.wat` | 81 |
| declaration-name | `tests/types/probe_arc227_stone2_defrecord.wat` | 68 |
| declaration-name | `wat-scripts/fixes/positional-ctor-to-map.wat` | 56 |
| declaration-name | `tests/rete/probe_arc251_8d_make_rule_quote_boundary.wat` | 48 |
| declaration-name | `wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat` | 48 |
| declaration-name | `wat-tests/rete/differential-fuzz-scalars.wat` | 48 |
| declaration-name | `wat-tests/core/core-arithmetic.wat` | 45 |
| type | `wat-scripts/fmt/rules/kwargs.wat` | 1072 |
| type | `tests/collection/bundle_capacity_over_panic.wat` | 502 |
| type | `tests/collection/bundle_capacity_try_propagate.wat` | 403 |
| type | `tests/collection/bundle_capacity_accessors.wat` | 401 |
| type | `tests/collection/bundle_capacity_over_error.wat` | 319 |
| type | `wat-scripts/fmt/rules/siblings.wat` | 160 |
| type | `tests/services/probe_arc278_sift_arena.wat` | 154 |
| type | `wat-scripts/perf/grid/where-record.wat` | 137 |
| match-variant | `wat-scripts/perf/grid/where-exists.wat` | 122 |
| match-variant | `wat-scripts/scratch-pad/255-stone-h-1b-atom-wrong-arity.wat` | 120 |
| match-variant | `wat-scripts/scratch-pad/255-stone-h-1b-atom-success-calls.wat` | 108 |
| match-variant | `wat-scripts/perf/grid/where-or-conditions.wat` | 98 |
| match-variant | `tests/rete/probe_arc300_2_fix_defrule.wat` | 91 |
| match-variant | `tests/rete/probe_arc251_8d_make_rule_quote_boundary.wat` | 80 |
| match-variant | `wat-scripts/scratch-pad/probe-optA-retag.wat` | 74 |
| match-variant | `wat-scripts/scratch-pad/probe-selectables-homogeneity.wat` | 74 |
| reference | `wat-scripts/scratch-pad/255-stone-h-1b-atom-metadata-arity.wat` | 60 |
| reference | `wat-scripts/scratch-pad/probe-reland8-type-of-campaign-enums.wat` | 58 |
| reference | `wat-tests/gen.wat` | 40 |
| reference | `tests/services/probe_arc278_sift_rules_arena.wat` | 39 |
| reference | `wat-scripts/scratch-pad/255-stone-h-1a-holon-metadata-arity.wat` | 35 |
| reference | `tests/rete/probe_arc278_vsa_where_native_differential.wat` | 22 |
| reference | `wat-tests/holon/eval-coincident.wat` | 20 |
| reference | `tests/wat_lang/wat_arc144_special_forms.wat` | 19 |
| quote | `wat-scripts/scratch-pad/255-stone-h-1b-atom-success-calls.wat` | 168 |
| quote | `wat-scripts/scratch-pad/255-stone-o-iv-c-1-holon-sweep-apply.wat` | 96 |
| quote | `wat-scripts/scratch-pad/255-stone-o-iv-c-2-atom-sweep-apply.wat` | 95 |
| quote | `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat` | 77 |
| quote | `wat-tests/rete/differential-fuzz-scalars.wat` | 61 |
| quote | `wat-scripts/scratch-pad/255-stone-h-1b-atom-wrong-arity.wat` | 60 |
| quote | `wat-scripts/scratch-pad/probe-arc278-rules-cross-the-wire.wat` | 53 |
| quote | `wat-scripts/scratch-pad/probe-arc278-rules-ship-as-declared-payload.wat` | 53 |
| rete | `wat-tests/rete/differential-fuzz-scalars.wat` | 7 |
| rete | `tests/rete/probe_arc278_enum_variant_typo_keyword.wat` | 6 |
| rete | `tests/rete/probe_arc278_match_arm_body_ok.wat` | 3 |
| rete | `tests/rete/probe_arc278_match_arm_then_core_bare.wat` | 3 |
| rete | `tests/rete/probe_arc278_match_arm_then_rete_bare.wat` | 3 |
| rete | `tests/rete/probe_arc278_match_arm_then_wrapped.wat` | 3 |
| rete | `tests/rete/probe_arc278_enum_variant_typo.wat` | 2 |
| rete | `tests/rete/probe_arc278_enum_variant_typo_bad.wat` | 2 |
| map-key | `tests/types/probe_arc255_74_key_must_be_data__hashmap_call_arg_fn_key.wat` | 1 |
| map-key | `tests/types/probe_arc255_74_key_must_be_data__map_literal_fn_key.wat` | 1 |
| map-value | `wat-scripts/scratch-pad/255-probe-a-declaration-cannot-be-stored-unvalidated.wat` | 5 |
| map-value | `wat-scripts/scratch-pad/probe-255-defn-metadata-map.wat` | 5 |
| map-value | `tests/diagnostics/refusals_teach_the_dot_separator_s6_keyword_subpattern.wat` | 1 |
| map-value | `tests/function/recursive_patterns_t3.wat` | 1 |
| set | `tests/types/probe_arc255_74_key_must_be_data__set_literal_fn.wat` | 1 |

Heaviest file elsewhere: other call-head `docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_left_idx_latch.wat` 58; rust call-head `src/runtime.rs` 797, then `src/rete/reachability.rs` 416; stdlib quote `wat/service.wat` 675, stdlib call-head `wat/service.wat` 129.

## Runtime counter

Removed before the commit. There is no `[features]` table on this main. Origin numbers below are from that removed counter. They are not re-runnable from the committed example.

The counter watched `Value::wat__core__keyword` text containing `::`.

- `made-literal`: the keyword eval fallthrough, not inside a macro expansion.
- `made-macro`: the same fallthrough while `expand_macro_call` was inside `expand_template`. A macro that emits a keyword form counts as `macro` when the body is evaluated during expansion, and as `literal` when the expanded form is evaluated. Both happened for `:proof::from-macro`.
- `made-from-string`: `keyword_from_string_value` (`src/runtime.rs:5202`, builds `:{s}` at `:5207`) and `eval_keyword_from_string` (`:5222`, builds `:{s}` at `:5262`). Other `Value::wat__core__keyword` constructions were not in this column.
- `made-reflection`: the syn-listed sites in the intrinsic table below. `metadata-of` was not hooked.
- `made-intrinsic`: `:wat::holon::from-holon` only (`src/intrinsic/holon/atom.rs:244` and `:251`). It stayed 0.
- `use-compared`: `values_equal` keyword arm (`src/runtime.rs:5850`), one count per `::` operand.
- `use-rust-eq`: `PartialEq` (`src/value/value.rs:609`), one per `::` operand. It was 0 in every measured run.
- `use-hashed`: `Hash` (`src/value/value.rs:781`).
- `use-printed-edn`: `value_to_edn_with` (`src/edn/render.rs:4610`).
- `use-printed-render`: `render_value` (`src/value/observe.rs:202`).
- `use-lookup-text`: every `SymbolTable::get` (`src/value/symbol_table.rs:274`) whose path contains `::`. This is name lookup of text, including call dispatch, not only keyword values.
- `use-lookup-value`: a keyword value passed to `sym.get` in `eval_kernel_fn_forms` (`src/closure_extract.rs:552`).
- `eval-seen` / `disp-*`: every `::` keyword that enters the value-position eval arm, and which branch it took. Call heads do not enter that arm.

Uses fire for a `::` keyword value that reaches eq, hash, print, or lookup, whether or not its construction was in the made-table.

### Planted origin proof

One process, stdlib startup plus this body. The program returns a `TypeMismatch` from `fn-forms` because `:proof::from-string` is not a registered function. That error is after the lookup note, and its snapshot is what incremented `printed-render`. `proof ok`, `RC=0`.

The body compared `:proof::literal-data` with itself, compared two `keyword/from-string` of `"proof::from-string"`, stored `{:proof::map-key :proof::literal-data}`, called `compose-variant` of `:wat::core::Option` and `:Some`, expanded a macro whose body is the keyword `:proof::from-macro`, evaluated `:wat::core::i64/+` in value position, and passed the from-string keyword to `fn-forms`. Rust also built `Value::wat__core__keyword(":proof::printed")` and called `value_to_edn_with`.

| counter | proof | baseline (stdlib freeze, no user body) | delta |
|---|---:|---:|---:|
| made-literal | 7 | 1 | +6 |
| made-macro | 13 | 12 | +1 |
| made-from-string | 791 | 773 | +18 |
| made-reflection | 293 | 292 | +1 |
| made-intrinsic | 0 | 0 | 0 |
| use-compared | 4 | 0 | +4 |
| use-rust-eq | 0 | 0 | 0 |
| use-hashed | 4 | 0 | +4 |
| use-printed-edn | 1 | 0 | +1 |
| use-printed-render | 1 | 0 | +1 |
| use-lookup-value | 1 | 0 | +1 |
| use-lookup-text | 39450 | 39364 | +86 |
| eval-seen | 25 | 14 | +11 |
| disp-unit-variant | 4 | 0 | +4 |
| disp-def-value | 1 | 1 | 0 |
| disp-function | 0 | 0 | 0 |
| disp-keyword-value | 20 | 13 | +7 |

The baseline main was rejected as UselessMain (`:user::main` body the bare `nil`). The freeze had already loaded stdlib. The counts are that load. `compared` +4 is two `=` calls times two `::` operands. `hashed` +4 is the planted map; startup hashed 0. Why 4 rather than 2 was not determined. `from-string` +18 against 3 recorded `:proof::from-string` lines; the other 15 were not identified. `disp-unit-variant` +4 during the body was not identified. `disp-function` stayed 0, and `:wat::core::i64/+` was recorded as `literal`, so that value-position reference did not resolve to a function on this path.

Spellings recorded for makes only (the proof log is startup plus the body):

- literal, 5 unique, 7 lines: `:proof::literal-data` 3, `:proof::from-macro` 1, `:proof::map-key` 1, `:wat::core::i64/+` 1, `:wat::telemetry::Log` 1. The map key is `literal` because eval of the key node hits the fallthrough. `:wat::telemetry::Log` is the one startup literal.
- macro, 5 unique, 13 lines: `:wat::spawn::Locus/launch` 9, `:proof::from-macro` 1, `:wat::kernel::stdout-svc/start$impl` 1, `stdin-svc/start$impl` 1, `stderr-svc/start$impl` 1. `:proof::from-macro` is both macro and literal.
- from-string, 393 unique, 791 lines. Top: `:wat::query::Store::Reply` 18, `:wat::telemetry::Journal::Reply` 13, `:wat::cache::Cache::Reply` 10, `:wat::query::Store::Op` 10, `:wat::telemetry::Span::Reply` 9, then service `::Status` / `::Admin` / `::Op` / `::Reply` paths at 7. These are name-shaped variant paths built as keyword values during startup. The planted spelling `:proof::from-string` occurs 3 times.
- reflection, 208 unique, 293 lines. Twelve spellings at 4, among them `:wat::query::Store::Reply.Put`, `:wat::query::Store::Reply.Scan`, `:wat::cache::Cache::Reply.Get`, `:wat::cache::Cache::Reply.Put`, and the matching `RequestTooLarge` responses. The planted `:wat::core::Option.Some` occurs 1 time, which is the reflection delta.

Classified right: literal `:proof::literal-data`, from-string `:proof::from-string`, reflection `:wat::core::Option.Some`, macro `:proof::from-macro`.

### One stdlib startup

`made-from-string` 773, `made-reflection` 292, `made-macro` 12, `made-literal` 1 (`:wat::telemetry::Log`), `made-intrinsic` 0. `use-compared` 0, `use-hashed` 0, `use-printed-edn` 0, `use-lookup-value` 0, `use-lookup-text` 39364. Value position: `eval-seen` 14 = `disp-keyword-value` 13 + `disp-def-value` 1. Zero function resolutions at value position during that startup.

### `tests/function` sample

88 files, `*.wat` excluding `*.wat.bad`, each `startup_from_source` in one process. Bodies are not run. `SAMPLE_RC=0`. 24 files fail startup and still mostly freeze stdlib: `defn_bad_type.wat`, `defn_redef.wat`, `fn_rename_bare_fn_type.wat`, `fn_rename_legacy_lambda.wat`, `fn_rename_mixed_legacy.wat`, `fn_rename_multi_lambda.wat`, `fn_signature_body_mismatch.wat`, `fn_signature_malformed_args.wat`, `probe_check_scoped_param_resolution_handwritten.wat`, `probe_check_scoped_param_resolution_macro.wat`, `recursive_patterns_nonexhaustive.wat`, `stone18a_e01.wat` through `stone18a_e06.wat`, `variadic_define_amp_no_binder.wat`, `variadic_define_arity_err.wat`, `variadic_define_double_amp.wat`, `variadic_define_fixed_after_rest.wat`, `variadic_define_non_vector_rest.wat`, `variadic_define_strict_extra_args.wat`, `variadic_define_type_err.wat`.

| counter | 88 startups |
|---|---:|
| made-literal | 80 |
| made-macro | 1056 |
| made-from-string | 68024 |
| made-reflection | 25696 |
| made-intrinsic | 0 |
| use-compared | 0 |
| use-rust-eq | 0 |
| use-hashed | 0 |
| use-printed-edn | 0 |
| use-printed-render | 0 |
| use-lookup-value | 0 |
| use-lookup-text | 3219657 |
| eval-seen | 1216 |
| disp-def-value | 80 |
| disp-function | 0 |
| disp-keyword-value | 1136 |

`from-string` 88 × 773 = 68024, `reflection` 88 × 292 = 25696, `macro` 88 × 12 = 1056. Those three origins added nothing beyond one stdlib freeze per file. `literal` is 80, not 88. `disp-keyword-value` is 1136, not 88 × 13 = 1144. `lookup-text` is 3219657, not 88 × 39364 = 3464032. `disp-nil`, `disp-retired`, `disp-option`, and `disp-unit-variant` are 0. Spellings were off.

### `wat-tests`

90 files, one process each. `run_tests_from_dir("wat-tests")` in one process aborted: `wat-tests/counter-actor-proof-process.wat` deftest `counter-actor::process-proof` FAILED, then `thread 'wat-thread-peer::<anon>' has overflowed its stack`, `SUITE_RC=134`, no suite TSV. Per-file, 88 files exited 0 at the default stack. `wat-tests/deporder.wat` and `wat-tests/lint.wat` aborted at RC 134 on that stack and were retried with `RUST_MIN_STACK=33554432`. Both retries RC 0: deporder 4 passed, 0 failed; lint 10 passed, 0 failed. The two abort snapshots were excluded. The 88 default-stack files reported 391 passed and 40 failed; plus the two retries, 405 passed and 40 failed.

The 40 failures are deftest failures inside this in-process runner. They were not re-run under `cargo test --test kernel`. This score does not say `wat-tests` is green and does not say the 40 are pre-existing. Files, failure count: `kernel/services/ambient-stdio.wat` 5, `test.wat` 4, `core/core-nth.wat` 3, `core/core-nth-differential.wat` 3, `core/core-seq-walkers.wat` 2, and one each of `timer-env-grab-parity.wat`, `spawn/recv-budget-override.wat`, `spawn/multiline-roundtrip.wat`, `service-stop-resp.wat`, `service-signal-observer.wat`, `service-request-malformed.wat`, `service-parametric-messages.wat`, `service-parametric-bare-messages.wat`, `service-multiparam-init.wat`, `service-locus-parity.wat`, `service-init-parity.wat`, `service-hibernate-resume.wat`, `service-cache-lru.wat`, `service-cache-hologram.wat`, `service-admin-facet.wat`, `process/signal-user2-and-hangup-independent.wat`, `process/signal-user1-delivers-child-observes.wat`, `process/signal-terminate-kills-the-child-and-the-read-sees-it.wat`, `process/signal-reset-sigusr1-is-a-transition.wat`, `counter-actor-proof-process.wat`, `core/unknown-call-head-panics.wat`, `core/core-equality.wat`, `core/core-arithmetic.wat`.

Merged counters, 90 files:

| counter | total |
|---|---:|
| made-literal | 87827 |
| made-macro | 1125 |
| made-from-string | 103340 |
| made-reflection | 26594 |
| made-intrinsic | 0 |
| use-compared | 0 |
| use-rust-eq | 0 |
| use-hashed | 70 |
| use-printed-edn | 13 |
| use-printed-render | 0 |
| use-lookup-value | 0 |
| use-lookup-text | 24286456 |
| eval-seen | 448520 |
| disp-function | 238799 |
| disp-option | 118781 |
| disp-keyword-value | 88952 |
| disp-unit-variant | 1685 |
| disp-def-value | 303 |
| disp-nil | 0 |
| disp-retired | 0 |

Disposition sum 238799 + 118781 + 88952 + 1685 + 303 = 448520 = `eval-seen`. `disp-keyword-value` 88952 = `made-literal` 87827 + `made-macro` 1125. Every keyword-value disposition in this run is one of those two origins. from-string and reflection are additional constructions that do not pass the eval keyword arm. Spellings were off. The 87827 literals were not recorded as text.

### Origin verdict

| origin | what was measured | verdict |
|---|---|---|
| from-string | 773 name-shaped service paths per startup (393 unique in the proof log), plus the planted unresolved `:proof::from-string` | MIXED. A reader that rewrites source keywords does not see these values. |
| reflection | variant paths such as `:wat::query::Store::Reply.Put`; planted `:wat::core::Option.Some` | NAME-shaped keyword values. Same door gap. The 208 spellings were not each passed through `enter`. |
| macro | `:wat::spawn::Locus/launch` and the three `start$impl` names, plus planted `:proof::from-macro` which is also later `literal` | MIXED. Declared names and an unresolved `::` token. |
| literal | startup `:wat::telemetry::Log`; planted `:proof::literal-data` compared and hashed; 87827 suite hits with spellings off | The instrument does not decide DATA versus NAME. An unresolved `::` keyword can be stored and compared. Ruling needed. |
| intrinsic (`from-holon`) | 0 on the proof, the baseline, the function sample, and `wat-tests` | No measured `::` value. The verb still builds a keyword (`atom.rs:244` copies symbol text, `:251` builds `:{s}`). |

`use-lookup-text` is not a keyword-value use. One startup does it 39364 times.

### Constructions the made-table does not cover

The syn walk of `src/intrinsic` plus `src/reflect` produced the 15 TSV rows. `keyword/from-string` lives in `src/runtime.rs` and was hooked by name, not by that walk. These constructions also build a keyword value and were not hooked. A `::` value from one of them is absent from the made-columns. A use still counts if the value reaches eq, hash, print, or the hooked lookup.

| site | what it returns |
|---|---|
| `src/runtime.rs:5207` | `keyword_from_string_value`: `:{s}`, or `None` on a leading `:` or an angle-type head |
| `src/runtime.rs:5262` | `eval_keyword_from_string`, producer `:wat::keyword::from-string`. The shim is `src/intrinsic/keyword.rs:104`. Doc examples: `"foo"` → `:foo`, `"wat::type::i64"` → `:wat::core::i64` |
| `src/runtime.rs:7690` | `metadata-of` type token: the parsed keyword text |
| `src/runtime.rs:7712` | `metadata-of` arg name `:{name}` |
| `src/runtime.rs:7737` | `metadata-of` `:see` entries |
| `src/runtime.rs:7752` | `metadata-of` `:alias` |
| `src/runtime.rs:7951` | `metadata-of` map keys from the key string |
| `src/runtime.rs:7956` | `metadata-of` `:name`, the intrinsic FQDN |
| `src/record/update.rs:102` | `:{field-name}` |
| `src/rete/export.rs:193` | keyword of `name` |
| `src/rete/expr_ir/mod.rs:499` | keyword of `k` |
| `src/holon/ast.rs:59` and `:63` | symbol text, or `:{s}` |
| `src/kernel/peer.rs:420` | `:wat::kernel::__peer_crashed__` (`PEER_CRASHED_SENTINEL`, `peer.rs:256`) |
| `src/edn/render.rs:503` | `:{variant}` while rendering |

### Intrinsics the walk did list

The TSV prints the construction as `new(…)`. That is the visitor's view of `Arc::new`, not the returned text. What the source returns:

| site | returns |
|---|---|
| `src/intrinsic/holon/atom.rs:244` | symbol text as a keyword; symbol `"nil"` returns `Nil`. Producer `:wat::holon::from-holon` |
| `src/intrinsic/holon/atom.rs:251` | `:{s}` from a holon keyword |
| `src/intrinsic/reflect.rs:86` | intrinsic FQDN keyword, `:wat::intrinsic::examples` |
| `src/intrinsic/reflect.rs:205` | intrinsic FQDN keyword, rows |
| `src/reflect/verbs.rs:647` and `:652` | the caller's literal keyword text, `:wat::runtime::rename-callable-name` (`:user::my-double` is the comment's example) |
| `src/reflect/verbs.rs:875` | `:{arg-name}` for a bare binder |
| `src/reflect/verbs.rs:1052` | `:{field-name}`, `:wat::runtime::field-names-of` |
| `src/reflect/verbs.rs:1285` | `kw()`, `:{name}` |
| `src/reflect/verbs.rs:1527`, `:1542`, `:1572` | the type keyword string inside a TypeInfo record |
| `src/reflect/verbs.rs:1536` | each marker child as a keyword |
| `src/reflect/verbs.rs:1874` | parent enum keyword, or `None`, `:wat::runtime::variant-parent-of` |
| `src/reflect/verbs.rs:1962` | `compose_variant` text, for example a `:wat::cache::Lru.Hit` shape, `:wat::runtime::compose-variant` |

`type-of`'s doc says an example result can be the slash spelling `:wat.core/Option`. Not every returned type keyword contains `::`.

## Gates

| gate | result |
|---|---|
| no `src/` edit | `git diff --stat -- Cargo.toml src crates wat tests` empty before this commit |
| planted positions | `proof ok, 39 :: keyword(s)`, `RC=0` |
| planted origins | `proof ok`, `RC=0`, on the removed counter |
| clippy | `cargo clippy --release --all-targets -- -D warnings` finished in 14.22s, `RC=0` |
| floor | not run. Nothing under test changed |
| STOP-1 | did not fire. No `undecided:` row. `parse-fail-count` 0 |

## What the reader stone is walking into

Most source `::` keywords are names: 115132 of 116052 corpus hits, and all 1359 stdlib hits. The stdlib residue is quote, quasiquote, and unquote.

The split is not safe as "no `::` keyword is data."

1. Comparison operators `:wat::core::<`, `:wat::core::<=`, and the `i64` / `f64` / `rete` twins are call heads of real functions. `from_keyword` accepts them. `enter` refuses them because of `<`. A reader that follows `enter` leaves them as text. A reader that follows `from_keyword` turns them into pairs. That choice is a ruling.
2. `:fn(wat::core::i64)->wat::core::i64` is the same split on `(`.
3. Eight spellings are names that are also stored: `:probe::inc`, `:u::E.B`, `:wat::core::Option.None`, and the five `:wat::runtime::` metadata variants. `map-key`, `map-value`, and `set` are entirely those rows.
4. One stdlib startup already builds 773 `::` keyword values via `keyword/from-string` and 292 via reflection. They are service and variant paths. The reader never sees them. `wat-tests` then resolves 238799 `::` keywords to functions at value position and leaves 88952 as keyword values, while from-string adds another 103340 constructions that do not pass that arm.
5. A `::` keyword can be an opaque token. The planted `:proof::literal-data` was compared and hashed. `:wat::kernel::__peer_crashed__` is built in Rust as a keyword value. The literal origin does not decide which unresolved `::` keywords are data.

`flat` stays. The text bridge stays. Stone 4 and stone 5 of the old Name series were not started. `wat/kernel/services/stdio.wat` was not edited. `name-census.tsv` was not rewritten.
