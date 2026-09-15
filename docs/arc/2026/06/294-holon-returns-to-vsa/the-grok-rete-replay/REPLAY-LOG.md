# REPLAY-LOG — grok-rete #11–#60 onto `replay/grok-rete`

Branch: `replay/grok-rete`. Source: `origin/grok-rete` `37528f6e0` (git show only). Base: `de827fb4c`.
2a4: `c184348f7`. Addendum: `b342b13a2`. Tooling: `1d722096f`. **50 REPLAY commits.**
BRIEF-4b `cec02a996`. 2a4b `d19e65885`. **Not pushed.** Main untouched.
Tag `replay-pre-fold` = old tip `7d50e6edd` (pre-fold #50–#60).

## Checkpoints

| at | floor | clippy |
|---|---|---|
| #35 `f8503d77f` | `.floor/2026-09-14T01-00-58Z` Summary [216.862s] 5463 passed, 25 skipped exit=0 | 0 |
| #60 `2528967fb` (pre-fold) | `.floor/2026-09-14T01-43-05Z` Summary [216.702s] 5490 tests run: 5485 passed, 5 failed, 22 skipped **exit=100** — STOP-4, not re-run | not run |
| #60 `6bbe1b777` + 2a4b `d19e65885` | `.floor/2026-09-14T22-03-17Z` Summary [220.761s] 5495 tests run: 5495 passed, 22 skipped exit=0 | 0 |

## Steps

| N | C → replayed | kind | notes |
|---|---|---|---|
| 11 | `eebf75374` → `9607a439e` | NEW | LATENT (c3fefc5ab) `edn::to-string` → `edn::write`. fuzzer PASS |
| 12 | `a1fbda88a` → `1c659422b` | NEW | gen-selftest + gen_lib_laws PASS |
| 13–16 | `6b02228b8`…`2946d23ff` → `5bf9b8a6e`…`ff13caed0` | mod | merge-file rc=0; UNRESOLVED Gen/at ~ctor PersistentVector (same class) |
| 17 | `9ec12c34b` → `b4c0f1dea` | mod | LATENT write kept. fuzzer PASS |
| 18 | `b13efc3c9` → `dbd82eef1` | NEW | sampling-order-probe |
| 19 | `a39c28e10` → `24679fd9a` | mod | LATENT hunk → author C. fuzzer PASS ratchet 22 |
| 20 | `03e34f0f3` → `fcf066b2c` | rename | fuzzer PASS |
| 21 | `0c7071b40` → `1294f2fc3` | mod | gen_library PASS |
| 22 | `8eeff8adc` → `a8a95400f` | stdlib NEW | **TWO-PHASE.** 0 UNREGISTERABLE ReservedPrefix. include_str `../../wat/gen.wat`. gen_library PASS [0.315s]; fuzzer PASS [4.925s] |
| 23 | `33fa967f1` → `4d9fba979` | two-phase | bind. gen_library PASS |
| 24 | `6b87ba6fd` → `b0b31f979` | two-phase | tests → wat-tests/. modify/delete git rm. 22 deftest PASS |
| 25 | `81a37f21e` → `88513ed77` | two-phase | wat-tests gen + fuzz PASS |
| 26 | `e9a5e0156` → `ea5163aa6` | docs | cherry-pick -x |
| 27 | `c43473e38` → `87482fd02` | two-phase shared | 23 gen deftest PASS |
| 28 | `a20f063a6` → `6ac4a0a61` | two-phase | 3-way src/stdlib.rs → src/load/stdlib.rs (comment path) |
| 29 | `762771096` → `0644b5bc6` | two-phase shared | test-no-negative-card PASS |
| 30–32 | `078926b13`…`23554ebd5` → `9440fcfbd`…`33312ec48` | two-phase | gen + fuzz PASS |
| 33 | `2fcd6272e` → `ff44528be` | two-phase | modify/delete sampling-order-probe git rm |
| 34 | `ef25ccffa` → `6beb8b57a` | code | wat-tests gen PASS |
| 35 | `d2a590825` → `f8503d77f` | two-phase | **checkpoint green** |
| 36–41 | `b7a3d70b2`…`26cb50517` → `2f6d2b3af`…`1ec6566bf` | two-phase | gen + fuzz PASS; #39 shared purity.rs auto-merged |
| 42 | `37f04c402` → `96acc5c57` | NEW wat-tests | touch tests/kernel/test.rs so wat::test! rescans. 5 pat deftest PASS |
| 43 | `fddedc205` → `cb6d8ffd8` | two-phase shared | docs + gen-patterns |
| 44 | `6d96ce127` → `074713fdb` | code | P6 P7 PASS |
| 45 | `6511e91a0` → `515b34456` | rs | gen_doc_surface_matches PASS |
| 46 | `04f1a0550` → `84c90f7ba` | code | family B. fuzzer divergences PASS |
| 47 | `73883d00e` → `5fecb2e41` | shared rs | auto-merged. no named tests |
| 48 | `ee1fe443b` → `3c925fa17` | docs | cherry-pick -x |
| 49 | `b2939f12b` → `ea9aedd2e` | code | families A+C. fuzzer PASS |
| 50 | `4354afc6e` → `6a5b92609` | shared + composition | validate.rs conflict as before. **Folded:** recorded migration dropped 24 dead binds in main's fmt rules. Was `3ee40f787`. |
| 51 | `aaa62272e` → `027a78011` | rs | cherry-pick of old `6d2f83b75`. auto-merged |
| 52 | `014f99452` → `6fee919cf` | code | was `25cd7eaf6`. fuzz retraction PASS |
| 53 | `d758a7ba9` → `9cfdfc58d` | shared | was `2faa14838`. accumulate max PASS |
| 54 | `2361bf8b3` → `66fa44357` | code | was `f6d73fefd`. scalar fuzzer PASS |
| 55 | `fc949d002` → `77dd78794` | code | was `ec7cf39a1`. scalar tests PASS |
| 56 | `fcfe1b291` → `118d6d48d` | docs | was `50240389c`. cherry-pick -x |
| 57 | `6241ae1d8` → `8105e22d6` | code | was `7ecc337b4`. scalar tests PASS |
| 58 | `17d010638` → `419935bcf` | shared | was `3f57263a8`. tms path-independence PASS |
| 59 | `841914cc8` → `957278f6e` | shared trap | was `1fd0f5b21`. re-expressed `fixpoint_error` |
| 60 | `d039b29e5` → `6bbe1b777` | shared trap + composition | verifier. **Folded:** `rune:lint(one-variant-separator, namespace)` above `config.rs` `leaf(head)`. Was `2528967fb`. |

convert.sh reports across the batch: era UNREADABLE (expected); UNRESOLVED `:wat::gen::Gen/at`, `~ctor`, `:wat::core::PersistentVector` on `wat/gen.wat` (same class as #14–#21 on `lib/gen.wat`). No UNREGISTERABLE ReservedPrefix after 2a4.

## STOP

**STOP-4 at pre-fold #60 closed by BRIEF-4b.** New #60 floor GREEN. Census baseline `.census/2026-09-14T22-02-27Z.txt` (2049 files, 200 failing). STOP-8 is the step gate from #61. Batch 2 is a separate brief.

---

# REPLAY-LOG — grok-rete #61–#125 onto `replay/grok-rete` (BRIEF-5)

Branch: `replay/grok-rete`. Source: `origin/grok-rete` `37528f6e0` (git show only). Base: `de827fb4c`.
Start tip: stone-3. Census baseline `.census/2026-09-15T04-07-25Z.txt` files=2054.
**Not pushed.** Main untouched. 65 REPLAY commits #61–#125.
BRIEF-5b folded the two late FIX commits into #95 and #108. Tip files unchanged (`git diff` empty).

## Checkpoints

| at | floor | clippy |
|---|---|---|
| #93 `da5bcb3f0` | `.floor/2026-09-15T05-28-34Z` Summary [225.422s] 5530 tests run: 5530 passed, 22 skipped exit=0 | 0 |
| #125 fold tip `f8060baad` | `.floor/2026-09-15T08-11-53Z` Summary [227.439s] 5540 tests run: 5540 passed, 22 skipped exit=0 | 0 |

## Recurring re-expression (wat-in-rs-strings; convert.sh does not reach them)

- RETE_OPS homes: `:wat::rete::{i64,f64,string,vector,vec,linkedlist,map,keyword}::*`
- `crate::load::loader::InMemoryLoader`
- Vector ctor `(:wat::core::Vector :- [T] v)`; List ctor `(:wat::core::List v)` (`List/of` retired)
- unit-variant KEYWORD `:probe::E.A` (not constructor); match arms `[:probe::E.A {} true]` KEY-FIRST; defenum unit `:A :B` (no `[]` — that makes tagged-empty)
- Main-only RETE_OPS row `:wat::rete::core::variant-name` re-injected after each take-theirs
- LATENT #95: `:wat::core::keyword/{to,from}-string` → `:wat::keyword::{to,from}-string` / `:wat::rete::keyword::*`
- HEAD `WatAST::CharLit` added to exhaustive rewrite_field_refs / bind_field_refs leaf arms (#98)
- datamancer.rete.edn ABI regenerated whenever RETE_OPS grows
- diagnostic goldens kept HEAD (line numbers)

## Steps

| N | C → replayed | kind | notes |
|---|---|---|---|
| 61 | `635895348` → `d4a2b1fe7` | docs | cherry-pick -x REPLAY-prefixed |
| 62–67 | `979607007`…`bbeb1997d` → `dc40ca825`…`dfb7adaf4` | code/docs | batch-2 start; #64/#66 census |
| 68 | `86091edf7` → `47e632bc8` | shared | LociDiedError map peel |
| 69–70 | `48e331135`/`5c2c06267` → `ee6479715`/`200e3b0df` | docs | reworded onto REPLAY prefix |
| 71 | `ac9782e6b` → `2f708986a` | shared | InMemoryLoader + i64 home |
| 72 | `40b3fc4bd` → `f3654ffe4` | shared | PmContainsKey on `:wat::map::contains-key?` |
| 73 | `eb5b281c4` → `db92ebf54` | docs | |
| 74 | `6fad5d237` → `588fe40f2` | code | 55-row ledger; wrap field; Vector/List ctors. census `.census/2026-09-15T04-51-54Z.txt` files=2060 |
| 75 | `ecedc8f52` → `575f081ff` | docs | |
| 76 | `d07933919` → `8cd7bebe5` | code | 74 rows; variant-name cell. census `.census/2026-09-15T04-57-43Z.txt` |
| 77 | `f8dd00573` → `ed3d2a162` | docs | |
| 78 | `db17511f6` → `f49298403` | shared | reduce=foldl, PmNew. census `.census/2026-09-15T05-02-40Z.txt` |
| 79 | `c53486fcb` → `0efa3c683` | docs | |
| 80 | `07073c091` → `c7b830679` | shared | mapv/filterv; purity.rs kept HEAD. ABI `v1:3ca327ca5d648b83` |
| 81 | `7e90f32bf` → `00cd26a4b` | docs | |
| 82 | `bf6d86640` → `d9ea53614` | code | probe-tuple-observability.wat convert `--check` 0. census files=2061 |
| 83 | `365b61884` → `1625cd5ef` | shared | Tuple accessors. goldens HEAD. ABI `v1:196687399f76baa6` |
| 84–87 | `f961a7754`…`a409d04ab` → `f960d3fac`…`9116148e6` | docs | experiri proposal |
| 88 | `97eac5a38` → `3025e2daa` | shared | reduce total arity. 2 named tests. census files=2063 |
| 89 | `851818afa` → `544ea7c3a` | docs | |
| 90 | `05a1999a5` → `33fb368c7` | shared | inline computes. 18 tests. census `.census/2026-09-15T05-20-25Z.txt` |
| 91 | `763fed823` → `592ddcc38` | shared | matcher: keep HEAD dispatch; grok inner + Some(sym) in intrinsic/rete.rs. census files=2064 |
| 92 | `a549fd9b8` → `52484f371` | docs | |
| 93 | `b7fac5ec7` → `da5bcb3f0` | shared | export Op::Eval. **checkpoint floor GREEN** `.floor/2026-09-15T05-28-34Z` 5530/5530 clippy 0 |
| 94 | `3fb4aefa3` → `6f4999462` | shared | keyword equality inline. auto-merge |
| 95 | `7cb9994cb` → `165e61097` | shared | keyword converters. LATENT homes. **Folded:** RETE_MODULES + GAP_A (was `00cc59ff4`). ABI `v1:e423b3522b35d7a7` |
| 96 | `d4fe222c2` → `428e8cf22` | shared | inline any bool expr. auto-merge |
| 97 | `5cf17bbe7` → `f237a2270` | docs | |
| 98 | `1a97cf12b` → `33d79d58a` | shared | vector field-ref. CharLit arms. census `.census/2026-09-15T05-53-37Z.txt` files=2064 |
| 99 | `4c19b9029` → `5a37d744c` | shared | computed operand typed. CharLit + diagnostic home. census `.census/2026-09-15T05-59-38Z.txt` |
| 100 | `ad2286133` → `0e8c1884c` | shared | inline cond/let/match/if. Vector match-arms. census `.census/2026-09-15T06-04-37Z.txt` |
| 101 | `b7f54a17f` → `8a7c42136` | shared | ONE rule: keyword operand is field-ref or constant. convert.sh keyword→ctor over-rewrote; unit defenum `:A :B`; `decompose_variant` door. census `.census/2026-09-15T06-26-16Z.txt` files=2065 |
| 102 | `5f650bb39` → `b95514b0f` | docs | cherry-pick -x |
| 103 | `7f21de15f` → `b7bd31978` | code | stratify third hole. census `.census/2026-09-15T06-29-28Z.txt` files=2065 |
| 104 | `197093b8d` → `17919d102` | docs | cherry-pick -x |
| 105 | `85c87314d` → `145ee1dcd` | shared | termination diagnostic. convert `--check` 0. 5 named tests. census `.census/2026-09-15T06-32-06Z.txt` files=2066 |
| 106 | `f4c618d02` → `11edc321c` | code | coincident? fifth. convert `--check` 0. census `.census/2026-09-15T06-33-26Z.txt` files=2067 |
| 107 | `ef66360d3` → `d0344f28c` | docs | cherry-pick -x |
| 108 | `3f03b7d33` → `2ad83eda0` | shared | Ret::Is/NoScheme. classify_fallback_outcome in `src/holon/outcome.rs`. goldens HEAD. **Folded:** Ret doc ` ```text ` fence (was `1c6d6af3e`). census `.census/2026-09-15T06-39-53Z.txt` files=2067 |
| 109 | `1facc1f94` → `ef87a7e18` | docs | cherry-pick -x |
| 110 | `2c4c6a163` → `ae021ec88` | shared | `#holon` literal fold. eval_quote already pub(crate). to_holon_inner via `crate::holon`. goldens HEAD. census `.census/2026-09-15T06-45-18Z.txt` files=2067 |
| 111 | `c3caee1c1` → `ecdb33c67` | docs | cherry-pick -x |
| 112 | `39534d73f` → `9853fb949` | code | 7 scratch-pad probes convert `--check` 0. census `.census/2026-09-15T06-46-56Z.txt` files=2074 |
| 113–121 | `9abad4d73`…`75eea76d5` → `5d6b641a4`…`84f15efce` | docs | cherry-pick -x |
| 122 | `31c76474a` → `d91d945d4` | shared | hash-destructure. Map arm + HEAD KEY-FIRST Vector nested-variant. convert `--check` 0. census `.census/2026-09-15T06-51-37Z.txt` files=2076 |
| 123 | `3f2cb8322` → `3e9142db2` | docs | cherry-pick -x |
| 124 | `4e2043cc2` → `616b45d1c` | code | export-roundtrip probe convert `--check` 0. census `.census/2026-09-15T06-53-02Z.txt` files=2077 |
| 125 | `bff5b2179` → `3ac756c0f` | docs | cherry-pick -x |

BRIEF-5b fold: `00cc59ff4` into #95, `1c6d6af3e` into #108. FIX commits dropped. 65 subjects unchanged.

## STOP

None. Fold proof: tip files identical to pre-fold.

---

# REPLAY-LOG — grok-rete #126–#152 onto `replay/grok-rete` (BRIEF-6)

Branch: `replay/grok-rete`. Source: `origin/grok-rete` `37528f6e0` (git show only). Base: `de827fb4c`.
Start tip: BRIEF-6 `0ae232d3d`. Census start `.census/2026-09-15T06-53-02Z.txt` files=2077.
**Not pushed.** Main untouched. 27 REPLAY commits #126–#152.
#126 folded a golden recapture (`:line 1525`→`1526`) so no REPLAY commit is knowingly red.

## Checkpoints

| at | floor | clippy |
|---|---|---|
| #139 first floor (captured, not re-run) | `.floor/2026-09-15T19-06-20Z` Summary [223.564s] 5542 tests run: 5541 passed, 1 failed, 22 skipped **exit=100** | not run |
| #139 fold tip `0de34ca3c` | `.floor/2026-09-15T19-13-39Z` Summary [226.425s] 5542 tests run: 5542 passed, 22 skipped exit=0 | 0 |
| #152 `55969600a` | `.floor/2026-09-15T19-36-41Z` Summary [228.990s] 5546 tests run: 5546 passed, 22 skipped exit=0 | 0 |

## Recurring re-expression

- #126 trap door: `src/edn_shim.rs` → `src/edn/render.rs`; `src/string_ops.rs` → `src/string/mod.rs`. `value_to_edn_with` / `value_to_json_natural` / `value_to_edn_string_with` return `Result`; named `value_to_edn_string_lossy` for infallible sites; try-send encode hoisted in `src/kernel/message.rs`. HEAD H-2 tagged-map encoding kept.
- Composition into #126: grok's new test `call -> Result<Value, String>` met main's `no_error_flattening_helper` — rewritten to return `RuntimeError`. freeze.rs lossy-door moved `rust_caller_span!` 1525→1526; golden recaptured.

## Steps

| N | C → replayed | kind | notes |
|---|---|---|---|
| 126 | `5696835f1` → `ef8ded525` | shared trap + composition | edn::write reports. **Folded:** golden `:line 1526`. flattening-helper repair. census files=2078 |
| 127 | `f30d31af8` → `ab2a0c654` | docs | cherry-pick -x REPLAY-prefixed |
| 128 | `c99202e1a` → `4c10f56c3` | code | 2 scratch-pad probes convert `--check` 0. census files=2080 |
| 129 | `59feffad3` → `d72f6d89c` | code | ci grid speed script. census files=2080 |
| 130 | `ec5df8939` → `590d50b15` | docs | |
| 131 | `142f24b05` → `20afd7776` | shared | HEAD diagnostic comment kept (chase CLOSED). census files=2080 |
| 132 | `3f4cf13f5` → `9337ddb4b` | docs | |
| 133 | `4fbdb9901` → `5d5604447` | docs | |
| 134 | `6fa13308e` → `3de67e0cd` | code | finite-domain-bool probe convert `--check` 0. census files=2081 |
| 135 | `692af57fd` → `163114662` | docs | |
| 136 | `36f9f2845` → `3cbee5a91` | docs | |
| 137 | `48010beaa` → `e63012792` | shared | config.rs auto-merged. kind(lib) 1471. census files=2081 |
| 138 | `486b46abe` → `ff8f6d575` | docs | |
| 139 | `56565a78c` → `0de34ca3c` | code | finite-typed computed head. 6 named tests. **checkpoint floor GREEN** after fold |
| 140 | `e440b1029` → `3692f6c97` | code | finite-domain cap re-derived. convert `--check` 0 |
| 141 | `5c7e8e66f` → `e47bf2ccd` | docs | |
| 142 | `1166fc877` → `76e35c655` | shared | counting allocator, unwired |
| 143 | `3c5ac7bd1` → `b0cb6ce57` | shared | per-session memory ceiling. convert `--check` 0 |
| 144 | `8c10ee490` → `ce92441dd` | code | kill global counters |
| 145 | `f8fccd69e` → `8ab3b6dbb` | docs | |
| 146 | `c0c2745e3` → `cf635a41d` | docs | |
| 147 | `12cdf4081` → `086a13131` | docs | |
| 148 | `d93d3e454` → `9eb4735b1` | docs | |
| 149 | `f006f77d5` → `298988662` | shared | ceiling covers every round. census files=2084 |
| 150 | `b5240f30b` → `674a21617` | docs | |
| 151 | `52213d3b0` → `d61a518bc` | shared trap | session is the boundary. merge-file: took converted C. 8 named tests. census files=2086 |
| 152 | `1a8ad61a0` → `55969600a` | docs | cherry-pick -x. **checkpoint floor GREEN** |

## Captured red (not re-run)

`.floor/2026-09-15T19-06-20Z` ARM: `probe_supervisor_select_lost::select_prime_yields_lost_when_process_child_crashes` at `tests/process/probe_supervisor_select_lost.rs:202` — golden `:line 1525`, actual `1526` (`src/freeze.rs` `rust_caller_span!`). Folded into #126.

## STOP

None remaining. Do not push. Main untouched.
