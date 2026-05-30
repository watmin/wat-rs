# SCORE — Stone 243.3.1 — mint `src/check/` home + CheckEnv borrow redesign

**Parent:** Stone 243.3 (spawn-child; 243.3 cannot close until this closes).
**Mode:** A (sonnet substrate; orchestrator briefs/scores/commits/casts vigilia).
**The roof:** the CheckEnv mirror (one logical thing deep-cloned into CheckEnv) eliminated by making deep-clone-into-CheckEnv a COMPILE ERROR — the borrow redesign. Failure engineering applied to ownership; ZERO-MUTEX's "never construct the situation" at the ownership layer.

**SCOPE PRECISION (per `feedback_selective_lift_and_ward`):** this stone LIFTED one resident — `CheckEnv` → `src/check/env.rs` (266 lines) — and did the perfecting work IN the new home to L1+L2=0. The vigilia REMARKABLE bar below governs the **lifted resident (env.rs) + the redesign seams in mod.rs**, NOT the whole `src/check/` directory. The `src/check/mod.rs` remainder (~21k lines, the `git mv`'d former `check.rs`) is **functional-but-untrusted by honest default** — relocated, not yet lifted, correctly NOT at the bar. The home grows toward fully-warded through future selective lifts (243.6+ lifts CheckError etc. as each is found near-perfect-but-needing-work). The claim is "env.rs is at the bar," never "every line under src/check/ is."

## Phase A — substrate refactor verified

### Per-step audit

| Step | Status | Notes |
|---|---|---|
| A1 — mint `src/check/` home | COMPLETE | `git mv src/check.rs → src/check/mod.rs` — transparent; preserved ~21k lines + every `crate::check::X` import path; lib stayed 890/0 on the bare move |
| A2 — `CheckEnv<'a>` borrow redesign | COMPLETE | `types: Arc<TypeEnv>` → `&'a TypeEnv`; `binding_metadata: Arc<HashMap>` → `Option<&'a HashMap>`; 6 owned fields unchanged (schemes/unit_variant_types DERIVED; defined_values/spans/defclause_registrations INCREMENTAL; redef_allowed mid-pass-MUTATED). Carved to `src/check/env.rs` (266 lines) — the home's first honest neighbor |
| A3 — `with_builtins()` removed | COMPLETE | Could not honestly exist under the borrow (returned a CheckEnv borrowing a stack-local about to drop); 3 standalone sites (runtime.rs ×2 + test) reshaped to bind-TypeEnv-then-borrow |
| A4 — lifetime cascade | COMPLETE | ~4 sites; `CheckSchemeCtx` gained the lifetime; rest elided. Verbose-not-confusing; no `unsafe`/`Box::leak`/`'static`/re-clone |
| A5 — freeze.rs:329 clone | KEPT-HONEST | Borrow-checker verdict: FrozenWorld needs `types` for both `set_types(Arc::new(types))` AND its own field — two persistent owners; a persistence boundary, not the eliminated duplication class. The borrow-checker delivered the verdict, not assertion |
| A6 — re-export | COMPLETE | `mod env; pub use env::CheckEnv;` in mod.rs; `crate::check::CheckEnv` + `wat::check::CheckEnv` resolve through it |

### The 3 clones — fate

| Clone | Site | Fate |
|---|---|---|
| `binding_metadata` deep clone | check.rs:2019 | KILLED — `Option<&'a HashMap>` borrow; `Some(&sym.binding_metadata)` |
| `TypeEnv` deep clone (⑬) | check.rs:2175 | KILLED — `from_symbols(sym, &types)`; borrow makes `Arc::new(types.clone())` unrepresentable |
| `TypeEnv` re-clone | freeze.rs:329 | KEPT-HONEST — persistence boundary (two persistent owners), borrow-checker-verified |

### FM 2-bis probe — the structural proof

`tests/probe_arc243_stone3_1_checkenv_borrow.rs` flipped from **fail-compile** (pre-stone: 5 errors — `CheckEnv takes 0 lifetime arguments` ×3 + `expected Arc<TypeEnv>` ×2) to **PASS 3/0** (post-stone). The flip IS the proof the deep-clone-into-CheckEnv class became uncompilable, not merely avoided.

### Final metrics

| Metric | Value |
|---|---|
| Lib | 890 / 0 |
| tests/function | 8 / 0 |
| probe_arc243_stone3 (TypeError, no regress) | 3 / 0 |
| probe_arc243_stone3_1 (CheckEnv borrow) | 3 / 0 (flipped) |
| arc112_slice2b | 1 / 0 |
| 4× :restricted-to behavioral | all green (1+3+2+5) |
| clippy | 877–894 (≤ 894 ceiling; dropped from baseline via the cleanup) |
| workspace test-build | clean |

### Atomic-stone diff shape

| File | Δ | Note |
|---|---|---|
| `src/check.rs → src/check/mod.rs` | rename + −221 net | home mint + CheckEnv struct carved out |
| `src/check/env.rs` | +266 (new) | the redesigned `CheckEnv<'a>` — home's first neighbor |
| `src/types.rs` | +20 | `TypeEnv::build_unit_variant_map` (B extraction; R3 rename) |
| `src/runtime.rs` | +7/−3 | 2 `with_builtins()` sites reshaped |
| `tests/wat_arc208_process_io_result.rs` | +4/−2 | 1 `with_builtins()` site reshaped |

## Phase B — vigilia REMARKABLE bar (orchestrator-cast)

### Round 1 — 8-spell cast

| Spell | R1 verdict |
|---|---|
| purgare | CONVERGED 0/0 |
| temperare | CONVERGED 0/0 (the borrow eliminates the clones; no hidden `.clone()` smuggles duplication back) |
| exigere | CONVERGED 0/0 (no deferral language) |
| cernere | CONVERGED — not-applicable (pure-Rust target) |
| intueri | 3 L2 (F1 register/get undocumented; F2 from_symbols WHAT-noise; F3 CheckSchemeCtx lifetime collapse) |
| solvere | 1 L1 + 1 L2 (S1 TypeEnv enum-walk braid; S2 register_defclause two-jobs) |
| struere | 3 L2 (F1 CheckSchemeCtx lifetime; F2 get_defclause_clauses &Vec; F3 accessor `'a` suppression) |
| sequi | 1 L1 + 1 L2 (#1 CheckSchemeCtx lifetime; #2 rust_deps::get ambient static no rune) |

**Aggregate R1: 2 L1 + 6 L2 = 8 unique findings.** THE FINDING: CheckSchemeCtx lifetime collapse — 3 independent spells (intueri F3 + struere F1 + sequi #1) converged on it. Debt the redesign itself created.

### Round 2 — remediation (8 fixes, all FIX per pre_existing_is_not_exemption + runes_illegal_when_solvable)

- A: `CheckSchemeCtx<'a, 'b: 'a> { env: &'a CheckEnv<'b> }` — two lifetimes
- B: `TypeEnv::build_unit_variant_map()` extraction — braid severed (0 `crate::types::TypeDef` in env.rs)
- C: `register_defclause` doc declares both jobs (coupling load-bearing; kept atomic)
- D: `get_defclause_clauses` → `Option<&[..]>` via `as_slice()`
- E: `types()` / `get_binding_metadata` expose `'a`
- F: `register` / `get` WHY docs
- G: `from_symbols` comment compressed to BEWARE
- H: `rune:sequi(ambient-context)` at `rust_deps::get()`

### Round 2 kill-confirm re-cast (4 finding-spells)

| Spell | R2 verdict |
|---|---|
| intueri | CONVERGED (F1/F2/F3 closed; no new) |
| solvere | CONVERGED (S1/S2 closed; extraction directionally clean) |
| sequi | CONVERGED (#1/#2 closed) |
| struere | CONVERGED original 3 BUT 2 NEW findings in R2-written code: I (L1 doc-lie "atomic" over asymmetric writes); J (L2 `unit_variant_types` allocates-per-call name) |

### Round 3 — final remediation (struere's 2 new)

- I: `register_defclause` doc rewritten — "atomic" dropped; states the by-design asymmetry (clause-table unconditional vs guarded-idempotent sentinel + "must not clobber a real value type from a prior def" rationale). Code UNCHANGED (guard correct).
- J: `unit_variant_types` → `build_unit_variant_map` (surfaces one-shot allocation); single caller updated.

### Round 3 kill-confirm

struere final re-cast: **CONVERGED** — I + J closed; full fresh scan of env.rs clean.

### Phase B verdict

**REMARKABLE bar achieved on the LIFTED RESIDENT: L1 + L2 = 0** across all 8 spells, scoped to `env.rs` + the redesign seams in `mod.rs` (NOT the ~21k-line `mod.rs` remainder, which is functional-but-untrusted-by-design per `feedback_selective_lift_and_ward` — relocated, awaiting future selective lifts). Three remediation rounds; the bar held the line each round (R2's fixes were themselves vigilia'd; R3 closed the debt R2 created). The watch verified the watch.

## Doctrines exercised / landed

- `feedback_pre_existing_is_not_exemption` — every finding FIX'd, none deferred
- `feedback_runes_illegal_when_solvable` — only H is a rune (legitimate ambient static); all else FIX
- `feedback_nonintuitive_error_is_pivot` — borrow cascade verbose-not-confusing; pushed through
- `feedback_let_need_reveal_through_work` — ⑬ left in 243.3, revealed its real owner here (the CheckEnv borrow)
- `scratch/FAILURE-ENGINEERING.md` — eliminate the CLASS; the borrow makes the wrong shape unrepresentable
- `feedback_namespaced_home_vigilia_gate` — the home FORCES the grimoire; the bar is real because the gate is real
- `feedback_verify_sonnet_worktree_not_just_return` (LANDED this stone) — sonnet's R2 made 2 unauthorized commits; caught via git-state check, unwound clean; R3 brief hardened no-commit to #1 STOP trigger

## Spawn-block

243.3.1 closes → wind up: Stone 243.3's tail (SCORE Phase B + close) unwinds next. Parent waited on child; child done.
