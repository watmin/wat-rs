# SCORE — STONE 255.92: the registries are keyed by `Name`

Struck 2026-10-04 on `main`. Drawn against `4d4087f03`. Executor grok, solo. Not pushed.

**STOP-1.** Keying the registries by `Name` makes two spellings that were distinct strings one pair, and existing programs resolve differently. Stone 3 (the `REG`/`DISPATCH` literal codemod), stone 4, deletion of `flat`, 5c-iii, 5c-iv, and 5d were not started. The red floor was not re-run.

Work commits, watmin `<john@shields.wtf>`:

| hash | subject |
|---|---|
| `ea5fd333943876130999489b2eaadd62c48b424b` | the registries are keyed by `Name` |
| `49e4b81db` | link `Self::from_keyword` on `Name::enter` |

The floor ran on `ea5fd3339`, with `git status --porcelain` empty. `49e4b81db` is one doc-comment character sequence after that floor. It was not given a second floor.

## STOP-1

`Name::enter` sends `:wat::core::Option/expect` and `:wat::core::Option::expect` to one pair, `{wat.core.Option, expect}`. The same collapse hits every other `/` member and its `::` twin. Lookups of either spelling hit the one entry.

The program that required them to stay apart is `tests/macros/probe_arc251_8d_macro_member_join_wrong_join.wat.bad`. It registers `:user::helper/of` and calls `:user::helper::of`. The file's header says a keyword author writing the other join against a stored `/` name must stay refused. On this floor the file starts up clean:

```
        FAIL [  21.651s] ( 261/6416) wat::lint every_wat_bad_fixture_actually_fails::every_wat_bad_fixture_actually_fails_shard_12
  stdout ───

    running 1 test
    test every_wat_bad_fixture_actually_fails::every_wat_bad_fixture_actually_fails_shard_12 ... FAILED

    failures:

    failures:
        every_wat_bad_fixture_actually_fails::every_wat_bad_fixture_actually_fails_shard_12

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 375 filtered out; finished in 21.63s

  stderr ───

    thread 'every_wat_bad_fixture_actually_fails::every_wat_bad_fixture_actually_fails_shard_12' (95624) panicked at /home/john/work/holon/wat-rs/tests/lint/every_wat_bad_fixture_actually_fails.rs:392:1:


    🔥 1 `.wat.bad` file(s) in shard 12/16 START UP CLEAN and do not declare why. `.wat.bad` claims a file fails to start up; nothing checked that claim until this gate, and a fixture that starts up fine makes every assertion resting on it a coincidence.

    THE FIX, one of two:

    1. If the test that drives it asserts `is_ok()`, or starts the world up and INVOKES (asserting the error comes at EVAL), the file is a valid program and the NAME is wrong — `git mv` it to `.wat` and update every `.rs` referrer. That is the common case: 13 of the first 16 were exactly this.

    2. If its test asserts `is_err()` and is `#[ignore]`d as RED-at-HEAD, the badness is BANKED against a substrate change that has not landed. Declare it: `;; rune:lint(bad-is-banked) — <what the substrate should do instead> banked-by: <the ignored test's fn name>`.

    ⛔ NOT a fix: adding a `:user::main`. This gate drives `startup_from_file`, which does not want one — that is the binary, and measuring this corpus through the binary is the error that got the first draft of this gate withdrawn.

      tests/macros/probe_arc251_8d_macro_member_join_wrong_join.wat.bad
          starts up CLEAN (startup_from_file returned Ok) but is named `.wat.bad`, and declares nothing
```

The assertion that fired is the shard-12 gate at `tests/lint/every_wat_bad_fixture_actually_fails.rs:392`. The named file is the whole finding.

The registry states the same fact directly. `probe_arc255_14_namespace_join::a_namespace_member_answers_to_both_surfaces_and_only_the_new_join` wants `:wat::spawn::process::runner-count` present and `:wat::spawn::process/runner-count` absent. The panic:

```
    thread 'probe_arc255_14_namespace_join::a_namespace_member_answers_to_both_surfaces_and_only_the_new_join' (162158) panicked at /home/john/work/holon/wat-rs/tests/services/probe_arc255_14_namespace_join.rs:137:5:
    the namespace join answered wrongly on 1 row(s):
      registry: :wat::spawn::process/runner-count present=true, want false — the respelled declaration must hold ONE key, not two
```

The arm is the `want false` row at `tests/services/probe_arc255_14_namespace_join.rs:106`. `SymbolTable::get` now enters both spellings as one `Name`, so the `/` key is present.

The wide failure is the same collapse inside `match`. `wat/kernel/services/stdio.wat` matches `RecvOutcome` and `WriteResponse` by keyword. The floor's repeated panic is `PatternMatchFailed`, scrutinee type `wat::core::Enum`, no arm matched, at `stdio.wat:215` (`stdio-write-out`) and `stdio.wat:310` (`stdio-read-frame`). That message is what `uuid`, `recursive_patterns`, `every_probe_runs`, and `every_recorded_migration_replays` die on. It is not a second mechanism.

`:wat::core::i64` and `:wat::type::i64` stay distinct (`wat.core` / `i64` and `wat.type` / `i64`). Unifying a keyword with the symbol `from_keyword` already produced is the 255.91 door, not this stop.

## STOP-2

A rendered parametric form is not a `Name`. `Name::enter` returns `None` when the spelling contains `(` or `<`, or a `:-` binder that is not a `::` leaf beginning with `-`. `:wat::core::->`, `:wat::core::->>`, and `:wat::bracket::-type-slot-name` are names. The first draft of the detector treated every `:-` byte pair as rendered and parked those three leaves in the side map. The startup measure after the fix does not.

`NameMap` / `NameSet` / `NameBTree` keep an enter-`None` spelling in `rendered` under that exact text. They are not forced into a `Name`.

`startup_bare` (`freeze::tests::rendered_keys_of_startup_bare`, PASS [0.444s] on the measure, then pinned in the committed test) reports 52 rendered type keys. Symbol registries: 0. Macro registry: 0. Fifteen are `parametric_extensions`, seventeen are `subtype_edges`, twenty are `subtype_parents`. Every one starts with `(`.

`parametric_extensions` and the same child on `subtype_edges`:

- `(:wat::cache::hologram-svc::Handle :- [:T])`
- `(:wat::cache::lru-svc::Handle :- [:K :V :T])`
- `(:wat::kernel::Process :- [:S :R])`
- `(:wat::kernel::Thread :- [:S :R])`
- `(:wat::kernel::stderr-svc::Handle :- [:T])`
- `(:wat::kernel::stdin-svc::Handle :- [:T])`
- `(:wat::kernel::stdout-svc::Handle :- [:T])`
- `(:wat::query::mem-store::Handle :- [:T])`
- `(:wat::query::sqlite-store::Handle :- [:T])`
- `(:wat::stream::Stream :- [:T])`
- `(:wat::telemetry::journal::Handle :- [:T])`
- `(:wat::telemetry::span::Handle :- [:T])`
- `(wat.type/List :- [:T])`
- `(wat.type/PersistentVector :- [:T])`
- `(wat.type/Vector :- [:T])`

`subtype_edges` also has `(wat.type/HashMap :- [:K :V])` and `(wat.type/HashSet :- [:T])`.

`subtype_parents`:

- `(:wat::capability::Dialable :- [(:wat::cache::Cache::Op :- [:K :V]) (:wat::cache::Cache::Reply :- [:K :V]) :T])`
- `(:wat::capability::Dialable :- [(:wat::cache::Cache::Op :- [:wat::holon::HolonAST :wat::holon::HolonAST]) (:wat::cache::Cache::Reply :- [:wat::holon::HolonAST :wat::holon::HolonAST]) :T])`
- `(:wat::capability::Dialable :- [:wat::kernel::StdErr::Op :wat::kernel::StdErr::Reply :T])`
- `(:wat::capability::Dialable :- [:wat::kernel::StdIn::Op :wat::kernel::StdIn::Reply :T])`
- `(:wat::capability::Dialable :- [:wat::kernel::StdOut::Op :wat::kernel::StdOut::Reply :T])`
- `(:wat::capability::Dialable :- [:wat::query::Store::Op :wat::query::Store::Reply :T])`
- `(:wat::capability::Dialable :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply :T])`
- `(:wat::capability::Dialable :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply :T])`
- `(:wat::capability::TypedCapability :- [(:wat::cache::Cache::Op :- [:K :V]) (:wat::cache::Cache::Reply :- [:K :V]) :T])`
- `(:wat::capability::TypedCapability :- [(:wat::cache::Cache::Op :- [:wat::holon::HolonAST :wat::holon::HolonAST]) (:wat::cache::Cache::Reply :- [:wat::holon::HolonAST :wat::holon::HolonAST]) :T])`
- `(:wat::capability::TypedCapability :- [:wat::kernel::StdErr::Op :wat::kernel::StdErr::Reply :T])`
- `(:wat::capability::TypedCapability :- [:wat::kernel::StdIn::Op :wat::kernel::StdIn::Reply :T])`
- `(:wat::capability::TypedCapability :- [:wat::kernel::StdOut::Op :wat::kernel::StdOut::Reply :T])`
- `(:wat::capability::TypedCapability :- [:wat::query::Store::Op :wat::query::Store::Reply :T])`
- `(:wat::capability::TypedCapability :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply :T])`
- `(:wat::capability::TypedCapability :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply :T])`
- `(:wat::core::Seqable :- [:T])`
- `(:wat::spawn::Locus :- [:wat::kernel::Transport.Shared])`
- `(:wat::spawn::Locus :- [:wat::kernel::Transport.Wire])`
- `(:wat::spawn::Spawned :- [:S :R])`

One closure extract, `wat_arc170_closure_extraction::t3_toplevel_defn_uses_user_types`, wrote no rendered closure keys. That dump ran before the leaf fix. The leaf fix moves spellings out of `rendered`. It does not add any.

## What is keyed by `Name`

`src/name_map.rs` is the map. The authoritative key is `Name`. The inserted spelling is retained so iteration, diagnostics, and `UseDeclarations::covers` still see the text that was stored. `NameSet::insert` keeps the first spelling when the pair is already present. `Name` is `Ord` by namespace then name, so `NameBTree` iteration is that order, not keyword-spelling order.

| map | was | now |
|---|---|---|
| `SymbolTable.functions`, `unit_variants`, `runtime_def_values`, `binding_metadata` | `HashMap<String, _>` | `NameMap` |
| `TypeEnv.types`, `subtype_edges`, `source_forms`, `parametric_extensions`, `generic_edges` | `HashMap<String, _>` | `NameMap` |
| `TypeEnv.builtin_names`, `subtype_parents` | `HashSet<String>` | `NameSet` |
| `MacroRegistry.macros` | `HashMap<String, MacroDef>` | `NameMap` |
| `CheckEnv.unit_variant_types` | `HashMap<String, TypeExpr>` | `NameMap` |
| `UseDeclarations.declared` | `HashSet<String>` | `NameSet` |
| `captured_deps`, `captured_types`, `captured_macros` | `BTreeMap<String, _>` | `NameBTree` |

`get(&str)` remains. It calls `Name::enter` once at the boundary. `get_name(&Name)` is the pair lookup on `TypeEnv`, `SymbolTable`, and `MacroRegistry`. Changing `get`'s signature would have been a hand sweep of the literal call sites. The compiler accepted the boundary. That is the transitional constructor. It is not stone 3's codemod. No literal was rewritten by hand to build a `Name`.

`TypeEnv::get` no longer calls `canonical_identity`. `contains` is `types` or `builtin_names`.

`wat_keyword_to_clojure_symbol` (`src/edn/render.rs`) still returns `None` for a trailing `::` and for no `::`. Otherwise it is `Name::from_keyword(kw).map(|name| name.to_string())`. One definition. Markers stay `None`.

`duplicate_defmacro_symbol_spelling_is_the_same_macro` asserts `len() == 1`, an empty residue, and `get` of the one name, for both the kwargs companion and the member join, then `expand_src` is still `is_ok()`.

`install_symbol_alias` returns before the ambiguous-join branch when `Name::enter(primary)` and `Name::enter(alias)` are the same pair. `symbol_alias` and `ambiguous_symbols` stay string maps. They index a clojure spelling that is not itself the macro key.

## Text that stayed text

These still hold name spellings. A `Name` does not flow into them. The compiler did not ask for them once `get` kept `&str`.

- `SymbolTable.acronym_registry`: namespace keyword to a list of acronym strings.
- `binding_metadata`'s inner map: metadata keywords such as `:restricted-to`. The outer key is the name.
- `CheckEnv`: `schemes`, `param_bounds`, `defined_values`, `defined_value_spans`, `defclause_registrations`, `defined_value_asts`, `corpus_values`. `extend_registrations` is `(protocol, type)` string pair to method names.
- `UseDeclarations::covers` is still a prefix of the stored spelling (`/` or `::`). `contains` is the pair.
- Closure: `captured_defs`, `dep_edges`, `type_edges`, `types_visited`, `deps_visited`, and the local sets.
- `surface_param_debt: Vec<(String, Span)>`.
- `reconstruct_call_path`'s cache is `HashMap<String, HashMap<String, Arc<str>>>` of namespace text to name text. It is a call-path cache, not a name registry.
- `rust_deps` `symbols` / `types` (`HashMap`/`HashSet<String>` at the two registry structs) are Rust symbol tables, not `UseDeclarations`.
- `is_builtin_primitive` still matches `wat::type::i64`-style strings. `classify` and `is_known_type` still call `canonical_identity` only to strip a colon for that list.

`canonical_type_key` still runs where a caller builds a string: `constructor_head_key` (`src/types.rs`), the type-registration keyword arms in that file, `src/intrinsic/holon/atom.rs` (the `HashMap` prefix test), and `rekey_type_member_functions` (`src/freeze/env.rs`), which still `format!`s `:{parent}` and `{denoted_parent}/{method}`. After this stone those two joins are one `Name`, so the remove-and-insert is a no-op when `enter` unifies them. The function was left in place. Those producers are the residue stone 3 would size. They are not registry keys anymore for `TypeEnv::get`.

## Census

`cargo run --release --example name_census -- --out /tmp/name-census-25592-after.tsv`. `CENSUS_RC=0`. `PARSE_FAIL 0`. The committed `name-census.tsv` was not rewritten.

NAME rows fell. Before, on this tree, before any 255.92 edit: src 59, crates 1, tests 5 (65). After: src 53, crates 1, tests 5 (59).

The six src NAME rows that left are `captured_deps`, `captured_types`, `captured_macros`, `subtype_edges`, `subtype_parents`, and `parametric_extensions`. The other registries in the table above were OTHER or STOP in the census (the key expression was a variable named `path`, or the file had no insert the classifier could see). They left the string-key rows too. MAPS src 377 → 362. OTHER src 7 → 3. STOP src 311 → 306. The three `rendered` maps in `src/name_map.rs` are STOP (`insert var spelling`), not NAME.

## Floor

Do not re-run `.floor/2026-10-04T05-34-06Z`.

Doctests exit 0. Nextest:

```
Summary [ 472.787s] 6416 tests run: 6088 passed (28 slow), 328 failed, 24 skipped
```

`FLOOR_RC=100`. Every failing block is `.floor/2026-10-04T05-34-06Z/ARM.txt` (1,310,900 bytes). Test-name set against `.floor/2026-10-04T04-53-19Z`, regex `^\s*(PASS|FAIL|SLOW)\s+\[[^\]]+\]\s+\(\s*\d+/\d+\)\s+(\S+)\s+(\S+)\s*$`: base 6415, this floor 6416, MISSING 0, EXTRA 1 (`wat freeze::tests::rendered_keys_of_startup_bare`).

Fuzz on this floor, over the 71.9 s budget:

```
PASS [  85.249s] (2465/6416) wat::kernel test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle
```

An isolated run of that one test, before the floor, was `PASS [  37.793s]` (`Summary [  37.804s] 1 test run: 1 passed, 6438 skipped`). The number the gate asked for is the floor's 85.249 s. It rose. It was not optimized, and the limit was not raised, because STOP-1 already stops the stone.

Clippy on the implementation tree, before the one-line link repair: `Finished release profile [optimized] target(s) in 12.58s`, `CLIPPY_RC=0`. `cargo clippy --release --all-targets -- -D warnings`.

Ignores, `git grep -hcE '^\s*#\[ignore' -- 'src/*.rs' 'tests/*.rs' | paste -sd+ | bc`: 18.

The doc-link judge on this floor is not empty. `cargo doc` finished in 12.97 s. The judge:

```
broken intra-doc links moved away from the frozen ledger:

1 NEW broken intra-doc link(s) — not in KNOWN_BROKEN_DOC_LINKS:
  crates/wat-reader/src/identifier.rs: [`from_keyword`] × 1
```

`49e4b81db` changes that link to [`Self::from_keyword`]. No ledger line was added. That commit was not floored.

## What was not done

No hand sweep of literals. No global interner. No `HashMap<String, Name>` spelling index. `flat` is still there. `fold_member_twin` was not ported. No `.wat` file was edited. The 328 failures were not patched. Stone 3 was not started.
