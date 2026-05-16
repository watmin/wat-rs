# Arc 198 SCORE — `:wat::core::def-restricted` substrate primitive + `:wat::core::defn-restricted` defmacro sugar

**BRIEF:** `BRIEF.md`
**EXPECTATIONS:** `EXPECTATIONS.md`

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `:wat::core::def-restricted` substrate primitive defined (parser + AST + eval + CheckEnv storage) | **YES** | Registry entry in `src/special_forms.rs` (insert at the comment block right after `:wat::core::def`); `infer_def_restricted` arm in `src/check.rs` (called from `infer_list` next to `:wat::core::def` arm); `extract_def_restricted_binding` + `extract_prefix_vec` helpers in `src/check.rs`; `CheckEnv.defined_value_restrictions: HashMap<String, Vec<String>>` field + `get_defined_value_restriction` / `register_defined_value_restriction` methods; `SymbolTable.defined_value_restrictions` field in `src/runtime.rs` (mirrored into CheckEnv at `from_symbols`); `try_parse_fn_shape_def_restricted` for fn-shape pre-registration; `register_defines` arm + `preregister_fn_defs_in_do` / `preregister_fn_defs_in_let` arms; `register_runtime_defs_form` arm; `dispatch_keyword_head` arm returning `DeclarationInExpressionPosition`; `is_mutation_form` + `is_declaration_form` extensions in `src/freeze.rs`. |
| B | Walker check for restricted caller-namespace defined + hooked into `check_program` | **YES** | `validate_def_restricted_caller_namespace` work via `walk_for_def_restricted_call` in `src/check.rs` (placed directly after Stone B's `walk_for_join_result_call`); `caller_matches_prefix_list` helper next to it; hooked into `check_program` at the loop right after Stone B's hook — `for (name, func) in sym.functions.iter() { walk_for_def_restricted_call(&func.body, name, &env, &mut errors); }`. New `CheckError::DefRestrictedCallerNotAllowed { callee, enclosing_fn, prefixes, span }` variant in `src/check.rs` with Display + Diagnostic impls (placed adjacent to `JoinResultUserNamespace` variant + impls). |
| C | `:wat::core::defn-restricted` defmacro defined in `wat/core.wat` | **YES** | Defmacro at `wat/core.wat` adjacent to existing `defn` defmacro: same rest-binder shape per arc 150; takes `(name [prefixes] & rest)` and expands to `(:wat::core::def-restricted ~name ~prefixes (:wat::core::fn ~@rest))`. |
| D | 5 new tests pass — positive prefix + negative prefix + exact-FQDN + multi-prefix + defn-restricted-expansion | **YES** | `tests/wat_arc198_def_restricted.rs` defines: `def_restricted_caller_inside_allowed_namespace_passes`, `def_restricted_caller_outside_allowed_namespace_fails`, `def_restricted_exact_fqdn_match_only_allows_named_caller`, `def_restricted_multi_prefix_whitelist_admits_either_namespace`, `defn_restricted_macro_expands_to_def_restricted_plus_fn`. `cargo test --release -p wat --test wat_arc198_def_restricted` → `test result: ok. 5 passed; 0 failed`. |
| E | `cargo build --release --workspace --tests` clean | **YES** | `Finished release profile [optimized] target(s) in 1m 03s`; no errors. 5 pre-existing dead-code warnings + 1 unused-mut + 1 unused-variable from existing tests; not introduced by this arc. |
| F | Workspace test failure count ≤ baseline (Stone B end: 4 pre-existing failures) | **YES** | `cargo test --release --workspace --no-fail-fast` → `error: 4 targets failed`. Per-target: `probe_lifeline_pipe_proof` (lifeline flake, pre-existing 2/100 trials); `wat::test` 176 passed / 1 failed (`deftest_wat_tests_tmp_totally_bogus`, pre-existing); `wat_arc170_program_contracts` 23 passed / 1 failed (`t6_spawn_process_factory_with_capture_round_trips`, pre-existing); `wat-cli::wat_cli` 14 passed / 1 failed (`startup_error_bubbles_up_as_exit_3`, pre-existing). NO new failures. New target `wat_arc198_def_restricted` adds 5 passes. |

**6/6 PASS.**

## Honest deltas

### Parser handling of Vec-of-keyword positional arg

No parser change needed. `WatAST::Vector` is parsed from `[...]` bracket syntax at parse time and reaches `infer_def_restricted` directly as `args[1]`. The challenge was avoiding the generic vector-at-value-position diagnostic (`MalformedForm` at `src/check.rs:4259`, "vector literals at value position are not supported in arc 167"). Solution: `infer_def_restricted` does NOT call `infer` on the prefix-vector arg — it extracts the prefixes directly via `extract_prefix_vec` and validates each entry is a `Keyword`. The expr arg (args[2] originally, args[3] in the 4-element shape) IS passed through `infer` so normal type-inference applies.

The arity expectation in `infer_def_restricted` is `args.len() == 3` (head + name + prefixes + expr in the original form means 3 args after the head). In `extract_def_restricted_binding` the items length is 4 (head + name + prefixes + expr). These are consistent — the discrepancy is just the arg-slice convention (sans head) vs items-slice convention (with head).

### AST representation choice — option (a)? (b)?

Neither. **Option (c) — extend existing storage maps, no AST change.** The `WatAST` enum is unchanged; `def-restricted` is recognized by its head keyword (`:wat::core::def-restricted`), same shape recognition as `def`. The "new variant" path would have required wider changes (parser, type checker, macro expansion, freeze pre-registration) and bought nothing — `def-restricted` IS structurally `def` plus a Vector arg. The storage extension lives in CheckEnv (`defined_value_restrictions: HashMap<String, Vec<String>>`) and SymbolTable (parallel field, mirrored at `from_symbols`).

This matches arc 157's `def` design — `def` didn't introduce a new AST variant either; it added `CheckEnv.defined_values` + `SymbolTable.defined_values` and reused `WatAST::List` recognition by head keyword.

### CheckEnv extension point

Added `defined_value_restrictions: HashMap<String, Vec<String>>` as a sibling field to `defined_values`. Mirrored from the same-named field on SymbolTable via `from_symbols`. Accessors: `get_defined_value_restriction` (walker reads) + `register_defined_value_restriction` (`collect_splice_defs_ctx` writes).

The mirror happens at CheckEnv construction; `register_defines` populates the SymbolTable map BEFORE `check_program` runs, so the walker sees the whitelist at the right time. The freeze pipeline order (per `src/freeze.rs:840-893`) is: `register_defines` (step 6) → `resolve_references` (step 7) → `check_program` (step 8) → `register_runtime_defs` (step 9). Restrictions are registered in step 6 (the same path that pre-registers fn-shape defs into `sym.functions`); step 8's walker reads them through CheckEnv.

### Walker hook landing

Inserted in `check_program` directly after Stone B's `validate_join_result_user_namespace` loop. Same iteration shape — `for (name, func) in sym.functions.iter()` — passing the FQDN as `enclosing_fn`. No need for substrate-namespace short-circuit (Stone B exempts `:wat::*` callers because Stone B's rule is hard-coded; arc 198's rule is declared at the binding site, so the whitelist itself decides whether substrate-namespace callers are allowed).

The walker recurses through `WatAST::List` and `WatAST::Vector` children identically to Stone B's walker. Call detection: if the first child of a List is a Keyword and that keyword names a restricted binding (per `env.get_defined_value_restriction`), check the enclosing fn FQDN against the whitelist.

### defmacro expansion shape

`(:wat::core::defn-restricted name [prefixes] & rest)` → `(:wat::core::def-restricted ~name ~prefixes (:wat::core::fn ~@rest))`. The defmacro takes the prefixes as a separate `AST<wat::core::nil>` positional arg (between name and rest), then splices it back via `~prefixes`. The rest-binder splices fn args/arrow/ret/body in directly. Mirrors the existing `defn` expansion shape almost line-for-line.

The prefix Vec is treated as an opaque AST node by the defmacro — the macro doesn't introspect its contents. The substrate primitive `def-restricted` validates the Vec at check time (each entry must be a Keyword).

### Empty whitelist semantics

`[]` matches nothing — every caller fails. `caller_matches_prefix_list(_, &[])` returns `false` because `Iterator::any` on an empty iterator returns `false`. The walker fires `DefRestrictedCallerNotAllowed` with `prefixes: vec![]` rendered as an empty bracket pair in the diagnostic.

This is the honest reading: a restriction with no allowed callers means no callers are allowed. A future arc could mint a sugar like `def-private` over this same primitive (whitelist = `[]`), making "substrate-only/no-callers" an explicit user-facing form. Arc 198 does not gate empty whitelists at the declaration site — the user has to mean it, and the walker fires uniformly the moment any call site is found. Documented in the `caller_matches_prefix_list` doc comment.

### Restricted-binding lookup at call site

The walker does a `HashMap::get` on the call-head keyword string against `env.defined_value_restrictions`. O(1) average lookup per call site. The lookup table is mirrored from SymbolTable at CheckEnv construction; no per-fn-body work. The walker still recurses through every node — the per-node cost is one HashMap lookup keyed on the head keyword (skipped entirely when the head isn't a Keyword).

### Workspace test count vs baseline

| Target | Baseline (Stone B end) | Post-arc-198 | Delta |
|---|---|---|---|
| `wat::wat_arc198_def_restricted` (NEW) | (did not exist) | **5 passed / 0 failed** | +5 passes |
| `wat::probe_lifeline_pipe_proof` | 1 fail (flake 1-2/100) | 1 fail (flake 2/100 this run) | unchanged (flake) |
| `wat::test` | 176 pass / 1 fail (`totally_bogus`) | 176 pass / 1 fail (`totally_bogus`) | unchanged |
| `wat::wat_arc170_program_contracts` | 23 pass / 1 fail (t6) | 23 pass / 1 fail (t6) | unchanged |
| `wat-cli::wat_cli` | 14 pass / 1 fail (`startup_error`) | 14 pass / 1 fail (`startup_error`) | unchanged |
| Every other target | passes | passes | unchanged |

**Net: +5 new passes; 0 new failures. Same 4 baseline targets failing with the same individual tests.**

### Substrate-discovery surprises

**Zero. Smooth ride.** Predicted 0-2 in EXPECTATIONS; actual: 0. The substrate's existing arc-157 (`def`) + arc-166 (fn-shape def pre-registration) machinery was a clean template — the `def-restricted` primitive is structurally `def` plus a positional Vec arg plus a per-binding metadata HashMap parallel to `defined_values`. Every implementation site had a sibling `def` arm that arc 198 cloned and adjusted.

The one judgment call was empty-whitelist semantics (documented in the relevant honest-delta section above): `[]` means no callers allowed. The other plausible reading ("error at declaration site") was rejected — restrictions are values, the substrate is permissive about the values themselves, and the walker is uniform. Letting `[]` be a real (if extreme) point in the design space keeps the primitive composable for future sugars.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60-90 min | ~75 min |
| Scorecard rows | 6/6 PASS | 6/6 PASS |
| Workspace fail count | ≤ baseline (4) | = baseline (4 — same individual tests) |
| New test count | 5 | 5 |
| AST representation chosen | (a) new variant OR (b) extended Def | (c) no AST change — head-keyword recognition + sibling CheckEnv / SymbolTable HashMap fields |
| Empty-whitelist semantics | TBD | `[]` matches nothing; every caller fails (documented; future sugar `def-private` could mint this explicitly) |
| Substrate-discovery surprises | 0-2 | 0 |
| Mode | Additive | Additive (no existing behavior changed; no existing tests modified) |

## STOP triggers encountered

**None reached.**

- "Parser doesn't accept Vec-of-keyword positional arg cleanly" — no issue; vectors parse fine; the only adjustment was skipping `infer` recursion into the prefix-vec inside `infer_def_restricted` to avoid the unrelated "vector at value position" diagnostic from `src/check.rs:4259`.
- "CheckEnv storage doesn't have an obvious place for per-binding metadata" — adjacent to `defined_values` was the obvious place; arc 157's design accommodated the extension without restructuring.
- "Defmacro expansion doesn't typecheck due to arg-shape mismatch" — the `defn` template at `wat/core.wat:201-206` was a clean reference; the only addition was a single extra positional `(prefixes :AST<wat::core::nil>)` arg before the rest-binder. The macro expansion typechecked on first compile.
- "Migration breaks existing tests" — no existing tests were modified; arc 198 is purely additive. All 4 baseline failures unchanged.
- ">5 unexpected substrate-finding surfaces" — 0 substrate-discovery surprises.

## What this enables

After arc 198 ships:

- **Stone B's hard-coded rule can collapse into a generic mechanism.** A follow-up step (separate from arc 198's scope per BRIEF) refactors `validate_join_result_user_namespace` to use two `def-restricted` declarations on the `*_join-result` substrate fns, eliminating the special-case walker rule. Stone B's `CheckError::JoinResultUserNamespace` variant would become orphaned scaffolding (arc 113 retirement precedent).
- **Future restricted forms come for free.** `defmacro-restricted`, `define-restricted`, etc. are all defmacro sugar over `def-restricted` — the walker mechanism doesn't care about the surface form. Empty-whitelist sugar `def-private` is a one-liner defmacro over `def-restricted` + `[]`.
- **Substrate-internal forms self-declare their callable surface.** Each `def-restricted` declaration reads at the binding site as "this binding is callable only from {whitelist}". The substrate teaches its own boundaries; future readers don't grep the walker for hard-coded rules.

The substrate teaches; we listen; we generalize once the pattern is real.

## Files touched

- `src/special_forms.rs` — new registry entry for `:wat::core::def-restricted`
- `src/check.rs` — new variant + Display + Diagnostic + CheckEnv field + accessors + `infer_def_restricted` arm + `extract_def_restricted_binding` + `extract_prefix_vec` + `walk_for_def_restricted_call` + `caller_matches_prefix_list` + `collect_splice_defs_ctx` arm + `check_program` hook
- `src/runtime.rs` — new `SymbolTable.defined_value_restrictions` field + Debug field + `try_parse_fn_shape_def_restricted` + `register_defines` arm + `preregister_fn_defs_in_do` arm + `preregister_fn_defs_in_let` arm + `register_runtime_defs_form` arm + `dispatch_keyword_head` arm
- `src/freeze.rs` — `is_mutation_form` + `is_declaration_form` extended to include `:wat::core::def-restricted`
- `wat/core.wat` — new `defn-restricted` defmacro adjacent to existing `defn`
- `tests/wat_arc198_def_restricted.rs` — 5 new tests (NEW file)
- `docs/arc/2026/05/198-defn-restricted/SCORE.md` — this file (NEW)
