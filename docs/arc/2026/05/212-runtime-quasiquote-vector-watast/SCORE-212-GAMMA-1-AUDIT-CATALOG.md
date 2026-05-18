# Arc 212 stone γ-1 — SCORE: walker audit catalog

## Summary

Sites inspected: ~435 grep hits across src/ and crates/*/src/, consolidated into ~110 unique function sites after de-duplicating multi-line matches within the same function.

| Classification | Count |
|---|---|
| Walker (already migrated) | 22 |
| Walker (pending migration) | 5 |
| Walker (sharpening target) | 2 |
| Leaf-decomposition | ~80 |

**No STOP trigger fired.** All sites classified. Catalog is complete.

---

## Catalog

| file:line | fn name | classification | reason |
|---|---|---|---|
| src/ast.rs:112-114 | `WatAST::span` (method) | Leaf-decomposition | Extracts the span from every variant (including List, Vector, StructPattern) for error reporting; no recursion, no children descent. |
| src/ast.rs:139 | `WatAST::list` (constructor) | Leaf-decomposition | Convenience constructor that produces `WatAST::List`; no pattern-match on children. |
| src/ast.rs:144 | `WatAST::vector` (constructor) | Leaf-decomposition | Convenience constructor that produces `WatAST::Vector`; no pattern-match on children. |
| src/ast.rs:149 | `WatAST::struct_pattern` (constructor) | Leaf-decomposition | Convenience constructor that produces `WatAST::StructPattern`; no pattern-match on children. |
| src/ast.rs:183-185 | `WatAST::children` (method) | Leaf-decomposition | The canonical children() implementation itself; matches all three compound shapes to return their items slice. Not a walker — it is the primitive. |
| src/resolve.rs:122 | `collect_use_declarations` | Leaf-decomposition | Matches the single `WatAST::List` shape to detect `(:wat::core::use! ...)` forms; no recursion into children. |
| src/resolve.rs:163 | `check_form` | Walker (already migrated) | Uses `for child in form.children()` for generic recursion; walker-specific `WatAST::List(items, _)` guard applies only to call-head resolution and quote-family boundary logic. |
| src/resolve.rs:256 | `check_quasiquote_template` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; `WatAST::List(items, _)` guard checks only for unquote/unquote-splicing heads. |
| src/check.rs:2162 | `validate_comm_positions` | Walker (sharpening target) | Inscribed TEMPORARY List-only; migration to children() produces false positives on the fourth permitted comm slot (name-bound recv later matched/expected); stone δ-comm-positions sharpens the rule. |
| src/check.rs:2311 | `validate_sandbox_scope_leak` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; `WatAST::List(items, _)` guard applies only to sandbox-primitive detection and inner-scope leak analysis. |
| src/check.rs:2392 | `check_calls_for_sandbox_leak` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; `WatAST::List(items, _)` guard applies only to call-head leak detection and sandbox-boundary guard. |
| src/check.rs:2547-2568 | `walk_for_arc170_legacy` | Walker (already migrated) | Explicit List and Vector arms covering all practically reachable compound shapes; StructPattern contains only bare Symbols (field names), never legacy keyword paths, so the arc 170 legacy detection cannot miss anything; classified as already-correct per EXPECTATIONS § 4. |
| src/check.rs:2754-2770 | `walk_for_bare_primitives` | Walker (pending migration) | Has explicit List and Vector arms but lacks StructPattern; BRIEF explicitly designates this function as the known pending walker (stone δ-bare-primitives). |
| src/check.rs:2970-2992 | `walk_for_legacy_stream` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; keyword-prefix detection applies only to `WatAST::Keyword` nodes. |
| src/check.rs:3009-3030 | `walk_for_legacy_telemetry_service` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; legacy-path detection applies only to `WatAST::Keyword` nodes. |
| src/check.rs:3054-3083 | `walk_for_legacy_lru_cache_service` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; legacy-path detection applies only to `WatAST::Keyword` nodes. |
| src/check.rs:3112-3151 | `walk_for_legacy_kernel_queue` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; legacy-name detection applies only to `WatAST::Keyword` nodes. |
| src/check.rs:3170-3192 | `walk_for_bare_legacy_console` | Walker (pending migration) | Has explicit List and Vector arms but lacks StructPattern; `:wat::console::` prefixed keywords cannot appear inside StructPattern in practice, but the structural gap makes it an unmigrated walker per arc 212 doctrine. |
| src/check.rs:3218-3241 | `walk_for_def_restricted_call` | Walker (pending migration) | Has explicit List and Vector arms but lacks StructPattern; call heads are always in List position so the practical coverage is complete, but structural gap makes it a pending migration site. |
| src/check.rs:3280-3346 | `walk_for_deadlock` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; `WatAST::List(items, _)` guard applies only to sandbox-boundary and let-form scope construction. |
| src/check.rs:3415 | `parse_binding_for_typed_check` | Leaf-decomposition | Dead-code helper (arc 133 retired); extracts one `(name :type)` pair binding shape; no recursion into children. |
| src/check.rs:3619 | `collect_process_calls` | Walker (sharpening target) | Inscribed TEMPORARY List-only; migration to children() produces false positives because the walker's rule has no scope-boundary awareness (inner-let scopes would be conflated with outer scope tracking); stone δ-process-scope sharpens the rule. |
| src/check.rs:3695-3725 | `collect_process_stdin_and_joins` | Walker (pending migration) | Has explicit List and Vector arms but lacks StructPattern; recurses into both List and Vector children to find `Process/stdin` and `Process/join-result` calls. |
| src/check.rs:3738-3761 | `contains_join_on_thread` | Walker (already migrated) | Uses `node.children().iter().any(...)` for generic recursion; `WatAST::List(items, _)` guard applies only to `Thread/join-result` head detection. |
| src/check.rs:3830-3992 | `walk_for_pair_deadlock` | Walker (already migrated) | Uses `for child in node.children()` for generic recursion; `WatAST::List(items, _)` guard applies only to sandbox-boundary, let-form scope construction, comm-primitive exclusion, and call-site structural rule. |
| src/check.rs:4007 | `check_call_for_pair_deadlock` | Leaf-decomposition | Decomposes one `WatAST::List` call form to classify its Symbol arguments; no recursion into children — called from `walk_for_pair_deadlock` after it lands on a call site. |
| src/check.rs:4075 | `trace_to_pair_anchor` | Leaf-decomposition | Traces a binding-chain from a name to a make-channel anchor by matching specific `WatAST::List` shapes (first/second/make-channel); no generic recursion. |
| src/check.rs:4125 | `extend_pair_scope_with_tuple_destructure` | Leaf-decomposition | Decomposes one let-binding shape to extract tuple-destructure entries; matches specific `WatAST::List` and `WatAST::Vector` positions; no generic recursion. |
| src/check.rs:4221 | `derive_type_ann_from_rhs` | Leaf-decomposition | Matches specific `WatAST::List` shapes (channel-creating calls) to infer type annotations; no generic recursion. |
| src/check.rs:4279-4296 | `parse_binding_for_pair_check` | Leaf-decomposition | Extracts `(name, type-ann, rhs)` from one let-binding `WatAST::List`; no recursion. |
| src/check.rs:4391 | `check_form` (check.rs version) | Leaf-decomposition | Thin wrapper that calls `infer`; does not pattern-match on WatAST itself. |
| src/check.rs:4488-4524 | `infer` | Leaf-decomposition | Top-level type-inference dispatcher; matches each WatAST variant to infer its type, delegating List to `infer_list`; not a walker (does not recurse generically — delegates per-variant). |
| src/check.rs:4536+ | `infer_some_constructor` | Leaf-decomposition | Processes specific arity/shape of one constructor call; no generic recursion. |
| src/check.rs:4583+ | `infer_ok_constructor` | Leaf-decomposition | Same pattern as `infer_some_constructor`. |
| src/check.rs:4631+ | `infer_err_constructor` | Leaf-decomposition | Same pattern as `infer_some_constructor`. |
| src/check.rs:4679+ | `infer_list` | Leaf-decomposition | Dispatches on the List head keyword to the correct inference handler; no generic recursion. |
| src/check.rs:5669+ | `infer_match` | Leaf-decomposition | Decomposes `(:wat::core::match ...)` arms and patterns; matches specific `WatAST::List` shapes for arm and pattern positions; no generic recursion. |
| src/check.rs:6032+ | `detect_match_shape` | Leaf-decomposition | Classifies match scrutinee shapes; matches specific `WatAST::List` patterns; no generic recursion. |
| src/check.rs:6110+ | `pattern_coverage` | Leaf-decomposition | Matches arm-pattern shapes (Some, Ok, Err, enum variants, wildcards); matches specific `WatAST::List` heads; no generic recursion. |
| src/check.rs:6498+ | `check_subpattern` | Leaf-decomposition | Validates sub-pattern structure within a match arm; matches specific `WatAST::List` positions; no generic recursion. |
| src/check.rs:6922 | `ast_variant_name_check` | Leaf-decomposition | Returns a string name for each `WatAST` variant including List/Vector/StructPattern; pure classification, no recursion. |
| src/check.rs:6948+ | `infer_if` | Leaf-decomposition | Infers types for `(:wat::core::if ...)` branches by calling `infer` on each sub-form; dispatches List arm via `infer`; no generic recursion itself. |
| src/check.rs:7076+ | `infer_do` | Leaf-decomposition | Decomposes `(:wat::core::do ...)` body forms; iterates sub-forms and calls `infer`; no generic recursion. |
| src/check.rs:7114+ | `infer_cond` | Leaf-decomposition | Decomposes `(:wat::core::cond ...)` clauses; matches `WatAST::List` clause shapes; no generic recursion. |
| src/check.rs:7175-7208 | `infer_let` (inner binding helpers) | Leaf-decomposition | Decomposes flat-Vector let-binding pairs and delegates to `process_let_binding`; matches specific shapes; not a generic walker. |
| src/check.rs:7287+ | `infer_let` | Leaf-decomposition | Processes `(:wat::core::let ...)` bindings and body; matches specific `WatAST::Vector` and `WatAST::List` shapes for binding desugaring; not a generic walker. |
| src/check.rs:7305-7359 | `infer_let` (StructPattern binder section) | Leaf-decomposition | (Continuation of `infer_let`) StructPattern binder decomposition for struct-destructure let bindings; single-shape handler, no recursion. |
| src/check.rs:7445+ | `infer_def` | Leaf-decomposition | Processes `(:wat::core::define ...)` form; matches specific List positions; no generic recursion. |
| src/check.rs:7566+ | `infer_def_restricted` | Leaf-decomposition | Processes `(:wat::core::def-restricted ...)` form; matches specific shapes; no generic recursion. |
| src/check.rs:7628-7779 | `extract_prefix_vec` / `extract_redef_setter` | Leaf-decomposition | Small helpers that extract specific slot values from known List/Vector shapes; no recursion. |
| src/check.rs:7762+ | `collect_and_register_splice_defs` | Leaf-decomposition | Top-level wrapper; delegates to `collect_splice_defs_ctx`. |
| src/check.rs:7771+ | `collect_splice_defs_ctx` | Leaf-decomposition | Processes top-level `def`, `def-restricted`, `do`, `let` forms only; matches the outer List head then recurses only into `do`/`let` body children by explicit keyword match, not generic recursion. |
| src/check.rs:7865+ | `extract_def_binding` | Leaf-decomposition | Extracts `(name, type, span)` from one `(:wat::core::def ...)` form; no recursion. |
| src/check.rs:7903+ | `extract_def_restricted_binding` | Leaf-decomposition | Extracts `(name, type, span, prefixes)` from one `(:wat::core::def-restricted ...)` form; no recursion. |
| src/check.rs:7945+ | `extract_prefix_vec` | Leaf-decomposition | Extracts keyword prefixes from one `WatAST::Vector`; no recursion. |
| src/check.rs:7968+ | `extract_redef_setter` | Leaf-decomposition | Checks if a `WatAST::List` form is a `set-redef!` call; no recursion. |
| src/check.rs:8019+ | `validate_def_positions_in_forms` | Leaf-decomposition | Validates that `def` only appears at top-level positions in a forms block; matches List shapes; no generic recursion. |
| src/check.rs:8140+ | `infer_try` | Leaf-decomposition | Decomposes `(:wat::core::try ...)` forms; matches specific shapes; no generic recursion. |
| src/check.rs:8250+ | `infer_option_try` | Leaf-decomposition | Decomposes `(:wat::core::Option/try ...)` forms; matches specific shapes; no generic recursion. |
| src/check.rs:8354+ | `infer_option_expect` | Leaf-decomposition | Decomposes `(:wat::core::Option/expect ...)` forms; matches specific shapes; no generic recursion. |
| src/check.rs:8455+ | `infer_result_expect` | Leaf-decomposition | Decomposes `(:wat::core::Result/expect ...)` forms; matches specific shapes; no generic recursion. |
| src/check.rs:8566+ | `infer_kernel_readln` | Leaf-decomposition | Decomposes `(:wat::kernel::readln ...)` forms; matches specific shapes; no generic recursion. |
| src/check.rs:8636+ | `find_binding_span` | Leaf-decomposition | Iterates a flat list of binding `WatAST` nodes looking for a specific name; matches `WatAST::List` two-element pairs; no generic recursion into children. |
| src/check.rs:8682+ | `check_let_for_scope_deadlock_inferred` | Leaf-decomposition | Reads inferred type maps to detect sibling-deadlock; matches `WatAST::List` binding shapes for name extraction; not a generic walker. |
| src/check.rs:8712-8738 | `check_let_for_scope_deadlock_inferred` (binding decompose section) | Leaf-decomposition | Decomposes flat-Vector or List-of-pairs binding shapes; extracts field names from Vector/List/StructPattern binders; single-purpose, no generic recursion. |
| src/check.rs:8855+ | `sender_originates_from_thread_pipe` | Leaf-decomposition | Iterates binding `WatAST` nodes to find a specific name's RHS; matches `WatAST::List` binding pairs; no generic recursion. |
| src/check.rs:8882+ | `rhs_is_thread_input_extractor` | Leaf-decomposition | Matches one `WatAST::List` shape to test for `Thread/input` or `Process/input` call; no recursion. |
| src/check.rs:8913+ | `spawn_thread_fn_body_has_no_recv` | Leaf-decomposition | Iterates binding nodes to find a spawn call and inspect its inline fn body using `node_contains_recv`; matches specific `WatAST::List` shapes; no generic recursion itself. |
| src/check.rs:8941+ | `rhs_spawn_fn_has_no_recv` | Leaf-decomposition | Matches specific `WatAST::List` shapes (spawn call, inline fn); delegates to `node_contains_recv` for body walk; not a generic walker itself. |
| src/check.rs:8975+ | `node_contains_recv` | Walker (already migrated) | Uses `node.children().iter().any(...)` for generic recursion; `WatAST::List(items, _)` guard applies only to recv/try-recv/select head detection. |
| src/check.rs:9317-9319 | `infer_spawn` | Leaf-decomposition | Error-message helper inside `infer_spawn` that names the ast variant; pattern-matches all three shapes for diagnostic string; no recursion. |
| src/check.rs:9359+ | `process_let_binding` | Leaf-decomposition | Decomposes one `(binder rhs)` pair into its binding shape (Symbol, Vector, StructPattern); matches specific shapes for scope extension; not a generic walker. |
| src/check.rs:9539+ | `infer_hashset_constructor` | Leaf-decomposition | Processes `(:wat::core::HashSet ...)` forms; matches specific shapes; no generic recursion. |
| src/check.rs:9621+ | `infer_comparison` | Leaf-decomposition | Processes comparison call forms; no WatAST::List direct match (operates on already-sliced args). |
| src/check.rs:9690+ | `infer_arithmetic` | Leaf-decomposition | Processes arithmetic call forms; operates on already-sliced args. |
| src/check.rs:9788+ | `infer_polymorphic_time_arith` | Leaf-decomposition | Processes time-arithmetic forms; no generic recursion. |
| src/check.rs:9893+ | `infer_form_matches` | Leaf-decomposition | Decomposes `(:wat::form::matches? subject pattern)` to check the pattern shape; matches one `WatAST::List` for pattern; no generic recursion. |
| src/check.rs:9922 | `infer_list` (inner section) | Leaf-decomposition | (Part of `infer_list`) Matches specific list-head keywords; delegates to specialized infer_* helpers; not a generic walker. |
| src/check.rs:10990 | `infer_fn` (body reconstruction) | Leaf-decomposition | (Part of `infer_fn`) Synthesizes a `WatAST::List` for the do-body wrapper; no pattern-match on children. |
| src/check.rs:11054 | `parse_fn_signature_for_check` | Leaf-decomposition | Extracts fn parameter names and types from one `WatAST::Vector`; no generic recursion. |
| src/check.rs:11108 | `parse_fn_signature_for_check_diag` | Leaf-decomposition | Diagnostic variant of the above; same shape; no generic recursion. |
| src/closure_extract.rs:499-646 | `walk_free_symbols` | Walker (already migrated) | Explicit arms for List (binding-form dispatch), Vector, and StructPattern; all three compound shapes recurse into children; classified as already-correct per EXPECTATIONS § 4 (explicit all-variant coverage). |
| src/closure_extract.rs:651+ | `walk_let_form` | Leaf-decomposition | Decomposes `(:wat::core::let [...] body)` bindings sequentially; matches specific Vector/StructPattern binder shapes; no generic recursion — delegates body to `walk_free_symbols`. |
| src/closure_extract.rs:710+ | `walk_fn_form` | Leaf-decomposition | Decomposes fn-signature Vector to extract param names; matches specific positions; no generic recursion. |
| src/closure_extract.rs:745+ | `walk_define_form` | Leaf-decomposition | Decomposes `(:wat::core::define ...)` signature shapes; matches specific positions; no generic recursion. |
| src/closure_extract.rs:802+ | `walk_struct_form` | Leaf-decomposition | Decomposes struct declaration fields; matches specific `WatAST::List` field shapes; no generic recursion. |
| src/closure_extract.rs:836+ | `walk_enum_form` | Leaf-decomposition | Decomposes enum declaration variants; matches specific shapes; no generic recursion. |
| src/closure_extract.rs:905+ | `walk_match_form` | Leaf-decomposition | Decomposes `(:wat::core::match ...)` arms and patterns; matches specific `WatAST::List` arm shapes; no generic recursion. |
| src/closure_extract.rs:955+ | `collect_pattern_bindings` | Leaf-decomposition | Extracts binding names from match arm patterns; matches specific `WatAST::List` and `WatAST::Vector` shapes; no generic recursion. |
| src/closure_extract.rs:1799 | `head_keyword` | Leaf-decomposition | Returns the head keyword string of a `WatAST::List`; no recursion. |
| src/closure_extract.rs:1834+ | `split_body_prelude` | Leaf-decomposition | Splits a `WatAST::List` do-body into prelude and tail; no recursion. |
| src/closure_extract.rs:1908+ | `rewrite_with_scope` | Walker (already migrated) | Explicit arms for List (binding-form dispatch), Vector, and StructPattern; all three compound shapes handled with correct semantics; classified as already-correct per EXPECTATIONS § 4. |
| src/closure_extract.rs:1963+ | `rewrite_let` | Leaf-decomposition | Rewrites let-binding positions under a new scope; matches `WatAST::Vector` inner and Vector/StructPattern binder shapes; not a generic walker (processes one structural form). |
| src/closure_extract.rs:2028+ | `rewrite_fn` | Leaf-decomposition | Rewrites fn-literal body under a new scope; matches `WatAST::Vector` args-vec; not a generic walker. |
| src/closure_extract.rs:1450-1706 | `encode_value_to_ast` / `encode_value_with_path` / related | Leaf-decomposition | AST construction helpers that produce `WatAST::List(...)` nodes; no pattern-matching on WatAST input. |
| src/closure_extract.rs:2067-2353 | `capture_define_form` / `type_def_to_ast` / `function_to_define_form` / related | Leaf-decomposition | AST construction helpers; produce `WatAST::List(...)` and `WatAST::Vector(...)` output nodes; no pattern-matching on children. |
| src/dispatch.rs:283 | `is_define_dispatch_form` | Leaf-decomposition | Matches one `WatAST::List` shape to test if a form is a `define-dispatch`; no recursion. |
| src/dispatch.rs:303 | `parse_define_dispatch_form` | Leaf-decomposition | Decomposes one `(:wat::core::define-dispatch ...)` form and its arms; matches specific `WatAST::List` positions; no generic recursion. |
| src/dispatch.rs:372 | `parse_arm` | Leaf-decomposition | Decomposes one dispatch arm `((pattern...) impl)` list; no recursion. |
| src/dispatch.rs:394 | `parse_arm` (pattern section) | Leaf-decomposition | (Continuation of `parse_arm`) Matches one `WatAST::List` for the pattern sub-list; no recursion. |
| src/form_match.rs:167 | `classify_clause` | Leaf-decomposition | Decomposes one `WatAST::List` clause form to classify it as Eq/Compare/And/Or/Not/Where; no recursion. |
| src/form_match.rs:259 | `classify_clause` (test helper) | Leaf-decomposition | Test helper that constructs `WatAST::List`; no pattern-matching. |
| src/freeze.rs:1335 | `refuse_mutation_forms` | Walker (pending migration) | Recurses into `WatAST::List(items, _)` children only (via `for child in items`); no Vector arm, no StructPattern arm, no `children()` call; will silently miss mutation-form detection inside bracketed forms. |
| src/hash.rs:160-184 | `write_canonical_wat` | Walker (already migrated) | Explicit arms for List, Vector, and StructPattern, each recursing into children with `for child in items { write_canonical_wat(child, out); }`; all three compound shapes covered; classified as already-correct per EXPECTATIONS § 4. |
| src/load.rs:492 | `match_load_form` | Leaf-decomposition | Decomposes one `WatAST::List` to detect and classify a load form; no recursion. |
| src/load.rs:757 | `scan_for_setter` | Walker (pending migration) | Has explicit List and Vector arms but lacks StructPattern; recurses into children looking for `(:wat::config::set-*! ...)` heads; structural gap makes it a pending migration site (setter detection inside StructPattern is not reachable in practice but doctrine requires children()). |
| src/load.rs:785-795 | `variant_name` (load.rs) | Leaf-decomposition | Returns a string name for each `WatAST` variant; pure classification, no recursion. |
| src/load.rs:793-794 | `variant_name` (load.rs, List/Vector arms) | Leaf-decomposition | (Same function) List and Vector arms of the variant-name classifier. |
| src/load.rs:1114 | (within `process_forms` or `process_single_load`) | Leaf-decomposition | Matches specific `WatAST::List` shapes for load form detection; no generic recursion. |
| src/load.rs:1502 | (within a nested form-processor) | Leaf-decomposition | `if let WatAST::List(items, _) = f` guard to extract form items at a known position; no generic recursion. |
| src/lower.rs:163 | `lower` | Leaf-decomposition | Top-level dispatcher that routes `WatAST::List` to `lower_call`, returns errors for all other shapes including Vector and StructPattern; no generic recursion. |
| src/lower.rs:178 | `lower` (Vector arm) | Leaf-decomposition | (Same function) Returns `LowerError::UnsupportedForm` for Vector; no recursion. |
| src/lower.rs:186 | `lower` (StructPattern arm) | Leaf-decomposition | (Same function) Returns `LowerError::UnsupportedForm` for StructPattern; no recursion. |
| src/lower.rs:264 | `lower_bundle` | Leaf-decomposition | Matches one `WatAST::List(items, _)` arg shape to extract the `(:wat::core::Vector :T item...)` bundle arg; no generic recursion. |
| src/macros.rs:261 | `expand_once` | Leaf-decomposition | Matches one `WatAST::List` to detect a macro call head; no recursion (single-level check only, per macroexpand-1 contract). |
| src/macros.rs:311-316 | `is_defmacro_form` | Leaf-decomposition | Matches one `WatAST::List` to test the defmacro head keyword; no recursion. |
| src/macros.rs:322 | `parse_defmacro_form` | Leaf-decomposition | Decomposes one `(:wat::core::defmacro ...)` form; no recursion. |
| src/macros.rs:360 | `parse_defmacro_signature` | Leaf-decomposition | Decomposes the defmacro signature list to extract name and params; matches specific List shapes; no generic recursion. |
| src/macros.rs:408 | `parse_defmacro_signature` (param section) | Leaf-decomposition | (Continuation) Decomposes one `(param :AST<T>)` pair List; no recursion. |
| src/macros.rs:499-576 | `expand_form` | Walker (already migrated) | Explicit List and Vector arms, each recursing into children; StructPattern is not a macro-expandable context (no unquote sites); the `other => Ok(other)` leaf arm handles it; classified as already-correct per EXPECTATIONS § 4 (explicit Vector arm IS the equivalent). |
| src/macros.rs:629-631 | `construct_keyword_of` (inner `ast_kind`) | Leaf-decomposition | Names each `WatAST` variant for error messages; pure classification, no recursion. |
| src/macros.rs:755 | `expand_template` (or enclosing `expand_macro_call`) | Leaf-decomposition | Matches one `WatAST::List` of length 2 for the pattern `(head arg)` unquote shape; no recursion. |
| src/macros.rs:800-991 | `walk_template` | Walker (already migrated) | Explicit List and Vector arms, each recursing into children with unquote/splice dispatch; `other => Ok(other.clone())` arm handles StructPattern as a leaf (correct: StructPattern contains only Symbols, never unquote forms); classified as already-correct per EXPECTATIONS § 4. |
| src/macros.rs:1012-1040 | `substitute_bindings` | Walker (already migrated) | Explicit List and Vector arms, each recursing into children; `other => other.clone()` handles StructPattern (no macro-parameter symbols inside StructPattern); classified as already-correct per EXPECTATIONS § 4. |
| src/macros.rs:1126-1131 | `unquote_argument` | Leaf-decomposition | Matches `WatAST::List` and `WatAST::Vector` shapes to extract items for eval-time unquote evaluation; no generic recursion. |
| src/macros.rs:1141 | `unquote_argument` (construction) | Leaf-decomposition | Constructs `WatAST::List` output; no pattern-matching on children. |
| src/macros.rs:1191 | `splice_argument` | Leaf-decomposition | Matches one `WatAST::List` to extract items for splicing; no generic recursion. |
| src/macros.rs:1212-1214 | `ast_variant_name` | Leaf-decomposition | Names each `WatAST` variant for error messages; pure classification, no recursion. |
| src/macros.rs:1263-2011 | Various macros.rs leaf helpers and `#[cfg(test)]` functions | Leaf-decomposition | These sites are either: (a) test assertion helpers inside `#[cfg(test)]` that construct specific `WatAST::List` shapes for structural assertions, or (b) defmacro parsing helpers that decompose specific known forms. None recurse generically through children. |
| src/parser.rs:189 | (parser construction) | Leaf-decomposition | Parser constructs `WatAST::List(list, span)` for parsed `(...)` forms; no pattern-matching on children. |
| src/parser.rs:197 | (parser construction) | Leaf-decomposition | Parser constructs `WatAST::Vector(items, span)` for parsed `[...]` forms; no pattern-matching on children. |
| src/parser.rs:230 | (parser construction) | Leaf-decomposition | Parser constructs `WatAST::StructPattern(items, span)` for parsed `{...}` forms; no pattern-matching on children. |
| src/parser.rs:256 | (parser construction) | Leaf-decomposition | Parser constructs `WatAST::List(...)` for spliced forms; no pattern-matching on children. |
| src/parser.rs:380-382 | `parse_form` (error arm) | Leaf-decomposition | Names `WatAST` variants for parse-error messages; pure classification, no recursion. |
| src/parser.rs:498 | (post-parse validator) | Leaf-decomposition | Matches one `WatAST::List` to validate that the top-level form has a Keyword head; no generic recursion. |
| src/config.rs:518 | `setter_head_of` | Leaf-decomposition | Extracts the head keyword from one `WatAST::List`; no recursion. |
| src/config.rs:530 | `setter_args_of` | Leaf-decomposition | Returns the args slice from one `WatAST::List`; no recursion. |
| src/config.rs:601-603 | `variant_name` (config.rs) | Leaf-decomposition | Names each `WatAST` variant for error messages; pure classification, no recursion. |
| src/runtime.rs:1731 | `eval_do` (or enclosing special-form handler) | Leaf-decomposition | Matches `WatAST::List(ref do_items, _)` to detect a do-form; no generic recursion as a walker. |
| src/runtime.rs:1784 | (same area, second match) | Leaf-decomposition | (Continuation of same handler) Second `WatAST::List` guard for do-form detection; no generic recursion. |
| src/runtime.rs:1936-2351 | Various `runtime.rs` form constructors | Leaf-decomposition | These sites construct `WatAST::List(...)`, `WatAST::Vector(...)` nodes as output; they are AST-building helpers (synthesize define forms, fn forms, accessor bodies, etc.); no pattern-matching on children. |
| src/runtime.rs:2254 | (extract from define form) | Leaf-decomposition | Matches one `WatAST::List` to extract a specific slot (items[0]..); no generic recursion. |
| src/runtime.rs:2389 | (extract args vector from define) | Leaf-decomposition | Matches `WatAST::Vector` to extract items from a fn-sig position; no recursion. |
| src/runtime.rs:2425-2480 | Various define-form extraction helpers | Leaf-decomposition | Each matches one specific `WatAST::List` shape to extract named slots (param, body, etc.); no generic recursion. |
| src/runtime.rs:2543 | (restricted item extraction) | Leaf-decomposition | Matches `Some(WatAST::List(restricted_items, _))` at a specific index; no recursion. |
| src/runtime.rs:2577 | (public item extraction) | Leaf-decomposition | Matches `Some(WatAST::List(public_items, _))` at a specific index; no recursion. |
| src/runtime.rs:2617 | (field handler in struct define) | Leaf-decomposition | Matches `WatAST::List(field_items, _)` to process one field declaration; no generic recursion. |
| src/runtime.rs:2679-2882 | Various runtime form extractors | Leaf-decomposition | Each matches one specific `WatAST::List` or `WatAST::Vector` shape to extract named slots from define/enum/fn forms; no generic recursion. |
| src/runtime.rs:2984 | (nested-let scan within do-form handler) | Leaf-decomposition | `WatAST::List` guard inside a pre-registration helper to detect nested define forms; no generic walker. |
| src/runtime.rs:3063 | (similar nested-define scan) | Leaf-decomposition | Second occurrence of the same pattern; same classification. |
| src/runtime.rs:3098-3157 | (form-slicing helpers) | Leaf-decomposition | Match specific `WatAST::List` positions to extract items by index; no generic recursion. |
| src/runtime.rs:3218 | `eval_match` (or enclosing handler) | Leaf-decomposition | Matches `WatAST::Keyword` or `WatAST::List` for a specific pattern position inside match evaluation; no generic recursion. |
| src/runtime.rs:3269-3431 | `eval_let` / `eval_match` related | Leaf-decomposition | Decompose specific arm/binding shapes; match `WatAST::List` at known positions; no generic recursion. |
| src/runtime.rs:3617-3656 | `eval_list` (head dispatch area) | Leaf-decomposition | Matches `WatAST::List` at the head position of a call form; delegates to `eval_list`; not a generic walker. |
| src/runtime.rs:3780 | (match-arm body destructure) | Leaf-decomposition | Matches `WatAST::Vector` for tuple-destructure binder in let/match; no generic recursion. |
| src/runtime.rs:3898 | (extract list items at known position) | Leaf-decomposition | Matches one `WatAST::List` to extract items from a specific structural position; no generic recursion. |
| src/runtime.rs:3933-4037 | `eval` | Leaf-decomposition | Top-level eval dispatcher; routes `WatAST::List` to `eval_list`, returns errors for Vector and StructPattern at value position; not a generic walker. |
| src/runtime.rs:4040+ | `eval_list` | Leaf-decomposition | Dispatches on the list head to the correct evaluation handler; no generic recursion (delegates per head keyword). |
| src/runtime.rs:5258-5296 | (quasiquote body builders / vector extraction) | Leaf-decomposition | Constructs `WatAST::List` output or matches `WatAST::Vector` at a specific slot; no generic recursion. |
| src/runtime.rs:5442-5741 | (fn/struct match eval helpers) | Leaf-decomposition | Decompose fn-literal body, match patterns, struct-destructure binders; match specific shapes at known positions; no generic recursion. |
| src/runtime.rs:5879-6038 | (let binding iteration helpers) | Leaf-decomposition | Iterate flat-Vector bindings and decompose each `WatAST::List` pair or `WatAST::Vector` binder; no generic walker. |
| src/runtime.rs:9019 | `walk_quasiquote` | Walker (already migrated) | Explicit List arm (unquote/quasiquote nesting dispatch + plain list child walk) and explicit Vector arm (child walk); StructPattern treated as leaf with code comment ("admits only bare Symbols at parse time; cannot contain unquotes"); classified as already-correct per EXPECTATIONS § 4. |
| src/runtime.rs:9225-9657 | Various runtime AST construction helpers | Leaf-decomposition | These produce `WatAST::List(...)` and `WatAST::Vector(...)` nodes as output (synthesize define/fn/struct/enum forms at runtime); no pattern-matching on WatAST input children. |
| src/runtime.rs:9918 | (sentinel construction) | Leaf-decomposition | Constructs a `WatAST::List` sentinel node; no pattern-matching on children. |
| src/runtime.rs:10604 | (keyword-dispatch head check) | Leaf-decomposition | Matches `WatAST::List(items, _) if !items.is_empty()` to extract the call head keyword for dispatch; no generic recursion. |
| src/runtime.rs:11886-12289 | Various runtime dispatchers and form-readers | Leaf-decomposition | Each matches one specific `WatAST::List` shape to extract items or head keywords for a specific dispatch path; no generic recursion. |
| src/runtime.rs:12281-12313 | `watast_to_holon` | Walker (already migrated) | Explicit arms for List, Vector, and StructPattern, each recursing into children via `items.iter().map(watast_to_holon)`; all three compound shapes covered; classified as already-correct per EXPECTATIONS § 4. |
| src/runtime.rs:13085-13140 | `holon_ast_to_watast` (or `encode_holon_to_watast`) | Leaf-decomposition | Constructs `WatAST::List(...)` nodes from `HolonAST` variants; the input is `HolonAST`, not `WatAST`; no WatAST pattern-matching. |
| src/runtime.rs:16007-16009 | `ast_variant_name` (runtime.rs) | Leaf-decomposition | Names each `WatAST` variant for error messages; pure classification, no recursion. |
| src/runtime.rs:19527-19573 | `step_form` | Leaf-decomposition | Dispatches to `step_list` for List, returns `NoStepRule` errors for Vector and StructPattern; not a generic walker. |
| src/runtime.rs:19590-19724 | `try_recognize_holon_value` | Leaf-decomposition | Matches `WatAST::List` to test if it is a holon-value constructor shape; decomposes specific List shapes (`:wat::holon::*` head checks); no generic recursion into arbitrary children. |
| src/runtime.rs:19724-19728 | `try_recognize_holon_value` (Vector/StructPattern) | Leaf-decomposition | Returns `None` for Vector and StructPattern (they are not holon-value shapes); no recursion. |
| src/runtime.rs:19851-20568 | Various `step_list` and step-sub-helpers | Leaf-decomposition | Decompose specific special-form shapes for step reduction; match `WatAST::List`, `WatAST::Vector`, `WatAST::StructPattern` at known positions within let/fn/match/do forms; not generic walkers. |
| src/runtime.rs:20499-20509 | `step_subst_rename` (or equivalent) | Leaf-decomposition | Matches List and Vector to substitute symbols in one node; explicit List and Vector arms; StructPattern treated as pass-through; similar to substitute_bindings but step-level. |
| src/runtime.rs:20995 | (final holon-step helper) | Leaf-decomposition | Matches `WatAST::List` to decompose a holon constructor call at step time; no generic recursion. |
| src/runtime.rs:22130 | (large dispatch arm in eval/step) | Leaf-decomposition | Matches `WatAST::List` at a specific position within a larger dispatch; no generic recursion. |
| src/runtime.rs:26422 | (StepValue arm) | Leaf-decomposition | Matches `StepValue::Next(WatAST::List(_, span))` to extract the span from a step result; no recursion. |
| src/test_runner.rs:558 | `source_has_config_setter` | Leaf-decomposition | Matches one `WatAST::List` per top-level form to check its head keyword; no recursion into children (uses `forms.iter().any(...)` over the top-level list, not recursive descent). |
| src/types.rs:1452 | `splice_type_decls_user` | Leaf-decomposition | Decomposes one `WatAST::List` form to detect `do`/`let` heads and splice type declarations from their bodies; recurses ONLY into `do`/`let` body children by explicit keyword match (not generic recursion); a structural specialization, not a generic walker. |
| src/types.rs:1515 | `splice_type_decls_stdlib` | Leaf-decomposition | Stdlib variant of `splice_type_decls_user`; same classification. |
| src/types.rs:1569 | `classify_type_decl` | Leaf-decomposition | Matches one `WatAST::List` head to classify it as struct/enum/newtype/typealias; no recursion. |
| src/types.rs:1595 | `parse_type_decl` | Leaf-decomposition | Extracts items from one `WatAST::List` type declaration form; no recursion. |
| src/types.rs:1677-1747 | `parse_struct_restricted` | Leaf-decomposition | Decomposes a four-slot struct-restricted form; matches specific `WatAST::Vector` and `WatAST::List` slot shapes; no generic recursion. |
| src/types.rs:1985 | `parse_field` | Leaf-decomposition | Decomposes one `(field-name :Type)` List into name + type; no recursion. |
| src/types.rs:2053 | `parse_enum_variant` | Leaf-decomposition | Decomposes one enum-variant form (keyword or tagged List); no recursion. |
| src/types.rs:2538-2540 | `ast_variant_name` (types.rs) | Leaf-decomposition | Names each `WatAST` variant for error messages; pure classification, no recursion. |
| crates/wat-macros/src/codegen.rs | (if any WatAST sites) | Leaf-decomposition | The `wat-macros` crate is a proc-macro crate operating on Rust TokenStream, not on WatAST; grep produced no `WatAST::List(` hits there. |

---

## Notes

### Surprises and calibration notes

**Counts vs prediction:** The EXPECTATIONS predicted ~80-120 inspected sites and ~60-100 Leaf-decompositions. The actual count of unique functions is toward the higher end (~110 unique function-level sites), with Leaf-decompositions dominating at ~80. Walker (already migrated) at 22 is higher than the predicted 12 because EXPECTATIONS only counted the 12 `children()` walkers but the explicit-all-arms walkers (`walk_free_symbols`, `rewrite_with_scope`, `watast_to_holon`, `write_canonical_wat`, `walk_quasiquote`, `walk_template`, `substitute_bindings`, `expand_form`, `walk_for_arc170_legacy`) also qualify as already-migrated per EXPECTATIONS § 4.

**Walker (pending migration) count: 5 vs predicted 1-3:** The BRIEF explicitly named only `walk_for_bare_primitives`. Four additional pending walkers surfaced:
1. `src/freeze.rs::refuse_mutation_forms` — List-only, no Vector arm, no StructPattern arm. Mutation-form detection inside bracketed `[...]` or `{...}` forms would be missed.
2. `src/load.rs::scan_for_setter` — List + Vector arms, no StructPattern arm.
3. `src/check.rs::walk_for_bare_legacy_console` — List + Vector arms, no StructPattern arm. (BRIEF listed as "already-correct" but lacks StructPattern; per strict arc 212 doctrine this is a structural gap.)
4. `src/check.rs::walk_for_def_restricted_call` — List + Vector arms, no StructPattern arm. (Same situation as above.)
5. `src/check.rs::collect_process_stdin_and_joins` — List + Vector arms, no StructPattern arm.

**Classification ambiguity note — the "already-correct" walkers with explicit List+Vector:** The BRIEF says `walk_for_bare_legacy_console` and `walk_for_def_restricted_call` are "already-correct walkers." The EXPECTATIONS § 4 says "explicit Vector arm IS the children() shape's manual equivalent." If the orchestrator treats "List + Vector = equivalent to children()" as the threshold for "already migrated," then items 3 and 4 above flip to Walker (already migrated). The catalog above marks items 3, 4, 5 as Walker (pending migration) based on strict structural gap (missing StructPattern arm) per arc 212 doctrine. The orchestrator decides whether the practical-unreachability of StructPattern in those contexts justifies the stricter classification.

**`refuse_mutation_forms` in `src/freeze.rs` (line 1335):** This is a previously-unaudited walker that the BRIEF did not pre-name. It has only a `WatAST::List` arm — no Vector, no StructPattern, no `children()`. A mutation-inducing head inside a let-binding vector (e.g., `[x (:wat::core::define ...)]`) would evade detection. This is the most structurally incomplete walker in the codebase.

**`collect_process_stdin_and_joins` in `src/check.rs` (around line 3689):** A previously-unaudited walker that was not in the BRIEF's pre-named list. It has explicit List + Vector arms but lacks StructPattern. It's the sibling to `collect_process_calls` (a sharpening target) but does not have the scope-boundary concern — it's a straightforward pending migration candidate.

**`walk_quasiquote` in runtime.rs (line 9019):** The BRIEF lists this as an "already-correct walker." The code comment at line 9071 confirms the design intent: "StructPattern admits only bare Symbols at parse time per `src/ast.rs:99`; cannot contain unquotes; treated as leaf." This is correct and justified. Classified as Walker (already migrated).

**`watast_to_holon` in runtime.rs (line 12281):** Not in the BRIEF's pre-named lists but clearly a walker (recurses into all three compound shapes). Classified as Walker (already migrated) via explicit all-variant coverage.

**`write_canonical_wat` in hash.rs (line 124):** Not in the BRIEF's pre-named lists. It's a serializer walker that explicitly handles List, Vector, and StructPattern with distinct tag bytes and recursive child serialization. Classified as Walker (already migrated).

**The crates (`crates/*/src/`):** No `WatAST::List(` or `WatAST::Vector(` or `WatAST::StructPattern(` hits appeared in `crates/wat-cli/src/`, `crates/wat-edn/src/`, `crates/wat-macros/src/`, `crates/wat-sqlite/src/`, `crates/wat-telemetry/src/`, `crates/wat-holon-lru/src/`, `crates/wat-lru/src/`, or `crates/wat-telemetry-sqlite/src/`. WatAST is a `wat-rs` core type; the crates operate on their own domain types, not directly on WatAST. All pattern-match sites are confined to `src/*.rs`.

### Summary for orchestrator — queuing candidates

Stones to queue based on this audit:
- **δ-bare-primitives** — `src/check.rs::walk_for_bare_primitives` (BRIEF-named, children() migration)
- **δ-comm-positions** — `src/check.rs::validate_comm_positions` (sharpening target, inscribed)
- **δ-process-scope** — `src/check.rs::collect_process_calls` (sharpening target, inscribed)
- **δ-refuse-mutation** — `src/freeze.rs::refuse_mutation_forms` (newly surfaced, List-only)
- **δ-scan-setter** — `src/load.rs::scan_for_setter` (newly surfaced, List+Vector, no StructPattern)
- **δ-process-stdin-joins** — `src/check.rs::collect_process_stdin_and_joins` (newly surfaced, List+Vector, no StructPattern)
- **δ-console / δ-def-restricted** — `walk_for_bare_legacy_console` + `walk_for_def_restricted_call` — pending only if orchestrator decides strict StructPattern gap matters (see ambiguity note above)
