# BRIEF — Arc 214 Slice 4 Stone 4.1 — Mint `:wat::program::Env` typealias

**Stone:** register `:wat::program::Env` as a typealias for `HashMap<keyword, HolonAST>`. Foundational; enables subsequent stones (4.2 accessor verbs; 4.3 unified spawn).
**Type:** Sonnet Mode A.
**Time budget:** 30-45 min target; 60 min STOP.
**Depends on:** arc 215 (both stones shipped); arc 214 DESIGN Slice 4 forward-correction.
**Unblocks:** Stone 4.2 (accessor verbs); Stone 4.3 (unified spawn).

## Goal

Mint the type name. Per `feedback_no_new_types` — typealias, not wrapper struct. The underlying IS `HashMap<keyword, HolonAST>`; the alias gives that shape a registered name for use in function signatures.

Per arc 214 DESIGN Slice 4 forward-correction Q4: `:wat::program::Env` is the wat-level program env namespace. Distinct from `:wat::process::Env` (OS env vars; separate concern; out of scope).

## Pre-flight verified

- Typealias mechanism: `env.register_builtin(TypeDef::Alias(AliasDef { name, type_params, expr }))` in `src/types.rs`
- Precedent: `:wat::core::Bytes` at `src/types.rs:440-447` (typealias for `Vec<u8>`)
- `:wat::program::*` namespace currently empty (clean mint; no collision)
- Baseline tests green (probe_arc215_stone2 13/13; probe_arc215_collection_literal_inference 12/12; probe_brace_map_literal 9/9; probe_hashmap_ctor_vector_symmetric 9/9)

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- `cargo test` is the verification path

## Your scope

1. **Register `:wat::program::Env` as typealias** in `src/types.rs`:
   - Find the existing `:wat::core::Bytes` registration (line 440-ish) as the pattern template
   - Add a new `env.register_builtin(TypeDef::Alias(AliasDef { ... }))` call for `:wat::program::Env`
   - Expression: `TypeExpr::Parametric { head: "wat::core::HashMap".into(), args: vec![TypeExpr::Path(":wat::core::keyword".into()), TypeExpr::Path(":wat::holon::HolonAST".into())] }`
   - Document the registration with a brief comment block (matches Bytes' commentary style); cite arc 214 Slice 4 forward-correction Q4

2. **Probe file** `tests/probe_arc214_slice4_stone1_program_env_typealias.rs` with ~6 probes:
   - Probe 1: `:wat::program::Env` parses as a valid type-keyword via `parse_type_expr`
   - Probe 2: `expand_alias(:wat::program::Env)` returns the underlying `HashMap<keyword, HolonAST>` shape
   - Probe 3: A function signature `(:user::test-fn (m :wat::program::Env) -> :wat::core::nil)` type-checks
   - Probe 4: Calling that function with `{:foo (:wat::holon::Atom 42)}` literal type-checks (V infers HolonAST from the explicit Atom)
   - Probe 5: Calling with empty `{}` literal type-checks (HM unification fills K + V from the param signature)
   - Probe 6: Calling with `{:foo "string"}` (V = String, not HolonAST) fails at check with TypeMismatch naming the V mismatch

3. **WAT-CHEATSHEET update** (`docs/WAT-CHEATSHEET.md`): brief mention of `:wat::program::Env` in the namespace section; reference the forward-correction in arc 214 DESIGN

4. **SCORE doc** at `docs/arc/2026/05/214-concurrency-toolkit/slice-4-program-env/SCORE-STONE-4.1.md`:
   - 10-row scorecard matching the EXPECTATIONS file
   - Mode declaration (A)
   - Honest deltas section
   - PASS/FAIL per row with citation

## NOT your scope

- Accessor verbs (`/get`, `/dig`, etc.) — Stone 4.2
- spawn-program' verb — Stone 4.3
- Kernel verbs (`send'`, `recv'`, etc.) — Stone 4.4
- Integration tests for ProgramEnv usage — Stone 4.5
- INTERSTITIAL entry — orchestrator-direct post-ship per `feedback_sonnet_no_realization_voice`
- WARD-PASS — out-of-zone per `feedback_ward_zone_comms_only`
- Commit + push — orchestrator commits after reviewing SCORE
- `:wat::process::Env` OS env vars — separate concern; not this arc

## STOP triggers

- STOP-1: typealias registration breaks something subtle in `expand_alias` or `parse_type_expr` — Bytes precedent should make this clean, but flag if not
- STOP-2: any existing test fails after the typealias registration — should be additive, but verify
- STOP-3: 60 min elapsed

## Verification

Single commands, one per line (firewall-friendly per `feedback_sonnet_bash_firewall`):

```
cargo build --release
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat
cargo test --release --test probe_arc215_stone2 -p wat
cargo test --release --test probe_arc215_collection_literal_inference -p wat
cargo test --release --test probe_brace_map_literal -p wat
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat
cargo clippy --release -- -D warnings
```

## When you finish

Report:
- Final PASS count out of 10
- Honest deltas
- Verification summary
- Elapsed time
- Anything you discovered that wasn't in the BRIEF

Don't commit. Orchestrator commits after reviewing SCORE.
