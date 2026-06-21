# ⛔ ARC 255 — CURRENT STATE (breadcrumb, 2026-06-21; replace in place)

**255 is ACTIVE. 278 is PARKED (255 unlocks its continuity). Design is LOCKED — read
`DESIGN.md` § "LOCKED RECORD MODEL" (the authoritative spec; the sections above it are the
derivation).**

## Why we pivoted (the catastrophic instance, grounded)
The resolver blanket-accepts ANY `:wat::*` head (`is_reserved_prefix → true`, walk.rs:189)
and the checker punts via a permissive `Infer` fallback (check.rs:9923) — so a typo'd/
retired/nonexistent builtin (`:wat::core::nonexistent-xyz?`) type-checks clean and only dies
at runtime. Double-punt; both layers say "not my job." Builder verdict: annihilation.

## The settled model (one line)
A registered name = **baseline (platform-guaranteed, always-concrete, enum-typed+no-Default)
⊕ per-kind `*Def` (structural; uniform `*Def` family — `FnDef` split fixes the `Function`
loner) ⊕ per-kind `*Meta` (closed wat-record schema; optionality via NAMED forced-match sum
`MetaField<T>=Unspecified|Specified(T)`, NOT raw Option/sentinels; evolve via Unspecified
defaults; fix-wat for rare breaks)**. Registry IS `sym`. Full spec: DESIGN.md LOCKED section.

## Strike sequence (hand-author seam + first home; delegate per-home repeats)
- **255.1b-i (NEXT) — FnDef split + type scaffold.** Split today's `Function`
  (env.rs:35, `name: Option`, `closed_env`) into **`FnDef`** (the def-record: name(req),
  type_params, params, param_types, ret_type, rest_param — a true `*Def`) + **`Function`**
  (the runtime closure VALUE: body + closed_env, anon-capable, metadata-free, unregistered).
  Then the leaf vocab (`Arity`, `Purity`/`Determinism`/`ExpandTime`/`DefKind` enums,
  `MetaField<T>`, baseline `Registration`, `DefDetail` sum, `NativeBuiltin`). ~31 Function
  construction sites (7 anon, `name: None`). `FunctionBody::{Wat,Native}` ALREADY exists
  (env.rs:22, 255.1a) — Native never constructed yet.
  **STRIKE 1 SCOPED (2026-06-21):** `Function {` at 31 sites — **26 runtime.rs · 4
  environment.rs · 1 function/eval.rs**. 7 are anon `name: None` (eval_fn @ function/eval.rs:62
  + comments) = pure VALUES (no FnDef). Registry slot today: `sym.functions:
  HashMap<String, Arc<Function>>` (symbol_table.rs:35). Def-fields (`param_types`/`ret_type`)
  read across ~8 files each = the FnDef surface (checker call-site machinery). Value-field
  `closed_env` read in 3 files (environment.rs, closure_extract.rs, runtime.rs). Cut plan:
  mint `FnDef`(def-record) + slim `Function`(closure value: body+closed_env+sig-link);
  `sym.functions` → holds the def (or a Registration wrapping it); named-fn constructions
  build FnDef+Function, the 7 anon build Function only. Mechanical but wide — drive it with
  the test cascade (each broken site names the next). Land green; floor 953/36/1.
- **255.1b-iii** — register builtins into `sym` from their homes (first home = small/pure
  template, e.g. `core::Bytes`); carve those arms out of runtime.rs's dispatch.
- **255.1b-iv** — resolver rewrite: delete blanket-accept; `sym` membership + retirement/
  near-match remedy. GATE: 254.R undefined-builtin probe green; full corpus green (cascade
  reveals unregistered real heads → register); bench no hot-path regression.
- **255.2** reflection verbs (child-namespaces/names/metadata-of); **255.3** consumers
  collapse (rete/purity.rs + macros::is_pure_total DELETE; is_effectful_op → :pure deriver);
  **255.N** inscription.

## Floor
lib 953 passed / 36 failed / 1 ignored; warnings 26. Shipped this session before the pivot:
collection seq HOFs 1a (`5ac9abdb`) + 1b List (`751d131d`). Use the wat migration toolkit
(fix-wat + retirement table + cascade) — memory `feedback_lean_on_wat_migration_toolkit`.

> ⛔ NEW INSTANCE: you did not live the design session above — it's a cache. recolligere:
> read DESIGN.md LOCKED section + this file, `git log --oneline -8`, before moving.
