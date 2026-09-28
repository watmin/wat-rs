# SCORE — STONE 255.67: cutover 2 of 7 — the corpus spells its hard primitives `wat.type/…`

Covers both agents' work: the first agent's codemod + partial stdlib conversion + three
compare-time fixes (RED); this agent's K1 (canonicalize-at-parse, GREEN) and the rest-of-corpus
conversion (RED again, STOPPED — a different bug class than K1's).

Commits, in order:
- `029ae9f2c`, `0ace00868`, `a0efa4ae3` — the recorded codemod `wat-scripts/fixes/types-to-wat-type.wat`,
  its replay fixture, the `wat.type/AST` rendering fix, the doc-directive grammar widening (first agent).
- `db5600f4c` — the draw. `4d5f238a5` — the ruling this stone starts from.
- `c9bfee0cf` — the AMEND doc itself (K1, builder-ruled).
- `a3abf38b5` — **this agent**: landed the first agent's uncommitted converted stdlib
  (`wat/**/*.wat`, 63 of 65 files) and three compare-time fixes, at a RED floor.
- `9d13e81a7` — **this agent**: K1. GREEN.
- `fd04778e4` — **this agent**: converted the rest of the corpus (2346 of 2513 remaining files).
  RED again — a different, new bug class. STOPPED per doctrine, not re-run.

## The codemod and its rules

`wat-scripts/fixes/types-to-wat-type.wat`, on the `wat/fix.wat` framework. Converts a keyword
`:wat::core::<one of the 24 hard primitives>` (or `:wat::WatAST`) to the symbol
`wat.type/<name>` (`wat.type/AST` for `WatAST`) **only in a type position**, via five local,
non-recursive, left-to-right position rules:

- **(A) HEAD-OF-BRACKET** — immediately followed by `:-`: `(Vector :- […])`.
- **(B) ARGS-VECTOR MEMBER** — inside a sequence itself immediately preceded by `:-`.
- **(C) POST-MARKER** — immediately follows `<- -> :-> :<`.
- **(D) BARE CHILD of `extend-type` / `derive`** — every non-head child of these two form heads.
- **(E) LAST CHILD of `typealias`** — the body, never the name being declared.

The 24: `i64 f64 u8 bigint rational char String bool keyword nil Value Never Fn Record Struct
Vector HashMap HashSet List Tuple PersistentVector PersistentMap Bytes AST`.

## The five bugs, one class

The red floor this agent inherited:

```
FLOOR RED — 2026-09-28T05-09-49Z
exit=100
     Summary [ 381.242s] 6213 tests run: 6110 passed (23 slow), 103 failed, 24 skipped
```

~170 `insert-all` container-admission failures, 38 `extend-type` method-resolution ("does not
implement surface method seq") failures, and a handful of `:T` rows. **Every one is the same
class**: a key or comparison built from one spelling of a type (`:wat::type::X` /
`wat.type/X`) checked against the other (`:wat::core::X`), because under the temporary door a
parsed type could carry either spelling and only SOME comparisons remembered to denote.

One arm of the `insert-all` class, verbatim (`.floor/2026-09-28T05-09-49Z/ARM.txt`):
```
        FAIL [   7.274s] ( 258/6213) wat::lint rete_compile_gate::every_wat_scripts_rete_rule_compiles_shard_05
  stdout ───

    running 1 test
    test rete_compile_gate::every_wat_scripts_rete_rule_compiles_shard_05 ... FAILED

    failures:

    failures:
        rete_compile_gate::every_wat_scripts_rete_rule_compiles_shard_05

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 363 filtered out; finished in 7.26s

  stderr ───

    thread 'rete_compile_gate::every_wat_scripts_rete_rule_compiles_shard_05' (2074227) panicked at /home/john/work/holon/wat-rs/tests/lint/rete_compile_gate.rs:299:1:


    🔥 1 wat-scripts/ file(s) in shard 5/16 DECLARE a rete rule that does not COMPILE. `every_wat_scripts_file_loads_on_the_current_runtime` only proves these LOAD (parse + type-check); the four-axis fence (pure ∧ det ∧ total ∧ rete-primitive, `wat/rete/compile.wat:463`) lives at rete COMPILE, one step further, and this gate is the one that reaches it.

    THE FIX: either the file is a genuine mistake (rewrite the fence to a total,pure, deterministic, rete-primitive expression — inline a registered `:wat::rete::` op rather than a bare user-fn call, mirroring`wat-scripts/fixes/to-faithful-clojure-net.wat`'s own repair), or delete it.
    ⛔ THIS GATE HAS NO EXEMPTION CATEGORY, BY DESIGN(`docs/arc/2026/06/278-rules-engine/strike-no-rule-that-cannot-compile/DESIGN.md`).There is no rune that waves a non-compiling rule through. If a genuinenegative needs to live in this corpus, minting the first exemption categoryis a deliberate act with its own argument — it does not belong in a fix forone red file.

      wat-scripts/perf/grid/where-or-inline.wat
          startup: #wat.check/CheckErrors {:message "6 type-check errors" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::rete::insert-all: parameter #2 expects (:wat::core::PersistentVector :- [:wat::core::Record]); got (:wat::core::PersistentVector :- [:woi::Reading])" ... :callee ":wat::rete::insert-all" :param "#2" :expected "(:wat::core::PersistentVector :- [:wat::core::Record])" :got "(:wat::core::PersistentVector :- [:woi::Reading])" :remedies []} ...]}

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
(the record type `wat.type/Record` — un-denoted, per-file, as the container root — no longer
matched the OLD `:wat::core::Record` key `insert-all`'s parameter type was checked against; K1
fixed this class by canonicalizing at parse instead of at this comparison.)

## K1 — canonicalize at parse

Builder ruling (`AMEND-STONE-255.67-K1-canonicalize-at-parse.md`): canonicalize a type spelling
the MOMENT it enters the system, to the OLD key, instead of relying on every comparison to
remember to denote.

**New: `canonical_type_key`** (`src/types.rs`, beside `denoted_type_path`) — membership-aware,
unlike the pre-existing `type_denotation`'s blind prefix rewrite: only the 24 closed hard-primitive
tails (`WAT_TYPE_HARD_PRIMITIVES`) denote to their old `:wat::core::<name>` home (`AST` →
`:wat::WatAST`); `:wat::type::Infer` keeps `denoted_type_path`'s existing marker carve-out;
anything else (`wat.type/Bogus`) stays raw, so the existing "not a member of wat.type" diagnostic
(`tests/types/probe_255_1_identity.rs`) still recognizes it. A blind rewrite here would have
silently laundered a non-member into a `:wat::core::Bogus` that never existed, losing that
diagnostic — verified by running the change against that test before committing.

**Applied at every site the amendment named:**

1. `parse_type_inner`'s plain-`Path` arm (`src/types.rs`) — was: only `nil` canonicalized (a
   `canonicalize && denoted == ":wat::core::nil"` special case); every OTHER converted primitive
   fell through and stored the raw `:wat::type::X` path. Reached via `parse_type_node` /
   `parse_type_form` / `parse_type_expr` / `parse_type_expr_from_source` for free (all four sit on
   `parse_type_inner`).
2. `parse_type_form`'s parametric HEAD (`src/types.rs`) — was "Identity only" **by design**, to
   preserve a non-member's origin for the same "not a member of wat.type" diagnostic.
   `canonical_type_key`'s membership guard makes it safe to also denote a genuine member here.
   **This is the site that fixed the 38 `extend-type` "does not implement surface method seq"
   failures**: `register_extend_type_surface_impls`'s `dispatch_type_base`
   (`src/declare/register.rs:404-411`) is built straight off this `TypeExpr::Parametric.head`
   via `format!(":{head}")` — the METHOD REGISTRATION KEY, not a comparison, so nothing at that
   site was denoting before.
3. `extend-type` / `derive`'s child, target and protocol_name (`src/types.rs`,
   `splice_type_decls`) — **both** the `Symbol` arm (the first agent's fix) **and** the `Keyword`
   arm (a literal `:wat::type::X` keyword — the retired spelling the ruling also names — left
   un-denoted by that fix) now denote.
4. `constructor_head_key` — cosmetic only; all 7 container heads it handles are members, so the
   output is byte-identical to before.
5. `freeze::env::rekey_type_member_functions` — denotes `parent` before building the rekeyed
   `Type/method` name (theoretical for the current corpus — no `::`-qualified defn name currently
   collides with this — kept in step for the same reason the `derive` fix was kept theoretical).

**Which of the first agent's three fixes K1 makes redundant:**

- `src/function/subsume.rs` `value_matches_type_by_name`'s compare-time re-denotation of a
  Parametric head is now a no-op for a real corpus clause — the head it reads off the declared
  clause is already canonical after `parse_type_form`'s fix (#2 above). **Kept** — harmless, and
  it is the general-case fallback for a head that somehow still arrives un-denoted.
- The `extend-type`/`derive` Symbol-arm fixes in `src/types.rs` are directly superseded IN PLACE
  — this stone's K1 commit edits the same lines to also cover the Keyword arm.
- `src/check.rs` `check_legacy_user_main_signature` is **NOT** made redundant: it inspects the
  raw `WatAST` return-type node directly (`items[4]`) and never calls `parse_type_node`, so K1's
  parse-time door never reaches it. Still necessary.

## Gates — K1 floor

```
Summary [ 379.336s] 6213 tests run: 6213 passed (21 slow), 24 skipped
```
`.floor/2026-09-28T05-45-45Z`, exit=0. Every `insert-all` container-admission failure, every
`extend-type` seq/method-resolution failure, and every `:T` row from the inherited red
(`.floor/2026-09-28T05-09-49Z`, quoted below) is gone. STOP-1/STOP-2 (the K1 amendment's own
triggers) never fired — the amendment's step 3 `:T`-row chase was never needed.

`cargo clippy --release --all-targets -- -D warnings`: clean (rc 0), both after K1 and again
after the corpus conversion below.

## Finishing the stone — convert the rest of the corpus

Brief step 2 / amendment step 4. List re-derived: `git ls-files '*.wat' '*.wat.bad' | grep -v
'^wat-scripts/fixes/' | grep -v '^wat/'` (2522 candidates), minus the 9 unreadable `.wat.bad`
fixtures the first agent listed (verified all 9 present in the candidate list before excluding):
`docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/{substrate,test}.wat.bad`,
`tests/cli/wat_grep__malformed.wat.bad`,
`tests/types/probe_arc214_lexer_primed_generic_head_{primed,unprimed}_space.wat.bad`,
`tests/types/probe_arc232_generic_method_type_application.wat.bad`,
`tests/types/struct_destructure_non_symbol.wat.bad`,
`tests/value/wat_arc220_char_supplementary_plane.wat.bad`,
`tests/wat_lang/wat_arc072_letstar_parametric_whitespace.wat.bad` — 2513 files.

**Dry-run** on an 8-file random sample, copied to `/tmp`, diffed against the live tree: every
diff converted only a type position (`x <- :wat::core::i64` → `x <- wat.type/i64`,
`-> :wat::core::i64` → `-> wat.type/i64`, `(:wat::core::Vector :- […])` →
`(wat.type/Vector :- […])`); nothing touched a same-named function/form head or an unrelated
namespace (`:wat::grep::Node`, `:wat::holon::HolonAST` untouched).

**Applied** to all 2513 (`printf '[...]' | ./target/release/wat ./wat-scripts/fixes/types-to-wat-type.wat`,
70s wall): 2346 files changed, 167 had nothing to convert.

**Idempotence**: a third application over the already-converted corpus produced 0 changes —
`sha256sum` of all 2513 files after the 2nd application vs after the 3rd application: empty
diff.

**Census** — pre `.census/2026-09-28T04-23-04Z.txt` (2302 files) vs post
`.census/2026-09-28T05-57-29Z.txt` (2305 files, 52s), with the union of the wat/ stdlib's 65
files (first agent's conversion) and this run's 2513 as the OWNED/exempt list:
```
census-diff: no STOP-8
```
exit 0.

**Delta** — `scripts/replay/delta.sh` (its OWN default 179-file canary and its OWN default
codemod, `to-faithful-clojure.wat` — unrelated to this stone's `types-to-wat-type.wat`):
```
  ORIG-CLEAN  160/179
  CONV-CLEAN  158/179
  NEW         2   (orig clean -> converted broken)
  RECOVERY    0   (orig broken -> converted clean)

NEW files:
  wat-scripts/probes/arc-170/probe-c1-clean-surface.wat
  wat/holon/Ngram.wat
[delta] exit=0. RECOVERY 0.
```
RECOVERY 0 (no forged green). Both NEW files independently verified **not** caused by this
stone: copied each file's PRE-255.67 (unconverted) source into an isolated dir and ran
`to-faithful-clojure.wat` on it directly — both reproduce the identical breakage
(`wat/holon/Ngram.wat`: `DuplicateMacro :wat::holon::Ngram`, a stdlib-vs-live-copy collision
artifact of the delta harness itself; `probe-c1-clean-surface.wat`: 5 pre-existing type-check
errors including an `UnknownCallee :robe/work::kwargs-check`) — neither has anything to do with
a `wat.type/` spelling.

## STOP — the floor after the corpus conversion is RED

```
Summary [ 374.359s] 6213 tests run: 6156 passed (22 slow), 57 failed, 24 skipped
```
`.floor/2026-09-28T06-00-51Z`, exit=100. **Not re-run**, per doctrine. Full ARM captured at
`.floor/2026-09-28T06-00-51Z/ARM.txt` (7083 lines, ANSI-stripped, verbatim) and quoted in the
landing commit `fd04778e4`.

This is a **different bug class than K1's** — none of the three mechanisms below is a "two
spellings compared" bug. All 57 failing test names:

```
wat rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter
wat::collection probe_hashmap_ctor_vector_symmetric::probe_p7_odd_pair_count_rejected
wat::diagnostics probe_arc242_stone2_value_position_doctrine::contract_03_keyword_type_in_body_rejected_with_remedy
wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_3_type_mismatch_renders_non_keyword_head
wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_4_type_mismatch_renders_non_vector_spread
wat::function fn_signature::fn_body_type_mismatch_surfaces
wat::function fn_signature::malformed_args_vector_clear_error
wat::function probe_arc237_stone2_defclause_substrate::probe_01_single_clause_defclause_basic
wat::function probe_arc237_stone2_defclause_substrate::probe_02_multi_arity_dispatches_by_arity
wat::function probe_arc237_stone2_defclause_substrate::probe_03_same_arity_different_types_dispatches_by_type
wat::function probe_arc237_stone2_defclause_substrate::probe_04_typeunion_arg_accepts_via_bounded_existential
wat::function probe_arc237_stone2_defclause_substrate::probe_05_shared_return_type_applies_to_all_clauses
wat::function probe_arc237_stone2_defclause_substrate::probe_06_per_clause_return_types_pick_at_call_site
wat::function probe_arc237_stone2_defclause_substrate::probe_09_runtime_computes_correct_result
wat::function probe_arc237_stone2_defclause_substrate::probe_10_single_clause_defclause_equivalent_to_defn
wat::function variadic_define::parse_error_double_ampersand_in_define_signature
wat::function variadic_define::parse_error_fixed_param_after_rest_binder
wat::function variadic_define::parse_error_rest_binder_with_non_vector_type
wat::function variadic_define::parse_error_rest_marker_without_binder
wat::function variadic_define::signature_of_defn_variadic_define_returns_rest_shape
wat::function variadic_define::strict_arity_define_unchanged_by_arc150
wat::function variadic_define::variadic_define_arity_error_below_fixed_arity
wat::function variadic_define::variadic_define_rest_binding_is_a_vec_value
wat::function variadic_define::variadic_define_type_error_on_mismatched_rest_arg
wat::function variadic_define::variadic_define_uses_foldl_over_rest_args
wat::function variadic_define::variadic_define_with_many_rest_args
wat::function variadic_define::variadic_define_with_no_fixed_params_only_rest
wat::function variadic_define::variadic_define_with_no_fixed_params_zero_args_returns_seed
wat::function variadic_define::variadic_define_with_one_rest_arg
wat::function variadic_define::variadic_define_with_zero_rest_args_binds_empty_vec
wat::macros probe_arc209_macro_param_type_enforced::lying_macro_param_type_is_rejected_at_macro_def
wat::macros probe_arc258_stone2b_macro_error::contract_03_macro_error_surfaces_its_message
wat::program probe_arc213_program_edn_roundtrip::t1_program_to_edn_is_plain_edn
wat::resolve probe_arc255_register_variant_is_its_own_door::a_defn_with_a_dotted_name_is_still_refused_end_to_end
wat::resolve probe_arc255_register_variant_is_its_own_door::defn_then_variant_ctor_collide_at_check
wat::resolve probe_arc255_register_variant_is_its_own_door::variant_ctor_then_defn_collide_at_check
wat::resolve probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defn_may_not_claim_a_rust_name
wat::resolve probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defn_may_not_claim_a_wat_name
wat::resolve probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defstruct_may_not_claim_a_wat_name
wat::resolve probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_typealias_may_not_claim_a_wat_name
wat::types newtype::newtype_rejects_inner_type_at_arg_position
wat::types probe_arc234_stone3c_keyword_accessor::probe_3_unknown_field_on_record_errors
wat::types probe_arc251_type_the_polymorphic_accessor::record_monomorphic_lie_is_refused
wat::types probe_arc251_type_the_polymorphic_accessor::record_parametric_lie_is_refused
wat::types probe_arc251_type_the_polymorphic_accessor::unknown_field_on_a_known_receiver_is_refused
wat::types probe_arc251_type_the_polymorphic_accessor::variant_parametric_lie_is_refused
wat::types probe_arc255_equality_domain_gate::equality_refuses_a_non_equatable_declared_type
wat::types probe_arc258_stone1_if_inference::contract_03_branch_mismatch_rejected_for_the_right_reason
wat::types probe_arc283_1_rename_typearg::rename_reaches_type_arguments
wat::types tuple::legacy_tuple_lowercase_redirects_via_pattern2_poison
wat::wat_lang wat_arc136_do_form::do_empty_form_is_malformed
wat::wat_lang wat_arc153_nil_rename::mixed_empty_list_body_with_nil_sig_now_rejected
wat::wat_lang wat_arc153_nil_rename::value_position_empty_list_now_rejected
wat::wat_lang wat_arc153_nil_rename::value_position_nil_against_i64_recipient_fires_type_mismatch
wat::wat_lang wat_arc157_def::def_type_mismatch_via_registered_type
wat::wat_lang wat_idempotent_redeclare::define_divergent_body_errors
wat::wat_lang wat_idempotent_redeclare::typealias_divergent_errors
```

### Mechanism A — ~33 failures — golden EDN diagnostic snapshots drift

A `.rs` test does `assert_eq!` against a hardcoded EDN literal of an expected `TypeError` /
`CheckError` / `RuntimeError`, **including its exact `:col`/`:end` byte position**, for a
`.wat`/`.wat.bad` FIXTURE this stone correctly converted. `wat.type/i64` is a different length
than `:wat::core::i64`, so the fixture's line width shifted and the golden's pinned column is
stale — the diagnostic ITSELF is correct (same message, same error variant), only the recorded
byte offset disagrees. Representative, verbatim (both members of the `left`/`right` `assert_eq!`
pair, from the ARM):

```
thread 'probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_typealias_may_not_claim_a_wat_name' panicked at tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket.rs:104:5:
assertion `left == right` failed: EDN data mismatch
--- actual (raw) ---
#wat.type/ReservedPrefix {:message "type name :wat::core::SneakyA uses a reserved prefix (:wat::, :rust::, :$bound::); user types must use their own prefix" :location #wat.core/Span {:file "tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__typealias_wat.wat" :line 2 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 2 :col 57}}} :causes [] :name ":wat::core::SneakyA"}
--- expected (raw) ---
#wat.type/ReservedPrefix {:message "type name :wat::core::SneakyA uses a reserved prefix (:wat::, :rust::, :$bound::); user types must use their own prefix" :location #wat.core/Span {:file "tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__typealias_wat.wat" :line 2 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 2 :col 60}}} :causes [] :name ":wat::core::SneakyA"}
```
(actual `col 57`, expected `col 60` — a 3-column drift, exactly `len(":wat::core::")` −
`len("wat.type/")` = 12 − 9 = 3, on ONE converted token in that line.)

Same "EDN data mismatch" shape, verified individually: `probe_arc255_register_variant_is_its_own_door::a_defn_with_a_dotted_name_is_still_refused_end_to_end`
(col 58 vs 61, a 3-column drift again), `probe_arc251_type_the_polymorphic_accessor::record_monomorphic_lie_is_refused`,
`newtype::newtype_rejects_inner_type_at_arg_position`, `wat_arc136_do_form::do_empty_form_is_malformed`,
`probe_arc209_macro_param_type_enforced::lying_macro_param_type_is_rejected_at_macro_def`,
`wat_idempotent_redeclare::define_divergent_body_errors`. The remaining members of this bucket
(`probe_arc234_stone3c_keyword_accessor`, `probe_arc255_equality_domain_gate`,
`probe_arc258_stone1_if_inference`, `probe_arc258_stone2b_macro_error`,
`probe_arc283_1_rename_typearg`, `tuple::legacy_tuple_lowercase_redirects_via_pattern2_poison`,
`wat_arc153_nil_rename` ×3, `wat_arc157_def`, `wat_idempotent_redeclare::typealias_divergent_errors`,
`probe_diagnostic_value_snapshot_in_errors` ×2, `probe_arc242_stone2_value_position_doctrine`,
`fn_signature` ×2, `probe_hashmap_ctor_vector_symmetric`, `probe_arc213_program_edn_roundtrip`) were
not individually re-verified under this STOP, but share the identical panic message shape
("EDN data mismatch") in the ARM.

### Mechanism B — 23 failures — two ad hoc recognizers never learned the Symbol surface

Two hand-rolled type-annotation recognizers only match `WatAST::Keyword` and silently take the
"not this shape" fallback on a `WatAST::Symbol` (`wat.type/…`) — never calling any type parser,
so there is no denotation for K1 to apply:

- **`src/declare/parse.rs:773-776`**, `try_parse_user_variadic_def_fn_form`'s return-type slot
  for `(:wat::core::def :name (:wat::core::fn [args… & rest <- T] -> RET body))`:
  ```rust
  let ret_type = match &fn_items[3] {
      WatAST::Keyword(k, _) => parse_type_keyword(k)?,
      _ => return Ok(None),
  };
  ```
  A `wat.type/i64` return type hits `_ => return Ok(None)` — the WHOLE function silently reports
  "this is not a variadic def+fn form" (not an error), so `:my::sum-of` and its siblings in
  `tests/function/variadic_define.wat` never register at all. Verbatim, the resulting symptom
  (reproduced directly with `./target/release/wat --check tests/function/variadic_define.wat`,
  outside the test harness, identical):
  ```
  thread 'variadic_define::variadic_define_with_many_rest_args' panicked at tests/function/variadic_define.rs:34:41:
  startup: #wat.resolve/UnresolvedReferences {:message "7 unresolved references" ... [#wat.resolve/UnresolvedReference {:path ":my::sum-of" :context "call head — not a builtin, not a registered function" ...} ... #wat.resolve/UnresolvedReference {:path ":my::add-all" ...}]}
  ```
  All 15 `variadic_define::*` failures share this one root: `:my::sum-of`, `:my::sum`,
  `:my::count-rest`, `:my::add-all` never register (the file's one NON-variadic, strict-arity
  `:my::add_strict` — no rest binder — registers fine, confirming the mechanism is specific to
  the rest-binder recognizer).

- **`src/function/parse.rs:829-834`**, `parse_defclause_form`'s shared `-> :T` detection:
  ```rust
  match (&after_name[0], &after_name[1]) {
      (arrow, WatAST::Keyword(k, _)) if crate::types::is_return_arrow(arrow) => {
          let ret = parse_type_keyword(k)?;
          (Some(ret), 4usize)
      }
      _ => (None, 2usize), // items[2..] = clauses
  }
  ```
  `(:wat::core::defclause :p05::pick -> wat.type/i64 ([x <- wat.type/i64] x) …)` — the `wat.type/i64`
  after `->` is a Symbol, so this falls to `_ => (None, 2usize)`: NO shared return type is
  detected, `clause_offset` stays 2, and `clause_items` becomes `[-> wat.type/i64 ([x <- …] x) …]`
  — the bare Symbol `wat.type/i64` is now read as if it were the first CLAUSE, which must be a
  list, so `parse_defclause_clause` refuses it:
  ```
  thread 'probe_arc237_stone2_defclause_substrate::probe_01_single_clause_defclause_basic' panicked at tests/function/probe_arc237_stone2_defclause_substrate.rs:67:29:
  single-clause defclause should parse + type-check: #wat.runtime/MalformedForm {:message "malformed :wat::core::defclause form: each defclause clause must be a list `([args] -> :Ret body)` or `([args] body)` (with shared return); got symbol" :location ... :line 34 :col 35 ...} :head ":wat::core::defclause" :reason "each defclause clause must be a list `([args] -> :Ret body)` or `([args] body)` (with shared return); got symbol"}
  ```
  All 8 `probe_arc237_stone2_defclause_substrate::*` failures come from the ONE fixture
  (`probe_arc237_stone2_defclause_substrate.wat`) failing to load at all (a single
  `:p05::pick` defclause with the shared-return-type sugar), which fails every test that
  `startup`s it.

K1 does not reach either site — they never call `parse_type_node`/`parse_type_expr` for the
Symbol arm, so there is nothing for `canonical_type_key` to denote. This is a **parse-time
recognizer gap** (a missing arm), structurally different from the "two spellings compared" class
K1 fixes.

### Mechanism C — 1 failure — a static corpus differential, not investigated further

`rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter`:
```
thread '...' panicked at src/rete/kernel/tests/where_tree_branch_differential.rs:500:5:
assertion `left == right` failed: the non-uniform `where-*` axes must match NON_UNIFORM exactly — a new axis, a deleted one, or one that changed its row-driver shape all land here
  left: [...37 axis names...]
 right: [...26 axis names (NON_UNIFORM, hardcoded)...]
```
The live/computed set (`left`) has 12 more `where-*` axis names than the hardcoded expected list
(`right`): `where-boolean`, `where-collection`, `where-control`, `where-inline-computed`,
`where-inline-keyword`, `where-join-order`, `where-multivar`, `where-nesting`, `where-numeric`,
`where-record`, `where-shapes`, `where-string`. Ran in 0.00s (a static scan, not a live
evaluation), so this reads the corpus's own `where`-clause text directly — the corpus-wide
conversion evidently changed what the scan classifies as non-uniform. Not investigated further
under the STOP; may be related to Mechanism B (an ad hoc recognizer's shape check) or may be
independent.

## STOP — per doctrine

This is reported, not worked around: the corpus conversion (commit `fd04778e4`) is landed AS
RED, exactly as the amendment's own step 1 landed the first agent's red state — reverting it
would discard verified-correct, dry-run-checked, idempotent, brief-mandated conversion work to
dodge the red, which is itself a form of working around the STOP. The floor was **not** re-run
after the red was captured. Nothing beyond K1's own mandate (parse-time denotation) was
attempted — mechanisms B and C need new recognizer code (not a denotation fix) and mechanism A
needs either updated goldens or a judgment call on which fixtures a golden-sensitive test should
exempt from the corpus conversion; both are calls for the builder, not a rider improvising past a
STOP.

## AMEND-2 — finish green (this agent)

Commits: `01b488ea6` (B/A/C cure) and this SCORE append. Executed
`AMEND-2-STONE-255.67-finish-green.md` against the 57-failure red left by `fd04778e4`.

### Mechanism B — the two named recognizers, plus two more found by the same search

- **`src/declare/parse.rs:773-776`, `try_parse_user_variadic_def_fn_form`'s return-type slot** —
  was `WatAST::Keyword(k, _) => parse_type_keyword(k)?, _ => return Ok(None)`. A `wat.type/i64`
  Symbol hit the `None` fallback, so the whole variadic def+fn form silently read as "not this
  shape" and never registered (15 `variadic_define::*` failures — `UnresolvedReference` for
  `:my::sum-of` etc). Routed through `parse_type_node` (accepts `Keyword` or `Symbol`), error
  path converts `TypeError` → `RuntimeError` the same way `parse_type_keyword` did.
- **`src/function/parse.rs:829-834`, `parse_defclause_form`'s shared `-> :T` detection** — same
  Keyword-only match; a `wat.type/i64` shared return type went undetected, so `clause_offset`
  stayed at 2 and the bare type Symbol was read as the first CLAUSE, refused as "got symbol" (8
  `probe_arc237_stone2_defclause_substrate::*` failures — one shared fixture failing to load).
  Same `parse_type_node` routing.
- **Found by the search, same shape, routed the same way:**
  - `src/declare/parse.rs:637`, `try_parse_variadic_def_fn_form` — the STDLIB-only sibling of the
    first recognizer (`register_stdlib_defines`'s privileged path, `allow_reserved=true`).
    Identical Keyword-only bug; not currently reachable by any corpus fixture (no stdlib
    variadic-def+fn form happens to use a converted return type today) but fixed defensively —
    it is the same recognizer class at the same registration phase (step 6, pre-`normalize_symbol_refs`).
  - `src/function/parse.rs:~1279` (was `~1279`), `parse_extend_type_form`'s per-method `-> :T`
    return-type slot inside an `extend-type`/`derive` impl body — matched `Keyword` and `List`
    (the `(Head :- [T])` reference form) but had no `Symbol` arm; a bare `wat.type/i64` method
    return type fell to the `_ => (body_items, nil)` wildcard, which reads NO annotation at all
    and hands the whole `[-> wat.type/i64 body...]` back as the method BODY — `->` then evaluates
    as an unbound-symbol reference at runtime. **This one is corpus-live** (every `extend-type`
    method with an atomic converted return type, e.g. `wat/seq.wat`'s four `Seqable` `seq` impls
    use the `(Head :- [T])` List form so they were unaffected, but other stdlib/`wat-tests` files
    use the bare-symbol form) — see the self-correction below.
  - `src/intrinsic/holon/atom.rs`, `eval_holon_from_holon` (`:wat::holon::from-holon`'s optional
    3-arg `-> :T` HashMap-vs-HashSet disambiguation hint) — ran POST-normalize (eval time, not
    registration), so a pre-normalize `Symbol` never reaches it; `normalize_type_slot`
    (`src/resolve/normalize.rs`) converts it to a `WatAST::Keyword` first — but to the RAW
    `:wat::type::HashMap` spelling (`ns_to_wat_path`, not K1's `canonical_type_key`), and the
    site's own check was a raw `k.starts_with(":wat::core::HashMap")` — never denoted. A
    `:wat::type::HashMap` keyword compared unequal to the `:wat::core::HashMap` prefix, so
    `_hint_is_hashmap` silently read `false` for a well-formed hint. Not caught by
    `probe_arc216_stone3_hashmap_roundtrip.rs`'s own assertions (an empty HashMap vs HashSet both
    have length 0), so this was a live, silent misclassification, not a test failure. Routed
    through `canonical_type_key` (K1's door), accepting both `Keyword` and `Symbol`. This
    recognizer's fix RETIRED the ledger row `("src/intrinsic/holon/atom.rs",
    "eval_holon_from_holon", 1, "Ax1")` in `tests/lint/keyword_heresy_ledger.rs` —
    `LEDGER_TOTAL` 149 → 148 (the ledger's own gate, `the_heresy_ledger_matches_its_frozen_census`,
    caught this and named the exact row; updated by hand per the gate's own instruction).
  - **Checked and found already correct** (not a bug, no action): `src/macros/parse.rs`'s
    `defmacro` return-type check (already widened, "old keyword-only check refused" per its own
    comment); `src/types/surface.rs:429`'s method-member return type (already routed through
    `parse_type_node`); `src/function/parse.rs:1059-1108`'s `extend-type` target/protocol-name
    slots (already handle `Keyword`/`List`/`Symbol`); `src/check.rs`'s `parse_bracket_type_keyword`
    (Keyword-only, used by `infer_persistentmap_constructor`/`infer_list_constructor`'s literal
    `:- [K V]` VALUE-constructor bracket — genuinely reachable in principle, but the corpus has no
    site using a hard-primitive type in that specific literal-constructor bracket position today,
    grep-verified; theoretical, like K1's own `rekey_type_member_functions` note, left unfixed);
    `src/declare/parse.rs:955-957`'s `parse_type_slot` (Keyword-only, but it is ALREADY a tracked,
    cataloged row in the heresy ledger — `("src/declare/parse.rs", "parse_type_slot", 1, "Ax1")` —
    so it is a known, accounted-for gap, not a silent one, and out of THIS stone's scope).

**Self-correction, found by the floor, not by review:** the first edit to
`parse_extend_type_form`'s method-return-type match REPLACED the pre-existing
`node @ WatAST::List(_, _) =>` arm with the new `node @ WatAST::Symbol(_, _) =>` arm instead of
adding the Symbol arm alongside it — deleting List-form return-type handling entirely. This
passed every narrow/filtered nextest run (none of those runs touched an `extend-type` method
using the `(Head :- [T])` List return form) and passed `cargo build`/`clippy` clean, but the
**first full `scripts/floor.sh`** went RED at 54 failures — `UnboundSymbol {:name "->"}` at
`wat/seq.wat:87/90/93` (the `PersistentVector`/`List`/`Stream` `Seqable` impls; `Vector`'s impl at
line 84 also failed) and every test transitively depending on `Seqable`/`foldl`/`seq`
(`deftest_wat_tests_core_core_seqable_*`, `*_seq_walkers_*`, `*_foldl_*`, the `Bigram`/`Trigram`/
`Sequential`/`Ngram` holon deftests, `probe_stone118_3b_seqable_parametric_satisfaction::*`,
`probe_stone_118_b2c_surface_arm_never_dispatches::*`, `map_indexed`, `seqable_to_stream`,
`probe_arc118_2z_takewhile_lazy`, `probe_arc255_22_an_edge_declares_its_type_parameters`) — 53 of
the 54, plus the unrelated, expected `keyword_heresy_ledger` ratchet trip (below). Per doctrine
the red floor (`.floor/2026-09-28T06-37-21Z`, `Summary [ 376.710s] 6213 tests run: 6159 passed
(20 slow), 54 failed, 24 skipped`) was **not** re-run; the ARM was read, the exact arm named
(`node @ WatAST::List(_, _)` missing, `_` wildcard firing, `->` handed to the body as a literal
call head), the diagnosis made from the panic's own file:line:col (`wat/seq.wat:90:15`, the `->`
token), and the List arm restored ALONGSIDE the Symbol arm (both present now). Re-verified with a
64-test targeted run (all pass), then a full re-floor (below) confirms only the one known STOP
remains.

### Mechanism C — cured, spelling duplicate confirmed (not new shapes)

`src/rete/kernel/tests/where_tree_branch_differential.rs`'s `classify` function keys a `where-*`
axis as UNIFORM by finding the literal substring
`"defn :{ns}::row-count [] -> :wat::core::i64 "` in the FILE'S OWN TEXT (a raw string scan of the
`.wat` source, not a parsed-AST comparison — the one site in this hunt that isn't AST-based at
all). The corpus conversion rewrote every `where-*.wat`'s `row-count` return type to
`-> wat.type/i64 ` (verified directly, e.g. `wat-scripts/perf/grid/where-boolean.wat:50`), so the
marker never matched any of the 12 previously-uniform axes named in the ARM
(`where-boolean`, `where-collection`, `where-control`, `where-inline-computed`,
`where-inline-keyword`, `where-join-order`, `where-multivar`, `where-nesting`, `where-numeric`,
`where-record`, `where-shapes`, `where-string`) — they fell through to `None` (non-uniform) and
inflated `found_other` past `NON_UNIFORM`. Cured by trying both spellings for the marker (the raw
scan's own "door", since there is no AST here for `parse_type_node`/`canonical_type_key` to act
on). STOP-2 never fired — confirmed spelling duplicates, not genuinely new axis shapes: with the
fix, `where_tree_branch_agrees_with_the_reference_filter` passes and, critically, its OWN
correctness assertion (the tree-filter branch derives the identical fact multiset as the
reference branch, on every one of the now-uniform axes) also passes — the 12 axes are not just
"classified the same", they behave identically to before the corpus conversion.

### Mechanism A — 32 of 33 goldens recaptured; 1 STOP (more than a position changed)

All 33 confirmed individually (not just by panic-message shape) before touching anything: ran the
33 tests RED first, then for each compared actual vs. the existing golden with a position-blind
(digit-masked) diff — every one but `probe_arc283_1_rename_typearg::rename_reaches_type_arguments`
showed identical message/kind/name/field text with only numeric `:col`/`:end.col` values (and, in
one case, the fixture's own faithfully-converted `:node` spelling — see below) differing.

**25 recaptured via `UPDATE_EDN=1` (the `assert_edn_matches_file!` capture path)** — ran the 25
tests under `UPDATE_EDN=1`, which parses+validates `actual` as EDN (STOP-1 if it didn't) and
overwrites the `.edn` golden verbatim; then `git diff` on every touched `.edn` reviewed by hand:

| test | file | old → new position |
|---|---|---|
| `probe_hashmap_ctor_vector_symmetric::probe_p7_odd_pair_count_rejected` | `tests/collection/probe_hashmap_ctor_vector_symmetric__odd_pair_count.edn` | `:end.col` 28 → 25 |
| `probe_arc242_stone2_value_position_doctrine::contract_03_keyword_type_in_body_rejected_with_remedy` | `tests/diagnostics/probe_arc242_stone2_value_position_doctrine__contract_03_keyword_type_in_body_rejected_with_remedy.edn` | `:col` 50 → 47, `:end.col` 65 → 62 |
| `probe_diagnostic_value_snapshot_in_errors::probe_3_type_mismatch_renders_non_keyword_head` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors__probe_3_type_mismatch_renders_non_keyword_head.edn` | `:col` 76 → 73, `:end.col` 91 → 88 |
| `probe_diagnostic_value_snapshot_in_errors::probe_4_type_mismatch_renders_non_vector_spread` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors__probe_4_type_mismatch_renders_non_vector_spread.edn` | `:col` 125 → 122, `:end.col` 127 → 124 |
| `fn_signature::fn_body_type_mismatch_surfaces` | `tests/function/fn_signature__fn_body_type_mismatch_surfaces.edn` | `:col` 112 → 103, `:end.col` 113 → 104 |
| `fn_signature::malformed_args_vector_clear_error` | `tests/function/fn_signature__malformed_args_vector_clear_error.edn` | `:col` 42 → 39, `:end.col` 43 → 40 |
| `probe_arc209_macro_param_type_enforced::lying_macro_param_type_is_rejected_at_macro_def` | `tests/macros/probe_arc209_macro_param_type_enforced__lying_macro_param_type_is_rejected_at_macro_def.edn` | `:end.col` 56 → 53 |
| `probe_arc258_stone2b_macro_error::contract_03_macro_error_surfaces_its_message` | `tests/macros/probe_arc258_stone2b_macro_error__contract_03_macro_error_surfaces_its_message.edn` | `:col` 50 → 47, `:end.col` 63 → 60 |
| `probe_arc213_program_edn_roundtrip::t1_program_to_edn_is_plain_edn` | `tests/program/probe_arc213_program_edn_roundtrip__program_frame.edn` | many `:col`/`:end-col` positions shifted (a full-program AST roundtrip); ALSO three `:node` fields changed from `:wat.core/i64`/`:wat.core/nil` to `wat.type/i64`/`wat.type/nil` — judged in-scope (see note below), not a STOP |
| `newtype::newtype_rejects_inner_type_at_arg_position` | `tests/types/newtype__newtype_rejects_inner_type_at_arg_position.edn` | `:col` 77 → 74, `:end.col` 82 → 79 |
| `probe_arc234_stone3c_keyword_accessor::probe_3_unknown_field_on_record_errors` | `tests/types/probe_arc234_stone3c_keyword_accessor__probe3_unknown_field.edn` | `:col` 62 → 59, `:end.col` 74 → 71 |
| `probe_arc251_type_the_polymorphic_accessor::record_monomorphic_lie_is_refused` | `tests/types/probe_arc251_type_the_polymorphic_accessor__record_mono_lie.edn` | `:col` 64 → 61, `:end.col` 70 → 67 |
| `probe_arc251_type_the_polymorphic_accessor::record_parametric_lie_is_refused` | `tests/types/probe_arc251_type_the_polymorphic_accessor__record_param_lie.edn` | `:col` 86 → 80, `:end.col` 92 → 86 |
| `probe_arc251_type_the_polymorphic_accessor::unknown_field_on_a_known_receiver_is_refused` | `tests/types/probe_arc251_type_the_polymorphic_accessor__unknown_field.edn` | `:col` 62 → 59, `:end.col` 74 → 71 |
| `probe_arc251_type_the_polymorphic_accessor::variant_parametric_lie_is_refused` | `tests/types/probe_arc251_type_the_polymorphic_accessor__variant_param_lie.edn` | `:col` 91 → 85, `:end.col` 99 → 93 |
| `probe_arc255_equality_domain_gate::equality_refuses_a_non_equatable_declared_type` | `tests/types/probe_arc255_equality_domain_gate__fn_operand_refused.edn` | `:end.col` 84 → 78 (also flat → pretty EDN formatting, cosmetic, data-equal) |
| `probe_arc258_stone1_if_inference::contract_03_branch_mismatch_rejected_for_the_right_reason` | `tests/types/probe_arc258_stone1_if_inference__contract_03_branch_mismatch_rejected_for_the_right_reason.edn` | `:col` 79 → 76, `:end.col` 82 → 79 |
| `tuple::legacy_tuple_lowercase_redirects_via_pattern2_poison` | `tests/types/tuple__legacy_tuple_lowercase_redirects_via_pattern2_poison.edn` | `:col` 110 → 98, `:end.col` 127 → 115 |
| `wat_arc136_do_form::do_empty_form_is_malformed` | `tests/wat_lang/wat_arc136_do_form__do_empty_form_is_malformed.edn` | `:col` 54 → 51, `:end.col` 68 → 65 |
| `wat_arc153_nil_rename::mixed_empty_list_body_with_nil_sig_now_rejected` | `tests/wat_lang/wat_arc153_nil_rename__paren_body_err.edn` | `:col` 61 → 58, `:end.col` 63 → 60 |
| `wat_arc153_nil_rename::value_position_empty_list_now_rejected` | `tests/wat_lang/wat_arc153_nil_rename__paren_form_err.edn` | `:col` 60 → 57, `:end.col` 62 → 59 |
| `wat_arc153_nil_rename::value_position_nil_against_i64_recipient_fires_type_mismatch` | `tests/wat_lang/wat_arc153_nil_rename__value_position_nil_against_i64_recipient_fires_type_mismatch.edn` | `:col` 51 → 48, `:end.col` 54 → 51 |
| `wat_arc157_def::def_type_mismatch_via_registered_type` | `tests/wat_lang/wat_arc157_def__def_type_mismatch_via_registered_type.edn` | `:col` 65 → 62, `:end.col` 71 → 68 |
| `wat_idempotent_redeclare::define_divergent_body_errors` | `tests/wat_lang/wat_idempotent_redeclare__define_divergent_body_errors.edn` | two `:end.col` fields (one per cause), 94 → 88 and 94 → 88 |
| `wat_idempotent_redeclare::typealias_divergent_errors` | `tests/wat_lang/wat_idempotent_redeclare__typealias_divergent_errors.edn` | `:end.col` 52 → 49 |

**Note on `probe_arc213_program_edn_roundtrip`:** unlike the other 24, this golden's diff also
shows `:node` field VALUES changing (`:wat.core/i64` → `wat.type/i64`, `:wat.core/nil` →
`wat.type/nil`). Judged in-scope, not a STOP: the test is a roundtrip of the FIXTURE'S OWN source
(a `.wat`/`.wat.bad` fixture legitimately converted by this stone's codemod) back to EDN — the
`:node` field is showing exactly what is now IN the fixture, which is the intended, correct effect
of the conversion, not an unrelated golden's content being disturbed. Confirmed by: after
recapture, `actual == golden` (the test passes cleanly; nothing needed forcing).

**3 more recaptured by hand** (the macro used, `assert_edn_eq!(out, include_str!(...))`, has no
`UPDATE_EDN` support — `run_check`'s exact `wat --check <fixture>` command reproduced directly,
diffed against the existing golden to confirm position-only, then the golden file overwritten with
the new `wat --check` output verbatim):

| test | file | old → new position |
|---|---|---|
| `probe_arc255_register_variant_is_its_own_door::defn_then_variant_ctor_collide_at_check` | `tests/resolve/probe_arc255_register_variant_is_its_own_door__defn_then_variant.edn` | `:end.col` 61 → 58 |
| `probe_arc255_register_variant_is_its_own_door::variant_ctor_then_defn_collide_at_check` | `tests/resolve/probe_arc255_register_variant_is_its_own_door__variant_then_defn.edn` | `:end.col` 61 → 58 |
| `probe_arc255_register_variant_is_its_own_door::a_defn_with_a_dotted_name_is_still_refused_end_to_end` | `tests/resolve/probe_arc255_register_variant_is_its_own_door__dotted_defn.edn` | `:end.col` 61 → 58 |

**4 more, same manual method, other file:**

| test | file | old → new position |
|---|---|---|
| `probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defn_may_not_claim_a_wat_name` | `tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defn_wat.edn` | `:end.col` 62 → 59 |
| `probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defn_may_not_claim_a_rust_name` | `tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defn_rust.edn` | `:end.col` 60 → 57 |
| `probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defstruct_may_not_claim_a_wat_name` | `tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defstruct_wat.edn` | `:end.col` 67 → 64 |
| `probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_typealias_may_not_claim_a_wat_name` | `tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__typealias_wat.edn` | `:end.col` 60 → 57 |

25 + 3 + 4 = 32. Every recapture individually diff-reviewed before being trusted; none showed a
message, kind, or name change — only the numeric position (and, for the one AST-roundtrip test,
the fixture's own intended spelling reflected verbatim).

### STOP-1 — `probe_arc283_1_rename_typearg::rename_reaches_type_arguments` — more than a position changed, not touched

```
thread 'probe_arc283_1_rename_typearg::rename_reaches_type_arguments' (4131304) panicked at /home/john/work/holon/wat-rs/tests/types/probe_arc283_1_rename_typearg.rs:25:5:
assertion `left == right` failed
  left: "(:wat::core::defn :u::f [xs <- (:wat::core::Vector :- [:t::New]) y <- :t::OldExtra] -> :t::New (:t::New/make xs))"
 right: "(:wat::core::defn :u::f [xs <- (wat.type/Vector :- [:t::New]) y <- :t::OldExtra] -> :t::New (:t::New/make xs))"
```

This is NOT an EDN position golden — it's `assert_eq!` on a plain STRING, the output of
`:wat::fix::rename-keyword-prefix` run over a raw string LITERAL embedded in
`tests/types/probe_arc283_1_rename_typearg.wat` (`":t::Old"` → `":t::New"`, simulating a
text-level rename, deliberately using the OLD `:wat::core::Vector` spelling inside a Rust STRING
argument — opaque text, never touched by the type-position codemod, and correctly still
`:wat::core::Vector` in `left`/actual). The golden file
`tests/types/probe_arc283_1_rename_typearg__renamed.wat`, however, IS a `.wat` file matching the
corpus glob, and the 2513-file conversion (`fd04778e4`) rewrote ITS OWN top-level AST content
(`:wat::core::Vector` → `wat.type/Vector`, confirmed via `git show fd04778e4 -- <that path>`) —
even though this file's actual JOB is to be the byte-exact STRING golden of the rename tool's
output on unrelated text, not a type-checked program whose own annotations should track the
`wat.type/` migration. The corpus conversion touching this file was collateral, not intentional:
`right`/golden now disagrees with `left`/actual because the GOLDEN was mutated, not because the
tool's behavior changed. This is squarely **STOP-1** ("a golden where more than a position
changed") — the entire type spelling changed, not a byte offset, and unlike every other case
above, `actual` here is NOT the corrected value to capture (capturing it would launder the
codemod's collateral damage into a permanent, correct-looking golden). **Left untouched, still
RED, reported per doctrine — a call for the builder**: either exempt this one golden file from
future corpus-wide codemod runs (it is not "wat source" in the normal sense, it's comparison
data), or restore its `:wat::core::Vector` spelling by hand with a note explaining why it is
exempt.

## Gates (AMEND-2, this agent)

**`cargo clippy --release --all-targets -- -D warnings`**: clean, rc 0 (run twice — after the B/C
fix and again after the final state).

**`scripts/replay/census.sh --diff .census/2026-09-28T04-23-04Z.txt .census/2026-09-28T06-55-45Z.txt <owned>`**
(owned = the union of `wat/**/*.wat` and the 2513-file "rest of corpus" list, same derivation as
the prior agent's run): `census-diff: no STOP-8`, exit 0.

**`scripts/replay/delta.sh`** (its own default 179-file canary + `to-faithful-clojure.wat`,
unrelated to this stone's codemod):
```
  ORIG-CLEAN  160/179
  CONV-CLEAN  158/179
  NEW         2   (orig clean -> converted broken)
  RECOVERY    0   (orig broken -> converted clean)

NEW files:
  wat-scripts/probes/arc-170/probe-c1-clean-surface.wat
  wat/holon/Ngram.wat
```
RECOVERY 0. Identical to the prior agent's independently-verified run (same 2 NEW files, same
root causes — a stdlib-vs-live-copy `DuplicateMacro` collision in the delta harness itself, and 5
pre-existing type-check errors including `UnknownCallee :robe/work::kwargs-check` — neither has
anything to do with `wat.type/`).

**`scripts/floor.sh`, first run (after the B/C/A cures, before the self-correction below):**
```
Summary [ 376.710s] 6213 tests run: 6159 passed (20 slow), 54 failed, 24 skipped
```
`.floor/2026-09-28T06-37-21Z`, exit=100. Not re-run. 53 of the 54 traced to the
`parse_extend_type_form` self-correction (above: the List arm was deleted, not added-alongside);
the 54th (`keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census`) is the ledger's
own ratchet correctly firing on the `eval_holon_from_holon` cure (a row went from 1 occurrence to
0 — "the good direction", per the gate's own message — requiring `LEDGER_TOTAL`/the row updated by
hand, done above).

**`scripts/floor.sh`, final run (after the self-correction and the ledger update):**
```
Summary [ 377.758s] 6213 tests run: 6212 passed (21 slow), 1 failed, 24 skipped
```
`.floor/2026-09-28T06-49-04Z`, exit=100. The one failure is the STOP-1 case above,
`probe_arc283_1_rename_typearg::rename_reaches_type_arguments`, reported and left untouched per
doctrine — every other test in the 6213-test suite is green, including all of B, C, and 32 of A.

## AMEND-3 — X-G: goldens are `.wat.golden` (this agent)

Per `AMEND-3-STONE-255.67-goldens-are-not-programs.md`. Method: re-derived SCORE-255.58 §1's
37-row table from the tree first — every `include_str!` call site and its `.wat` file re-read,
line numbers cross-checked against the table; all 37 present, unchanged, at the same lines.

### 1. Renamed (37, `git mv <f> <f>.golden`), test's `include_str!` path updated to match, nothing
   else in the 9 test files changed

- `tests/resolve/probe_arc251_fix_source_local_rules__contract-01-arrow-in-binder.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-02-post-arrow-scalar.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-04-head-inverts.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-05-full-fn-literal.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-06a-less-than.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-06b-less-equal.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-07-greater-than.wat`
- `tests/resolve/probe_arc251_fix_source_head_rule__contract-01-bare-call-head-inverted.wat`
- `tests/resolve/probe_arc251_fix_source_head_rule__contract-02-strip-and-head-compose.wat`
- `tests/resolve/probe_arc251_fix_source_head_rule__contract-03-nested-heads.wat`
- `tests/resolve/probe_arc251_fix_source_head_rule__contract-04-data-keyword-head.wat`
- `tests/resolve/probe_arc258_stone3_fix_source__contract-05-nested-do-if.wat`
- `tests/resolve/probe_arc258_stone3_fix_source__contract-06-preserves-option-expect.wat`
- `tests/resolve/probe_arc258_stone3_fix_source__contract-07-end-to-end-clean.wat`
- `tests/resolve/probe_arc251_decl_migrator__c01-typealias-type-slot.wat`
- `tests/resolve/probe_arc251_decl_migrator__c02-defn-drop-type-params.wat`
- `tests/resolve/probe_arc251_decl_migrator__c04-user-type-preserved.wat`
- `tests/resolve/probe_arc251_decl_migrator__c05-newtype-type-slot.wat`
- `tests/resolve/probe_arc251_decl_migrator__c06-typeunion-core-members.wat`
- `tests/resolve/probe_arc251_decl_migrator__c07-typeunion-user-members.wat`
- `tests/resolve/probe_arc251_decl_migrator__c08-defenum-variant-tags.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-01a-scalar-i64.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-01b-scalar-user.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-06-tuple.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c01a-core-fqdn-i64.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c01b-core-fqdn-string.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c03a-legacy-i64.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c03b-legacy-string.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c03c-legacy-bool.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c04-user-type-namespace.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c06-user-type-two-segment.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c07a-type-var-t.wat`
- `tests/resolve/probe_arc251_type_namespace_fix__c07b-type-var-k.wat`
- `tests/resolve/probe_arc251_fix_text_comment_faithful__probe-comment-faithful.wat`
- `tests/resolve/probe_arc251_fix_text_comment_faithful__once-many-comments-idempotent.wat`
- `tests/resolve/probe_arc269_rename_keyword_prefix__swap-prefix-comment-faithful.wat`
- `tests/types/probe_arc283_1_rename_typearg__renamed.wat`

Test files edited (`include_str!` path only, `.wat` → `.wat.golden`): `probe_arc251_fix_source_local_rules.rs`,
`probe_arc251_fix_source_head_rule.rs`, `probe_arc258_stone3_fix_source.rs`, `probe_arc251_decl_migrator.rs`,
`probe_arc251_keyword_to_type_form.rs`, `probe_arc251_type_namespace_fix.rs`,
`probe_arc251_fix_text_comment_faithful.rs`, `probe_arc269_rename_keyword_prefix.rs`,
`probe_arc283_1_rename_typearg.rs`.

### 2. Restored to pre-conversion bytes

Only `tests/types/probe_arc283_1_rename_typearg__renamed.wat.golden`, per the amendment's step 3 (this
is the STOP-1 file from AMEND-2). Content set to `git show fd04778e4^:tests/types/probe_arc283_1_rename_typearg__renamed.wat`
verbatim (byte-diffed against that blob — identical, including the absence of a trailing newline on
both sides).

Checked whether `fd04778e4` touched any of the OTHER 36 renamed goldens: `git show fd04778e4
--name-only` against the full 37-name list — only `probe_arc283_1_rename_typearg__renamed.wat`
appears. **No other renamed golden needed restoration.**

### 3. Dead fixtures deleted (6, `git rm`)

Re-verified each is read by nothing: grepped the whole tree (not just `--include=*.rs`) for each
file name. Each hit found was either the file itself, a stale hardcoded corpus-snapshot list inside
`wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat` (a one-off arc-278
reconnaissance script, not a test, not part of any gate — confirmed its `main` is a literal 2000+
line path snapshot from that arc, unrelated to this stone), or prose mentions in two historical SCORE
docs (`SCORE-STONE-255.57...`, `SCORE-STONE-255.66...`). No `.rs` file under `tests/` references any
of the six.

- `tests/resolve/probe_arc251_keyword_to_type_form__contract-02-parametric.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-03-nested-parametric.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-04-type-var-bare.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-05-multi-arg.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-07-empty-tuple.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-08-nested-tuple.wat`

### 4. Gates

**`cargo clippy --release --all-targets -- -D warnings`**: clean, rc 0.
```
   Compiling wat v0.1.0 (/home/john/work/holon/wat-rs)
    Checking with-loader-example v0.1.0 (/home/john/work/holon/wat-rs/examples/with-loader)
    Checking console-demo v0.1.0 (/home/john/work/holon/wat-rs/examples/console-demo)
    Finished `release` profile [optimized] target(s) in 12.00s
```

**`scripts/replay/census.sh`** (fresh census, `.census/2026-09-28T07-14-58Z.txt`, 2262 files) then
**`scripts/replay/census.sh --diff .census/2026-09-28T04-23-04Z.txt .census/2026-09-28T07-14-58Z.txt`**:
```
census-diff: no STOP-8
```
rc 0. `comm` between the two path lists confirms the diff is exactly: the 43 files this stone
removed from `*.wat` (37 renamed + 6 deleted, listed above) leaving the census, plus 3 files added by
the prior (AMEND-2) agent's work between the PREV stamp and now (`tests/function/probe_arc255_67_wat_type_container_defclause_dispatch.wat`,
`tests/program/wat_arc170_slice_1e_user_main_nil_new_spelling.wat`, `tests/types/probe_arc255_67_cutover_types.wat`)
— unrelated to this amendment. 2302 → 2262 = -43 +3, no other change, no rc flip.

**`scripts/replay/delta.sh`** (default 179-file canary):
```
  ORIG-CLEAN  160/179
  CONV-CLEAN  158/179
  NEW         2   (orig clean -> converted broken)
  RECOVERY    0   (orig broken -> converted clean)

NEW files:
  wat-scripts/probes/arc-170/probe-c1-clean-surface.wat
  wat/holon/Ngram.wat
```
`.delta/2026-09-28T07-16-17Z`, exit=0. Identical NEW/RECOVERY to the prior agent's run — none of this
amendment's 43 touched files are in the delta's 179-file canary list; unrelated.

**`scripts/floor.sh`**:
```
Summary [ 376.945s] 6213 tests run: 6212 passed (22 slow), 1 failed, 24 skipped
```
`.floor/2026-09-28T07-05-20Z`, exit=100.

### STOP-1 (new) — `probe_arc170_edn_bridge_unspellable::c02_control_ordinary_forms_stay_plain_edn` —
not a renamed/restored golden's own test. Not re-run. Not worked around.

```
        FAIL [   0.012s] (3874/6213) wat::program probe_arc170_edn_bridge_unspellable::c02_control_ordinary_forms_stay_plain_edn
  stdout ───

    running 1 test
    test probe_arc170_edn_bridge_unspellable::c02_control_ordinary_forms_stay_plain_edn ... FAILED

    failures:

    failures:
        probe_arc170_edn_bridge_unspellable::c02_control_ordinary_forms_stay_plain_edn

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 60 filtered out; finished in 0.00s

  stderr ───

    thread 'probe_arc170_edn_bridge_unspellable::c02_control_ordinary_forms_stay_plain_edn' (290240) panicked at /home/john/work/holon/wat-rs/tests/program/probe_arc170_edn_bridge_unspellable.rs:103:45:
    an ordinary corpus file must be readable: Os { code: 2, kind: NotFound, message: "No such file or directory" }
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Mechanism.** `tests/program/probe_arc170_edn_bridge_unspellable.rs:99-103` hardcodes the path
`tests/resolve/probe_arc251_fix_source_head_rule__contract-01-bare-call-head-inverted.wat` and reads
it with `std::fs::read_to_string` — NOT `include_str!` — as "an ordinary corpus file", the deliberate
CONTROL fixture for that test's EDN-bridge structural check (comment at its own :79-96: "an ordinary
program crosses as PLAIN EDN"). That file is ALSO, independently, one of the 37 goldens named in
SCORE-255.58 §1 and this amendment's step 1: the expected OUTPUT of
`probe_arc251_fix_source_head_rule::contract_01...` (row 8 of the table). It plays both roles at
once — X-G output golden for one test, ordinary corpus INPUT sample for a wholly unrelated one. The
37-file re-derivation in this amendment (like SCORE-255.58's original measurement) found every
`include_str!` call site; neither pass searched for `std::fs::read_to_string`/`read_to_string`
call sites naming these same paths, so this second consumer was invisible to the method both times.

Renaming the file to `.wat.golden` (this amendment's step 2, uncontested — its OWN test,
`probe_arc251_fix_source_head_rule.rs`, is green) silently removed it from where
`probe_arc170_edn_bridge_unspellable.rs:101` still looks, which is why the floor went red on a test
this amendment's work list never names.

**Not fixed.** Per doctrine ("STOP-1: a floor red that is not a renamed or restored golden's own
test... STOP" / "A STOP trigger means stop and report, not work around"), the rename is left in
place exactly as this amendment ordered it, `probe_arc170_edn_bridge_unspellable.rs` is left
untouched, and the floor is left red — a call for the builder. Two shapes of cure, either legitimate:
(a) repoint `probe_arc170_edn_bridge_unspellable.rs:101` at a different, still-`.wat` ordinary
corpus file (the property under test does not care WHICH ordinary file it samples, only that it stays
plain, un-golden, un-renamed corpus), or (b) treat this one golden as the STOP-2 case the amendment
anticipated for a file that cannot be cleanly classified golden vs. program — since it is, here,
BOTH — and revert just its rename, leaving `tests/resolve/probe_arc251_fix_source_head_rule__contract-01-bare-call-head-inverted.wat`
un-renamed (its own test does not require the `.wat.golden` suffix to pass; only the AMENDMENT's
motivating property, keeping it out of a future corpus-wide codemod, would be lost for this one file).
Nothing else in the 6213-test suite is affected: every other renamed/restored/deleted site's own
tests are green (the full 6212/6213 pass count reflects that — the ONE failure is this one).

**Gate note:** clippy, census-diff, and delta above are unaffected by this STOP (none re-runs the
floor; none depends on the failing test) and are reported as run, verbatim, per the amendment's step
5. The floor itself was run exactly once and is not re-run per STOP-1.

## AMEND-3 cure — orchestrator's ruling on the new STOP-1: option (a), keep the rename

Orchestrator's ruling: the rename stands (it follows directly from X-G — a golden is not an
ordinary corpus program), and `probe_arc170_edn_bridge_unspellable.rs` gets repointed at a genuine
ordinary program instead. No new builder ruling needed; this is the amendment's own consequence,
not a fresh design question.

### 1. Chosen replacement file and why

`tests/program/probe_arc170_edn_bridge_unspellable.rs:99-103` (`c02_control_ordinary_forms_stay_plain_edn`)
repointed from the renamed golden to **`tests/collection/probe_arc257_native_map_set.wat`**. Only
the path string changed; no comment named the old file by name (checked: the only occurrence of
`fix_source_head_rule` or `contract-01-bare-call-head-inverted` anywhere in this test file was that
one path literal).

Why this file still exercises the property: it is a genuine ordinary corpus PROGRAM, not a golden
and not a `.wat.bad` — confirmed by `tests/collection/probe_arc257_native_map_set.rs` loading and
RUNNING it via `call_beside_value(file!(), ":t::probeN-...")` (the co-located-fixture convention;
`file!()` resolves to this exact `.wat` by Rust module-path convention), and its own three tests are
green in the same floor below. Content (749 bytes):

```
(:wat::core::defn :t::probe1-map-single [] -> wat.type/i64
  (:wat::core::let
    [m {:a 42}]
    (:wat::core::length m)))

(:wat::core::defn :t::probe2-map-multi [] -> wat.type/i64
  (:wat::core::let
    [m {:x 10 :y 20}]
    (:wat::core::length m)))

(:wat::core::defn :t::probe3-set-contains [] -> wat.type/bool
  (:wat::core::let
    [s #{1 2 3}]
    (:wat::core::contains? s 2)))
```

It has everything the orchestrator asked the replacement to carry: `defn` (×3), calls
(`:wat::core::length`, `:wat::core::contains?`), `let`, and native collection literals (a `{}` map ×2,
a `#{}` set) — richer than the one-form golden it replaces, and still small enough to keep the
control fast. It still discriminates C02's actual property (an ordinary program, including its map
literals, crosses `program_to_edn` → parses back as PLAIN EDN with no tag beyond the two declared
span-carriage tags) because nothing in it is a wat-unspellable lexeme (C01's job) or the fixture's
OWN job is being a golden (this stone's whole point) — it is simply representative corpus content.

### 2. Search for other non-`include_str!` readers of the 37 renamed / 6 deleted names

Searched every `tests/**/*.rs` and `src/**/*.rs` (not just `include_str!` sites) for each of the 37
renamed and 6 deleted basenames, individually. Result:

- **37 renamed names**: every hit outside the one already-fixed `probe_arc170_edn_bridge_unspellable.rs`
  call is an `include_str!` site — i.e. the renamed golden's own test, already repointed to
  `.wat.golden` in the prior commit. No other `.rs` file under `tests/` or `src/` names any of them.
- **One non-`.rs` hit, inert, not repointed**: `tests/types/probe_arc255_54_class_hits.txt` (a
  checked-in data snapshot, 3 lines naming `probe_arc251_fix_source_local_rules__contract-06a-less-than.wat`,
  `-06b-less-equal.wat`, `-07-greater-than.wat`) is referenced by exactly one `.rs` line
  (`probe_arc255_54_classes.rs:107`), inside `collect_compared_types`, a `#[ignore]`d "one-shot
  collector… not part of the floor" that only ever WRITES this file fresh from a live
  `git ls-files '*.wat' '*.wat.bad'` sweep — it never reads the checked-in `.txt` to compare against
  anything, and being `#[ignore]`d it never runs in `scripts/floor.sh`. The stale snapshot itself is
  not consumed by any test; next time someone runs it with `--ignored` it will naturally stop naming
  the renamed files (they no longer match `git ls-files '*.wat'`). Left untouched — not a live
  consumer, nothing to repoint.
- **6 deleted dead-fixture names**: zero hits anywhere in `tests/**/*.rs` or `src/**/*.rs` (confirmed
  again with this broader, non-`include_str!`-scoped search) — the 255.67-amendment deletion stands
  unchanged.

### 3. Gates, after the repoint

**`scripts/floor.sh`**:
```
     Summary [ 377.850s] 6213 tests run: 6213 passed (19 slow), 24 skipped
```
`.floor/2026-09-28T07-22-37Z`, exit=0. **All green** — including
`probe_arc170_edn_bridge_unspellable::c02_control_ordinary_forms_stay_plain_edn` and all three
`probe_arc257_native_map_set` tests. No red anywhere in the 6213-test suite. Not re-run (this is the
one and only run after the repoint).

**`cargo clippy --release --all-targets -- -D warnings`**: clean, rc 0.
```
   Compiling wat v0.1.0 (/home/john/work/holon/wat-rs)
    Checking with-loader-example v0.1.0 (/home/john/work/holon/wat-rs/examples/with-loader)
    Checking console-demo v0.1.0 (/home/john/work/holon/wat-rs/examples/console-demo)
    Finished `release` profile [optimized] target(s) in 12.06s
```

The stone lands green.
