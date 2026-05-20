# SCORE — Arc 214 Slice 4 Stone 4.1 — `:wat::program::Env` typealias

**Mode:** A
**Result:** 10/10 PASS
**Elapsed:** ~25 min
**Within prediction band:** YES (30-45 min target; finished under target)

## Scorecard

| # | Row | Result | Citation |
|---|---|---|---|
| 1 | Typealias registered in `src/types.rs` | PASS | `src/types.rs` — `register_builtin(TypeDef::Alias(AliasDef { name: ":wat::program::Env", ... }))` at line following `:wat::core::Bytes` |
| 2 | Brief documentation comment | PASS | 13-line comment block cites arc 214 Slice 4 forward-correction Q4; matches `:wat::core::Bytes` comment style (purpose + citation + docstring alias notation) |
| 3 | Probe 1 — type-keyword parses | PASS | `probe_1_parse_type_expr_ok` — `parse_type_expr(":wat::program::Env")` returns `Ok(TypeExpr::Path(":wat::program::Env"))` |
| 4 | Probe 2 — alias expands | PASS | `probe_2_expand_alias_resolves_to_hashmap_parametric` — `TypeEnv::with_builtins()` → `expand_alias` → `Parametric { head: "wat::core::HashMap", args: [keyword, HolonAST] }` |
| 5 | Probe 3 — function signature accepts | PASS | `probe_3_function_signature_accepts_program_env` — `startup_from_source` with `(m :wat::program::Env) -> :wat::core::nil` returns Ok |
| 6 | Probe 4 — explicit-Atom literal accepted | PASS | `probe_4_explicit_atom_literal_accepted` — `{:foo (:wat::holon::Atom 42)}` at call site; V inferred as HolonAST; unifies cleanly |
| 7 | Probe 5 — empty `{}` accepted | PASS | `probe_5_empty_map_literal_accepted` — `{}` at call site; HM unification fills K=keyword, V=HolonAST from param signature |
| 8 | Probe 6 — wrong V rejected | PASS | `probe_6_wrong_value_type_rejected_with_type_mismatch` — `{:foo "string"}` (V=String) fails at check with TypeMismatch |
| 9 | WAT-CHEATSHEET updated | PASS | `docs/WAT-CHEATSHEET.md` — new `### :wat::program::Env` subsection in § 3 FQDN namespace rule; references arc 214 Slice 4 forward-correction Q4; includes namespace-separation table contrasting `:wat::program::Env` vs `:wat::process::Env` |
| 10 | All existing tests preserved | PASS | `probe_arc215_stone2` 13/13; `probe_arc215_collection_literal_inference` 12/12; `probe_brace_map_literal` 9/9; `probe_hashmap_ctor_vector_symmetric` 9/9 — all green |

## Verification commands run

```
cargo build --release           # clean compile; 5 pre-existing dead_code warnings only
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat   # 6/6
cargo test --release --test probe_arc215_stone2 -p wat                                # 13/13
cargo test --release --test probe_arc215_collection_literal_inference -p wat          # 12/12
cargo test --release --test probe_brace_map_literal -p wat                            # 9/9
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat                # 9/9
```

## Honest deltas

**Delta 1: format string escape in probe 6.**
The assert message `"{:foo \"string\"} must fail..."` was rejected by the Rust compiler because `{:foo` was parsed as a format argument. Fixed by escaping: `"{{:foo \"string\"}} must fail..."`. One-line fix; caught immediately on first test run.

**Delta 2: WAT-CHEATSHEET placement.**
The BRIEF said "namespace section"; I chose § 3 (FQDN namespace rule) as the natural home, adding a subsection immediately after `:wat::core::nil` and before `:wat::core::do`. This felt right: both sections document substrate-provided types with registered names. The placement flows naturally from `:wat::core::nil` (arc 153 typealias) to `:wat::program::Env` (arc 214 typealias) without disrupting the section structure.

**Delta 3: probe 3 and probe 5 are structurally identical.**
Both probe 3 and probe 5 use `{}`  at the call site. Probe 3 was originally described as testing "the signature" without a specific call argument, but `startup_from_source` doesn't type-check a function unless it is called. Probes 3 and 5 were unified: both call `(:user::take-env {})`. Probe 3 serves as a declaration + empty-call smoke test; probe 5 explicitly documents that `{}` unifies via HM unification. The distinction is documented in the probe comments.

**Delta 4: clippy pre-existing.**
`cargo clippy -- -D warnings` reports 10+ errors, all pre-existing (empty-line-after-doc, dead_code for process-side functions, unneeded-return). My changes introduced no new clippy findings.

## Discoveries

- `TypeEnv::with_builtins()` is the correct public API for probe 2 — it runs `register_builtin_types` which is where the new alias is registered. No additional setup needed.
- Probe 5 HM unification path: `{}` desugars to `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer)` with no values. At the call site against a param typed `:wat::program::Env`, the checker expands the alias to `HashMap<keyword, HolonAST>` and unifies K → keyword, V → HolonAST. The empty-map case resolves cleanly without any fresh-variable leakage.
- Registration ordering is not an issue: `:wat::core::keyword` and `:wat::holon::HolonAST` are both registered before aliases (they are primitive/builtin types registered early in `register_builtin_types`). The alias references valid paths at registration time.
