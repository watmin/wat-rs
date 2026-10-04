# SCORE — STONE 255.91: the name is the pair

Struck 2026-10-04 on `main`. Drawn against `61d54d1bf`. Executor grok, solo. Not pushed. Stone 2 (registries keyed by `Name`), the literal codemod, `CMP`/`BUILD`, deletion of `flat`, 5c-iv, and 5d were not started. 255.89 stays on `origin/cutover-5c-iii`.

The work is four commits, watmin `<john@shields.wtf>`:

| hash | subject |
|---|---|
| `b928f45b43acbbec29dccddc380d59c8a983cb39` | the type, `from_keyword`, the census example (the old bin is still in this commit) |
| `78c96f52736eb27523ec3d08c1cc456a5bd2a1a8` | delete `src/bin/name_census.rs` |
| `9d74f18402bdee997b66a37b019446476fcc2b2c` | the member keyword and its symbol are one `Name` |
| `4db92dc4e065e0625c76d4a4f4337988ee51b9a4` | link `Name`'s `Display` at [`std::fmt::Display`] |

`4db92dc4e065e0625c76d4a4f4337988ee51b9a4` is the tree the acceptance floor ran. `git status --porcelain` was empty before that floor and is empty before this score.

## The agreement

`origin/cutover-5c-iii` holds converted corpus `d45406b02297abb70a623713be9e8b3b2f85ecab` beside pre-image `9d85da5a518d4909b76df02c0ffa85549bd53a25`. The zip is `git diff -U0 9d85da5a5 d45406b02 -- *.wat`, comments stripped, tokens of alnum plus `_.:+*/<>=!?'-$&` (`$` stays inside the token). Arrow tokens `:-` `<-` `->` `:->` are dropped. Equal-length token lists zip by position; a pair is recorded only when the old token changed and is a keyword (`:` and `::`). Unequal lists use `SequenceMatcher(autojunk=False)` on equal-length replace spans.

The table that zip wrote, `/tmp/kw-pairs-v2.tsv`, has 11105 rows and 11105 distinct keywords. The occurrence column sums to 114412. The script's `AMBIGUOUS` is a keyword with more than one symbol image; a table with one row per keyword has none. `Name::from_keyword` against `Identifier::bare(s)` on that full table: the Rust test printed `KW_PAIRS_FULL 11105` and passed (zero disagreements). The floor does not set `KW_PAIRS_FULL`. It ran the committed sample.

Committed fixture `crates/wat-reader/tests/fixtures/keyword-symbol-pairs.tsv`: 2 header lines and 305 data rows. Every shape is in it:

| shape | keyword | symbol |
|---|---|---|
| colon-path | `:a2::Bad` | `a2/Bad` |
| member-join | `:acc::CountF/g` | `acc.CountF/g` |
| type-twin | `:cons::Consumer::PageState` | `cons.Consumer/PageState` |
| marker | `:my::` | `my` |
| division | `:wat::bigint::/` | `wat.bigint//` |
| wat.type | `:wat::type::Infer` | `wat.type/Infer` |

The header's two forced rows are in the file: `:wat::core::i64` → `wat.core/i64` (1 occurrence in the full table, not `wat.type/i64`) and `:wat::cache::Lru/get` → `wat.cache.Lru/get` (6). `:wat::cache::Lru::get`, `:a::b::c`, and `:a::b/c` are not in the corpus. The equality probe constructs the last two. Other confirmed images: `:my::kernel::` → `my.kernel` (7), `:wat::core::/` → `wat.core//` (9), `:wat::i64::/` → `wat.i64//` (81), `:wat::type::i64` → `wat.type/i64` (11).

`from_keyword` is the port of `wat_keyword_to_clojure_symbol` (`src/edn/render.rs`) plus the trailing-`::` marker rule. No position argument. No closed-set `wat.type/` special case inside it. `wat_keyword_to_clojure_symbol` was not replaced. A keyword with no `::` (`:k`, `:else`, `:T`, bare `:i64`) is `None`.

Disagreements: 0. STOP-2 did not fire.

## The type and the one resolution function

`Name { namespace: Arc<str>, name: Arc<str> }` lives in `crates/wat-reader/src/identifier.rs`. `Eq` compares contents, with an `Arc::ptr_eq` fast path on each field. `Hash` hashes the text, not the pointer. There is no interner. `Display` is the one stringification: `namespace/name`, or the bare `name` when the namespace is `$bound`. The intra-doc links say [`std::fmt::Display`].

`Identifier` holds that `Name` (`pair`), the scope set, and `flat` as a print cache. `PartialEq` and `Hash` are `(pair, scopes)`. `bare` still splits on the first `/`. `into_bound` is still `{$bound, <whole spelling>}`, `flat` unchanged, idempotent when the ident is not a reference. `receiver()` still reads `flat.contains('/')`; that is an accessor. `Debug` still prints `flat` under the field name `name`.

`local_spelling` is the one spelling a binder and a body reference share. A reference borrows `flat`. A binder borrows `pair.name`. `same_local` is equal scopes and equal local spellings. A slashed local (`into_bound("foo/bar")` versus `bare("foo/bar")`) is not pair-equal. Both local spellings are `foo/bar`, so `same_local` holds and both `env_key`s are `foo/bar`. Slash-less locals already share the pair.

`substitute` (`src/runtime.rs`) matches with `ident.same_local(target)`. `env_key` and `scope_divergent_binder` (`src/scope/resolution.rs`) borrow `local_spelling()`.

`ast_same_identity` (`src/macros/registry.rs`): keyword/keyword stays `canonical_identity`. Symbol/symbol stays `sa == sb`, which is now pair equality. The keyword/symbol cross arm is `id.is_reference() && Name::from_keyword(k).as_ref() == Some(id.pair())`. `fold_member_twin` was not ported.

That cross arm makes `:wat::core::Option/expect` and `wat.core.Option/expect` one `Name`. The old door kept them apart. The program that pinned the old divergence is `macros::tests::duplicate_defmacro_symbol_spelling_is_the_same_macro`. The second registration is now a no-op (`Ok([])`). The brief required this comparison, so the test was updated to expect `member.is_ok()`. That is the cure in `9d74f18402bdee997b66a37b019446476fcc2b2c`. It is not STOP-1: the brief named the cross arm, and no other program was edited to follow it. `$bound/foo` and `foo` are the same pair; the reader refuses `$bound/x` in user source. Keyword/keyword was not changed.

Floor of the tree before that cure, `.floor/2026-10-04T04-19-50Z`. Do not re-run it. `Summary [ 395.153s] 6415 tests run: 6414 passed (28 slow), 1 failed, 24 skipped`. The arm:

```
        FAIL [   0.007s] (1366/6415) wat macros::tests::duplicate_defmacro_symbol_spelling_is_the_same_macro
  stdout ───

    running 1 test
    test macros::tests::duplicate_defmacro_symbol_spelling_is_the_same_macro ... FAILED

    failures:

    failures:
        macros::tests::duplicate_defmacro_symbol_spelling_is_the_same_macro

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1401 filtered out; finished in 0.00s

  stderr ───

    thread 'macros::tests::duplicate_defmacro_symbol_spelling_is_the_same_macro' (2930666) panicked at src/macros/tests.rs:606:5:
    a member join is not the :: identity; got Ok([])
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

The assertion that fired is the member-join one. The kwargs-companion assertion in the same test had already passed.

The next floor, `.floor/2026-10-04T04-29-44Z`, is also not the acceptance floor. Do not re-run it. Nextest was green: `Summary [ 393.955s] 6415 tests run: 6415 passed (27 slow), 24 skipped`. The doc-link judge then exited 1:

```
1 NEW broken intra-doc link(s) — not in KNOWN_BROKEN_DOC_LINKS:
  crates/wat-reader/src/identifier.rs: [`Display`] × 2

  FIX THE LINK. Do NOT add a line to this ledger.
```

Three [`Display`] links in `identifier.rs` were rewritten to [`std::fmt::Display`]. No ledger line was added. A targeted re-judge of that doc build printed `JUDGE=0`.

## The census example

`examples/name_census.rs` is the instrument. `[[bin]] name_census` is gone. `syn` and `proc-macro2` are back in `[dev-dependencies]` with the comment they had, plus one sentence that this stone moved the census so an example can link them. `cargo run --release --example name_census -- --out /tmp/name-census-25591.tsv` finished with `PARSE_FAIL 0`. The committed `name-census.tsv` was not rewritten. It is the 255.90 capture, taken on the 255.89 tree. `tests/rete/probe_arc255_89_identity.rs` is in that TSV and absent from this tree. Section totals are therefore not a numeric match to the TSV. This run:

| section | src | crates | tests |
|---|---|---|---|
| LIT | 15830 (REG 1879, DISPATCH 663, CMP 891, BUILD 311, MSG 521, WAT 196, OTHER 11369) | 663 (REG 1, DISPATCH 20, CMP 146, BUILD 26, MSG 32, WAT 0, OTHER 438) | 5545 (REG 75, DISPATCH 0, CMP 1180, BUILD 63, MSG 868, WAT 40, OTHER 3319) |
| MAP | 377 (NAME 59, OTHER 7, STOP 311) | 3 (1 / 0 / 2) | 47 (5 / 0 / 42) |
| SURFACE | 200 (IDENTITY 199, PRINT 1) | 51 (49 / 2) | 10 (10 / 0) |
| EQ | 1 | 0 | 0 |
| IDMAP | 0 | 1 | 0 |
| DOOR | 169, all IDENTITY | 2, both `canonical_identity` | 35, all `canonical_identity` |

PRINT is still 3 across the three trees. Doors sum to 206, all IDENTITY (`canonical_identity` 88+2+35, `fact_class_key` 18, `ns_to_wat_path` 49, `canonical_type_key` 14). The pair probe's `==` is now pair equality, and the variable is still named `flat_eq`. The into_bound line is `flat_eq false pair_eq false bound_ns "$bound" bound_name "wat.core/x" flat "wat.core/x"`. The 255.90 score printed `flat_eq true` for that same call.

## Gates

Acceptance floor `.floor/2026-10-04T04-37-58Z` at `4db92dc4e065e0625c76d4a4f4337988ee51b9a4`. Doctests: wat 5 passed, 1 ignored; wat_edn 3 passed; wat_macros 4 ignored. Nextest:

```
Summary [ 396.106s] 6415 tests run: 6415 passed (28 slow), 24 skipped
```

`doc-link-judge.log` in that directory is 0 bytes. `scripts/floor.sh` exits 0 only when nextest and the doc-link judge are both 0. The wrapper printed `[floor] doc-link exit=0` and `FLOOR_RC=0`.

Test-name set against `.floor/2026-10-03T13-09-11Z` (6412 `PASS` lines of the form `(n/N)`, keyed by binary and test): **MISSING 0**. EXTRA 3, the tests this stone added:

- `wat scope::resolution::tests::slashed_binder_and_body_reference_share_an_env_key`
- `wat-reader identifier::tests::from_keyword_matches_bare_of_the_converted_symbol` — `PASS [   0.006s] (6285/6415)`
- `wat-reader identifier::tests::the_name_is_the_pair` — `PASS [   0.005s] (6299/6415)`

The slashed-binder test passed at `PASS [   0.019s] (2163/6415)`.

Cost. The fuzz deftest on this floor: `PASS [  72.223s] (2371/6415) wat::kernel test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle`. The same line on `.floor/2026-10-03T13-09-11Z` is `PASS [  71.238s]`. The brief's 71.2s is that run. No limit was raised.

Clippy `cargo clippy --release --all-targets -- -D warnings` on the tree of `9d74f18402bdee997b66a37b019446476fcc2b2c` (the commit before the three doc-link characters): `Finished release profile [optimized] target(s) in 12.42s`, `CLIPPY_RC=0`. An earlier run of the same command, before the macro-test cure, finished in 14.34s, also rc 0. Clippy was not re-run on the doc-link commit.

Ignores. The SEAM command `git grep -hcE '^\s*#\[ignore' -- 'src/*.rs' 'tests/*.rs' | paste -sd+ | bc` printed `18`. This stone added none.

## STOPs

STOP-1 did not fire. STOP-2 did not fire.
