# BRIEF — Stone 243.3.1 — mint `src/check/` home + CheckEnv borrow redesign

You are sonnet. Stone 243.3.1 — the failure-engineering roof on the CheckEnv mirror. Two moves in ONE stone: **(A)** mint the `src/check/` namespaced home; **(B)** redesign `CheckEnv` to BORROW its immutable inputs (`types`, `binding_metadata`) instead of deep-cloning them, making the duplication structurally unrepresentable. The redesigned `CheckEnv<'a>` is born as `src/check/env.rs` — the home's first honest neighbor.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Read FIRST (in order)

1. `docs/COMPACTION-AMNESIA-RECOVERY.md` (skim)
2. `scratch/FAILURE-ENGINEERING.md` — the discipline this stone embodies (eliminate the CLASS by making the wrong shape structurally unavailable)
3. `docs/arc/2026/05/243-conformare-error-shape/DESIGN.md` — § The CheckEnv mirror + the stone chain
4. **`docs/arc/2026/05/243-conformare-error-shape/DESIGN-STONE-243.3.1.md`** — THE design. Field classification table, target struct shape, constructor reshape, trap-doors. This BRIEF is the execution contract; the DESIGN is the spec. Read it fully.
5. `tests/probe_arc243_stone3_1_checkenv_borrow.rs` — the FM 2-bis probe (3 contracts). Currently FAILS to compile (disconfirmed at HEAD). Must COMPILE + PASS 3/0 post-stone.
6. `src/check.rs` — the ~21k-line file you will carve. Read the CheckEnv struct (1938-1989), from_symbols (2001-2021), the constructors (2023-2045), get_binding_metadata (2111), the redef_allowed mutation (2449).

## The failure being eliminated (read this as the WHY)

`CheckEnv` snapshots/deep-clones SymbolTable's + TypeEnv's immutable data because it OWNS instead of BORROWS. Three clone instances:
- `check.rs:2019` — `Arc::new(sym.binding_metadata.clone())` deep-clones the metadata HashMap
- `check.rs:2175` — `Arc::new(types.clone())` deep-clones the entire TypeEnv (finding ⑬)
- `freeze.rs:329` — `Arc::new(types.clone())` re-clones the TypeEnv into FrozenWorld (T2 — may be honest; see below)

The roof: make `CheckEnv` BORROW, so deep-clone-into-CheckEnv becomes a **compile error**. Not avoided — unrepresentable.

## Phase A — the carve (do this FIRST, verify green, THEN redesign)

### Step A1 — git mv to home
```
git mv src/check.rs src/check/mod.rs
```
This preserves all ~21k lines AND every `crate::check::X` import path (a module's own items are addressable as `crate::check::X` whether it's `check.rs` or `check/mod.rs`). Build must still be green after the bare move — verify with `cargo build --release --lib` before touching anything else. **This is the single most important checkpoint: the move alone changes nothing semantically.**

### Step A2 — confirm green post-move
```
cargo test --release --lib -p wat 2>&1 | tail -1   # expect 890/0 unchanged
```
If anything breaks from the bare move, STOP and surface — a clean `git mv` of a module to `mod.rs` should be transparent.

## Phase B — the borrow redesign + carve to env.rs

### Step B1 — reshape CheckEnv to borrow (in check/mod.rs first, or directly as you extract)

Per DESIGN-STONE-243.3.1 § Target struct shape:
```rust
pub struct CheckEnv<'a> {
    schemes: HashMap<String, TypeScheme>,             // OWNED (derived) — unchanged
    unit_variant_types: HashMap<String, TypeExpr>,    // OWNED (derived) — unchanged
    types: &'a TypeEnv,                                // was Arc<TypeEnv> — NOW BORROW
    defined_values: HashMap<String, TypeExpr>,        // OWNED (incremental) — unchanged
    defined_value_spans: HashMap<String, Span>,       // OWNED (incremental) — unchanged
    binding_metadata: Option<&'a HashMap<String, HashMap<String, WatAST>>>,  // was Arc<HashMap> — NOW BORROW
    redef_allowed: bool,                              // OWNED-MUTABLE (mid-pass) — unchanged
    defclause_registrations: HashMap<String, Vec<(Vec<TypeExpr>, TypeExpr, bool)>>,  // OWNED — unchanged
}
```

ONLY `types` and `binding_metadata` change. The other 6 stay exactly as they are (they are DERIVED or INCREMENTAL or mid-pass-MUTATED — NOT mirrors). Do NOT borrow them (T4).

### Step B2 — reshape the constructors
- `from_symbols(sym: &'a SymbolTable, types: &'a TypeEnv) -> CheckEnv<'a>`:
  - `types` is now the borrow (drop the `Arc<TypeEnv>` param).
  - `binding_metadata: Some(&sym.binding_metadata)` (drop the `Arc::new(.clone())`).
- `with_builtins_and_types(types: &'a TypeEnv) -> CheckEnv<'a>` — borrow.
- `with_types(types: &'a TypeEnv) -> CheckEnv<'a>` (private) — borrow.
- `with_builtins()` — **REMOVE IT** (T1). It builds an inline TypeEnv and wraps it; under the borrow it would return a CheckEnv borrowing a stack-local that's about to drop — impossible. Every caller binds the TypeEnv first then calls `with_builtins_and_types(&types)`. There are 3 standalone call sites (runtime.rs:3636, runtime.rs:12703, and tests). Each becomes a two-liner. For standalone construction `binding_metadata = None`.
- `get_binding_metadata`'s body: `self.binding_metadata.and_then(|m| m.get(name))` (it already returns `Option<&HashMap<…>>` — no signature change).

### Step B3 — the call-site cascade (substrate-as-teacher)
~72 `&CheckEnv` / `&mut CheckEnv` parameter sites across `check/mod.rs`, `src/function/infer.rs`, `src/function/mod.rs`. Let the borrow checker drive: build, read each error, add the lifetime where rustc names it. Most `&CheckEnv` elide to `&CheckEnv<'_>`; explicit `<'a>` only where a function ties input and output lifetimes.

**CRITICAL — `feedback_nonintuitive_error_is_pivot`:** a VERBOSE cascade (many sites, each naming itself) is mechanical — push through. A CONFUSING error (you can't tell what rustc wants, or the fix feels like fighting the type system) is a DESIGN SIGNAL — STOP, surface verbatim, do not force. Verbose ≠ confusing. If you find yourself adding `unsafe`, `Box::leak`, `'static` transmutes, or `.clone()` to "make the borrow checker happy" — STOP. Those are the anti-patterns; the borrow is supposed to be clean.

### Step B4 — the call sites that build CheckEnv
- `check.rs:2175` (now check/mod.rs): `CheckEnv::from_symbols(sym, &types)` — drop `Arc::new(types.clone())`. `types` is already `&TypeEnv` in `check_program`'s signature, so just pass it through.
- The 3 `with_builtins()` standalone sites: bind a `let types = TypeEnv::with_builtins();` then `CheckEnv::with_builtins_and_types(&types)`.

### Step B5 — T2: the freeze.rs:329 clone (verify, don't assume)
`freeze.rs:329` does `symbols.set_types(Arc::new(types.clone()))`. This clone persists types into the FrozenWorld, which OUTLIVES check_program's borrow. After B4, `check_program` borrows `&types` and RETURNS before freeze.rs:329 runs (the borrow is released). So freeze.rs:329 may legitimately remain — FrozenWorld OWNS its types for the program lifetime (persistence, NOT the duplication class).
- **Verify via the borrow checker:** if `FrozenWorld::freeze` can take ownership of the existing `types` value (it's `let mut types` on the stack, moved into freeze at freeze.rs:883-890), then `set_types(Arc::new(types))` — without `.clone()` — may work, killing the third clone.
- If removing the `.clone()` causes a borrow/move error (types used again after), the clone is HONEST — keep it, and note in your return why (persistence boundary, not duplication).
- Surface your verdict on freeze.rs:329 explicitly.

### Step B6 — carve CheckEnv to src/check/env.rs
Once the redesign compiles + tests pass in mod.rs:
- Move the `CheckEnv<'a>` struct + its `impl<'a> CheckEnv<'a>` block + the constructors → `src/check/env.rs`.
- `check/mod.rs` adds `mod env;` + `pub use env::CheckEnv;` (and `TypeScheme` if it lives with env — check what travels).
- Move only what's cohesive with CheckEnv. The walkers, infer, CheckError, etc. STAY in mod.rs (they're 243.6's neighbors, not this stone's).
- Verify the re-export preserves `crate::check::CheckEnv` + `wat::check::CheckEnv` (the probe imports the latter).

## Gates (ALL must hold post-stone)
```
cargo test --release --lib -p wat 2>&1 | tail -1                                   # >= 890 / 0
cargo test --release --test function 2>&1 | tail -1                                # 8 / 0
cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 | tail -1 # 3 / 0 (no regression)
cargo test --release --test probe_arc243_stone3_1_checkenv_borrow 2>&1 | tail -1   # 3 / 0 (now PASSES)
cargo test --release --test arc112_slice2b_process_send_recv 2>&1 | tail -1        # 1 / 0 (stays green)
cargo build --release --tests --workspace                                          # exit 0
cargo clippy --release 2>&1 | grep -cE "^warning:"                                 # <= 894
```

The :restricted-to integration tests are the BEHAVIORAL half — they must stay green (the borrowed binding_metadata read-through must check restricted calls identically):
```
cargo test --release --test wat_arc198_slice2_stone_1_inventory_wiring 2>&1 | tail -1
cargo test --release --test wat_arc198_slice2_stone_2_attribute 2>&1 | tail -1
cargo test --release --test wat_arc198_slice2_stone_3_apply 2>&1 | tail -1
cargo test --release --test wat_arc198_def_restricted 2>&1 | tail -1
```

## STOP triggers (REJECTION — surface verbatim)
1. The bare `git mv` (A1) breaks the build — a transparent move should not; investigate before proceeding
2. Lib < 890 / function < 8 / probe_arc243_stone3 < 3 / probe_arc243_stone3_1 not passing / arc112 not 1
3. clippy > 894 / workspace build fails
4. Any :restricted-to integration test regresses (the binding_metadata borrow broke read-through)
5. A CONFUSING borrow-checker error (not just verbose) — pivot, surface; do NOT force
6. You reach for `unsafe`, `Box::leak`, `'static` transmute, `Rc`/`Arc` re-wrapping, or a `.clone()` to satisfy the borrow — STOP; the borrow is meant to be clean
7. T4 violation — borrowing `schemes`/`unit_variant_types`/incremental fields (only `types` + `binding_metadata` borrow)
8. CheckEnv escapes to heap/field storage anywhere (would break the borrow premise — investigation says it never does; if it now does, surface)
9. holon-rs touched (STOP-5)
10. 180 min elapsed (the cascade is real; the bound is generous)
11. INTERSTITIAL touched / commit attempted / vigilia cast by sonnet

## Discipline
- **Sonnet writes substrate** (`feedback_sonnet_writes_substrate`) — you do the Rust; orchestrator briefs/scores/commits/casts vigilia.
- **Failure engineering** — the borrow makes the clone unrepresentable; that's the whole point. Don't settle for "avoided."
- **DO NOT commit** — orchestrator commits after vigilia REMARKABLE bar + SCORE.
- **DO NOT cast vigilia/conformare** — orchestrator-cast post-strike.
- **DO NOT write INTERSTITIAL.**

## Post-strike return (≤ 250 words)
- Phase A: bare `git mv` clean? (yes/no)
- Phase B: the 2 fields borrowed; the 3 clones' fate (binding_metadata ✓, check.rs:2175 ✓, freeze.rs:329 — KEPT-honest or KILLED, with the borrow-checker verdict)
- `with_builtins()` removal + the 3 standalone sites reshaped
- Call-site cascade: how many sites gained the lifetime; any that needed explicit `<'a>` vs elided
- CheckEnv carved to src/check/env.rs; re-export verified
- All gates (lib/function/both probes/arc112/clippy/workspace + the 4 :restricted-to tests)
- Any CONFUSING-error pivots or trap-door encounters (T1-T5)
- Honest deltas (line count; any place the borrow forced a non-obvious restructure)

## Predicted band
**90-180 min Mode A.** The carve is mechanical; the borrow reshape is small (2 fields, 4 constructors); the ~72-site lifetime cascade is the bulk — verbose but borrow-checker-guided. T2 (freeze.rs:329) is the one genuine investigation. The vigilia REMARKABLE bar follows (orchestrator-cast, separate round).
