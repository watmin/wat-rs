# DESIGN — Stone 243.3.1 — mint `src/check/` home + CheckEnv borrow redesign

**Parent:** Stone 243.3 (spawn-child; 243.3 cannot close until this closes).
**Pivot:** 2026-05-30. Pulled forward from the sketched 243.6 because the CheckEnv mirror is a LIVE failure (failure engineering: stop + eliminate now) and "through the roof" requires the grimoire requires a namespaced home.

## The failure being eliminated (the CLASS, not the symptom)

**Class:** a struct snapshots/deep-clones another struct's immutable data because it OWNS instead of SHARES/BORROWS. The duplication is one logical thing stored in two places.

**Instances (all in the check pass):**
1. `CheckEnv.binding_metadata = Arc::new(sym.binding_metadata.clone())` — check.rs:2019. Deep clone of `HashMap<String, HashMap<String, WatAST>>`.
2. `CheckEnv::from_symbols(sym, Arc::new(types.clone()))` — check.rs:2175. Deep clone of the entire `TypeEnv` (finding ⑬).
3. `symbols.set_types(Arc::new(types.clone()))` — freeze.rs:329. SECOND deep clone of the same `TypeEnv` into the persisted FrozenWorld.

**The roof (failure-engineering grade):** don't *avoid* the clone — make the clone-into-CheckEnv **structurally unrepresentable**. A field of type `&'a TypeEnv` cannot hold an owned copy; the compiler rejects `Arc::new(x.clone())` into it. Option A (make fields `Arc`, share by handle) only *avoids* the clone by convention — a future edit can write `Arc::new(x.clone())` and reconstruct the duplication. Option B (borrow) makes the situation never constructible. **B is the path** (four-questions verdict: A fails Honest; B earns the third checkmark).

## Field classification (verified by topology investigation 2026-05-30)

| Field | Kind | Redesign disposition |
|---|---|---|
| `schemes: HashMap<String, TypeScheme>` | DERIVED (computed from `sym.functions` via `derive_scheme_from_function` + builtins) | OWNED — stays. Not a mirror; it's a transform. |
| `unit_variant_types: HashMap<String, TypeExpr>` | DERIVED (walked from TypeEnv enum decls) | OWNED — stays. |
| `types: Arc<TypeEnv>` | MIRROR (deep-cloned at 2175) | **BORROW → `&'a TypeEnv`.** Read-only after the register phase (freeze.rs:780-782 build it before check). |
| `defined_values` | INCREMENTAL (built during check_program loop) | OWNED — stays. |
| `defined_value_spans` | INCREMENTAL | OWNED — stays. |
| `binding_metadata: Arc<HashMap<…>>` | MIRROR (deep-cloned at 2019) | **BORROW → `Option<&'a HashMap<…>>`.** Read-only after register phase (populated freeze.rs:862, before check). None in standalone constructors. |
| `redef_allowed: bool` | OWNED-MUTABLE (seeded from sym, then mutated mid-pass at check.rs:2449 on `set-redef!`) | OWNED — stays. NOT a mirror; it diverges from `sym.redef_allowed` during the pass. |
| `defclause_registrations` | INCREMENTAL | OWNED — stays. |

**Only 2 fields borrow:** `types` and `binding_metadata`. The rest are legitimately owned (derived or incremental or mid-pass-mutable).

## Target struct shape

```rust
pub struct CheckEnv<'a> {
    schemes: HashMap<String, TypeScheme>,
    unit_variant_types: HashMap<String, TypeExpr>,
    types: &'a TypeEnv,                                   // was Arc<TypeEnv> (cloned)
    defined_values: HashMap<String, TypeExpr>,
    defined_value_spans: HashMap<String, Span>,
    binding_metadata: Option<&'a HashMap<String, HashMap<String, WatAST>>>,  // was Arc<HashMap> (cloned)
    redef_allowed: bool,
    defclause_registrations: HashMap<String, Vec<(Vec<TypeExpr>, TypeExpr, bool)>>,
}
```

`get_binding_metadata` already returns `Option<&HashMap<…>>` — its body becomes `self.binding_metadata.and_then(|m| m.get(name))`. No caller signature change.

## Constructor reshape

| Constructor | Before | After |
|---|---|---|
| `from_symbols(sym: &SymbolTable, types: Arc<TypeEnv>)` | clones types into Arc; deep-clones binding_metadata | `from_symbols(sym: &'a SymbolTable, types: &'a TypeEnv) -> CheckEnv<'a>` — `types` borrowed; `binding_metadata: Some(&sym.binding_metadata)` |
| `with_builtins_and_types(types: Arc<TypeEnv>)` | owns Arc | `with_builtins_and_types(types: &'a TypeEnv) -> CheckEnv<'a>` |
| `with_types(types: Arc<TypeEnv>)` | private | `with_types(types: &'a TypeEnv)` |
| `with_builtins()` | builds inline TypeEnv, wraps Arc | **TRAP-DOOR (T1).** Cannot return `CheckEnv<'static>` borrowing a stack-local TypeEnv. RESOLUTION: caller binds the TypeEnv first. The 3 standalone call sites become two-liner: `let types = TypeEnv::with_builtins(); let env = CheckEnv::with_builtins_and_types(&types);`. `with_builtins()` itself is REMOVED (it cannot honestly exist under the borrow — keeping it would force a leak or a static). Standalone `binding_metadata` = `None`. |

## Call-site cascade (verified counts)

- `&CheckEnv` / `&mut CheckEnv` parameter sites: **~72** across `src/check.rs`, `src/function/infer.rs`, `src/function/mod.rs`. Each gains the lifetime (`&CheckEnv` → `&CheckEnv<'_>` or explicit `<'a>` where needed; most elide). **The borrow checker is the substrate-as-teacher** — every site that needs the annotation names itself. `feedback_nonintuitive_error_is_pivot`: if an error gets *confusing* (not just verbose), STOP and surface — confusing means the design is wrong, not the cascade.
- Standalone `with_builtins()` call sites: **3** (runtime.rs:3636, runtime.rs:12703, tests) → each becomes the two-liner.
- `from_symbols` call site: **1** (check.rs:2175) → `CheckEnv::from_symbols(sym, &types)` (drop `Arc::new(.clone())`).
- freeze.rs:329 second clone: addressed by making `types` flow as a borrow/Arc that's shared, not re-cloned. (If `types` upstream stays a bare `TypeEnv`, freeze.rs:329's clone-into-FrozenWorld may legitimately remain — FrozenWorld PERSISTS the types and outlives the stack. This clone is the ONE that may be honest: FrozenWorld owns its types for the program lifetime. VERIFY: if FrozenWorld can take ownership of the existing `types` rather than cloning, the clone dies; if FrozenWorld must coexist with check's borrow, it stays. Sonnet determines via the borrow checker; surface the verdict.)

## The home carve (the namespaced-home mint)

This stone is BORN as a home, not edited in flat check.rs:
1. `git mv src/check.rs src/check/mod.rs` — preserves all ~21k lines + every `crate::check::X` import path (mod.rs re-exports its own contents automatically).
2. Carve the redesigned `CheckEnv<'a>` + its impl block + `from_symbols`/`with_*` constructors → `src/check/env.rs`. `mod.rs` adds `mod env; pub use env::CheckEnv;` (+ `TypeScheme` if it travels with env).
3. `src/check/env.rs` is the home's FIRST honest neighbor. The remaining ~21k lines stay in `mod.rs` until future stones (243.6+) carve more neighbors (error.rs, walkers, infer).
4. vigilia REMARKABLE bar (L1+L2=0) on `src/check/` as a whole — the grimoire now fires on this home every commit. This is WHY the redesign rides the carve: the bar is only real when the home exists.

## FM 2-bis disconfirming probe

The probe asserts the POST-state contract; it must FAIL to compile at HEAD (pre-stone) and PASS post-stone. Shape — a compile-level assertion that CheckEnv borrows:

```rust
// tests/probe_arc243_stone3_1_checkenv_borrow.rs
//! FM 2-bis — Stone 243.3.1. CheckEnv borrows its immutable inputs;
//! deep-clone-into-CheckEnv is structurally unrepresentable.

// CONTRACT 1: CheckEnv carries a lifetime (borrows, not owns-by-clone).
// Pre-stone CheckEnv has NO lifetime param → this fails to compile.
fn _contract_checkenv_is_lifetimed<'a>(_e: &wat::check::CheckEnv<'a>) {}

// CONTRACT 2: from_symbols takes types BY REFERENCE, not Arc<TypeEnv>.
// Verified at the type level — pre-stone signature is Arc<TypeEnv>.
// (Asserted via a function pointer coercion to the post-stone signature.)

// CONTRACT 3 (runtime, behavioral): a program that exercises binding_metadata
// (a :restricted-to call-site check) still type-checks correctly after the
// borrow redesign — the read-through path works identically.
```

Pre-stone: CheckEnv has no `<'a>` → contract 1 fails to compile → probe disconfirms (intended). Post-stone: compiles + passes 3/0.

## Gates (must hold)

- lib ≥ 890 / 0
- tests/function 8 / 0
- probe_arc243_stone3 (TypeError) 3 / 0 — must not regress
- probe_arc243_stone3_1 (this stone) 3 / 0 — must pass post-stone
- workspace test-build clean
- clippy ≤ 894
- vigilia on `src/check/` : L1 + L2 = 0 (REMARKABLE bar)

## Trap-doors

| # | Risk | Resolution |
|---|---|---|
| **T1** | `with_builtins()` can't borrow a stack-local TypeEnv | Remove it; callers bind TypeEnv first + use `with_builtins_and_types(&types)`. 3 sites. |
| **T2** | freeze.rs:329 clone may be honest (FrozenWorld persists types beyond check's borrow) | Sonnet verifies via borrow checker: if FrozenWorld can OWN the existing types, clone dies; if it must coexist with check's `&types`, the clone stays as a legitimate ownership boundary (NOT the eliminated class — persistence ≠ duplication). Surface the verdict honestly. |
| **T3** | Lifetime infects a signature that stores CheckEnv beyond check_program's frame | Investigation verified CheckEnv NEVER escapes to heap (0 Box/Arc/Vec/Option/field storage). If a new escape surfaces, STOP — the premise broke. |
| **T4** | `schemes`/`unit_variant_types` tempting to also borrow | NO — they're DERIVED (computed), not mirrors. Borrowing them would require computing-then-borrowing, which doesn't help. They stay owned. Scope discipline: only `types` + `binding_metadata` borrow. |
| **T5** | A confusing borrow-checker error mid-cascade | `feedback_nonintuitive_error_is_pivot`: confusing ≠ verbose. Verbose-but-mechanical = push through (substrate-as-teacher). Confusing = STOP, surface — design defect. |

## What this stone does NOT do

- Does NOT carve CheckError (that's 243.6, growing the now-existing home).
- Does NOT fuse the walker chain or fold collect_hints (243.6).
- Does NOT touch SymbolTable's own fields (binding_metadata stays `HashMap` on SymbolTable — only CheckEnv stops cloning it; SymbolTable is the owner).
- Does NOT thread `Arc<SymbolTable>` end-to-end (infeasible — freeze mutates symbols post-check).

## Spawn-block

243.3.1 is a spawn-child of 243.3. 243.3 closes ONLY after 243.3.1 closes. Wind down from completion: 243.3.1 lands → SCORE → close → THEN 243.3's tail (SCORE Phase B + close) unwinds.
