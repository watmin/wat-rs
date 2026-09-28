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
