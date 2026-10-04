# SCORE — STONE 255.90: measure — the name is the pair

Measured 2026-10-04 on `main` after the brief commit `a54a5e197` (drawn against `ddb1261b5`). Executor grok, solo. A measuring stone: no cure, no edit under `wat/` or `tests/`, no corpus edit, `flat` still stored, `fold_member_twin` not reverted. 5c-iv and 5d were not started. No floor. The counts below are the census bin's stdout and the rows in `name-census.tsv` from that same run (`CENSUS_RC=0`, `PARSE_FAIL 0`). The brief's text-grep figures are replaced by this run.

## The pair, measured

`print_pair_probe` builds `Identifier::bare` in process and prints both equalities. Stdout:

```
PAIR samples 6
PAIR spell ":a::b/c" ns ":a::b" name "c" flat ":a::b/c" ref true
PAIR spell "a.b/c" ns "a.b" name "c" flat "a.b/c" ref true
PAIR spell "wat.core.Option/expect" ns "wat.core.Option" name "expect" flat "wat.core.Option/expect" ref true
PAIR spell ":wat::core::Option/expect" ns ":wat::core::Option" name "expect" flat ":wat::core::Option/expect" ref true
PAIR spell "wat.core//" ns "wat.core" name "/" flat "wat.core//" ref true
PAIR spell "wat.core/x" ns "wat.core" name "x" flat "wat.core/x" ref true
PAIR into_bound flat_eq true pair_eq false bound_ns "$bound" bound_name "wat.core/x" flat "wat.core/x"
```

`bare` splits on the first `/` and leaves `::` in the namespace. `:a::b/c` and `a.b/c` are different pairs under flat-equality and under `(namespace, name)` equality. The end-state example in the ruling (`:a::b/c` and `a.b/c` as one pair) needs the `::` annihilation in front of it. This stone does not do that annihilation.

`into_bound` of `wat.core/x` keeps `flat` and rewrites the stored pair to namespace `$bound` and name `wat.core/x`. Flat-equality holds. Pair-equality does not. That is the measured disagreement.

`PartialEq` (`crates/wat-reader/src/identifier.rs:128`) and `Hash` (`:135`) use `flat` + `scopes`. `as_str` (`:238`) returns `flat`.

## Where equality is relied on

The TSV has two `==` rows on `WatAST::Symbol`'s `Identifier`, and one map keyed by `Identifier`.

| tree | file:line | what |
|---|---|---|
| src | `src/macros/registry.rs:198` | `sa == sb` in `ast_same_identity`, both `WatAST::Symbol` |
| src | `src/runtime.rs:14217` | `ident == target` in `substitute` |
| crates | `crates/wat-reader/src/identifier.rs:634` | `HashSet<Identifier>` inside `identifiers_are_hashable`; verdict NAME, evidence `insert bare` |

`src/runtime.rs:14210` says Identifier equality covers the name and the scope set. The implementation compares `flat` and the scopes. `substitute` replaces a `Symbol` equal to `target`. `into_bound` keeps `flat`, so a binder and the body reference compare equal today and the substitution hits. Under `(namespace, name, scopes)` they compare unequal and the substitution misses. That site depends on the bound form and the unbound form being equal.

No `==` site depends on them being unequal. Today the bound form and the unbound form compare equal, so no branch can observe an inequality between them.

`registry.rs:198` compares two symbol nodes. They diverge under pair-equality only in the `into_bound` shape (same `flat`, different stored pair). The hash test inserts `bare("x")` and a scoped copy and expects length 2. The scopes already differ, so a pair-equality `Hash` still separates them.

`src` and `tests` have zero `HashMap`/`HashSet` keyed by `Identifier`. The false rows from a `Symbol` pattern that is not `WatAST::Symbol` are absent: `crates/wat-edn/src/value.rs:92` and `crates/wat-reader/src/parser.rs:394` are not in the EQ section. The taint requires a path segment `WatAST` and a last segment `Symbol`.

## The instrument

`src/bin/name_census.rs` parses every `.rs` file under `src/`, `crates/`, and `tests/` with `syn` and writes `docs/arc/2026/06/255-builtin-registry/name-census.tsv`. It reports. It does not gate. It skips its own source: the TSV has 0 rows whose path contains `name_census` (23068 lines, header included, 22114 `LIT` rows).

The package lists `[[bin]]` entries, which turns off cargo's bin autodiscovery. `Cargo.toml` gains a `name_census` bin or the file does not build. `syn` and `proc-macro2` were dev-dependencies. A bin does not link dev-dependencies (`cannot find crate syn` was the compile error). They now sit in `[dependencies]`. The `wat` and `cargo-wat` bins do not name `syn`. `proc-macro2` already had feature `span-locations`, so `span.start().line` is populated. Those manifest stanzas are the instrument. The brief's "only the bin, the TSV, and the SCORE" meets this: `git diff a54a5e197` is `Cargo.toml`, the bin, the TSV, and this score. `git diff ddb1261b5` also contains the brief file, which is commit `a54a5e197`, the orchestrator's, not an edit by this stone.

Classifier priority, applied in this order, so a literal is not left unclassified:

1. `WAT` — embedded wat: the text contains `(wat.` or `wat.core/` or `(defn` or `(defrecord` or `(defenum` or a newline plus `(def`, and it contains a newline or `(wat.`.
2. `DISPATCH` — the literal is a match or `if-let` pattern.
3. `CMP` — `==` / `!=`, or `starts_with` / `ends_with` / `strip_prefix` / `strip_suffix` / `contains`, or the first two arguments of `assert_eq` / `assert_ne`. An `if` or `while` marks the whole condition `CMP`, so a `format!` inside a condition is `CMP`.
4. `BUILD` — `format!` / `format_args!` / `concat!` and the literal assembles a name (`{}` together with `::`, or a name-shaped fragment).
5. `REG` — a `wat_*` attribute argument, or a `const` / `static` / insert key that is name-shaped.
6. `MSG` — `panic` / `expect` / `todo` / `unimplemented` / `compile_error` / `println` / `eprintln` / `write` / `writeln`, or a format template that does not assemble a name, or a match-arm body whose text is not name-shaped.
7. `OTHER` — a doc attribute (`doc comment`), a match arm that yields a name-shaped string (`match arm yields a name`), or the residual (`no registration, dispatch, compare, build, message, or wat form`).

A doc attribute is `OTHER`, not `REG`.

Map keys: `String`, `str`, `&str`, `Arc<str>`, `Arc<String>` on `HashMap` / `BTreeMap` / `HashSet` / `BTreeSet`. An `Identifier` key is an `IDMAP` row, not a `MAP` row. Evidence is the `insert` / `entry` in the same file whose receiver names the field. Name evidence is a door call, `as_str` / `leaf` / `path` / `to_string` / `bare` / `into_bound` / `add_scope`, or a `::` or leading-`:` string. Other-text evidence is a path-like string, or a variable named `path`, `file`, `dir`, `label`, `msg`, `message`, `text`, or `filename`. Both bits, or neither, is `STOP`. A nested `HashMap<String, HashMap<String, _>>` emits two rows at the same line (`src/value/symbol_table.rs:16` twice, `src/types.rs:573` twice). The denominators below count rows, which is what the bin printed.

## Proof

Planted in `/tmp/name-census-proof.rs` (not committed; `tests/` is untouched). One literal of each class, plus a second `REG` (attribute) and a second `CMP` (`starts_with`). Exact text match, except the wat marker which is a substring of a raw string. Stdout:

```
PROOF REG :wat::census::Reg line 1
PROOF REG :wat::census::RegAttr line 3
PROOF DISPATCH :wat::census::Dispatch line 8
PROOF CMP :wat::census::CmpEq line 14
PROOF CMP :wat::census::CmpPre line 18
PROOF BUILD :wat::census::{} line 22
PROOF MSG missing :wat::census::Msg here line 26
PROOF WAT :wat::census::Wat line 29
PROOF OTHER std::census::Other line 35
PROOF ok 9
```

Known callers on this TSV:

| class | file:line | row |
|---|---|---|
| PRINT | `src/edn/render.rs:908` | `WatAST::Symbol` arm, `out.push_str(ident.as_str())` |
| IDENTITY | `src/edn/render.rs:5350` | `assert_eq!(id.as_str(), "wat.core/Error")` |
| IDENTITY | `src/edn/render.rs:1326`, `:5323`, `:5362`, `:5374` | `as_str` consumed as a key or an equality |
| PRINT | `crates/wat-doc/src/print.rs:291` and `:364` | `out.push_str(id.as_str())` |
| DOOR IDENTITY | `src/types.rs:223` | `canonical_identity(p) == INFER_TYPE_PATH` inside `denoted_type_path` |

`Finished dev profile [unoptimized + debuginfo] target(s) in 0.29s`. `CENSUS_RC=0`.

## 1. `::` literals

A row is one string literal whose text contains `::`. Denominator is `LITERALS` for that tree.

| class | src / 15827 | crates / 710 | tests / 5577 | all / 22114 |
|---|---:|---:|---:|---:|
| REG | 1879 | 0 | 75 | 1954 |
| DISPATCH | 663 | 20 | 0 | 683 |
| CMP | 890 | 186 | 1210 | 2286 |
| BUILD | 310 | 26 | 62 | 398 |
| MSG | 520 | 32 | 865 | 1417 |
| WAT | 196 | 3 | 42 | 241 |
| OTHER | 11369 | 443 | 3323 | 15135 |

`OTHER` by the `key` column (the reason):

| why | src | crates | tests |
|---|---:|---:|---:|
| doc comment | 7377 | 291 | 703 |
| no registration, dispatch, compare, build, message, or wat form | 3158 | 130 | 2610 |
| match arm yields a name | 834 | 22 | 10 |

Top 20 files, `src`:

| n | file |
|---:|---|
| 2174 | `src/check.rs` |
| 1595 | `src/runtime.rs` |
| 621 | `src/types.rs` |
| 504 | `src/intrinsic/mod.rs` |
| 478 | `src/remedy/retirement.rs` |
| 468 | `src/intrinsic/holon/atom.rs` |
| 431 | `src/edn/render.rs` |
| 418 | `src/intrinsic/special/rete_alias.rs` |
| 334 | `src/rete/reachability.rs` |
| 273 | `src/rete/vocabulary.rs` |
| 256 | `src/intrinsic/time.rs` |
| 230 | `src/rete/purity.rs` |
| 223 | `src/reflect/verbs.rs` |
| 194 | `src/freeze.rs` |
| 174 | `src/intrinsic/string.rs` |
| 162 | `src/collection/transform.rs` |
| 162 | `src/intrinsic/f64.rs` |
| 155 | `src/collection/infer.rs` |
| 153 | `src/intrinsic/collection.rs` |
| 151 | `src/collection/eval.rs` |

Top 20 prefixes, `src` (the identifier segment before the first `::`, leading colons stripped; an empty segment is `empty`):

| n | prefix |
|---:|---|
| 12547 | wat |
| 323 | Value |
| 225 | empty |
| 208 | crate |
| 185 | my |
| 156 | WatAST |
| 124 | rust |
| 105 | user |
| 61 | TypeExpr |
| 57 | probe |
| 44 | i64 |
| 41 | TypeDef |
| 36 | TypeEnv |
| 35 | Self |
| 34 | RecvError |
| 34 | std |
| 32 | RuntimeErrorKind |
| 31 | Op |
| 29 | fan |
| 29 | libc |

Top 20 files, `crates`: `wat-doc/src/lib.rs` 106, `wat-reader/src/identifier.rs` 95, `wat-macros/src/discover.rs` 83, `wat-reader/src/lexer.rs` 63, `wat-reader/src/parser.rs` 53, `wat-macros/src/lib.rs` 33, `wat-edn/src/value.rs` 31, `wat-macros/src/edn_doc.rs` 31, `wat-reader/src/ast.rs` 27, `wat-to-edn-derive/src/lib.rs` 27, `wat-macros/src/wat_intrinsic.rs` 25, `wat-source-derive/src/lib.rs` 24, `wat-macros/src/codegen.rs` 23, `wat-doc/src/print.rs` 17, `wat-edn/src/lib.rs` 15, `wat-macros/src/wat_special_form.rs` 12, `wat-edn/src/vocab.rs` 8, `wat-edn/src/json.rs` 7, `wat-edn/tests/comprehensive.rs` 5, `wat-reader/tests/clj_oracle_source_parity.rs` 5.

Top 20 prefixes, `crates`: wat 350, empty 62, my 29, WatAST 24, rust 19, Value 13, probe 10, wat_doc 10, wat_edn 10, Symbol 9, DocError 8, a 8, crate 8, rs 7, Cow 6, Self 6, user 6, Tag 5, TypeExpr 5, serde_json 5.

Top 20 files, `tests`: `tests/function/wat_arc170_closure_extraction.rs` 101, `tests/lint/rete_names_in_wat_scripts_resolve.rs` 72, `tests/rete/probe_arc278_export.rs` 62, `tests/types/probe_arc237_stone1_typeunion_substrate.rs` 61, `tests/types/probe_arc227_stone2_defrecord.rs` 49, `tests/rete/probe_arc278_55_slice_one_vocabulary.rs` 47, `tests/services/probe_arc255_24_defservice_declares_what_it_emits.rs` 44, `tests/lint/keyword_heresy_ledger.rs` 39, `tests/rete/probe_arc278_P6_delta_asymmetric_join.rs` 38, `tests/program/wat_arc170_program_contracts.rs` 37, `tests/lint/nested_program_starts.rs` 34, `tests/types/probe_arc237_sA_hierarchy.rs` 34, `tests/types/probe_arc226_stone1_type_predicates.rs` 32, `tests/collection/probe_collection_transform_ops.rs` 31, `tests/cli/wat_cli.rs` 29, `tests/collection/probe_arc216_stone3_hashmap_roundtrip.rs` 27, `tests/wat_lang/wat_arc098_form_matches_runtime.rs` 27, `tests/collection/probe_arc216_stone5c_hashmap_native_storage.rs` 26, `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` 26, `tests/resolve/probe_arc251_type_namespace_fix.rs` 26.

Top 20 prefixes, `tests`: user 1485, wat 1479, t 526, my 523, probe 298, Value 157, empty 87, p 56, myapp 46, u 40, WatAST 29, test 27, i64 26, RuntimeErrorKind 25, RecvOutcome 24, HolonAST 23, diag 22, StartupError 20, geo 20, p255_12 17.

Every `LIT` row is in the TSV (`section`, `tree`, `class`, `file`, `line`, `key` = why, `detail` = the literal text).

## 2. Text-keyed maps

| | rows | NAME | OTHER | STOP |
|---|---:|---:|---:|---:|
| src | 376 | 59 | 7 | 310 |
| crates | 3 | 1 | 0 | 2 |
| tests | 48 | 5 | 0 | 43 |
| all | 427 | 65 | 7 | 355 |

`NAME` rows (65):

```
src	NAME	src/check/env.rs:158	HashMap String	defined_value_asts | insert var name
src	NAME	src/closure_extract.rs:682	BTreeMap String	captured_deps | insert var name
src	NAME	src/closure_extract.rs:684	BTreeMap String	captured_types | insert var name
src	NAME	src/closure_extract.rs:693	BTreeMap String	captured_macros | insert var name
src	NAME	src/closure_extract.rs:698	BTreeMap String	captured_defs | insert var k
src	NAME	src/closure_extract.rs:708	BTreeMap String	dep_edges | entry var consumer; entry var name
src	NAME	src/closure_extract.rs:708	BTreeSet String	dep_edges | entry var consumer; entry var name
src	NAME	src/closure_extract.rs:710	BTreeMap String	type_edges | entry var name
src	NAME	src/closure_extract.rs:710	BTreeSet String	type_edges | entry var name
src	NAME	src/closure_extract.rs:758	BTreeSet String	locals | insert var name
src	NAME	src/closure_extract.rs:1305	BTreeSet String	locals | insert var name
src	NAME	src/closure_extract.rs:1407	BTreeSet String	locals | insert var name
src	NAME	src/closure_extract.rs:2688	HashMap &str	by_name | insert var cb
src	NAME	src/closure_extract.rs:2698	HashMap &str	by_name | insert var cb
src	NAME	src/closure_extract.rs:2699	BTreeSet String	locals | insert var name
src	NAME	src/closure_extract.rs:2777	HashMap &str	by_name | insert var cb
src	NAME	src/closure_extract.rs:2847	HashMap &str	by_name | insert var cb
src	NAME	src/edn/render.rs:3157	HashMap String	by_key | insert var kw
src	NAME	src/edn/render.rs:3993	HashMap String	by_key | insert var kw
src	NAME	src/edn/render.rs:4067	HashMap String	by_key | insert var kw
src	NAME	src/edn/render.rs:4142	HashMap String	by_key | insert var kw
src	NAME	src/freeze/env.rs:617	HashMap String	meta | insert :: literal
src	NAME	src/intrinsic/mod.rs:544	HashSet String	membership_names | insert var name
src	NAME	src/rete/alpha_tree.rs:330	HashMap String	out | entry var idx; insert var var
src	NAME	src/rete/compiled_cond.rs:393	HashMap String	scope | insert var name; insert var var
src	NAME	src/rete/compiled_cond.rs:453	HashMap String	scope | insert var name; insert var var
src	NAME	src/rete/compiled_cond.rs:472	HashMap String	scope | insert var name; insert var var
src	NAME	src/rete/compiled_cond.rs:696	HashMap String	scope | insert var name; insert var var
src	NAME	src/rete/compiled_cond.rs:810	HashMap String	scope | insert var name; insert var var
src	NAME	src/rete/compiled_cond.rs:829	HashMap String	scope | insert var name; insert var var
src	NAME	src/rete/export.rs:1661	HashMap String	idx | insert var class
src	NAME	src/rete/expr_ir/mod.rs:310	HashMap String	slots | insert var name
src	NAME	src/rete/expr_ir/mod.rs:369	HashMap String	slots | insert var name
src	NAME	src/rete/kernel/fire/rules.rs:82	HashMap String	production_id_by_rule | insert var rname
src	NAME	src/rete/kernel/tests/alpha_discrimination.rs:75	HashMap String	field_names_cache | entry var fact_class
src	NAME	src/rete/kernel/tests/alpha_discrimination.rs:253	HashMap String	field_names_cache | entry var fact_class
src	NAME	src/rete/kernel/tests/alpha_discrimination.rs:403	HashMap String	field_names_cache | entry var fact_class
src	NAME	src/rete/kernel/tests/where_tree_branch_differential.rs:131	BTreeMap &str	counts | entry var x
src	NAME	src/rete/matcher.rs:415	HashSet String	out | insert var var
src	NAME	src/rete/purity.rs:910	HashSet String	seen | insert var fqdn; insert var name
src	NAME	src/rete/purity.rs:1072	HashSet String	seen | insert var fqdn; insert var name
src	NAME	src/rete/purity.rs:1158	HashSet String	seen | insert var fqdn; insert var name
src	NAME	src/rete/purity.rs:1471	HashSet String	seen | insert var fqdn; insert var name
src	NAME	src/rete/purity.rs:2040	HashSet String	seen | insert var fqdn; insert var name
src	NAME	src/rete/purity.rs:2080	HashSet String	gray | insert var head; insert var root_name
src	NAME	src/rete/purity.rs:2082	HashSet String	black | insert var head
src	NAME	src/rete/purity.rs:2096	HashSet String	gray | insert var head; insert var root_name
src	NAME	src/rete/purity.rs:2097	HashSet String	black | insert var head
src	NAME	src/rust_deps/mod.rs:154	HashMap String	symbols | insert var sym
src	NAME	src/rust_deps/mod.rs:155	HashSet String	types | insert var decl
src	NAME	src/rust_deps/mod.rs:225	HashMap String	symbols | insert var sym
src	NAME	src/rust_deps/mod.rs:226	HashSet String	types | insert var decl
src	NAME	src/special_forms.rs:84	HashMap String	m | insert var name
src	NAME	src/types.rs:573	HashMap String	map | entry var ns
src	NAME	src/types.rs:573	HashMap String	map | entry var ns
src	NAME	src/types.rs:1122	HashMap String	subtype_edges | entry var child
src	NAME	src/types.rs:1127	HashSet String	subtype_parents | insert var parent
src	NAME	src/types.rs:1149	HashMap String	parametric_extensions | entry var child
src	NAME	src/types.rs:2175	HashMap String	bindings | insert var bare; insert var rep
crates	NAME	crates/wat-macros/src/lib.rs:975	BTreeSet String	s | insert var site
tests	NAME	tests/lint/gen_doc_surface_matches.rs:61	BTreeSet String	out | insert var leaf; insert var name
tests	NAME	tests/lint/keyword_heresy_ledger.rs:246	BTreeSet String	stringy | insert var f
tests	NAME	tests/lint/keyword_heresy_ledger.rs:536	BTreeSet String	stringy | insert var f
tests	NAME	tests/lint/rete_citation_resolves.rs:435	BTreeSet String	stems | insert var s
tests	NAME	tests/lint/rete_citation_resolves.rs:437	BTreeSet String	basenames | insert var b
```

`OTHER` rows (7):

```
src	OTHER	src/edn/bridge.rs:249	HashMap String	index | insert var file
src	OTHER	src/load/loader.rs:1057	HashMap String	source_files | insert var path
src	OTHER	src/load/loader.rs:1058	HashMap String	payload_files | insert var path
src	OTHER	src/rust_deps/mod.rs:278	HashSet String	declared | insert var path
src	OTHER	src/value/symbol_table.rs:33	HashMap String	functions | entry var path; insert var path
src	OTHER	src/value/symbol_table.rs:42	HashMap String	unit_variants | insert var path
src	OTHER	src/value/symbol_table.rs:108	HashMap String	runtime_def_values | insert var path
```

`STOP` rows (355). These are STOP-1. The insert key is neither clearly a name nor clearly other text, or the file has no `insert`/`entry` that names the binding, or both kinds of evidence are present. They are not a stone until a ruling says which are names.

```
src	STOP	src/check/env.rs:78	HashMap String	schemes | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/check/env.rs:81	HashMap String	param_bounds | no insert or entry in this file names this binding
src	STOP	src/check/env.rs:95	HashMap String	unit_variant_types | no insert or entry in this file names this binding
src	STOP	src/check/env.rs:109	HashMap String	defined_values | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/check/env.rs:113	HashMap String	defined_value_spans | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/check/env.rs:127	HashMap String	binding_metadata | no insert or entry in this file names this binding
src	STOP	src/check/env.rs:127	HashMap String	binding_metadata | no insert or entry in this file names this binding
src	STOP	src/check/env.rs:143	HashMap String	defclause_registrations | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/check/env.rs:175	HashMap String	corpus_values | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/check/env.rs:179	HashMap String	bounds | no insert or entry in this file names this binding
src	STOP	src/check.rs:1466	HashMap String	meta | no insert or entry in this file names this binding
src	STOP	src/check.rs:2190	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:2209	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:2474	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:2525	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:2577	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:2656	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:2904	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:2915	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:5624	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:5663	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:5674	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:5846	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:6593	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:6694	HashSet String	covered_enum_variants | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/check.rs:7106	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var s
src	STOP	src/check.rs:7295	HashMap String	type_param_mapping | no insert or entry in this file names this binding
src	STOP	src/check.rs:7413	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var s
src	STOP	src/check.rs:7654	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var s
src	STOP	src/check.rs:8263	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:8500	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:8559	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:8719	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:8905	HashMap String	_locals | no insert or entry in this file names this binding
src	STOP	src/check.rs:8936	HashMap String	clause_locals | insert seen, key is neither a name nor other text: insert var arg_ident
src	STOP	src/check.rs:9179	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:9696	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:9813	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:9912	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:9988	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10064	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10160	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10215	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10290	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10398	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10603	HashMap String	_locals | no insert or entry in this file names this binding
src	STOP	src/check.rs:10648	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10843	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10918	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:10985	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11048	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11122	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11229	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11272	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11339	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11412	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11507	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11559	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11670	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11752	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11828	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11923	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:11998	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12090	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12152	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12199	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12271	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12360	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12401	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12445	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12578	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:12702	HashMap String	rhs_scope | no insert or entry in this file names this binding
src	STOP	src/check.rs:12707	HashMap String	new_bindings | insert seen, key is neither a name nor other text: insert var fname; insert var name; insert var var_name
src	STOP	src/check.rs:13198	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13312	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13349	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13382	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13419	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13452	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13488	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13520	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13554	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13652	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13757	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:13966	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14076	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14184	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:14252	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14432	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14520	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14623	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14732	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14837	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:14917	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:15243	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:15457	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:15541	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:15599	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:15654	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:15740	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:15879	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:15998	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16069	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16125	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16165	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16237	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16268	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16335	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16466	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16492	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16583	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16636	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16682	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16732	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16799	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:16870	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:17103	HashSet String	set1 | no insert or entry in this file names this binding
src	STOP	src/check.rs:17554	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var s
src	STOP	src/check.rs:18121	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:18167	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:18409	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:18421	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:18462	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:18538	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:18590	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/check.rs:25015	HashSet String	names | insert seen, key is neither a name nor other text: insert key; insert var n
src	STOP	src/check.rs:25840	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/check.rs:25890	HashMap String	locals | insert seen, key is neither a name nor other text: insert var env_key; insert var name; insert var rest_name
src	STOP	src/closure_extract.rs:221	BTreeSet String	body_locals | insert seen, key is neither a name nor other text: insert var rest
src	STOP	src/closure_extract.rs:678	HashSet String	captured_locals | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/closure_extract.rs:703	HashSet String	types_visited | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/closure_extract.rs:705	HashSet String	deps_visited | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/closure_extract.rs:1065	BTreeSet String	outer_locals | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:1131	BTreeSet String	outer_locals | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:1346	BTreeSet String	outer_locals | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:1704	BTreeSet String	dep_locals | insert seen, key is neither a name nor other text: insert var rest
src	STOP	src/closure_extract.rs:1825	BTreeSet String	out | insert seen, key is neither a name nor other text: insert var head_kw; insert var p
src	STOP	src/closure_extract.rs:1835	BTreeSet String	out | insert seen, key is neither a name nor other text: insert var head_kw; insert var p
src	STOP	src/closure_extract.rs:1888	BTreeMap String	edges | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:1888	BTreeSet String	edges | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:1891	BTreeSet String	node_set | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:1893	BTreeMap String	indeg | insert seen, key is neither a name nor other text: entry var from; insert var n
src	STOP	src/closure_extract.rs:1894	BTreeMap String	effective_edges | insert seen, key is neither a name nor other text: entry var to
src	STOP	src/closure_extract.rs:1915	BTreeSet String	emitted | insert seen, key is neither a name nor other text: insert var n
src	STOP	src/closure_extract.rs:2686	BTreeSet String	outer_locals | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:2778	BTreeSet String	outer_locals | no insert or entry in this file names this binding
src	STOP	src/closure_extract.rs:2848	BTreeSet String	outer_locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:35	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:158	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:289	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:429	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:637	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:696	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:708	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:719	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:979	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1046	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1114	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1183	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1267	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1317	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1380	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1446	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1498	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1568	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1625	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1683	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/collection/infer.rs:1761	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/declare/parse.rs:348	HashMap String	meta | insert seen, key is neither a name nor other text: insert var key_str
src	STOP	src/declare/parse.rs:360	HashMap String	FnShapeMetadata | no insert or entry in this file names this binding
src	STOP	src/declare/register.rs:385	HashMap String	surface_type_subst | no insert or entry in this file names this binding
src	STOP	src/declare/register.rs:648	HashMap String	meta | no insert or entry in this file names this binding
src	STOP	src/declare/register.rs:673	HashMap String	meta | no insert or entry in this file names this binding
src	STOP	src/declare/register.rs:1846	BTreeSet String	declared_rete_defns | no insert or entry in this file names this binding
src	STOP	src/freeze/env.rs:66	BTreeSet String	declared_rete_defns | no insert or entry in this file names this binding
src	STOP	src/freeze.rs:474	BTreeSet String	declared_rete_defns | no insert or entry in this file names this binding
src	STOP	src/freeze.rs:533	BTreeSet String	declared_rete_defns | no insert or entry in this file names this binding
src	STOP	src/function/infer.rs:83	HashMap String	outer_locals | no insert or entry in this file names this binding
src	STOP	src/function/infer.rs:164	HashMap String	exposed_bounds | no insert or entry in this file names this binding
src	STOP	src/function/infer.rs:167	HashMap String	mapping | insert seen, key is neither a name nor other text: insert var tp
src	STOP	src/function/infer.rs:255	HashMap String	bounds | no insert or entry in this file names this binding
src	STOP	src/function/infer.rs:294	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/function/parse.rs:1144	HashMap String	impl_clauses | insert seen, key is neither a name nor other text: insert var method_name
src	STOP	src/intrinsic/mod.rs:530	HashMap &str	entries | insert seen, key is neither a name nor other text: insert var entry
src	STOP	src/intrinsic/mod.rs:672	HashMap &str	impls_by_fqdn | insert seen, key is neither a name nor other text: entry var submission
src	STOP	src/intrinsic/mod.rs:678	HashMap &str	eval_handler_by_fqdn | insert seen, key is neither a name nor other text: insert var submission
src	STOP	src/intrinsic/mod.rs:684	HashMap &str	tail_handler_by_fqdn | insert seen, key is neither a name nor other text: insert var submission
src	STOP	src/intrinsic/mod.rs:3021	HashMap &str	seen | insert seen, key is neither a name nor other text: entry var s
src	STOP	src/intrinsic/special/ann_form.rs:87	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/forms.rs:85	HashMap String	_locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/holon_literal.rs:55	HashMap String	_locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/macroexpand.rs:111	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/quasiquote.rs:88	HashMap String	_locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/quote.rs:93	HashMap String	_locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/stream_lazy.rs:114	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/struct_to_form.rs:89	HashMap String	locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/special/use_form.rs:114	HashMap String	_locals | no insert or entry in this file names this binding
src	STOP	src/intrinsic/string.rs:875	HashMap String	kwargs | insert seen, key is neither a name nor other text: insert var key_name
src	STOP	src/intrinsic/string.rs:876	HashSet String	used | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/load/loader.rs:457	HashSet String	visited | insert seen, key is neither a name nor other text: insert var fetched
src	STOP	src/load/loader.rs:476	HashSet String	visited | insert seen, key is neither a name nor other text: insert var fetched
src	STOP	src/load/loader.rs:495	HashSet String	visited | insert seen, key is neither a name nor other text: insert var fetched
src	STOP	src/macros/expand.rs:1348	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:1564	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:1633	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:1669	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:2033	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:2127	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:2333	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:2379	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/expand.rs:2431	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var param; insert var rest_name
src	STOP	src/macros/registry.rs:55	HashMap String	macros | insert seen, key is neither a name nor other text: insert var def
src	STOP	src/macros/registry.rs:59	HashMap String	symbol_alias | insert seen, key is neither a name nor other text: insert var alias
src	STOP	src/macros/registry.rs:63	HashSet String	ambiguous_symbols | insert seen, key is neither a name nor other text: insert var alias
src	STOP	src/reflect/verbs.rs:1646	HashSet String	before | no insert or entry in this file names this binding
src	STOP	src/resolve/normalize.rs:76	HashSet String	prev | no insert or entry in this file names this binding
src	STOP	src/resolve/quote.rs:25	HashSet String	scope | no insert or entry in this file names this binding
src	STOP	src/resolve/walk.rs:72	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:256	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:295	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:352	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:363	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:369	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:392	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:419	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:434	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:464	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:493	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:545	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/resolve/walk.rs:575	HashSet String	scope | insert seen, key is neither a name nor other text: insert var id
src	STOP	src/rete/alpha_tree.rs:322	HashMap String	var_to_field | no insert or entry in this file names this binding
src	STOP	src/rete/alpha_tree.rs:351	HashMap String	var_to_field | no insert or entry in this file names this binding
src	STOP	src/rete/alpha_tree.rs:389	HashMap String	var_to_field | no insert or entry in this file names this binding
src	STOP	src/rete/clause.rs:474	HashSet &str	known | no insert or entry in this file names this binding
src	STOP	src/rete/compiled_cond.rs:749	HashMap String	names | insert seen, key is neither a name nor other text: insert var reserved
src	STOP	src/rete/compiled_cond.rs:793	HashMap String	names | insert seen, key is neither a name nor other text: insert var reserved
src	STOP	src/rete/compiled_cond.rs:833	HashMap String	names | insert seen, key is neither a name nor other text: insert var reserved
src	STOP	src/rete/compiled_rhs.rs:119	HashMap String	CompiledRhsByRule | no insert or entry in this file names this binding
src	STOP	src/rete/export.rs:1994	HashMap String	m | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/arm.rs:1190	HashSet String	rule_names | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/fire/delta.rs:350	HashSet &str	scan_classes | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/fire/mod.rs:1263	HashSet &str	wanted | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/fire/mod.rs:1265	HashMap &str	idx | insert seen, key is neither a name nor other text: entry var a
src	STOP	src/rete/kernel/fire/mod.rs:1390	HashSet &str	wanted | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/fire/pass/alpha.rs:46	HashMap String	map | insert seen, key is neither a name nor other text: insert var class
src	STOP	src/rete/kernel/fire/pass/alpha.rs:124	HashSet &str	scan_classes | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/fire/rules.rs:120	HashSet String	stratum_rule_names | insert seen, key is neither a name nor other text: insert var rname
src	STOP	src/rete/kernel/fire/rules.rs:384	HashSet &str	wanted | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/session.rs:157	HashMap String	QueryMemory | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/session.rs:167	HashMap String	AlphasByType | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/session.rs:172	HashMap String	LeafAidsByClass | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/stratify.rs:228	HashMap String	type_strata | insert seen, key is neither a name nor other text: insert var p
src	STOP	src/rete/kernel/stratify.rs:275	HashMap String	type_strata | insert seen, key is neither a name nor other text: insert var p
src	STOP	src/rete/kernel/stratify.rs:311	HashMap String	type_strata | insert seen, key is neither a name nor other text: insert var p
src	STOP	src/rete/kernel/stratify.rs:974	HashMap String	reach | insert seen, key is neither a name nor other text: entry var c; entry var k
src	STOP	src/rete/kernel/tests/accum_alpha_cost.rs:855	HashMap String	std_map | insert seen, key is neither a name nor other text: insert var u
src	STOP	src/rete/kernel/tests/accum_cost.rs:535	HashSet &str	wanted | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/tests/accum_cost.rs:539	HashMap &str	idx | insert seen, key is neither a name nor other text: entry var a
src	STOP	src/rete/kernel/tests/accum_cost.rs:557	HashMap &str	idx | insert seen, key is neither a name nor other text: entry var a
src	STOP	src/rete/kernel/tests/accum_cost.rs:575	HashMap &str	idx | insert seen, key is neither a name nor other text: entry var a
src	STOP	src/rete/kernel/tests/cascade_cost.rs:48	BTreeSet &str	names | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/tests/mod.rs:514	HashMap &str	samples | insert seen, key is neither a name nor other text: entry var name
src	STOP	src/rete/kernel/tests/mod.rs:516	HashMap &str	pairs | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/rete/kernel/tests/pass_semantics.rs:605	HashSet String	locs | no insert or entry in this file names this binding
src	STOP	src/rete/kernel/tests/stratify_numbers.rs:104	HashMap String	m | no insert or entry in this file names this binding
src	STOP	src/rete/matcher.rs:433	HashSet String	bound | no insert or entry in this file names this binding
src	STOP	src/rete/matcher.rs:462	HashSet String	bound | no insert or entry in this file names this binding
src	STOP	src/rete/purity.rs:1982	BTreeSet String	declared | no insert or entry in this file names this binding
src	STOP	src/rete/purity.rs:3086	BTreeMap String	by_ns | insert seen, key is neither a name nor other text: entry var ns
src	STOP	src/rete/purity.rs:3105	BTreeSet &str	known | no insert or entry in this file names this binding
src	STOP	src/rete/purity.rs:3106	BTreeSet &str	live | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:291	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:357	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:381	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:425	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:776	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:830	HashMap String	field_types | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:831	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:970	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/mod.rs:1274	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/typing.rs:565	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/typing.rs:721	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/typing.rs:889	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/validate/typing.rs:1040	HashMap String	binds | no insert or entry in this file names this binding
src	STOP	src/rete/vocabulary.rs:1644	HashMap &str	BY_NAME | no insert or entry in this file names this binding
src	STOP	src/special_forms.rs:55	HashMap String	REGISTRY | no insert or entry in this file names this binding
src	STOP	src/types/defstruct.rs:43	HashMap String	ParsedStructMeta | no insert or entry in this file names this binding
src	STOP	src/types/defstruct.rs:54	HashMap String	field_restrictions | insert seen, key is neither a name nor other text: insert var field_sym
src	STOP	src/types/defstruct.rs:153	HashMap String	field_restrictions | insert seen, key is neither a name nor other text: insert var field_sym
src	STOP	src/types.rs:730	HashMap String	field_restrictions | no insert or entry in this file names this binding
src	STOP	src/types.rs:1103	HashMap String	types | insert seen, key is neither a name nor other text: insert var e; insert var name
src	STOP	src/types.rs:1116	HashSet String	builtin_names | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/types.rs:1137	HashMap String	source_forms | insert seen, key is neither a name nor other text: insert var def_name
src	STOP	src/types.rs:1158	HashMap String	generic_edges | insert seen, key is neither a name nor other text: entry var key
src	STOP	src/types.rs:1330	HashSet String	s | insert seen, key is neither a name nor other text: insert var p
src	STOP	src/types.rs:4083	HashMap String	acronyms | no insert or entry in this file names this binding
src	STOP	src/types.rs:4677	HashMap String	acronyms | no insert or entry in this file names this binding
src	STOP	src/types.rs:4861	HashMap String	acronyms | no insert or entry in this file names this binding
src	STOP	src/types.rs:7509	HashMap String	mapping | no insert or entry in this file names this binding
src	STOP	src/types.rs:7531	HashMap String	mapping | no insert or entry in this file names this binding
src	STOP	src/types.rs:7586	HashSet String	visiting | insert seen, key is neither a name nor other text: insert var name; insert var qualified
src	STOP	src/types.rs:7725	HashSet String	visiting | insert seen, key is neither a name nor other text: insert var name; insert var qualified
src	STOP	src/types.rs:7842	HashSet String	visiting | insert seen, key is neither a name nor other text: insert var name; insert var qualified
src	STOP	src/value/environment.rs:165	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/value/environment.rs:236	HashMap String	bindings | insert seen, key is neither a name nor other text: insert var name
src	STOP	src/value/symbol_table.rs:16	HashMap String	BindingMetadata | no insert or entry in this file names this binding
src	STOP	src/value/symbol_table.rs:16	HashMap String	BindingMetadata | no insert or entry in this file names this binding
src	STOP	src/value/symbol_table.rs:150	HashMap String	acronym_registry | no insert or entry in this file names this binding
src	STOP	src/value/value.rs:448	HashMap String	metadata | no insert or entry in this file names this binding
src	STOP	src/value/value.rs:490	HashMap String	impl_clauses | no insert or entry in this file names this binding
crates	STOP	crates/wat-macros/src/discover.rs:334	HashMap String	aliases | insert seen, key is neither a name nor other text: insert var alias
crates	STOP	crates/wat-macros/src/lib.rs:804	HashMap String	seen_names | insert seen, key is neither a name nor other text: insert var sanitized
tests	STOP	tests/cli/every_recorded_migration_replays.rs:184	HashMap String	b_counts | insert seen, key is neither a name nor other text: entry var line
tests	STOP	tests/cli/every_recorded_migration_replays.rs:350	HashSet String	spec_unused | no insert or entry in this file names this binding
tests	STOP	tests/lint/census_emitted_name_is_read_or_declared.rs:128	BTreeSet &str	read_names | no insert or entry in this file names this binding
tests	STOP	tests/lint/census_name_read_by_a_cost_test_is_emitted.rs:456	BTreeSet String	names | insert seen, key is neither a name nor other text: insert var lit
tests	STOP	tests/lint/census_name_read_by_a_cost_test_is_emitted.rs:458	BTreeSet String	count_names | insert seen, key is neither a name nor other text: insert var lit
tests	STOP	tests/lint/census_name_read_by_a_cost_test_is_emitted.rs:461	BTreeSet String	computed_helpers | insert seen, key is neither a name nor other text: insert var helper
tests	STOP	tests/lint/floor_never_reads_a_clock.rs:378	HashSet String	tainted | insert seen, key is neither a name nor other text: insert var name
tests	STOP	tests/lint/floor_never_reads_a_clock.rs:541	HashSet String	tainted | insert seen, key is neither a name nor other text: insert var name
tests	STOP	tests/lint/gen_doc_surface_matches.rs:59	BTreeSet String	want | no insert or entry in this file names this binding
tests	STOP	tests/lint/gen_doc_surface_matches.rs:61	BTreeSet String	want | no insert or entry in this file names this binding
tests	STOP	tests/lint/keyword_heresy_ledger.rs:246	BTreeSet String	doors | insert seen, key is neither a name nor other text: insert var name
tests	STOP	tests/lint/keyword_heresy_ledger.rs:246	BTreeSet String	kw_consts | insert seen, key is neither a name nor other text: insert var n
tests	STOP	tests/lint/keyword_heresy_ledger.rs:246	BTreeSet String	kw_lists | insert seen, key is neither a name nor other text: insert var n
tests	STOP	tests/lint/keyword_heresy_ledger.rs:247	BTreeMap String	fn_params | insert seen, key is neither a name nor other text: insert var n
tests	STOP	tests/lint/keyword_heresy_ledger.rs:249	BTreeMap String	scopes | no insert or entry in this file names this binding
tests	STOP	tests/lint/keyword_heresy_ledger.rs:536	BTreeSet String	kw_consts | insert seen, key is neither a name nor other text: insert var n
tests	STOP	tests/lint/keyword_heresy_ledger.rs:536	BTreeSet String	kw_lists | insert seen, key is neither a name nor other text: insert var n
tests	STOP	tests/lint/keyword_heresy_ledger.rs:592	BTreeSet String	doors | insert seen, key is neither a name nor other text: insert var name
tests	STOP	tests/lint/nested_program_starts.rs:165	HashMap String	lets | no insert or entry in this file names this binding
tests	STOP	tests/lint/nested_program_starts.rs:261	HashMap String	lets | no insert or entry in this file names this binding
tests	STOP	tests/lint/nested_program_starts.rs:272	HashMap String	lets | no insert or entry in this file names this binding
tests	STOP	tests/lint/nested_program_starts.rs:355	HashMap String	lets | no insert or entry in this file names this binding
tests	STOP	tests/lint/nested_program_starts.rs:582	HashMap String	srcs | insert seen, key is neither a name nor other text: insert var p
tests	STOP	tests/lint/one_member_join.rs:152	HashMap String	worlds | insert seen, key is neither a name nor other text: entry var rel
tests	STOP	tests/lint/rete_bind_generators.rs:90	HashSet String	out | insert seen, key is neither a name nor other text: insert var name
tests	STOP	tests/lint/rete_citation_resolves.rs:431	BTreeSet String	rust | no insert or entry in this file names this binding
tests	STOP	tests/lint/rete_citation_resolves.rs:433	BTreeSet String	wat | no insert or entry in this file names this binding
tests	STOP	tests/lint/rete_citation_resolves.rs:529	BTreeSet &String	distinct | no insert or entry in this file names this binding
tests	STOP	tests/lint/rete_citation_resolves.rs:545	BTreeSet String	seen | insert seen, key is neither a name nor other text: insert var token
tests	STOP	tests/lint/rete_citation_resolves.rs:693	BTreeSet String	in_prose | no insert or entry in this file names this binding
tests	STOP	tests/lint/rete_compile_gate.rs:111	BTreeSet String	out | insert seen, key is neither a name nor other text: insert var ns
tests	STOP	tests/lint/rete_compile_gate.rs:161	BTreeSet String	namespaces | no insert or entry in this file names this binding
tests	STOP	tests/lint/rete_engine_label_names_its_evidence.rs:631	BTreeSet String	quals | no insert or entry in this file names this binding
tests	STOP	tests/lint/rete_engine_label_names_its_evidence.rs:704	BTreeSet String	quals | no insert or entry in this file names this binding
tests	STOP	tests/lint/rete_names_in_wat_scripts_resolve.rs:533	BTreeMap String	fields | insert seen, key is neither a name nor other text: insert var name
tests	STOP	tests/lint/rete_names_in_wat_scripts_resolve.rs:875	BTreeSet String	seen | insert seen, key is neither a name nor other text: insert var token
tests	STOP	tests/lint/rete_names_in_wat_scripts_resolve.rs:1053	BTreeSet String	node_accessors | no insert or entry in this file names this binding
tests	STOP	tests/macros/probe_hygiene_scopes_reader_gate.rs:104	HashSet &str	allowed | no insert or entry in this file names this binding
tests	STOP	tests/macros/probe_macro_eval_prevalidated_caller_gate.rs:78	HashSet &str	allowed | no insert or entry in this file names this binding
tests	STOP	tests/reflection/probe_arc278_registry_census.rs:49	BTreeSet String	names | no insert or entry in this file names this binding
tests	STOP	tests/reflection/probe_arc278_registry_census.rs:72	BTreeMap &String	multi | no insert or entry in this file names this binding
tests	STOP	tests/rete/probe_arc278_compiled_where_ops.rs:102	HashMap String	per_file | name evidence and other-text evidence: entry var path
tests	STOP	tests/rete/probe_arc278_compiled_where_ops.rs:102	HashMap String	per_file | name evidence and other-text evidence: entry var path
```

Four of the seven `OTHER` rows are names in the registration reading and other-text in the variable-name reading, because the parameter is called `path` and the other-text rule treats that name as a filesystem path. Listed here, still counted as `OTHER` in the table above, and not reclassified after the fact:

- `src/value/symbol_table.rs:33` `functions: HashMap<String, Arc<Function>>`. `register_function(&mut self, path: String, …)` and `function_entry(path)`.
- `src/value/symbol_table.rs:42` `unit_variants: HashMap<String, EnumValue>`. `register_unit_variant(&mut self, path: String, …)`. The field doc says the key is a keyword path.
- `src/value/symbol_table.rs:108` `runtime_def_values: HashMap<String, Value>`. `register_def_value(&mut self, path: String, …)`.
- `src/rust_deps/mod.rs:278` `UseDeclarations.declared: HashSet<String>`. `declare(&mut self, path: String)`. The type doc says the set records `(:wat::core::use! :rust::...)` declarations.

The other three `OTHER` rows are file tables: `src/edn/bridge.rs:249` `OriginTable.index` (`insert var file`, a `Span` file string), `src/load/loader.rs:1057` `source_files`, `src/load/loader.rs:1058` `payload_files`.

`src/value/symbol_table.rs:16` `BindingMetadata = HashMap<String, HashMap<String, WatAST>>` is two `STOP` rows (no insert in that file names the binding). The outer map is the FQDN layer the brief named.

## 3. Identifier's string surface

A row is one call of `as_str`, `leaf`, `path`, `receiver`, or `flat`, or a free `leaf(` / `path(` in a file that imports that name (or in `crates/wat-reader/src/identifier.rs`). `IDENTITY`: the string becomes a key, is compared, or is passed to the door. `PRINT`: it reaches output.

| | calls | IDENTITY | PRINT |
|---|---:|---:|---:|
| src | 214 | 213 | 1 |
| crates | 51 | 49 | 2 |
| tests | 11 | 11 | 0 |
| all | 276 | 273 | 3 |

Call shape on this TSV: `src` `method.as_str` IDENTITY 177, `method.receiver` IDENTITY 36, `method.as_str` PRINT 1. `crates` `method.as_str` IDENTITY 29, `free.leaf` 7, `free.path` 6, `method.receiver` 5, `method.leaf` 1, `method.path` 1, `method.as_str` PRINT 2. `tests` `method.as_str` IDENTITY 11.

The three `PRINT` rows are the two printers and the render arm named in the proof (`render.rs:908`, `wat-doc` `print.rs:291`, `print.rs:364`).

Top 20 surface files, `src`: `src/types.rs` 24, `src/runtime.rs` 23, `src/macros/expand.rs` 18, `src/resolve/normalize.rs` 15, `src/check.rs` 14, `src/rete/validate/typing.rs` 12, `src/rete/expr_ir/mod.rs` 11, `src/closure_extract.rs` 10, `src/declare/parse.rs` 6, `src/edn/render.rs` 6, `src/form_match.rs` 5, then 3 each in `src/edn/bridge.rs`, `src/reflect/verbs.rs`, `src/resolve/walk.rs`, `src/rete/compiled_rhs.rs`, `src/rete/kernel/arm.rs`, `src/rete/kernel/stratify.rs`, `src/rete/matcher.rs`, `src/rete/validate/mod.rs`, `src/scope/resolution.rs`.

`crates`: `crates/wat-reader/src/identifier.rs` 25, `crates/wat-source-derive/src/lib.rs` 16, `crates/wat-doc/src/lib.rs` 4, `crates/wat-doc/src/print.rs` 2, `crates/wat-macros/src/discover.rs` 2, `crates/wat-reader/src/ast.rs` 2.

`tests`: `tests/function/wat_arc170_closure_extraction.rs` 4, `tests/lint/nested_program_starts.rs` 3, then 1 each in `tests/lint/decl_identity.rs`, `tests/lint/rete_compile_gate.rs`, `tests/lint/rete_header_claims_are_asserted.rs`, `tests/wat_lang/wat_arc144_special_forms.rs`.

## 4. Door callers

Every door row is `IDENTITY`. `PRINT` is 0 on every door in every tree. The TSV `DOOR` section is the 247-row list (file, line, which function).

| door | src | crates | tests | all |
|---|---:|---:|---:|---:|
| `canonical_identity` | 104 | 25 | 37 | 166 |
| `fact_class_key` | 18 | 0 | 0 | 18 |
| `ns_to_wat_path` | 47 | 2 | 0 | 49 |
| `canonical_type_key` | 14 | 0 | 0 | 14 |
| all | 183 | 27 | 37 | 247 |

The brief's 133 `canonical_identity` sites are replaced by 166. `fact_class_key` is `src/edn/render.rs:3748` and calls `canonical_identity` (that inner call is its own row at `:3749`). `canonical_type_key` is `src/types.rs:373` and calls `canonical_identity` at `:374`. `ns_to_wat_path` is `crates/wat-reader/src/identifier.rs:492`.

Files with the most door rows: `src/types.rs` 36, `src/edn/render.rs` 23, `crates/wat-reader/src/identifier.rs` 23, `tests/services/probe_arc255_87_name_not_substring.rs` 20, `src/runtime.rs` 14, `src/rete/expr_ir/mod.rs` 8, `src/check.rs` 7, `src/macros/registry.rs` 7, `src/declare/parse.rs` 6, `src/resolve/normalize.rs` 6, `src/rete/collect.rs` 6.

## 5. Hot path

Scratch only. Feature `name_census_probe` on `wat-reader`, forwarded from the `wat` package. `#[track_caller]` on `Identifier::as_str` and `canonical_identity`. Atomics for the two call counts. Allocation counts inside `canonical_identity` (each `to_string` / `format!`), `ns_to_wat_path` (`replace` plus `format!`, counted as 2), `fold_member_twin` (each `format!`), and `Identifier::bare` (3 when the spelling has a `/`, 2 when it does not: the `flat` arc, the namespace arc, and a name arc only on the slash arm). Thread-local site maps flush every 50_000 hits and on thread drop, to `/tmp/name-census-hot.txt`. The feature, the probe module, and every call-site edit were removed before this commit (`git grep name_census_probe` is empty).

One test, release, the existing 90s limit left where it is:

`cargo nextest run --release --offline --features name_census_probe -E 'test(=test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle)'`

```
Finished `release` profile [optimized] target(s) in 0.06s
PASS [  47.133s] (1/1) wat::kernel test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle
Summary [  47.143s] 1 test run: 1 passed, 6456 skipped
HOT_RC=0
```

A first filter, `test(=deftest_wat_tests_rete_fuzz_test_native_matches_oracle)`, matched 0 tests (`HOT_RC=4`). The binary is `wat::kernel` and the test name is `test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle`. That 0-test run is not a measurement.

This run's counters:

```
CANON 90566722
AS_STR 157556885
ALLOC 97145563
```

`AS_STR` is `Identifier::as_str` only. `CANON` is `canonical_identity` only.

Top call sites, copied from the probe file. The hottest `canonical_identity` site is `src/types.rs:374`, the call inside `canonical_type_key` (71_123_771 of 90_566_722). Next: `src/types.rs:1346` `TypeEnv::get` (6_046_111), `src/types.rs:7279` the parse-path canonicalize (3_849_844), `src/runtime.rs:949` a reference-symbol call head (2_555_072), `src/edn/render.rs:3763` `type_denotation` (2_231_724). The hottest `as_str` site is `src/scope/resolution.rs:81`, `env_key` borrowing `flat` when the scope set is empty (91_614_503 of 157_556_885). Next: `src/runtime.rs:1785` `ident.as_str() == "nil"` (54_536_722). `src/match_arm.rs:110` is on both lists: it calls `canonical_identity(id.as_str())` (1_259_169 each).

```
CANON 90566722
AS_STR 157556885
ALLOC 97145563
SITE canon 71123771 src/types.rs:374
SITE canon 6046111 src/types.rs:1346
SITE canon 3849844 src/types.rs:7279
SITE canon 2555072 src/runtime.rs:949
SITE canon 2231724 src/edn/render.rs:3763
SITE canon 1259169 src/match_arm.rs:110
SITE canon 755959 src/form_match.rs:272
SITE canon 649780 src/edn/render.rs:3749
SITE canon 576014 src/rete/clause.rs:354
SITE canon 288701 /home/john/.rustup/toolchains/1.97.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ops/function.rs:250
SITE canon 246264 src/rete/purity.rs:1419
SITE canon 193291 src/types.rs:223
SITE canon 162322 src/rete/clause.rs:166
SITE canon 151176 src/edn/render.rs:1370
SITE canon 128031 src/rete/clause.rs:425
SITE canon 87132 src/rete/clause.rs:293
SITE canon 74725 src/edn/render.rs:1535
SITE canon 67979 src/runtime.rs:7556
SITE canon 41184 src/rete/kernel/node.rs:231
SITE canon 22176 src/rete/eval_insert.rs:145
SITE canon 19605 src/types.rs:291
SITE canon 6816 src/rete/expr_ir/mod.rs:590
SITE canon 5759 src/macros/eval.rs:177
SITE canon 5095 src/declare/typevar.rs:230
SITE canon 4303 src/check.rs:982
SITE canon 3611 src/check.rs:1585
SITE canon 2069 src/macros/expand.rs:1899
SITE canon 1698 src/macros/registry.rs:196
SITE canon 1296 src/rete/kernel/arm.rs:259
SITE canon 1296 src/rete/kernel/arm.rs:450
SITE canon 777 src/function/parse.rs:758
SITE canon 720 src/declare/register.rs:581
SITE canon 703 src/macros/expand.rs:1927
SITE canon 675 src/types.rs:1305
SITE canon 634 src/macros/expand.rs:263
SITE canon 278 src/types.rs:6236
SITE canon 276 src/resolve/boundary.rs:108
SITE canon 175 src/runtime.rs:5156
SITE canon 111 src/types.rs:845
SITE canon 96 src/macros/registry.rs:202
SITE as_str 91614503 src/scope/resolution.rs:81
SITE as_str 54536722 src/runtime.rs:1785
SITE as_str 2555072 src/runtime.rs:949
SITE as_str 2553040 crates/wat-reader/src/ast.rs:311
SITE as_str 1259169 src/match_arm.rs:110
SITE as_str 1168771 src/form_match.rs:253
SITE as_str 755959 src/form_match.rs:272
SITE as_str 704057 src/rete/clause.rs:414
SITE as_str 601394 src/rete/matcher.rs:263
SITE as_str 448201 src/edn/render.rs:1326
SITE as_str 298978 src/types.rs:6832
SITE as_str 274491 src/runtime.rs:8989
SITE as_str 246264 src/rete/purity.rs:1419
SITE as_str 96433 src/scope/resolution.rs:94
SITE as_str 67979 src/runtime.rs:7556
SITE as_str 49248 src/edn/bridge.rs:472
SITE as_str 41184 src/rete/kernel/node.rs:231
SITE as_str 37728 src/rete/kernel/node.rs:235
SITE as_str 29584 crates/wat-reader/src/ast.rs:222
SITE as_str 27831 src/rete/matcher.rs:776
SITE as_str 22176 src/rete/eval_insert.rs:145
SITE as_str 21229 src/macros/expand.rs:725
SITE as_str 19387 src/macros/expand.rs:761
SITE as_str 12852 src/rete/matcher.rs:465
SITE as_str 11906 src/resolve/normalize.rs:241
SITE as_str 11880 src/rete/compiled_rhs.rs:149
SITE as_str 11880 src/rete/compiled_rhs.rs:177
SITE as_str 11880 src/rete/compiled_rhs.rs:178
SITE as_str 7667 src/resolve/normalize.rs:128
SITE as_str 6816 src/rete/expr_ir/mod.rs:590
SITE as_str 6646 src/types.rs:6948
SITE as_str 5759 src/macros/eval.rs:177
SITE as_str 4303 src/check.rs:982
SITE as_str 3611 src/check.rs:1585
SITE as_str 2592 src/rete/expr_ir/mod.rs:465
SITE as_str 2275 src/resolve/normalize.rs:1094
SITE as_str 2275 src/resolve/normalize.rs:1095
SITE as_str 2069 src/macros/expand.rs:1899
SITE as_str 1961 src/check.rs:7659
SITE as_str 1842 src/macros/expand.rs:734
```

One `canonical_identity` site is `library/core/src/ops/function.rs:250` (288_701). `track_caller` attributes that hit to the `Fn` impl that invoked the function, not to a wat source line.

The interned pair removes the `String` return of `canonical_identity` (and the allocs inside `bare` / `ns_to_wat_path` / `fold_member_twin`) and the `as_str` spelling read. On this deftest that is 90_566_722 door calls, 157_556_885 `as_str` calls, and 97_145_563 allocations. The 47.133s is this single-test scratch run. It is not a re-run of `.floor/2026-10-04T02-26-49Z`, and the 90s limit was not raised.

## Proposed split

A proposal for the orchestrator. None of these stones is started. The 355 `STOP` maps, and the four `OTHER` rows whose parameter is named `path`, stay unresolved until a ruling. They are not sized into a stone.

1. **The `Name` type and `Identifier` equality.** Gate 2's equality half. Size: `Eq`/`Hash` in `identifier.rs`, the two `==` sites, the one `HashSet<Identifier>` test, and `into_bound`. `substitute` (`src/runtime.rs:14217`) has to keep a binder equal to its body reference, or move in the same stone. The six sample spellings do not merge: `:a::b/c` and `a.b/c` stay two pairs until `::` is annihilated. The door's `String` return moves here too: 247 call sites, every one `IDENTITY`, zero `PRINT` (`canonical_identity` 166, `ns_to_wat_path` 49, `fact_class_key` 18, `canonical_type_key` 14).
2. **The registries.** Gate 2's "no string map key for a name". Size: the 65 `NAME` rows. The 355 `STOP` rows and the four ambiguous `path` parameters are outside this size.
3. **`REG` and `DISPATCH` literals**, by a recorded Rust-literal codemod. Gate 3's literal half for those two classes. Size: `REG` 1954, `DISPATCH` 683, together 2637 literals (`src` holds 1879 and 663 of them).
4. **`CMP` and `BUILD`.** The comparisons and the format/concat that assembles a name (the class that mangled generics). Size: `CMP` 2286, `BUILD` 398, together 2684. `tests` holds 1210 of the `CMP` rows.
5. **`flat` deleted last.** Gate 2's printer-only string, after the callers read the pair. Size: 276 surface calls (273 `IDENTITY`, 3 `PRINT`) plus the hot-path cost in section 5. The three `PRINT` sites are the printers (`render.rs:908`, `wat-doc` `print.rs:291` and `:364`) and become `format!("{namespace}/{name}")`.

`WAT` is 241 literals (src 196, crates 3, tests 42). That is 5c-iv's embedded source, and 5c-iv was not started. `MSG` is 1417. `OTHER` is 15135, of which 8371 are doc comments (7377 src + 291 crates + 703 tests). Those are gate 3's remaining text, not a gate-2 stone.

`fold_member_twin`'s capitalization rule stays as the brief recorded it: ruled out, reverted by a later stone, not by this one.

## Gates

| gate | result |
|---|---|
| planted classes | `PROOF ok 9`. Each planted class, the attribute `REG`, and the `starts_with` `CMP` classified as planted. The known `PRINT` caller and the known `IDENTITY` caller landed on those classes. |
| clippy | `cargo clippy --release --offline --all-targets -- -D warnings`. `Finished release profile [optimized] target(s) in 14.33s`. `CLIPPY_RC=0`. |
| floor | not run. Nothing under `tests/` changed. |
| diff | against `a54a5e197`: `Cargo.toml`, `src/bin/name_census.rs`, `name-census.tsv`, this score. The `[[bin]]` stanza and the `syn` / `proc-macro2` dependency move are the instrument, as above. |

## STOP-1

A `STOP` means stop. No cure stone was started.

- 355 text-keyed maps (310 src, 2 crates, 43 tests) whose key evidence is ambiguous or missing. The rows are the block in section 2.
- 4 maps the instrument called `OTHER` because the key variable is named `path`, which the registration reading treats as a keyword path or a `:rust::` declaration: `symbol_table.rs` `functions`, `unit_variants`, `runtime_def_values`, and `UseDeclarations.declared`. They need a ruling before they join the 65 `NAME` registries.
