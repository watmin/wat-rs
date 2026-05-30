# BRIEF — rust_deps WARD — lift `custodia` + annihilate all failure domains (one stone)

You are sonnet. Ward the `src/rust_deps/` home — the oldest code in wat-rs, never formally warded. ONE stone: (A) lift the ownership cells into a new `custodia.rs` resident (intueri-named), (B) annihilate every finding the 7-spell vigilia surfaced. The home earns its `vigilatum` stamp only when ALL of it is clean. "Warded" means failure domains found and annihilated — not "compiles," not "looks clean."

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## ‼️ COMMIT DISCIPLINE ‼️
You make ZERO git mutations — NO commit, add, stash, reset; NO scratch files outside the named edits. `git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits the stone atomically after a vigilia re-cast confirms the ward. If you think a commit is needed, STOP and say so. (A prior sweep this session breached this — it is the #1 rejection trigger.)

## Read first
1. `docs/VIGILATUM.md` — the ward marker doctrine (you are earning the home its stamp)
2. `scratch/FAILURE-ENGINEERING.md` — "warded" = annihilation, not convergence-checkbox
3. `src/rust_deps/mod.rs` + `src/rust_deps/marshal.rs` in full (you'll edit both + mint custodia.rs)

## Pre-spawn state
HEAD `7552f158`. Working tree clean. Gates baseline: lib 890/0; clippy ≤894; workspace build clean. The home is `mod.rs` (280) + `marshal.rs` (776). NOTE: the "from_wat" hits in runtime.rs are the `:wat::holon::from-wat` BUILTIN — a DIFFERENT concept; do NOT touch them. The FromWat TRAIT lives only in marshal.rs + 1 codegen site.

## PHASE A — lift `custodia` (intueri placement verdict)

`marshal.rs` is a Level-2 name-lie: it promises "marshalling" but houses ownership primitives used across 8+ consumer sites that do zero marshalling. Carve them out.

**A1.** Mint `src/rust_deps/custodia.rs`. Move from marshal.rs (≈ lines 382–527):
- `struct ThreadOwnedCell<T>` + its `unsafe impl<T: Send> Sync` + `impl<T> ThreadOwnedCell<T>` (its methods)
- `struct OwnedMoveCell<T>` + its `unsafe impl Send/Sync` + `impl<T> OwnedMoveCell<T>`
- the section-divider doc comment introducing the cells
- **Verify marshal.rs:453 `impl<T> ThreadOwnedCell<T> {}`** — if it's a stray EMPTY impl block, it's dead code: DELETE it (don't carry it to custodia). If it has content the grep truncated, move it. Report which.

`custodia.rs` module doc (first line will get the vigilatum stamp at the END by the orchestrator — leave room; start the doc at line 1 normally):
```rust
//! `custodia` — ownership-scope custody primitives for the `:rust::` bridge.
//!
//! `ThreadOwnedCell<T>` and `OwnedMoveCell<T>` enforce the substrate's
//! ZERO-MUTEX ownership discipline: single-thread custody and single-move
//! custody, respectively. They hold custody of their inner value against
//! cross-thread access and double-consume — the cell IS the guard.
//! Consumed across the shim ecosystem (runtime, io, hologram, the shim
//! crates); not a marshalling concern (hence carved from marshal.rs).
```

**A2.** `mod.rs`: add `pub mod custodia;` beside `pub mod marshal;`. Move the `ThreadOwnedCell`/`OwnedMoveCell` re-exports from the `pub use marshal::{...}` line to a new `pub use custodia::{OwnedMoveCell, ThreadOwnedCell};`. External `crate::rust_deps::ThreadOwnedCell` paths MUST still resolve (surface re-export preserved) — `git grep "rust_deps::ThreadOwnedCell\|rust_deps::OwnedMoveCell"` to confirm the consumer paths, verify they still compile.

**A3.** `marshal.rs` module doc: drop the ownership-cells from its "what this provides" list (the existing "Scope discipline" paragraph already scoped them out — make structure match words).

## PHASE B — annihilate the findings

### B1 — exigere ×4 deferral-lies (FIX — affirmative-present)
- `mod.rs:40-43` — strip the `001-caching-stack/DESIGN.md` citation entirely (that arc is **DISCARDED 2026-04-29** — a deferral pointing at a dead tracker). Replace with affirmative present-scope: "Current implementation: program-global set-insert (one declaration anywhere enables it everywhere)." Drop the "planned upgrade" wish.
- `mod.rs:169-170` (`with_wat_rs_defaults`) — "Future wat-rs defaults, if any, would register here." → affirmative: "Currently a no-op alias for `new()` — wat-rs ships zero default shims (LRU moved to the sibling `wat-lru` crate)."
- `mod.rs:258-259` (`UseDeclarations`) — "Per-file enforcement is a planned upgrade." → "Scope: program-global (one use! anywhere enables the symbol everywhere)." Drop the planned-upgrade clause.
- `marshal.rs:13-14` — "`Vec`/tuple impls land when a caller demands them." → affirmative present-list of what IS impl'd (the Vec + tuple impls now EXIST — the comment is also stale). State the current impl set; drop the "land when demanded."

### B2 — intueri findings (FIX)
- **get()→registry rename.** `mod.rs:248` `pub fn get()` → `pub fn registry()`. Cascade the 3 LIVE call sites: `resolve.rs:95`, `check.rs:14675`, `runtime.rs:6204` (`crate::rust_deps::get()` → `::registry()`). Update the 3 COMMENT mentions (`compose.rs:128,177`, `harness.rs:156`) + the rune text at `check.rs:14671` that says "rust-deps registry … get()". `git grep "rust_deps::get()"` to confirm zero stragglers after.
- `marshal.rs` `rust_opaque_arc` doc — delete the "…actually no — callers use `downcast_ref_opaque` below" self-correction artifact. State the settled contract: "Validate `v` is a `RustOpaque` with the expected type path and return the inner `Arc`. Callers pass this to `downcast_ref_opaque` for a typed reference to the payload."
- `marshal.rs:306` `FromWat for Value` — add one line naming the invariant: "This impl is infallible — it accepts any `Value` without error (the only infallible `FromWat` member)."

### B3 — sequi ×2 missing runes (FIX — callers cleaned, roof)
Add `rune:sequi(ambient-context)` immediately before the `registry()` call (post-rename) at both `resolve.rs:95` and `runtime.rs:6204`, matching the existing rune at check.rs:14671:
```rust
// rune:sequi(ambient-context) — rust-deps registry is a write-once dispatch
// table installed at startup; threading it through every resolver/eval
// signature would bloat every call site for a read-only config surface,
// not domain state.
```

### B4 — temperare T1/T2 (FIX — these are CELL methods; they move to custodia AND get fixed there)
- **T1** `with_mut` (now in custodia.rs): `self.ensure_owner(op, span.clone())?` → `self.ensure_owner(op, span)?` (span is owned, used once; the clone-then-drop is pure waste).
- **T2** `ensure_owner` error path: `std::thread::current().id()` is called twice (condition + format!). Bind once: `let current = std::thread::current().id();` before the `if`, use `current` in both.

### B5 — temperare T3 — FromWat `Span` → `&Span` (FIX — the obvious flaw; proven O(n) waste)
The `FromWat::from_wat` trait takes `span: Span` BY VALUE, forcing a `.clone()` per element in `Vec`/`Option`/`Result`/tuple impls (span is invariant across the whole iteration — proven O(n) Arc churn).
- Trait sig (marshal.rs:50): `fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError>;`
- Update **10 impl blocks** (i64, f64, bool, String, (), Option, the tuple macro at ~191, Result, Vec, Value) — each takes `span: &Span`; the `TypeMismatch { span, ... }` constructions become `span: span.clone()` ONLY at the LEAF error site (where the owned Span is actually consumed into the error) — the recursive calls (`T::from_wat(x, op, span)`) now pass the borrow with NO clone.
- The 5 internal recursive calls (marshal.rs:169,217,255,256,281) drop `.clone()` — pass `span`.
- The ~26 test call sites: `Span::unknown()` → `&Span::unknown()` (bind a `let s = Span::unknown();` where needed for the borrow to outlive the call).
- The 1 external codegen site `crates/wat-macros/src/codegen.rs:167` — the emitted `from_wat(... , span)` call: emit `&span` (verify the macro's span var is owned at that point; if so, `&` it).
- **ToWat is UNTOUCHED** (asymmetric — `to_wat(self) -> Value`, no span).
- Borrow-checker is the teacher: VERBOSE cascade = push through; CONFUSING error = STOP + surface. No `unsafe`/`leak`/`'static` to satisfy a lifetime.

### B6 — temperare T4 — double type-path check (JUDGE in-flight)
`rust_opaque_arc` (≈363) and `downcast_ref_opaque` (≈539) both check `inner.type_path != expected_path`. When called in sequence (canonical path) it's checked twice. BUT `downcast_ref_opaque` is documented as independently callable, so its check can't be removed without an unsound entry point. VERDICT: **leave both** — the redundancy is defense-in-depth for the independent entry point (intueri + struere both judged it legitimate). Do NOT remove; do NOT rune (it's correct as-is). Confirm in your return you left it intentionally.

## Gates (all must hold)
```
cargo test --release --lib -p wat 2>&1 | tail -1            # 890/0
cargo test --release --test function 2>&1 | tail -1         # 8/0
cargo build --release --tests --workspace                   # clean (the codegen + cell-move touch the workspace)
cargo clippy --release 2>&1 | grep -cE "^warning:"          # <= 894
```
Plus the rust_deps marshal tests (the ~26 from_wat test calls you migrated) must pass:
```
cargo test --release --lib -p wat marshal 2>&1 | tail -1
```

## STOP triggers (REJECTION)
1. Any gate regresses · 2. ANY git mutation (see top) · 3. CONFUSING borrow error (not verbose) — pivot+surface · 4. `unsafe`/`Box::leak`/`'static`/re-clone to satisfy a lifetime · 5. Touching the `:wat::holon::from-wat` builtin in runtime.rs (different concept) · 6. ToWat changed (only FromWat takes span) · 7. holon-rs touched · 8. scope creep beyond the named findings + the custodia carve · 9. INTERSTITIAL touched · 10. 120 min elapsed

## Return paragraph (≤ 250 words)
- A: custodia.rs minted; cells moved; marshal.rs:453 stray-impl verdict (deleted/moved); mod.rs re-exports preserved (consumer paths confirmed compiling)
- B1: 4 deferral rewrites (confirm the DISCARDED-arc cite stripped)
- B2: get→registry (confirm grep clean) + doc artifact + infallibility line
- B3: 2 runes added
- B4: T1/T2 fixed in custodia
- B5: T3 — trait sig changed; impl count touched; call/test/codegen sites migrated; ToWat confirmed untouched; any confusing-error pivots
- B6: T4 left defensive (confirm)
- all gates; CONFIRM no commits/git mutations/scratch files

## Predicted band
**75-120 min Mode A.** The custodia carve + T3 trait migration are the substance; the rest are targeted edits. T3's cascade is contained (marshal.rs + 1 codegen line) but touches ~40 sites — verbose, borrow-checker-guided.
