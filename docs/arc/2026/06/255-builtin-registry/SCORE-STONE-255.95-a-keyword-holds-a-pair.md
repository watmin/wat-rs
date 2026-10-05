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
