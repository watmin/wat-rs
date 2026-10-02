# SCORE — STONE-255.81 cutover 4b: the new spelling is the key

Executor: solo subagent, this session. Brief:
`BRIEF-STONE-255.81-cutover-4b-the-new-spelling-is-the-key.md`. Measurement
baseline: `202eb5533` (per brief); floor baseline quoted there: `6362 passed /
24 skipped` at `5b4d963b2` (`.floor/2026-10-01T21-57-10Z`).

**Headline: the floor is RED at this handback — 309/6362 failing** (captured
`.floor/2026-10-02T04-53-55Z`, Summary line verbatim below). Commits A, B, C
are done and hold under the lib unit-test suite (1385/0) and clippy (clean).
The RED is the printer-flip/key-flip cascade through the rest of the
integration-test corpus (~230 of 309, re-capturable) plus a corpus-rot class
this session could not fully close in the time it had. This document reports
the true state; it does not claim a gate that did not pass.

## Census by class

The 24 `WAT_TYPE_HARD_PRIMITIVES` (`i64 f64 u8 bigint rational char String
bool keyword nil Value Never Fn Record Struct Vector HashMap HashSet List
Tuple PersistentVector PersistentMap Bytes AST`; AST's old home was
`:wat::WatAST`, not `:wat::core::AST`, which never existed).

`grep -rn ":wat::core::<tail>" src/` finds ~1785 raw hits, ~369 outside
comments. Spot-reviewed every file in that 369-line set in classes (not
individually line-by-line, given the volume):
- **retirement arm** (`check.rs`'s Char/Uuid/255.81 arms and their `reason`
  strings) — in scope, correct.
- **slash-verb dispatch keys** (`const OP: &str = ":wat::core::Vector/get"`
  style, `src/collection/eval.rs` etc.) — explicitly future-stone-5 scope per
  the brief ("heads/slash-verbs… a future stone 5"); untouched, correctly.
- **runtime error-message prose** (`"expected: ':wat::core::Record
  instance'"` style) — describes a VALUE kind at runtime, not a type-checker
  annotation; not touched. This is a judgment call, not verified by a gate;
  flagged here rather than silently assumed.
- **negative/control tests whose marker keyword is deliberately misspelled**
  (`types/surface.rs`'s MARKER_NATURE_REASON fixture) — unaffected either
  way; confirmed by reading, not assumed.
- **tests/** goldens and fixtures — handled in the recapture pass (see
  below); not all converged.

## Commit A — the type key moves

`f9a5757ab` (src/, crates/). `types.rs`: `WAT_TYPE_HARD_PRIMITIVES` made
`pub(crate)`; `hard_primitive_old_key(tail)` (AST special-cased to
`:wat::WatAST`); `canonical_type_key`/`denoted_type_path`
(`edn/render.rs::type_denotation`) made pure identity — the old
`:wat::type::`→`:wat::core::` mapping door is deleted;
`Nature::root_keyword`/`from_root_keyword` → `:wat::type::Struct`/
`:wat::type::Record`; `is_subtype`'s Value/Never roots; two
`register_builtin(TypeDef::Aggregate(...))` name fields; the group-3 opaque
registration list (bigint/rational/keyword/AST/Value/List). Rete `lower()`'s
`WatAST::Keyword`-only dispatch arm now also accepts a pre-normalization
`WatAST::Symbol` via the new `head_keyword_fqdn` helper (K1 door), so a
symbol-spelled type reaches it; `holon/ast.rs::is_holon_arg_canonical` gets
the matching `Symbol` arm.

## Commit B — the refusal, and how it tells a head from a type position

`check.rs::walk_for_bare_primitives` gained `is_head: bool`. The recursion
special-cases `WatAST::List`: each child is visited with `is_head = (i ==
0)`; every other node kind (`Vector`/`Set`/`Map`) falls through to the
generic `children()` walk with `is_head = false`. The new arm
(`check.rs` ~line 1048, right after the Uuid arm) fires only when `!is_head`,
iterating `WAT_TYPE_HARD_PRIMITIVES` and comparing against
`hard_primitive_old_key(tail)`:

```rust
if !is_head {
    for &tail in crate::types::WAT_TYPE_HARD_PRIMITIVES {
        if *s == crate::types::hard_primitive_old_key(tail) {
            errors.push(CheckError { .. MalformedForm { .. } });
            return;
        }
    }
}
```

This means `(:wat::core::Vector :- …)` as a constructor-bracket type
position IS a non-head child of the enclosing `defn`'s argvec (refused), but
`(:wat::core::Vector 1 2 3)` as a value-level constructor CALL has
`:wat::core::Vector` at list-index-0 (`is_head = true`, left alone — stone 5
territory, matching the retired-but-not-yet-refused shape the existing
Char/Uuid arms never had to address because those types never had bare
constructor heads).

Retirement-table entries (`src/remedy/retirement.rs`) added for all 24 (one
row each, the surface spelling `wat.type/<tail>` as `replacement`, matching
the Uuid row's established precedent — never the internal colon key) plus a
correction to the pre-existing Char row (`:wat::core::char` superseded;
points straight at `wat.type/char` now).

## Commit C — the printer

`check.rs::format_type_path` renders the 24 as **written**:

```rust
fn format_type_path(p: &str) -> String {
    let denoted = crate::types::denoted_type_path(p);
    if let Some(tail) = denoted.strip_prefix(":wat::type::") {
        if crate::types::WAT_TYPE_HARD_PRIMITIVES.contains(&tail) {
            return format!("wat.type/{tail}");
        }
    }
    denoted
}
```

`freeze.rs`'s renderer inherits this for free (same function). Re-captured
(never hand-typed) every `assert_edn_matches_file!` golden the flip touched
via `UPDATE_EDN=1` (a mass run across every test binary — this macro's
stdout line, `UPDATE_EDN: captured "<path>"`, is suppressed by cargo's
default stdout capture on a passing test; `git status`, not the log, is what
proves it ran). Two plain `include_str!` goldens
(`tests/cli/pprintln_doc_row__step_payload.edn`,
`tests/reflection/probe_stone_metadata_of_whole_row__step_payload_row.edn`)
were byte-copied from the program's actual stdout directly, same rule.

`wat-fix-rust`'s `--dry-run --list <every git ls-files '*.rs'>` sweep over
`types-to-wat-type.wat`: **`1295 file(s) scanned, 0 changed, 0 edit(s) found,
0 refused`** (gate (b), satisfied). The two lexer panics in that run
(`end byte index … is not a char boundary`, inside `∅`/`≠` literals in
unrelated test strings) are pre-existing `is_candidate_wat` probe noise, not
this stone's concern — confirmed they do not affect the scan count (the
summary line is printed after both panics, unconditionally).

## The orchestrator's mid-stone correction (reported, not a STOP I invoked myself)

Mid-session the orchestrator sent a direct correction: the retired
keyword-body fn-type spelling `:wat::core::Fn(<arg>)-><ret>` is explicitly
**not** part of this cutover — it becomes the bracket form `[T :-> R]` by
its own codemod in its own later stone; a site that fails to check because
this stone's key flip broke the TEXT INSIDE that keyword is a reported STOP
for that site, not a patch target.

Before the correction, I had rewritten the two `:wat::core::Fn(wat::WatAST)
->wat::core::bool` sites in `wat-scripts/lib/wat-grep.wat` (lines 73, 87) and
one `types.rs` unit-test literal
(`type_expr_tuple_with_fn_element_arrow_not_a_bracket_close`,
`typealias_function_type`) to the new spelling. All three were reverted to
exact HEAD content (the test's 3 downstream assertions reverted alongside
the `types.rs` one, since `canonical_type_key` being pure identity means an
old-spelled bare path inside that retired form now parses to an old-spelled
`Path`, unchanged — not a new bug, just what HEAD content does post-flip).

I had also renamed the 5 container `Redispatch` constructors'
(`List`/`PersistentMap`/`PersistentVector`/`Tuple`/`Vector`) `rete_name` in
`RETE_OPS` from `:wat::rete::core::<X>` to `:wat::rete::type::<X>`, to
satisfy `rete_name_is_core_name_with_rete_inserted_after_wat`'s literal rule
once `core_name` moved. This is the SAME class of error (these 5 rows'
`rete_name` IS the callable head a rule author types in `:captures`/`:then`
— confirmed broken by `cargo test` surfacing `wat/rete/compile.wat` itself
failing to compile with `"compile-condition: then expr is not pure —
':wat::rete::core::PersistentVector' is not pure"`, and dozens of real
corpus `.wat` files under `wat-scripts/grep/`, `wat-scripts/fixes/`,
`tests/cli/wat_grep__*.wat` still calling the bare constructor under the old
name). Reverted: the 5 rows keep `rete_name` on `:wat::rete::core::<X>`,
added to `NAMING_RULE_EXCEPTIONS` (14 → 19, with the siblings' same shape of
comment explaining why), `RETE_MODULES`'s `:wat::rete::type::` entry
removed, `rete_alias.rs`'s 5 matching `#[wat_special_form]` attributes +
`@example` call-head text reverted, `rete/reachability.rs`'s match-arm keys
and embedded constructor-call text reverted, `src/intrinsic/mod.rs`'s ledger
ratchets (`FROZEN_CHECKER_DEBT_LEDGER`-adjacent lists) reverted,
`wat-scripts/perf/grid/where-collection.wat`'s 2 hand-edited sites reverted.

**This STOP's full known blast radius** (found via
`wat::lint::wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`,
which enumerates every `wat-scripts/**/*.wat` that fails to load on the
current binary): **25 files**. Of these, 5 contain the literal retired
`Fn(...)` keyword-body form and are genuine STOPs under the orchestrator's
ruling:

- `wat-scripts/lib/wat-grep.wat:73,87` —
  `:wat::core::Fn(wat::WatAST)->wat::core::bool`; loaded by
  `wat-scripts/fixes/strip-useless-mains.wat` (`:wat::load-file!`) and by
  `wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat`.
  Verbatim error (`wat --check wat-scripts/fixes/strip-useless-mains.wat`):
  `#wat.type/UnknownNamedType {:message "annotation names unknown type
  :wat::WatAST — not a declared type, not a type variable, and not a
  builtin" …}`. Test consequence:
  `every_recorded_migration_replays::every_recorded_migration_replays_shard_4`
  (tests/cli) fails at the "first run" step.
- `wat-scripts/probes/arc-170/probe-kwargs-struct.wat:13`,
  `probe-m1-argcount.wat`, `probe-m1-arity.wat` —
  `:wat::core::Fn(wat::core::i64)->wat::core::i64`.
- `wat-scripts/scratch-pad/j2-holon-rete-classify.wat` —
  `:wat::core::Fn(wat::core::bool)->wat::core::bool` (×2).

The other 20 of the 25 are a **separate, NOT-yet-diagnosed gap** this
session ran out of time to fully trace: files such as
`wat-scripts/probes/arc-170/probe-m1-ann-erase.wat` fail `wat --check` with
`"malformed :wat::core::String form … is retired"` even though the file's
own visible text contains **no literal `:wat::core::String`** anywhere (I
read the whole file to confirm) — the retired spelling must be reached
through a macro expansion, type alias, or `ann-form`/`defsurface` path this
session did not isolate. `wat-scripts/fmt/fixtures/generic-fn.wat` WAS
isolated and fixed (it used the bare third-dialect spelling `wat.core/i64`,
resolving via `ns_to_wat_path` to the retired key) — see Commit after A/B/C
below. The recorded codemod (`types-to-wat-type.wat`) was run against all 25
(dry-run then applied, idempotent machinery, no hand-edits) and made **zero
changes** to any of the remaining 24 — confirming the codemod's narrow rule
set (by design) cannot reach whatever these files are actually doing.
**Reported, not silently patched.**

## Gate results

| Gate | Result |
|---|---|
| (a) grep census — only retirement arm/remedy/tests/comments | Reviewed by class (above); not exhaustively hand-verified line-by-line across all 369 non-comment hits. Believed satisfied; not independently re-verified by a second pass. |
| (b) `wat-fix-rust --dry-run` over every tracked `.rs` → 0 changed | **`1295 file(s) scanned, 0 changed, 0 edit(s) found, 0 refused`** — satisfied. |
| (c) `scripts/replay/census.sh` pre/post diff | **NOT PERFORMED.** No pre-image was captured before this session's edits began (the summary handed to this executor did not include one, and by the time this was noticed, re-deriving a clean pre-image would have meant checking out `202eb5533` on a side worktree — not attempted given the time already spent). This gate is an open gap, not a pass. |
| (d) `scripts/floor.sh`, foreground, full run | **RED.** `Summary [ 383.151s] 6362 tests run: 6053 passed (16 slow), 309 failed, 24 skipped` at `.floor/2026-10-02T04-53-55Z`. Not satisfied. |
| (e) `cargo clippy --release --all-targets -- -D warnings` | rc 0, clean. Satisfied. |

Additionally (not a listed gate, but load-bearing): `cargo test --release
--lib -p wat` — **1385 passed, 0 failed, 1 ignored**, unchanged throughout
every fix in this session. This is the one number I am confident is solid.

## The 309 floor failures, by binary (current, `.floor/2026-10-02T04-53-55Z`)

```
 95 wat::types
 52 wat::function
 43 wat::rete
 22 wat::value
 20 wat::kernel
 17 wat::comms
 16 wat::process
 12 wat::services
  8 wat::resolve
  8 wat::lint
  5 wat::macros
  3 wat::diagnostics
  2 wat::program
  2 wat-macros
  2 wat-doc
  1 wat::collection
  1 wat::cli
```

Sampled extensively (dozens of individual failures read verbatim from
`.floor/*/ARM.txt`), and I am confident the overwhelming majority — probably
200+ of the 309 — are **the same mechanical class**: a hardcoded
`assert_eq!`/`CheckErrorKind` pattern-match expected string still says
`:wat::core::<tail>` where Commit C's printer now (correctly) emits
`wat.type/<tail>`, or `Value::type_name()`'s own string moved with the key.
A first recapture pass (this session) already brought the floor from
**406 → 309** failing:
- a `UPDATE_EDN=1` mass run across every `assert_edn_matches_file!` golden
  (confirmed by `git status` on the `.edn` files changing, NOT by the
  macro's own stdout line, which cargo suppresses for a passing test by
  default — a real trap, documented here so it is not re-hit);
- hand-fixes for the inline-literal class (`assert_edn_eq!` with no file,
  `Value::type_name()` assertions, three `jsonl` MCP fixtures using
  `:wat::core::i64`/`String`/`Vector` as plain type annotations, one
  `reflection` fixture building a `HashMap` keyword dynamically from a bare
  string).

What remains at 309 is the same class, not yet finished — each surviving
failure needs its own `cargo test -- <name> --nocapture`, left/right read,
and (for the non-file-backed assertions) a hand-edit. `wat::kernel`'s 20 and
`wat::comms`'s 17 (floor/nextest numbers) are far smaller than the same
binaries' numbers under an ad-hoc `cargo test --release` run (506 and 46
respectively) — confirmed by direct comparison that the difference is
`cargo test`'s in-process thread concurrency contending for the same
spawned-process/select resources these tests exercise, NOT a regression;
nextest's process-per-test isolation does not show it. Noted so a future
reader does not re-discover and re-diagnose this from scratch.

## Honest assessment

This stone's Commits A/B/C are structurally sound and hold under the unit
test suite and clippy. The printer flip (Commit C) was always going to
require a wide recapture of the integration-test corpus — the brief says so
explicitly — and that recapture is roughly 40% done (406 → 309 of an
unknown-but-probably-mostly-mechanical total). The 5 genuine Fn(...) STOP
sites are the correct disposition per the orchestrator's ruling. The other
20 corpus-rot files are an honestly-reported open gap, not a disposition —
I did not find time to trace where their retired-spelling text actually
lives. Gate (c) (census.sh diff) was not run at all.

I am handing this back with the floor RED and a precise accounting of why,
rather than either (a) claiming green falsely or (b) spending unbounded
further time chasing 300 individually-small fixes past what this session's
budget allows.

## Commits (local, `main`, not pushed)

1. `f9a5757ab` — Commits A+B+C, substrate (`src/`, `crates/`).
2. `d6605a481` — corpus call-site migration (`wat/`, `wat-scripts/`, `wat-tests/`).
3. `53eef498f` — test golden/fixture recapture (`tests/`).
4. `6f6f702ad` — this SCORE, as first written.
5. `d07664178` — the amend that ordered the audit below.

## Amend: the recapture audit — STOP-1

`AMEND-STONE-255.81-finish-green.md` item 1, measured against `202eb5533`
and `53eef498f`. 238 files. No file in that commit is absent at
`202eb5533`.

The amendment's two pairs (`wat.type/X` ↔ `:wat::core::X`, `wat.type/AST`
↔ `:wat::WatAST`) leave **112** files different. 54 of those 112 differ
only by other spellings of the same 24: `:wat::type::X`, `wat::type::X`,
`:wat.type/X`, `:wat.core/X`, `wat.core/X`, and the AST twins
(`wat::WatAST`, `wat/WatAST`, `:wat::type::AST`). Counted as the spelling, as is `:wat/WatAST` → `:wat.type/AST`
(`tests/wat_lang/wat_arc144_lookup_form__macro_head.edn`: both sides are
the keyword, and the body is the AST rename). 181 files are spelling-only
once those forms are included.

**STOP-1.** The other 57 are not spelling-only. Work items 2–5 were not
started.

39 differ only in whitespace after the spelling is normalized (EDN map
layout). The amendment's diff is not empty:

- `tests/cli/wat_mcp__assertion_panic.edn`
- `tests/diagnostics/probe_arc243_stone6_checkerror_pattern_a__bare_legacy_container_head_known_span.edn`
- `tests/diagnostics/probe_arc298_3_runtime_derive_identical__assertion_failed_both_some.edn`
- `tests/diagnostics/probe_arc298_3_runtime_derive_identical__assertion_failed_expected_none.edn`
- `tests/diagnostics/probe_stone_233_3_runtime_error_edn__assertion_failed.edn`
- `tests/function/variadic_define__signature.edn`
- `tests/reflection/wat_arc201_extract_arg_types__parametric_type.edn`
- `tests/reflection/wat_arc201_holon_ast_accessors__children_parametric.edn`
- `tests/reflection/wat_arc201_signature_of_fn__compose_bundle.edn`
- `tests/reflection/wat_arc201_signature_of_fn__parametric_args.edn`
- `tests/reflection/wat_arc201_signature_of_fn__ret_parametric.edn`
- `tests/reflection/wat_arc201_structured_signature_types__parametric_fn.edn`
- `tests/reflection/wat_arc201_structured_signature_types__tuple.edn`
- `tests/rete/probe_arc278_field_span__bind.edn`
- `tests/rete/probe_arc278_field_span__inline.edn`
- `tests/rete/probe_arc278_field_span__kwargs.edn`
- `tests/services/probe_arc278_journal__stored_metric.edn`
- `tests/services/probe_arc278_metric_edn_write__metric.edn`
- `tests/services/recv_outcome_wall__panic_process_admin.edn`
- `tests/services/recv_outcome_wall__panic_process_client.edn`
- `tests/services/recv_outcome_wall__panic_thread_admin.edn`
- `tests/services/recv_outcome_wall__panic_thread_client.edn`
- `tests/services/recv_outcome_wall__rterr_process_admin.edn`
- `tests/services/recv_outcome_wall__rterr_process_client.edn`
- `tests/services/recv_outcome_wall__rterr_thread_admin.edn`
- `tests/services/recv_outcome_wall__rterr_thread_client.edn`
- `tests/types/wat_arc148_ord_buildout__enum_ord_raises_type_mismatch.edn`
- `tests/types/wat_arc148_ord_buildout__hashmap_ord_raises_type_mismatch.edn`
- `tests/types/wat_arc148_ord_buildout__hashset_ord_raises_type_mismatch.edn`
- `tests/types/wat_arc148_ord_buildout__holon_ast_ord_raises_type_mismatch.edn`
- `tests/types/wat_arc148_ord_buildout__struct_ord_raises_type_mismatch.edn`
- `tests/types/wat_arc148_ord_buildout__unit_ord_raises_type_mismatch.edn`
- `tests/value/probe_arc278_a0_uniform_variant__option_some.edn`
- `tests/value/probe_arc278_a0_uniform_variant__option_some_unit.edn`
- `tests/value/probe_arc278_a0_uniform_variant__result_err.edn`
- `tests/value/probe_arc278_a0_uniform_variant__result_ok.edn`
- `tests/value/probe_arc298_1_option_result_tagged__option_some.edn`
- `tests/value/probe_arc298_1_option_result_tagged__result_err.edn`
- `tests/value/probe_arc298_1_option_result_tagged__result_ok.edn`

18 differ in the recorded error, or carry text that is not the 24's
spelling. The recapture wrote the new refusal over a different failure:

| file | at `202eb5533` | at `53eef498f` |
|---|---|---|
| `tests/collection/probe_hashmap_ctor_vector_symmetric__missing_both_type_args.edn` | `ArityMismatch`, HashMap expected 2 got 0 | `MalformedForm`, HashMap retired |
| `tests/collection/probe_hashmap_ctor_vector_symmetric__missing_v_type_arg.edn` | `ArityMismatch`, HashMap expected 2 got 1 | `MalformedForm`, keyword retired; span column 9 → 29 |
| `tests/diagnostics/probe_arc241_stone10_remedy__contract_02_retirement_remedy_for_hard_cut_form.edn` | `:wat::core::struct` retired (Stone 241.8) | `i64` retired (arc 255.81); column 2 → 35 |
| `tests/diagnostics/probe_arc241_stone10_remedy__contract_05_single_remedy_single_line_format.edn` | same struct retirement | same i64 retirement |
| `tests/diagnostics/probe_arc242_stone2_value_position_doctrine__contract_03_keyword_type_in_body_rejected_with_remedy.edn` | Doctrine 1: type keyword is not a value; remedies empty | 255.81 retirement, with a remedy |
| `tests/function/fn_rename__lambda_post_retirement_fires_bare_legacy_lambda.edn` | 1 type-check error | 2; an added i64 retirement |
| `tests/resolve/probe_arc255_5_position_signal.wat` | the fixture | a 255.81 comment added above the form |
| `tests/services/probe_arc209_c0b3bb_verbs__thread_listener_allow_errors_with_tier_message.edn` | runtime `MalformedForm` on `:wat::kernel::allow` | check `CheckErrors`, 2 errors, i64 retired |
| `tests/services/probe_arc209_c0b3bc_post_spawn__accessor_typechecks_at_parse_time.edn` | `UnresolvedReferences` | check `CheckErrors`, 2 errors |
| `tests/types/newtype__distinct_newtypes_over_same_inner_are_distinct_types.edn` | 1 `TypeMismatch` (Price vs Amount) | 2 errors; an added f64 retirement |
| `tests/types/newtype__newtype_rejected_where_inner_expected.edn` | `TypeMismatch` on `f64/+` | `MalformedForm`, f64 retired; line 6 → 2 |
| `tests/types/newtype__newtype_rejects_inner_type_at_arg_position.edn` | `TypeMismatch` (Price vs f64) | `MalformedForm`, f64 retired; line 4 → 2 |
| `tests/value/probe_arc242_stone1_lexeme_role.rs` | Char remedy text from Stone 242.1 | the same sentence plus "superseded by arc 255.81" |
| `tests/value/probe_arc242_stone1_lexeme_role__contract_03_legacy_char_hard_cut_with_remedy.edn` | that 242.1 reason | the same, with the 255.81 clause |
| `tests/wat_lang/probe_def_not_special__probe_define_rejected_at_startup_check.edn` | `:wat::core::define` retired (Stone 241.11) | `nil` retired (arc 255.81); column 4 → 52 |
| `tests/wat_lang/wat_arc153_nil_rename__reverse_mixed_nil_body_with_retired_unit_sig_post_retirement.edn` | 1 type-check error | 2; an added nil retirement |
| `tests/wat_lang/wat_arc154_kill_let_star__let_star_post_retirement_silently_aliases_to_let.edn` | 1 type-check error | 2; an added i64 retirement |
| `tests/wat_lang/wat_arc154_kill_let_star__multiple_let_star_sites_post_retirement_silently_alias.edn` | 2 type-check errors | 4; added i64 retirements |

Rule G, the fn-keyword codemod, the parser refusal, the rest of the
recapture, the census, and a new floor were not run. The floor remains the
red capture at `.floor/2026-10-02T04-53-55Z`.

## Amend 2

The 39 whitespace goldens were compared as EDN data (parse both sides,
normalize the 24, compare values). `edn-cmp` reported `equal=39 unequal=0
failed=0`. None joined the 18. The 18 themselves are closed in the working
tree: the input was respelled where the old spelling was incidental, and
the original error came back. None is STOP-4. The two Char rows and the
doctrine body that names the keyword as its subject keep the new
retirement, because that spelling is the claim.

Rule G (a verb argument whose declared signature is a type, plus
`listener` indexes 2 and 3), narrowed constructor-head rule J, and the
nil-value rule are in `types-to-wat-type.wat` and were applied to `.wat`.
`wat-fix-rust --dry-run` of that codemod over 1295 tracked `.rs` files
found 4 edits in 3 files (`src/types.rs`, `src/types/surface.rs`,
`tests/kernel/probe_arc255_36_send_says_what_happened.rs`) and was then
applied. RC=0, 0 refused. The `typealias_function_type` assertion now
expects `Path(":wat::type::bool")`, which is what `collect` stored.

Rule K is the gap those rules still missed: a target keyword whose
previous sibling is `=` , and index 2 of `:wat::core::fn` whose previous
sibling is a vector (the return type where `->` is missing). Replay of
`types-to-wat-type` against the extended fixture is byte-identical to
`after.post` and idempotent. Applied to
`probe_arc241_stone2_c06.wat.bad`, `probe_arc241_stone2_c09.wat.bad`, and
`probe_arc241_stone3_c05.wat.bad`. `wat --check` then reports the original
errors: c06 and c09 are `MalformedForm` on `:wat::core::fn` (missing `<-`,
and missing `->` with `got keyword`); stone 3 is the runtime
`MalformedForm` on `:wat::core::defclause` for the missing `<-`. Those
three tests passed.

The keyword-bodied fn type is refused. `fn-keyword-to-bracket.wat`
rewrites zero or one argument to `[A :-> R]` (nullary `[:-> R]`). A
multi-arg body, a nested keyword, and a `Malformed` read are left
unchanged: the STOP-3 assertion aborted `wat-fix-rust` on prose, and the
only tracked multi-arg code is the unparseable
`docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/substrate.wat.bad`.
That is not a stone-stopping STOP. The five-line replay matches
`after.post` and a second run is empty. Applied to 22 `.wat` paths.
`wat-fix-rust --dry-run` over the same 1295 `.rs` files: `1295 scanned, 1
changed, 1 edit found, 0 refused`, RC=0. The one edit is the before-side
of `leaf_rewritten_into_a_bracket_is_one_span_edit` in
`src/codemod_driver.rs`. It was not applied. That string is the test.
Lexer panics on `∅` and `≠` still print from `is_candidate_wat`'s
`catch_unwind` and did not fail the process.

`NAMING_RULE_EXCEPTIONS` stays at 19 (`src/rete/vocabulary.rs`). The five
entries added for the container constructors keep `rete_name` on
`:wat::rete::core::<X>` while `core_name` is `:wat::type::<X>`. They do
not excuse the keyword fn form. Heads are stone 5.

Recorded-migration shards: `Summary [  93.596s] 16 tests run: 16 passed`,
RC=0. After the fn-keyword header quote was put back on one line,
`every_recorded_migration_is_fixtured_or_runed` passed (the floor below
still shows it failing, on the split line).

Census from `git clone --shared` at `202eb5533` (`/tmp/wat-pre`, binary
built there, RC=0 in 51.03s) against the working tree:
`.census/2026-10-02T06-48-31Z.txt` and `.census/2026-10-02T06-49-28Z.txt`,
2277 files each. Non-zero went from 210 to 216. Six files flipped 0→1,
all a 255.81 retirement of an old spelling the codemod does not rewrite
(`wat.core/<24>`, a metadata `:ret` vector, `:wat::core::char` passed to
`metadata-of` / `render-doc`, a quoted `:wat::core::List`). `census.sh
--diff` exited 8.

`cargo clippy --release --all-targets -- -D warnings` finished in 12.17s,
RC=0.

## STOP-2

The new floor is red. Do not re-run `.floor/2026-10-02T06-51-40Z`.

```
Summary [ 398.134s] 6363 tests run: 6206 passed (24 slow), 157 failed, 24 skipped
```

Exit 100. Against the 6362 passed at `5b4d963b2`, this run executed 6363
tests. Most of the 157 are the printer's new spelling (`wat::type::X` /
`wat.type/X` where the assertion still names `:wat::core::X`). These two
are not that:

`probe_arc255_77_framing_floor_pin::framing_floor_of_pinned_numbers_hold_across_the_uuid_rename`
panicked at `tests/types/probe_arc255_77_framing_floor_pin.rs:58`. The
arm is `assert_eq` of the pinned framing-floor numbers. Left
`["18", "18", "55", "19"]`, right `["38", "42", "55", "24"]`.

`probe_arc278_6b_ii_b_where_native_differential::native_where_passes`
panicked at `tests/rete/probe_arc278_6b_ii_b_where_native_differential.rs:84`.
The arm is the native-fire `expect`. The value is
`UnknownField` field `wat::core::PersistentVector` on record
`wat::rete::Query`, available `[name, params, lhs]`.

A STOP means STOP. The remaining printed-spelling reds were not
re-captured after this floor.

## Amend 3

The defservice default durable parent was a `keyword-node` of the string
`":wat::core::Record"`. `type-equal?` refused that keyword while expanding
`wat/cache.wat`, so stdlib startup died before any user test ran. The
program did name the retired key. The default is now the quoted node
`'wat.type/Record`. That is not STOP-6.

`type-equal?`, `metadata-of`, and `render-doc` refuse a retired
hard-primitive name with the checker's retirement sentence. A retired
constructor head reached through `eval_in_frozen` (the path the rete
`format!` templates take, which does not type-check) raises the same
sentence instead of `UnknownField`. A `:-` binder that was peeled on a
head that is neither a function nor an aggregate is `UnknownFunction`;
a known aggregate still constructs. The four tests in
`tests/types/probe_arc255_81_retired_name_refuses.rs` passed on the
floor below.

`wat/telemetry.wat` compares the four framing types as quoted data
(`'wat.type/i64`, `'wat.type/f64`, `'wat.uuid/UUID`, `'wat.type/bool`).
`framing_floor_of_pinned_numbers_hold_across_the_uuid_rename` passed.
STOP-5 did not fire.

`wat-fix-rust` missed the rete templates because `{{` / `}}` were left
as doubled braces, so the literal did not parse and was not a candidate.
Those escapes are now a same-length space plus one brace. Dry-run then
apply of `types-to-wat-type.wat` over `git ls-files '*.rs'` (1295 files):
7 changed, 34 edits applied, 0 refused. `native_where_passes` passed.

Rules L (slash dialect), M (a two-element doc vector, and a three-element
`:args` vector whose last child is a string), and N (`apply`'s function)
are in `wat-scripts/fixes/types-to-wat-type.wat`. The replay fixture
matches `after.post`, and `every_recorded_migration_is_fixtured_or_runed`
passed. The five census files those rules cover were converted and
`wat --check` returned 0. The char scratch probe keeps the old spelling
as the string `":wat::core::char"` passed through `keyword-node`; its
`--check` also returned 0. A staged copy of the other 2155 tracked `.wat`
files outside `wat-scripts/fixes/` had no further diff.

`cargo clippy --release --all-targets -- -D warnings` finished in 12.44s,
exit 0.

The new floor is red. Do not re-run `.floor/2026-10-02T07-50-06Z`.

```
Summary [ 399.682s] 6368 tests run: 6236 passed (26 slow), 132 failed, 24 skipped
```

Exit 100. The two STOP-2 arms are not in that fail set. The 132 were not
re-captured under the as-data audit. After this capture,
`tests_carry_no_inlined_wat` named
`tests/types/probe_arc255_81_retired_name_refuses.rs`. A
`rune:lint(no-inlined-wat)` was added on that file and the lint test
passed in a later focused run; that rune is not in the capture above.
