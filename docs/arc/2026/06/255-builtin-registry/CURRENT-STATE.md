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

## Strike sequence (re-sequenced 2026-06-21 — registry-infra first; EVERYTHING below is IN 255, nothing left behind)
- **255.1b-i (DONE) — registry infra + FIRST home (`core::Bytes`), ZERO behavior.**
  `src/registry/{mod,bytes}.rs`: `NativeHandler` type + `BuiltinRegistry` (`name → handler`)
  + `registry()` (OnceLock); 2 Bytes dispatch arms route through `registry().lookup(head)`
  (consumes the registry → no dead_code); `eval_bytes_*` → `pub(crate)`. Floor 953/36/1,
  warnings 26, zero behavior change, registry `pub(crate)`.
  **WEIGH (caught a cheat):** the shadowdancer stored the full baseline (arity/purity/
  determinism/expand_time) UNREAD and made the module `pub` to hide the dead_code (the
  pub-leak silence-the-signal cheat). Reverted to `pub(crate)` + TRIMMED to the consumed core
  (`name → handler`). **The baseline metadata accretes consumer-by-consumer** ("satisfy by
  use"): `arity`→arity-check strike, `purity`/`determinism`→rete-query strike, `expand_time`→
  macro-gate strike, all→reflection (255.2). Each field lands WITH its reader — never dead,
  never silenced. (If the builder later wants the full baseline DATA recorded now and accepts
  loud dead_code as tracked forcing-signals, that's a floor-raise call — flagged, not taken.)
- **255.1b-ii… — per-home registration + carve repeats (shadowdancers).** Each home:
  register its builtins (baseline + handler) + route its dispatch arms through the registry,
  carving them out of runtime.rs's central match (the megafile dissolves). One home/strike,
  weighed against the corpus. Scalar/arith homes (`core::i64`/`f64`) gated on the hot-path
  bench (phf/generated dispatch if a HashMap lookup regresses).
- **255.1b-RESOLVE — the hole closes.** Once all builtins are registered: resolver rewrite —
  DELETE `is_reserved_prefix → true` blanket-accept; `:wat::*` head → registry/sym membership;
  unknown → UnresolvedReference + retirement/near-match remedy. GATE: 254.R undefined-builtin
  probe green; FULL corpus green (cascade reveals any unregistered real head → register it).
- **255.1c — the `FnDef` split (IN 255 — user-fn value-vs-def honesty; NOT left behind).**
  Split `Function` (env.rs:35; `name: Option` + `closed_env: Option` are CORRELATED — a
  disguised sum): extract `Signature` {type_params,params,param_types,ret_type,rest_param,
  rest_param_type}; `FnDef {name, sig, body}` (named top-level def, no env — what sym holds);
  `Closure {sig, body, env}` (anon fn-value, no name). `Value::wat__core__fn` carries the
  closure; named-fn-as-value converts FnDef→Closure (DESIGN this in its own pass — the
  Value::fn-sum / global-env question is unresolved). ~31 Function sites (7 anon). Drive with
  the cascade. THIS makes the `DefDetail` sum uniform (all `*Def`).
- **255.1d — `*Meta` layer** (per-kind closed wat-record schemas; `MetaField<T>` named
  forced-match sums; kwargs-construct; `:doc` common; user-form metadata).
- **255.2** reflection verbs (child-namespaces/names/metadata-of over the registry);
  **255.3** consumers collapse (rete/purity.rs + macros::is_pure_total DELETE; is_effectful_op
  → the `:pure` deriver); **255.N** inscription. ALL of these ship in 255.

## Floor
lib 953 passed / 36 failed / 1 ignored; warnings 26. Shipped this session before the pivot:
collection seq HOFs 1a (`5ac9abdb`) + 1b List (`751d131d`). Use the wat migration toolkit
(fix-wat + retirement table + cascade) — memory `feedback_lean_on_wat_migration_toolkit`.

> ⛔ NEW INSTANCE: you did not live the design session above — it's a cache. recolligere:
> read DESIGN.md LOCKED section + this file, `git log --oneline -8`, before moving.
