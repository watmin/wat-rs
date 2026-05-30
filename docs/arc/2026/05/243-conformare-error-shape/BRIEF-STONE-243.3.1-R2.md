# BRIEF — Stone 243.3.1 R2 — drive the vigilia REMARKABLE bar to L1+L2=0

You are sonnet. The 8-spell vigilia on Stone 243.3.1 (`src/check/` home + CheckEnv borrow redesign) surfaced 2 L1 + 6 L2 findings. This sweep closes ALL EIGHT to reach the namespaced-home REMARKABLE bar (L1+L2=0). Every finding is solvable + perf-OK → every one is a FIX (no runes except the one explicitly specified for H, which documents a legitimate ambient static).

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Pre-spawn state

Working tree carries the uncommitted Stone 243.3.1 work (src/check/mod.rs, src/check/env.rs, src/runtime.rs, tests/wat_arc208_process_io_result.rs). HEAD is `663a5a35`. Your R2 fixes JOIN that uncommitted set; orchestrator commits the whole stone atomically after this sweep + a vigilia re-cast confirms L1+L2=0.

**Gates baseline (all must hold post-sweep):**
- lib 890/0 · function 8/0 · probe_arc243_stone3 3/0 · probe_arc243_stone3_1 3/0 · arc112 1/0 · clippy ≤ 894 · workspace test-build clean
- 4× :restricted-to behavioral tests green (wat_arc198_slice2_stone_1/2/3 + wat_arc198_def_restricted)

## The discipline (read first)

**The borrow-checker is the teacher** (`feedback_nonintuitive_error_is_pivot`). A VERBOSE lifetime cascade is mechanical — push through. A CONFUSING error (you can't tell what rustc wants; the fix feels like fighting the type system) is a DESIGN SIGNAL — STOP, surface verbatim, do NOT force. If you reach for `unsafe`, `Box::leak`, `'static` transmute, `Rc`/`Arc` re-wrapping, or a `.clone()` to satisfy a lifetime — STOP. The borrow is meant to be clean.

## The 8 fixes

### A (L1) — `CheckSchemeCtx<'a>` lifetime collapse — mod.rs:~14709 [THE FINDING — 3 spells converged]

Current:
```rust
struct CheckSchemeCtx<'a> {
    env: &'a CheckEnv<'a>,        // collapses ctx-borrow lifetime with env-data lifetime
    locals: &'a HashMap<String, TypeExpr>,
    fresh: &'a mut InferCtx,
    subst: &'a mut Subst,
    errors: Vec<CheckError>,
}
```
`&'a CheckEnv<'a>` ties "how long this ctx borrows the CheckEnv" to "how long CheckEnv's inner data (`types`, `binding_metadata`) is borrowed." Two distinct relationships, one name — over-constrains call sites.

Target (two lifetimes; the inner data outlives the ctx's borrow of the env):
```rust
struct CheckSchemeCtx<'a, 'b: 'a> {
    env: &'a CheckEnv<'b>,
    locals: &'a HashMap<String, TypeExpr>,
    fresh: &'a mut InferCtx,
    subst: &'a mut Subst,
    errors: Vec<CheckError>,
}
```
Update `impl<'a> ... for CheckSchemeCtx<'a>` → `impl<'a, 'b: 'a> ... for CheckSchemeCtx<'a, 'b>` and the construction site in `dispatch_rust_scheme`. Let the borrow checker confirm the bound; if it wants a different precise form (e.g. elided `'b` at the construction site), follow what rustc names — the GOAL is the two lifetimes separated, not the exact syntax.

### B (L1) — TypeEnv enum-walk braided into CheckEnv ctor — env.rs:~156

Current (inside `with_types`):
```rust
for (name, def) in types.iter() {
    if let crate::types::TypeDef::Enum(e) = def {
        for variant in &e.variants {
            if let crate::types::EnumVariant::Unit(variant_name) = variant {
                let key = format!("{}::{}", name, variant_name);
                unit_variant_types.insert(key, TypeExpr::Path(name.clone()));
            }
        }
    }
}
```
The `crate::types::TypeDef::Enum` / `crate::types::EnumVariant::Unit` full paths are the tell — this structural knowledge belongs to `TypeEnv`, not `CheckEnv`.

Fix: add a method on `TypeEnv` in `src/types.rs` that yields the unit-variant map:
```rust
// in src/types.rs, impl TypeEnv
/// Map every unit-variant keyword path (`:enum::Variant`) to its enum type.
/// Consumed by the checker to resolve value-position unit-variant keywords.
pub fn unit_variant_types(&self) -> HashMap<String, TypeExpr> {
    let mut out = HashMap::new();
    for (name, def) in self.iter() {
        if let TypeDef::Enum(e) = def {
            for variant in &e.variants {
                if let EnumVariant::Unit(variant_name) = variant {
                    out.insert(format!("{}::{}", name, variant_name), TypeExpr::Path(name.clone()));
                }
            }
        }
    }
    out
}
```
Then `with_types` becomes `let unit_variant_types = types.unit_variant_types();`. The `crate::types::TypeDef`/`EnumVariant` references disappear from env.rs. (This method travels to `types/` at Stone 243.5 — it's the right home now and stays right.)

### C (L2) — `register_defclause` two jobs undeclared — env.rs:~245

The method registers the clause table AND writes a sentinel into `defined_values`/`defined_value_spans` (so keyword refs to the defclause name don't hit UnknownCallee). The coupling is LOAD-BEARING — forgetting the sentinel would be a bug, so they MUST stay atomic (do NOT split). The L2 is "does two jobs without declaring both." Fix = DECLARE both in the doc comment + make the name honest. Rewrite the doc:
```rust
/// Register a defclause's clause table AND a sentinel value-binding under
/// the same name. The sentinel (a `Var(u64::MAX)` in `defined_values`) is
/// load-bearing: it lets value-position keyword references to the defclause
/// name resolve here instead of failing UnknownCallee. Both writes are
/// atomic by design — a defclause without its sentinel is a bug.
```
Keep the method name `register_defclause` (the doc now declares the second job) OR rename to `register_defclause_with_value_binding` if you judge the name still hides it — your call; the bar is "both jobs visible."

### D (L2) — `get_defclause_clauses` returns `&Vec` — env.rs:~270

```rust
pub fn get_defclause_clauses(&self, name: &str) -> Option<&Vec<(Vec<TypeExpr>, TypeExpr, bool)>>
```
→ return the slice (callers only read):
```rust
pub fn get_defclause_clauses(&self, name: &str) -> Option<&[(Vec<TypeExpr>, TypeExpr, bool)]> {
    self.defclause_registrations.get(name).map(|v| v.as_slice())
}
```
Verify the call site (mod.rs ~6684, which `.clone()`s the result) still compiles — `.to_vec()` works on `&[T]` identically.

### E (L2) — accessors suppress `'a` — env.rs:~195, ~229

The accessors return data that lives `'a`, but elision ties the return to `&self`. Spell out `'a`:
```rust
pub fn types(&self) -> &'a TypeEnv { self.types }
pub fn get_binding_metadata(&self, name: &str) -> Option<&'a HashMap<String, WatAST>> {
    self.binding_metadata.and_then(|m| m.get(name))
}
```
(This composes with A — both are lifetime-honesty fixes. If rustc objects to `&'a` on `get_binding_metadata` because the `Option<&'a ...>` field needs `Copy`/deref handling, follow the borrow checker; the goal is the accessor exposes the real `'a` it holds.)

### F (L2) — `register` / `get` undocumented — env.rs:~184

Add WHY-level docs:
```rust
/// Register a function/builtin type scheme at `name`. Consumed by
/// `from_symbols` (user functions) and `register_builtins` (substrate primitives).
pub fn register(&mut self, name: String, scheme: TypeScheme) { ... }

/// Look up a function or builtin scheme by FQDN. For `def`-bound value types
/// use `get_defined_value_type`; for defclause dispatch use `get_defclause_clauses`.
pub fn get(&self, name: &str) -> Option<&TypeScheme> { ... }
```

### G (L2) — `from_symbols` WHAT-noise comment — env.rs:~125

The 6-line inline block before `env.binding_metadata = Some(&sym.binding_metadata)` repeats the struct + method docs. Compress to the one BEWARE the outer docs don't state:
```rust
// Read-only after freeze time — binding_metadata is populated before
// check_program runs; safe to borrow for the pass duration.
env.binding_metadata = Some(&sym.binding_metadata);
```

### H (L2) — missing rune at `rust_deps::get()` — mod.rs:~14671

The `let registry = crate::rust_deps::get();` reaches a write-once `OnceLock<RustDepsRegistry>` static — a read-only dispatch table, not domain state. Threading it through every infer signature would bloat every call site. This is a legitimate `ambient-context` — declare it conscious with a rune:
```rust
// rune:sequi(ambient-context) — rust-deps registry is a write-once dispatch
// table installed at startup; threading it through every infer/dispatch
// signature would bloat every call site for a read-only config surface, not
// domain state.
let registry = crate::rust_deps::get();
```

## Cadence (slow is smooth)

1. Baseline gates (confirm 890/0 · 8/0 · 3/0 · 3/0 · arc112 1/0 · clippy 894).
2. **A** (CheckSchemeCtx two lifetimes) → `cargo build --release --tests` → expect clean or borrow-checker guidance; iterate per rustc.
3. **B** (TypeEnv method extraction) → `cargo test --release --lib -p wat` (the enum-walk path is lib-tested) → 890/0.
4. **E** (accessor lifetimes — composes with A) → build.
5. **C/D/F/G** (env.rs doc + slice + comment) → build.
6. **H** (rune in mod.rs) → build.
7. Final gates: ALL of lib 890/0 · function 8/0 · probe_arc243_stone3 3/0 · probe_arc243_stone3_1 3/0 · arc112 1/0 · clippy ≤ 894 · workspace build clean · 4× :restricted-to green.
8. DO NOT COMMIT. DO NOT cast vigilia. Return paragraph.

## STOP triggers (REJECTION — surface verbatim)
1. Any gate regresses (lib<890 / function<8 / either probe<3 / arc112<1 / clippy>894 / workspace fails / any :restricted-to red)
2. A CONFUSING borrow error (not verbose) — pivot, surface, do NOT force
3. `unsafe` / `Box::leak` / `'static` transmute / re-clone / `Rc`/`Arc` re-wrap to satisfy a lifetime — STOP
4. B's extraction would require importing check-layer types INTO types.rs (it must not — the method uses only types.rs's own `TypeDef`/`EnumVariant`/`TypeExpr`) — if it does, surface
5. holon-rs touched (STOP-5)
6. Scope creep into mod.rs's 21k legacy beyond the named A + H sites
7. INTERSTITIAL touched / commit attempted / vigilia cast by sonnet
8. 60 min elapsed

## Return paragraph (≤ 200 words)
- Each fix A–H: landed (+ for A: the exact lifetime form rustc accepted; for B: confirm `crate::types::` paths gone from env.rs + the new method's location; for C: doc-vs-rename choice)
- Final gates (all 8 lines)
- Any confusing-error pivots or trap-doors
- Honest deltas (line counts; any place a lifetime forced a non-obvious form)

## Predicted band
**40-70 min Mode A.** Two L1s (A is the real one — lifetime separation; B is a clean extraction) + six small L2s. The lifetime work (A + E) is the substance; the rest is doc + type-tighten + one rune.
