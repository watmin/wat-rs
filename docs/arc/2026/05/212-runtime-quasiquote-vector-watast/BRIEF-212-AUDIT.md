# Arc 212 — BRIEF for slices γ + δ: comprehensive audit + walker migration to `children()`

**Slice scope:** Audit every function in `src/` and `crates/*/src/` that pattern-matches on `WatAST`. Classify each as Walker (recurses on children) or Leaf (decomposes one shape). For Walkers: migrate generic-recursion sites to use `WatAST::children()` (minted in slice β). Result: the "walker-skips-Vector" bug class is structurally eliminated.

**Origin:** Arc 212 expanded scope per failure-engineering doctrine. Slice α (t6 fix) revealed walker-divergence pattern in 9+ analogous walkers. Per-walker fixes would be N copies of the same logic. The honest fix is at the substrate layer: `WatAST::children()` ships once; walkers route through it; "miss Vector arm" structurally impossible.

**Closes:** Arc 212 closure-condition pre-requisite (slices γ + δ). When this work ships, slice ε (INSCRIPTION) becomes ready; arc 212 closes; arc 211 closure-condition #1 met (arc 213 is the other).

## The primitive (already minted in slice β)

```rust
// src/ast.rs (NEW method on WatAST)
impl WatAST {
    pub fn children(&self) -> &[WatAST] {
        match self {
            WatAST::List(items, _)
            | WatAST::Vector(items, _)
            | WatAST::StructPattern(items, _) => items,
            _ => &[],
        }
    }
}
```

This method is ALREADY shipped at the time you read this BRIEF. Verify with `grep -n "fn children" /home/watmin/work/holon/wat-rs/src/ast.rs`.

## Audit protocol

### Step 1: enumerate candidate sites

```bash
cd /home/watmin/work/holon/wat-rs
grep -rn "WatAST::List(" src/ crates/*/src/ 2>/dev/null | grep -v "//\|\.rs:[0-9]*:\s*//" > /tmp/audit-watast-list-sites.txt
wc -l /tmp/audit-watast-list-sites.txt
```

Expected: ~50 sites. Each is a candidate for classification.

### Step 2: classify each site

For each site, find the enclosing function (using surrounding code context). Classify:

**Walker** (must migrate): function recurses on children with the pattern:
```rust
match form {
    WatAST::List(items, _) => {
        // ... walker-specific logic (e.g., head-keyword checks) ...
        for child in items {
            walker_fn_name(child, ...);  // RECURSIVE CALL
        }
    }
    _ => { ... or fall-through ... }
}
```
OR the early-return form:
```rust
let WatAST::List(items, _) = node else { return; };
// ... logic ...
for child in items { walker_fn_name(child, ...); }
```

**Leaf-decomposition** (leave alone): function decomposes ONE shape without recursing on children. Examples:
- `if let WatAST::List(items, _) = ast { let head = items.first()?; ... }` — extracts head, doesn't recurse
- Parsers: `match form { WatAST::List(items, span) => (items, span), _ => return Err(...) }` — destructures for one parse path
- Classifiers: `fn variant_name(ast: &WatAST) -> &'static str { ... }` — labels variant; doesn't recurse
- Single-form handlers: `is_defmacro_form`, `is_define_dispatch_form`, etc. — check ONE shape

**Single-shape-walker** (case-by-case): walker that INTENTIONALLY only handles List (e.g., a verb-call detector). If the walker recurses but is asking "is this call shape X?" then it might be correct as-is. Document the reasoning; if unsure, migrate (the migration is safe — `children()` returns empty for non-compound shapes, so checks on call heads remain correct).

### Step 3: migrate each Walker

For each function classified as Walker:

**Before:**
```rust
fn walker_fn(node: &WatAST, ...) {
    match node {
        WatAST::List(items, _) => {
            // ... walker-specific logic (head checks, etc.) ...
            for child in items {
                walker_fn(child, ...);  // RECURSIVE CALL
            }
        }
        _ => {}
    }
}
```

**After:**
```rust
fn walker_fn(node: &WatAST, ...) {
    // Walker-specific List-head logic (if any) — pattern-match on List
    // explicitly for the per-shape checks the walker needs.
    if let WatAST::List(items, _) = node {
        // ... walker-specific logic (head checks, etc.) ...
    }
    // Generic recursion — arc 212 children() primitive handles
    // ALL compound shapes (List, Vector, StructPattern). The walker
    // cannot silently miss future AST variants.
    for child in node.children() {
        walker_fn(child, ...);
    }
}
```

**Critical:** preserve all walker-specific special-case logic. For example, `walk_for_deadlock` has special handling for `(:wat::core::let ...)` that skips sandbox boundaries. That logic STAYS. Only the generic-recursion site changes.

### Step 4: verify

After migrations:
1. `cargo build --release` — must compile clean
2. `cargo test --release --workspace --no-fail-fast 2>&1 | tail -20` — workspace count must NOT increase (currently 1: probe_lifeline_pipe_proof; that one is arc 213's territory and unrelated)
3. `cargo test --release --test wat_arc170_program_contracts` — 24/24 (t6 still passes; the substrate-level fix subsumes slice α)
4. SCORE-212-AUDIT.md inscribes:
   - Per-site classification (Walker / Leaf / Single-shape-walker, with reasoning)
   - Migration count
   - Per-walker post-migration verification (test or manual reasoning)
   - Workspace test result delta (expect: same count, no regressions)

## Known starting points (from orchestrator's preliminary audit)

**Confirmed Walkers (must migrate):**
- `src/resolve.rs::check_quasiquote_template` (~line 241) — recurses on List only; needs Vector + StructPattern via children()
- `src/check.rs::walk_for_bare_primitives` (~line 2673)
- `src/check.rs::walk_for_deadlock` (~line 3238) — note special-case for `:wat::core::let`
- `src/check.rs::walk_for_pair_deadlock` (~line 3765)
- `src/check.rs::walk_for_legacy_stream` (~line 2935)
- `src/check.rs::walk_for_legacy_telemetry_service` (~line 2974)
- `src/check.rs::walk_for_legacy_lru_cache_service` (~line 3019)
- `src/check.rs::walk_for_legacy_kernel_queue` (~line 3077)
- `src/check.rs::check_calls_for_sandbox_leak` (~line 2355)

**Confirmed already-correct (do NOT need migration but verify they use `children()` if it makes them simpler — optional):**
- `src/macros.rs::walk_template` — has explicit List + Vector arms
- `src/macros.rs::substitute_bindings` — has explicit List + Vector arms
- `src/runtime.rs::walk_quasiquote` — slice α added Vector arm
- `src/check.rs::walk_for_arc170_legacy` — has explicit List + Vector arms
- `src/check.rs::walk_for_bare_legacy_console` — has explicit List + Vector arms
- `src/check.rs::walk_for_def_restricted_call` — has explicit List + Vector arms
- Various in `src/closure_extract.rs` — verify each (preliminary grep showed multiple have Vector arms)

**Still to audit (~30+ sites):** `src/macros.rs` (many sites; most likely leaf-decomposition), `src/closure_extract.rs`, `src/types.rs`, `src/hash.rs`, `src/dispatch.rs`, `src/lower.rs`, `src/load.rs`, `src/config.rs`, `src/freeze.rs`, `src/form_match.rs`.

## Constraints

- DO NOT change walker-specific logic — only the generic-recursion site (the `for child in items { recurse(child) }` pattern)
- DO NOT add new test files (existing tests prove the migrations correct)
- DO NOT modify the `WatAST` enum or `children()` method (already minted in slice β)
- DO commit nothing — orchestrator commits atomically after independent verification
- DO use `git status --short` only; do not commit
- Workspace failure count target: 1 (only `probe_lifeline_pipe_proof`); ZERO regression

## Output structure (SCORE-212-AUDIT.md)

```markdown
# Arc 212 — SCORE: comprehensive walker audit + children() migration

## Summary
- N sites total inspected
- W classified as Walker (migrated)
- L classified as Leaf-decomposition (left)
- S classified as Single-shape-walker (case-by-case decision per site)
- M migration commits / Rust edits
- 0 regressions

## Per-site catalog
[Table or list: file:line, fn name, classification, migration done? evidence]

## Per-walker migration details
[For each migrated walker: before/after snippets; note walker-specific logic preserved]

## Verification
[cargo build output; cargo test workspace output; t6 still passes]

## Arc 211 tooling-validation extension
[Brief: the children() primitive demonstrates arc 211's tooling-doctrine — substrate owns; consumers benefit; failure class structurally eliminated]

## Mode classification (A/B/C/etc)
```

## Time prediction
60–90 min. Audit is bounded (~50 sites; per-site classification is fast); migration is uniform per-walker.

## STOP triggers
- Audit reveals a "Walker" whose recursion logic is fundamentally not migrable to `children()` (e.g., needs custom child-selection logic not captured by "all children") — surface explicitly in SCORE
- Workspace test count INCREASES from baseline (1) — something broke; surface the failing test + diagnose
- More than 20 walkers need migration (orchestrator's preliminary count was 9; significantly more would surface as a STOP for re-scoping)

## Tooling-proven-by-use validation

This slice is the SECOND validation pathway for arc 211 closure:
- Slice α validated arc 211b's panic-EDN format (precise diagnostic enabled the spot-fix)
- This slice validates arc 211's DOCTRINE OF SUBSTRATE OWNERSHIP — `children()` is the same shape as `process_stdio` (arc 211e) and `panic_hook::install` (arc 211a): substrate owns the discipline; consumers benefit; failure class structurally eliminated.

SCORE-212-AUDIT.md inscription should include a brief paragraph noting this doctrine cascade.

## Cross-references
- DESIGN.md § "Scope EXPANDED 2026-05-18 (post-slice-α)" — the locked expanded scope
- BRIEF-212.md — slice α's original BRIEF (preserved as historical record of the spot-fix that revealed the broader scope per `feedback_inscription_immutable`)
- SCORE-212.md — slice α's SCORE (preserved)
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition"
- DESIGN.md § "Scope EXPANDED 2026-05-18 (post-slice-α)" — the failure-engineering doctrine inscribed in-scope (component 3: eliminate the CLASS; substrate owns the recursion pattern; consumers benefit; "missed Vector arm in walker" structurally impossible)
