# SCORE — STONE 255.95: a keyword holds a pair

**Drawn against `main` @ `0b2dcc1e2`. Scored on `main` @ `983f37362` (the brief `107e678f0` and the amend are already that commit).** Executor: grok, solo. **STOP-2.** No `src/` change. No floor. The scratch counter was removed before this commit. The reader stone was not started.

`git diff 0b2dcc1e2` shows the brief and the amend. Those stay. This commit adds this score only.

## Item 1 — the 88,952, with spellings

A scratch counter (feature `kw_value_census`, removed; not in this tree) noted every `::` keyword value that stayed a keyword after the eval fallthrough in `src/runtime.rs` (the `WatAST::Keyword` arm, after nil, the retired bare variant, `:wat::core::Option.None`, `unit_variant`, `def_value`, and `sym.get`). The failed `sym.get` in that arm is not counted as a lookup. `from-string` and reflection are a different door; they are not these rows.

Populations, each file its own process, `RUST_MIN_STACK=33554432` (the peer-thread stack, not a time limit):

- `wat-tests/`: 90 files. Every example process printed `RC 0`, including `deporder.wat` and `lint.wat`. That RC is the process, not a deftest result. Passed and failed deftests were not summed. Do not read this as the suite being green. 255.94's in-process runner on the same directory was 405 passed and 40 failed; those 40 were not re-run here.
- `tests/function/` sample, the 88 files 255.94 used (no `*.wat.bad`): startup only. Bodies are not run. 24 logs begin `startup-fail` (the fixture does not start). They are listed below. The other 64 begin `startup-ok`.

Each file includes one stdlib freeze, the same accounting as 255.94's 88,952.

| population | files | spellings | hits |
|---|---:|---:|---:|
| wat-tests | 90 | 58 | 88952 |
| tests/function startup | 88 | 5 | 1136 |

The startup 5 spellings are a subset of the wat-tests 58. No new spelling. Startup hits 1136.

**DATA spellings: none.** The list is empty. **STOP-1 does not fire.**

### The rule

One rule for the table. A spelling is **NAME** when it denotes something declared and the program looks it up or would: a registry hit, or the site is a declared call head, a generated method, a service name, or a function name. Registry bits that count as declared: `fn`, `type`, `type-name`, `intrinsic`, `rust`, `macro`, `macro-name`. `enter` alone is not that. `eq`, `hash`, `edn`, and `render` are uses.

A spelling is **DATA** when it is an opaque token: compared, stored, or printed, and never resolved. A missed `sym.get` does not make a declared method into that token.

8 spellings hit a declared bit (87,783 hits). The other 50 (1,169 hits) are NAME by the site, not by a registry bit. 87783 + 1169 = 88952.

The heaviest site is the `file:line:col` with the largest attributed count. A tie takes the last site string. `Two` also has `wat-tests/rete/differential-fuzz-rules.wat:39:30` with 181 (184 + 181 = 365). `Alt` also has `wat-tests/rete/differential-fuzz-rules.wat:43:30` with 180. Each circular basis spelling has four sites of 1 in `wat-tests/holon/Circular.wat` (lines 14, 15, 21, 22); the table shows one of them.

| count | bits | class | spelling | heaviest site | site count |
|---:|---|---|---|---|---:|
| 26852 | enter+type+type-name+macro+macro-name | NAME | `:wat-tests::rete::tms::C` | `wat-tests/rete/differential-fuzz-tms.wat:54:39` | 26852 |
| 26852 | enter+type+type-name+macro+macro-name | NAME | `:wat-tests::rete::tms::D` | `wat-tests/rete/differential-fuzz-tms.wat:60:39` | 26852 |
| 16632 | enter+type+type-name+macro+macro-name | NAME | `:wat-tests::rete::fuzz::S2` | `wat-tests/rete/differential-fuzz.wat:54:86` | 16632 |
| 11088 | enter+type+type-name+macro+macro-name | NAME | `:wat-tests::rete::fuzz::S3` | `wat-tests/rete/differential-fuzz.wat:57:86` | 11088 |
| 5544 | enter+type+type-name+macro+macro-name | NAME | `:wat-tests::rete::fuzz::S4` | `wat-tests/rete/differential-fuzz.wat:60:86` | 5544 |
| 825 | enter | NAME | `:wat::spawn::Locus/launch` | `wat/service.wat:2401:21` | 825 |
| 365 | enter+type+type-name+macro+macro-name | NAME | `:wat-tests::rete::rules::Two` | `wat-tests/rete/differential-fuzz-rules.wat:38:30` | 184 |
| 360 | enter+type+type-name+macro+macro-name | NAME | `:wat-tests::rete::rules::Alt` | `wat-tests/rete/differential-fuzz-rules.wat:44:30` | 180 |
| 90 | enter | NAME | `:wat::kernel::stderr-svc/start$impl` | `wat/service.wat:2761:32` | 90 |
| 90 | enter | NAME | `:wat::kernel::stdin-svc/start$impl` | `wat/service.wat:2761:32` | 90 |
| 90 | enter | NAME | `:wat::kernel::stdout-svc/start$impl` | `wat/service.wat:2761:32` | 90 |
| 90 | enter+type+type-name | NAME | `:wat::telemetry::Log` | `wat/telemetry.wat:343:37` | 90 |
| 6 | enter+hash | NAME | `:wat-tests::recorder` | `wat-tests/service-telemetry-bridge.wat:44:1` | 6 |
| 5 | enter+hash+edn | NAME | `:wat::cache::hologram-svc` | `wat/cache.wat:392:1` | 5 |
| 5 | enter+hash | NAME | `:wat::cache::lru-svc` | `wat/cache.wat:203:1` | 5 |
| 4 | enter+hash | NAME | `:wat-tests::worker` | `wat-tests/service-telemetry-bridge.wat:63:1` | 4 |
| 4 | enter | NAME | `:wat::std::circular-cos-basis` | `wat-tests/holon/Circular.wat:22:10` | 1 |
| 4 | enter | NAME | `:wat::std::circular-sin-basis` | `wat-tests/holon/Circular.wat:22:10` | 1 |
| 2 | enter+hash | NAME | `:wat-tests::counter` | `wat-tests/service-locus-parity.wat:35:1` | 2 |
| 2 | enter+hash | NAME | `:wat-tests::hib-counter` | `wat-tests/service-hibernate-resume.wat:32:1` | 2 |
| 2 | enter+hash | NAME | `:wat-tests::mal-bag` | `wat-tests/service-request-malformed.wat:54:1` | 2 |
| 2 | enter+hash | NAME | `:wat-tests::pcache-svc` | `wat-tests/service-parametric-messages.wat:91:1` | 2 |
| 2 | enter | NAME | `:wat-tests::recorder/start$impl-thread` | `wat/service.wat:2760:32` | 2 |
| 2 | enter | NAME | `:wat-tests::worker/start$impl-thread` | `wat/service.wat:2760:32` | 2 |
| 1 | enter+hash+edn | NAME | `:wat-tests::admin-counter` | `wat-tests/service-admin-facet.wat:31:1` | 1 |
| 1 | enter | NAME | `:wat-tests::admin-counter/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::admin-counter/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter+hash | NAME | `:wat-tests::barebox-svc` | `wat-tests/service-parametric-bare-messages.wat:49:1` | 1 |
| 1 | enter | NAME | `:wat-tests::barebox-svc/start$impl` | `wat/service.wat:2761:32` | 1 |
| 1 | enter+hash | NAME | `:wat-tests::box-svc` | `wat-tests/service-parametric.wat:49:1` | 1 |
| 1 | enter | NAME | `:wat-tests::box-svc/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter | NAME | `:wat-tests::counter/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::counter/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter+hash+edn | NAME | `:wat-tests::deadline` | `wat-tests/timer-env-grab-parity.wat:31:1` | 1 |
| 1 | enter | NAME | `:wat-tests::deadline/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::deadline/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter | NAME | `:wat-tests::hib-counter/resume$impl-process` | `wat/service.wat:2882:31` | 1 |
| 1 | enter | NAME | `:wat-tests::hib-counter/resume$impl-thread` | `wat/service.wat:2884:33` | 1 |
| 1 | enter | NAME | `:wat-tests::hib-counter/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::hib-counter/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter | NAME | `:wat-tests::mal-bag/start$impl` | `wat/service.wat:2761:32` | 1 |
| 1 | enter+hash+edn | NAME | `:wat-tests::offset-counter` | `wat-tests/service-multiparam-init.wat:27:1` | 1 |
| 1 | enter | NAME | `:wat-tests::offset-counter/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::offset-counter/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter+hash | NAME | `:wat-tests::pair-svc` | `wat-tests/service-parametric-two-params.wat:55:1` | 1 |
| 1 | enter | NAME | `:wat-tests::pair-svc/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter | NAME | `:wat-tests::pcache-svc/start$impl` | `wat/service.wat:2761:32` | 1 |
| 1 | enter+hash+edn | NAME | `:wat-tests::resp-counter` | `wat-tests/service-stop-resp.wat:30:1` | 1 |
| 1 | enter | NAME | `:wat-tests::resp-counter/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::resp-counter/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter+hash | NAME | `:wat-tests::seeded-counter` | `wat-tests/service-init-parity.wat:29:1` | 1 |
| 1 | enter | NAME | `:wat-tests::seeded-counter/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::seeded-counter/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter | NAME | `:wat-tests::signal-observer/start$impl-process` | `wat/service.wat:2758:30` | 1 |
| 1 | enter | NAME | `:wat-tests::signal-observer/start$impl-thread` | `wat/service.wat:2760:32` | 1 |
| 1 | enter | NAME | `:wat-tests::worker/resume$impl-thread` | `wat/service.wat:2884:33` | 1 |
| 1 | enter | NAME | `:wat::cache::hologram-svc/start$impl` | `wat/service.wat:2761:32` | 1 |
| 1 | enter | NAME | `:wat::cache::lru-svc/start$impl` | `wat/service.wat:2761:32` | 1 |

### tests/function startup

| count | bits | spelling | heaviest site | site count |
|---:|---|---|---|---:|
| 792 | enter | `:wat::spawn::Locus/launch` | `wat/service.wat:2401:21` | 792 |
| 88 | enter | `:wat::kernel::stderr-svc/start$impl` | `wat/service.wat:2761:32` | 88 |
| 88 | enter | `:wat::kernel::stdin-svc/start$impl` | `wat/service.wat:2761:32` | 88 |
| 88 | enter | `:wat::kernel::stdout-svc/start$impl` | `wat/service.wat:2761:32` | 88 |
| 80 | enter+type+type-name | `:wat::telemetry::Log` | `wat/telemetry.wat:343:37` | 80 |

Startup-fail logs (24). The counter still recorded the stdlib freeze in each:

`tests/function/defn_bad_type.wat`, `defn_redef.wat`, `fn_rename_bare_fn_type.wat`, `fn_rename_legacy_lambda.wat`, `fn_rename_mixed_legacy.wat`, `fn_rename_multi_lambda.wat`, `fn_signature_body_mismatch.wat`, `fn_signature_malformed_args.wat`, `probe_check_scoped_param_resolution_handwritten.wat`, `probe_check_scoped_param_resolution_macro.wat`, `recursive_patterns_nonexhaustive.wat`, `stone18a_e01.wat`, `stone18a_e02.wat`, `stone18a_e03.wat`, `stone18a_e04.wat`, `stone18a_e05.wat`, `stone18a_e06.wat`, `variadic_define_amp_no_binder.wat`, `variadic_define_arity_err.wat`, `variadic_define_double_amp.wat`, `variadic_define_fixed_after_rest.wat`, `variadic_define_non_vector_rest.wat`, `variadic_define_strict_extra_args.wat`, `variadic_define_type_err.wat`.

## STOP-2 — the keyword is the lookup key and the stored token

Item 4 asks each stdlib caller of `keyword/from-string` what the name is for, and says to stop when one keyword is both a lookup key and a stored token. That caller is `wat.service/defservice` in `wat/service.wat`. The stone stops here. Items 2, 3, and 5 were not started. No caller was rewritten.

`serve-name` is `(wat.keyword/from-string (wat.string/interpolate "{b}::serve" :b fqdn-base))` at `wat/service.wat:981`. That keyword is the defined function: `(wat.core/defn ~serve-name …)` at `:2538` and again at `:2988`, and `~serve-name` is the call head through the serve body (first at `:1651`). The same spelling is handed to launch as a value: `(wat.keyword/from-string ~serve-name-str)` at `:2508`, `:2655`, `:2672`, `:2687`, `:2786`, `:2802`, and `:2817`. The comment at `wat/service.wat:2371` says why: a spliced literal `:fqdn::serve` would resolve to a function (`:2372`), so serve is passed as a runtime keyword, and the impl invokes it via `apply`. `wat/spawn.wat:289` records the same door, and `:290` is the `apply` of `keyword/from-string`: the generated child main reached `serve` that way because it did not resolve statically.

`Status::Started` is the same shape. `status-started-kw` is `compose-variant` of `(keyword/from-string "{b}::Status")` and `:Started` at `wat/service.wat:1158`. `status-started-str` (`:1165`) is that keyword's text, and start/resume pass `(keyword/from-string ~status-started-str)` (`:2660` and the five sibling launch calls). The comment at `:1161` says the launch surface takes the opaque keyword, and the thread tier resolves it via `apply`.

The generated methods are the same spelling twice. `start-impl-name` and `start-impl-call` (`:2590` and `:2592`) are both `keyword/from-string` of `{b}/start$impl`. The name is the `defn` at `:2698`. The call is the head spliced at `:2761`. The thread and process copies, and the resume copies (`:2594` through `:2614`), are that pair again. `:wat::kernel::stdout-svc/start$impl` in the table is one of these names (`:2761`).

A service name is stored as data on the process record and is also the declared service. `:name (wat.keyword/from-string ~fqdn-base)` is on each `wat.process/Service` map (`:2650` and the five sibling launch calls). `fqdn-kw` (`:189`) is that same text rebuilt as a keyword for the acronym lookup the comment names. The table's hashed service rows (`:wat::cache::hologram-svc`, `:wat::cache::lru-svc`, and the `wat-tests` service names) are this store: `hash` and, for some, `edn`, on a name the program also declares. They stayed NAME in item 1 because they denote the declaration. They are why item 4 cannot pick "make it a symbol" or "leave it a keyword" without a ruling.

`keyword/from-string` under `wat/` is 130 grep lines. Call sites are in `wat/service.wat`, `wat/bracket.wat`, `wat/core.wat`, `wat/fix.wat`, `wat/query.wat`, `wat/telemetry/span.wat`, `wat/rete/oracle/accum-pass.wat`, and `wat/rete/compile.wat`. `wat/spawn.wat:290` is the comment above, not a call. The other files were not classified one by one. One dual-use caller is enough to stop.

There is no runtime symbol value to switch these to. `Value` has `wat__core__keyword(Arc<String>)` (`src/value/value.rs:61`). It has no symbol variant. A symbol in a form is `WatAST::Symbol`. Item 4's "symbol constructor" would have to say which of those the stored token becomes, and the launch argument is required to stay data so it does not resolve early. That choice is the ruling.

## Amend — `$bare` was not applied

The amend (`983f37362`) rules that an unqualified keyword's namespace is `$bare`: `:k` is `{$bare, k}`, prints as `:k`, a namespaced keyword prints `:{namespace}/{name}`, no `Option`, and the reader refuses `$bare/x` and `:$bare/x` the way it refuses `$bound/x`. That representation is item 3. STOP-2 fired before it. `BARE_NAMESPACE` is not in `crates/wat-reader/src/identifier.rs`. `BOUND_NAMESPACE` is still the only reserved namespace constant (`:68`). `Name::from_keyword(":k")` is still `None` (`:150`, pinned by `the_name_is_the_pair`). `is_rendered_type_text` (`:219`) still refuses `(` and `<`. The sentinel is still the `::` spelling. `flat` stays. The text bridge stays.

## What did not run

No floor. No clippy. No ignore ledger. No fuzz cost. Nothing under `src/`, `crates/`, `wat/`, or `tests/` changes in this commit.

## Amend 2 — a symbol is a value

The amend (`f75e44ca077cbd6d1b43307a08659eb100341ce7`) accepts the STOP and rules S1. `0b2dcc1e2` is an ancestor of this score. The work is three commits on local `main`, none pushed:

- `059f2af8024281cb7ef54844438c5bc9601a5e93` — `Value::Symbol` holds a `Name`. `defservice` passes names as symbols. A keyword holds a `Name`. An unqualified keyword is `{$bare, name}`. `<` is a name character. `:$bare/x` and `$bare/x` are refused.
- `9854e3c66c7e30dfc5b52a58a348dcdc127ca68f` — a declared name written `ns/name` is namespaced. Bracket extends a keyword value as one pair. Enum-field maps and shipped def names keep the source `::` spelling through `canonical-identity`. A symbol is `Equatable`. `type-equal?` reads a symbol as the type it names.
- `f8eaaa16e55fdf755b0a7b41f769c56a40b8cfa1` — the `$bare` refusal had been inserted between `#[test]` and `bound_namespace_lookalike_but_not_a_slash_boundary_still_works`, so that test left the suite. The attribute is back. Three probes that no longer build a keyword with `Arc::new` drop the unused import.

`Value::Symbol` holds the same `Name` a keyword holds. `keyword_text` (`crates/wat-reader/src/identifier.rs:325`) prints `:k` when the namespace is `$bare`, otherwise `:{namespace}/{name}`. `symbol_text` (`:338`) prints the bare name when the namespace is `$bound` or `$bare`, otherwise `{namespace}/{name}`. `Name`'s `Display` is unchanged: it still writes `$bare/k` for an unqualified keyword. The keyword printer is the user-facing form.

`from_symbol_text` (`:272`): a string that contains `::` goes through `from_keyword_body`, so the symbol and the keyword are one pair; a string with no `::` splits at the first `/`. A written `$bare` or `$bound` namespace is `None`. The Rust match in `src/intrinsic/symbol.rs` accepts one string or a namespace and a name. The checker scheme (`src/check.rs:22002`) is one `String` plus a rest `String`, result `:wat::type::symbol`. The `#[wat_intrinsic]` handler takes `&[WatAST]`, which the macro records as `Arity::Variadic` (`crates/wat-macros/src/wat_intrinsic.rs`). Callers under `wat/` pass one string. There is no `wat.core/resolve`. Resolving a symbol value is `apply`. The pre-existing `:wat::holon::Reckoner/resolve` is a different verb.

`'` is quote sugar. The lexer emits `Token::Quote` (`crates/wat-reader/src/lexer.rs:487`). The parser wraps it as `:wat::core::quote` (`crates/wat-reader/src/parser.rs:450`).

`wat.runtime/TypeInfo.name` is `wat.type/symbol` (`wat/runtime-typeinfo.wat:76`). `wat.process/Service.name` is `wat.type/symbol` (`wat/process.wat:83`). `compose-variant` returns `Value::symbol` (`src/reflect/verbs.rs:1980`). The checker still types that verb as `:wat::type::keyword` (`src/check.rs:3356`) and types `variant-parent-of` as `Option` of `:wat::type::keyword` (`:3308`) while the eval returns a symbol (`:1888`).

`wat.fix/kw-text` runs `canonical-identity` on `":"` plus `keyword/to-string`, so a string-keyed enum map keeps the source `::` spelling `ast-name` returns (`wat/fix.wat:1551`).

### Remaining `keyword/from-string` calls under `wat/`

Grep of `from-string` under `wat/` this session. Six calls. None looks the result up as a function.

| site | what it builds |
|---|---|
| `wat/telemetry/span.wat:170` | metric `:name` keyword, the duration key's printer body plus `/count`, stored on `wat.telemetry/Metric` |
| `wat/telemetry/span.wat:171` | the same, plus `/duration` |
| `wat/service.wat:1563` | the variant leaf keyword (pascal of the op) passed to `compose-variant` beside the service `Op` symbol |
| `wat/service.wat:1567` | the same leaf beside the surface `Reply` symbol |
| `wat/service.wat:2076` | the same leaf beside the protocol `Op` symbol |
| `wat/service.wat:2079` | the same leaf beside the protocol `Reply` symbol |

`compose-variant` concatenates. It does not look the leaf up. Comments that still name `keyword/from-string` in `wat/core.wat`, `wat/spawn.wat`, `wat/query.wat`, `wat/rete/compile.wat`, `wat/rete/oracle/accum-pass.wat`, and elsewhere in `wat/service.wat` are comments. `wat/bracket.wat`, `wat/fix.wat`, `wat/query.wat`, `wat/rete/compile.wat`, and `wat/rete/oracle/accum-pass.wat` build names with `wat.core/symbol`.

The startup counter is gone. `kw_value_census` and `KW_FROM_STRING` are absent from the tree. The run that read 0 was before that removal and is not re-run here. `flat` stays. The text bridge stays. The reader stone was not started. `wat/kernel/services/stdio.wat` was not edited. `name-census.tsv` was not rewritten.

### Floors

Do not re-run `.floor/2026-10-05T06-25-09Z`. `raw.log:8396`:

```
Summary [ 422.177s] 6419 tests run: 6351 passed (28 slow), 68 failed, 24 skipped
```

Those 68 were cured in `9854e3c66`. Do not re-run `.floor/2026-10-05T07-00-54Z` either. Its `clean.log` Summary is `Summary [ 418.118s] 6420 tests run: 6420 passed (29 slow), 24 skipped`, and its test-name set against `.floor/2026-10-04T07-52-21Z` was MISSING 1: `wat-reader parser::tests::bound_namespace_lookalike_but_not_a_slash_boundary_still_works`. That is the attribute `f8eaaa16e` puts back.

Acceptance floor `.floor/2026-10-05T07-14-37Z` at `f8eaaa16e55fdf755b0a7b41f769c56a40b8cfa1`. Doctests exit 0. Nextest run `5bbac7a1-474f-4dae-baf6-9ca5f7f1de3f`. The script printed:

```
[floor]     Summary [ 423.928s] 6421 tests run: 6421 passed (29 slow), 24 skipped
[floor] exit=0. Log kept at .floor/2026-10-05T07-14-37Z/ regardless — a green run is evidence too.
[floor] doc-link exit=0. Log kept at .floor/2026-10-05T07-14-37Z/doc-link.log
```

`FLOOR_RC=0`. The same Summary is `clean.log:6459`. In-load fuzz on that floor, not the gate: `PASS [  70.153s] (2315/6421) wat::kernel test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle`.

Test-name set against `.floor/2026-10-04T07-52-21Z` (`clean.log:6451` `Summary [ 422.042s] 6417 tests run: 6417 passed (28 slow), 24 skipped`), regex `^\s*(PASS|FAIL|SLOW)\s+\[[^\]]+\]\s+\(\s*\d+/\d+\)\s+(\S+)\s+(\S+)\s*$`: base 6417, this floor 6421, MISSING 0, EXTRA 4:

- `wat resolve::registration::tests::printer_form_is_the_same_namespace`
- `wat-reader parser::tests::bare_namespace_is_refused_for_the_symbol_and_the_keyword`
- `wat::program wat_arc170_program_contracts::symbol_is_not_the_keyword_and_spellings_are_one_pair`
- `wat::program wat_arc170_program_contracts::symbol_value_applies_on_the_other_side_of_a_process`

### Cost

The gate is the fuzz deftest alone, six runs, the mean of the PASS clocks. Filter `test(/deftest_wat_tests_rete_fuzz_test_native_matches_oracle/)`, `cargo nextest run --release --offline`. No limit was raised.

Before, detached `0b2dcc1e2` in `/tmp/wat-25595-before`, `/tmp/kw95-fuzz-before.log`: 31.854, 31.951, 31.978, 32.062, 32.107, 32.014. Sum 191.966. Mean 31.994333333333334 s. All six `BEFORE_RC=0`.

After, `f8eaaa16e`, `/tmp/kw95-fuzz-after.log`: 32.016, 32.005, 31.940, 32.046, 31.881, 31.829. Sum 191.717. Mean 31.952833333333334 s. All six `AFTER_RC=0`.

31.952833333333334 ≤ 31.994333333333334.

### Clippy and ignores

`cargo clippy --release --all-targets -- -D warnings` on the tree that became `f8eaaa16e` finished in 14.40s. `CLIPPY_RC=0`.

Ignores, `git grep -hcE '^\s*#\[ignore' -- 'src/*.rs' 'tests/*.rs' | paste -sd+ | bc`: 18. `IGNORE_RC=0`. The floor's 24 skipped is nextest's skip count, including 5 via `profile.default.default-filter`.

STOP-1 did not fire (DATA empty, in the section above). STOP-2 was the ruling this amend accepts. STOP-3 did not fire: a `Name` is not rendered with `::`, no case rule was added, and one name is one key. Comparing a stored `Name` to a literal goes through the one keyword constructor.

## Amend 3 — the checker says what runs

Drawn against the score above (`13db3a39a`). The parent of this work is `0b893252d` (the amend markdown). The cure is `3bbb742c5c4b31133dc878195af9e9cdf1a1ac7c`. Not pushed.

### The class

A verb this stone moved from a keyword value to a symbol, and what the checker states for it.

| verb | eval returns | checker before this amend | after |
|---|---|---|---|
| `:wat::runtime::compose-variant` | `Value::symbol` (`src/reflect/verbs.rs:1980`) | infer `src/check.rs:3356` and the TypeScheme at `:23868` both `:wat::type::keyword` | both `:wat::type::symbol`. `@ret` matches the scheme. Arguments stay keyword. |
| `:wat::runtime::variant-parent-of` | `Option` of `Value::symbol` (`src/reflect/verbs.rs:1889`) | infer `:3310` and the TypeScheme at `:23844` both `Option` of `:wat::type::keyword` | both `Option` of `:wat::type::symbol`. `@ret` matches the scheme. The argument stays keyword. |
| `wat.runtime/TypeInfo.name`, reached by `type-of` and by `declared-types` | `Value::symbol` at `src/reflect/verbs.rs:1528`, `:1547`, and `:1581` | the field is `wat.type/symbol` (`wat/runtime-typeinfo.wat:76`); the record is registered from that file | agrees. The gate's `consume-type-name` passed before the cure and after it. |
| `:wat::runtime::metadata-of`'s `:name` | `Value::symbol` (`src/runtime.rs:8068`), intrinsic branch only | no TypeScheme and no `infer_list` arm. The comment at `:8065` still says "plain keyword value" | no scheme was added. A missing scheme is not a scheme that says keyword. |
| `wat.process/Service.name` | the field type is `wat.type/symbol` (`wat/process.wat:83`) | the checker reads that defrecord | agrees. The comment at `:54` still says the name is a keyword. |
| `wat.core/symbol` | `Value::symbol` | the scheme at `src/check.rs:22006` is already `:wat::type::symbol` | not a mismatch. |

`field-names-of` (`src/check.rs:4084`) and `extract-arg-names` (`:3999`) still return vectors of keywords at the checker and at eval. They were not flipped. `src/closure_extract.rs:568` builds a symbol only as an error snapshot.

`@example` lines on the two verbs still show a keyword result (`:wat::cache::Lru.Hit`, `:wat::core::Option`). Eval already returned a symbol on the green floor before this amend, and this floor is green with those lines left as written.

### The gate, red first

A `defn` parameter annotation is stored and not re-checked at run time. `value_matches_type_by_name` (`src/function/subsume.rs:78`) runs for `defclause` dispatch (`src/function/eval.rs:209`). The consumer is a `defclause` whose parameter is the type the checker stated.

`decompose_variant` splits on `.` (`crates/wat-reader/src/identifier.rs:800`). The literal `:wat::core::Option::Some` has no `.`, so `variant-parent-of` answers `None`. The registered spelling is `:wat::core::Option.Some`.

On the tree at `0b893252d`, nextest `6644662e-0764-4a93-92da-744165fe82db`, `REDPROOF_RC=100`. `type_info_name_matches_the_stated_field_type` passed. The other two failed at the keyword clause:

```
compose-variant consumer: #wat.runtime/NoMatchingClause {:message "no clause of :user::take-keyword matched (1 args); called with (wat::type::symbol `wat.core/Option.Some`); clause 0 skipped (arg 0: expected wat.type/keyword, got :wat::type::symbol)" :location #wat.core/Span {:file "tests/reflection/probe_arc255_95_checker_says_what_runs.wat" :line 14 :col 3 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 15 :col 63}}} :causes [] :name ":user::take-keyword" :called-arity 1 :called-args [{:type "wat::type::symbol" :rendered "wat.core/Option.Some" :provenance nil}] :attempted-clauses [#wat.kernel/ClauseAttempt {:clause-index 0 :declared-arity 1 :declared-arg-types ["wat.type/keyword"] :failure-reason #wat.kernel/ArgTypeMismatch {:position 0 :expected "wat.type/keyword" :got ":wat::type::symbol"}}]}
```

```
variant-parent consumer: #wat.runtime/NoMatchingClause {:message "no clause of :user::take-keyword matched (1 args); called with (wat::type::symbol `wat.core/Option`); clause 0 skipped (arg 0: expected wat.type/keyword, got :wat::type::symbol)" :location #wat.core/Span {:file "tests/reflection/probe_arc255_95_checker_says_what_runs.wat" :line 19 :col 3 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 22 :col 35}}} :causes [] :name ":user::take-keyword" :called-arity 1 :called-args [{:type "wat::type::symbol" :rendered "wat.core/Option" :provenance nil}] :attempted-clauses [#wat.kernel/ClauseAttempt {:clause-index 0 :declared-arity 1 :declared-arg-types ["wat.type/keyword"] :failure-reason #wat.kernel/ArgTypeMismatch {:position 0 :expected "wat.type/keyword" :got ":wat::type::symbol"}}]}
```

The cure points both doors and both `@ret` strings at `wat.type/symbol`, and points the same consumers at that stated type. Isolated re-run `849e1748-8eae-45c4-9b6f-361970ea5bb9`: 4 passed (`CURE_RC=0`), including `doc_arg_ret_types_match_checker_scheme`.

### Floors

Do not re-run `.floor/2026-10-05T07-53-11Z`. `clean.log:6476`:

```
Summary [ 413.581s] 6424 tests run: 6423 passed (28 slow), 1 failed, 24 skipped
```

`FLOOR_RC=100`. One arm, `refusals_teach_the_dot_separator::variant_parent_of_example_teaches_dot`, panicked at `tests/diagnostics/refusals_teach_the_dot_separator.rs:23`:

```
check.rs:2877: remedied input tests/diagnostics/refusals_teach_the_dot_separator_s3_variant_parent_of.wat refused: #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/ReturnTypeMismatch {:message ":u::f: body produces (:wat::core::Option :- [:wat::type::symbol]); signature declares (:wat::core::Option :- [wat.type/keyword])" :location #wat.core/Span {:file "tests/diagnostics/refusals_teach_the_dot_separator_s3_variant_parent_of.wat" :line 3 :col 3 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 3 :col 45}}} :causes [] :function ":u::f" :expected "(:wat::core::Option :- [wat.type/keyword])" :got "(:wat::core::Option :- [:wat::type::symbol])" :remedies []}]}
```

That signature now declares `Option` of `wat.type/symbol`. The whole block is `.floor/2026-10-05T07-53-11Z/ARM.txt`.

Acceptance floor `.floor/2026-10-05T08-02-01Z` on the tree that became `3bbb742c5`. Doctests exit 0. Nextest run `fa5d190b-3117-433e-8a43-b4f05966798a`. `clean.log:6459`:

```
Summary [ 419.687s] 6424 tests run: 6424 passed (29 slow), 24 skipped
```

`FLOOR_RC=0`. Doc-link exit 0. `doc-link-judge.log` is 0 bytes. The baseline this amend names, read from `.floor/2026-10-05T07-34-12Z/clean.log:6455`, is `Summary [ 407.230s] 6421 tests run: 6421 passed (28 slow), 24 skipped`.

Test-name set against that baseline, same regex as the section above: base 6421, this floor 6424, MISSING 0, EXTRA 3:

- `wat::reflection probe_arc255_95_checker_says_what_runs::compose_variant_result_matches_the_stated_type`
- `wat::reflection probe_arc255_95_checker_says_what_runs::type_info_name_matches_the_stated_field_type`
- `wat::reflection probe_arc255_95_checker_says_what_runs::variant_parent_matches_the_stated_type`

### Clippy and ignores

`cargo clippy --release --all-targets -- -D warnings` finished in 12.51s. `CLIPPY_RC=0`.

Ignores, the same `git grep` as the section above: 18. `IGNORES_RC=0`. The floor's 24 skipped is nextest's skip count, including 5 via `profile.default.default-filter`.

STOP-3 did not fire. `flat` stays. The text bridge stays. The reader stone was not started. `wat/kernel/services/stdio.wat` was not edited. `name-census.tsv` was not rewritten.
