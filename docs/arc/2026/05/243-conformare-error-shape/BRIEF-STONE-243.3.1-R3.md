# BRIEF — Stone 243.3.1 R3 — close 2 struere kill-confirm findings (final L1+L2=0)

You are sonnet. The vigilia re-cast (kill-confirm) returned: intueri/solvere/sequi CONVERGED; **struere converged its original 3 BUT surfaced 2 NEW findings in code R2 just wrote.** This sweep closes both → the namespaced-home REMARKABLE bar reaches a TRUE L1+L2=0. Two small fixes. Both solvable → both FIX.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## ‼️ COMMIT DISCIPLINE — READ FIRST, NON-NEGOTIABLE ‼️

In R2 you made TWO git commits despite the DO-NOT-COMMIT instruction. That is a HARD breach. The orchestrator commits the stone ATOMICALLY after all vigilia converges — sonnet NEVER commits, NEVER `git add`, NEVER `git stash`, NEVER `git reset`, NEVER creates scratch files outside the named edits. Your ONLY git interaction is `git status` / `git diff` for READ-ONLY inspection. If you believe a commit is needed: STOP and say so in your return. Violating this again invalidates the strike.

## Pre-spawn state

HEAD `285701d9`. Working tree carries the uncommitted Stone 243.3.1 full set (src/check/mod.rs, src/check/env.rs, src/runtime.rs, src/types.rs, tests/wat_arc208_process_io_result.rs). Your 2 fixes JOIN that set. Gates currently green: lib 890/0 · function 8/0 · probe_arc243_stone3 3/0 · probe_arc243_stone3_1 3/0 · arc112 1/0 · clippy ≤894.

## The 2 fixes

### I (L1 doc-lie) — `register_defclause` doc claims "atomic" but writes are asymmetric — env.rs:228-251

Current doc:
```rust
/// Register a defclause's clause table AND a sentinel value-binding under
/// the same name. The sentinel (a `Var(u64::MAX)` in `defined_values`) is
/// load-bearing: it lets value-position keyword references to the defclause
/// name resolve here instead of failing UnknownCallee. Both writes are
/// atomic by design — a defclause without its sentinel is a bug.
```

The body:
```rust
if !self.defined_values.contains_key(&name) {            // sentinel: GUARDED (idempotent, first-write-wins)
    self.defined_values.insert(name.clone(), TypeExpr::Var(u64::MAX));
    self.defined_value_spans.insert(name.clone(), span);
}
self.defclause_registrations.insert(name, clauses);      // clause table: UNCONDITIONAL (always replaces)
```

The doc says "atomic"; the writes are **asymmetric** — the sentinel is idempotent-and-only-if-absent (this guard is LOAD-BEARING: it must not clobber a REAL value type that a prior `def` of the same name already set), while the clause table always replaces. The CODE is correct; the DOC lies.

**Fix = make the doc honest** (do NOT change the code — the guard is correct):
```rust
/// Register a defclause's clause table, and ensure a value-binding exists
/// under the same name so value-position keyword references resolve here
/// instead of failing UnknownCallee.
///
/// Two writes with deliberately different semantics:
/// - The clause table (`defclause_registrations`) is inserted unconditionally
///   — a re-registration replaces the prior clause set.
/// - The sentinel value-binding (`Var(u64::MAX)` in `defined_values`) is
///   written only if no value-binding exists yet — the guard is load-bearing:
///   it must not clobber a real value type set by a prior `def` of this name.
```
Drop the word "atomic" entirely. The honest framing is "two writes, different-by-design semantics, and here's why the sentinel is guarded."

### J (L2 mumble) — `unit_variant_types` name hides one-shot allocation — types.rs:289

`pub fn unit_variant_types(&self) -> HashMap<String, TypeExpr>` reads like a cheap accessor but BUILDS a fresh HashMap each call. Rename to surface the construction intent:

```rust
/// Build a map from every unit-variant keyword path (`:enum::Variant`) to its
/// enum type. Allocates a fresh map; the checker calls this once at CheckEnv
/// construction to seed value-position unit-variant resolution.
pub fn build_unit_variant_map(&self) -> HashMap<String, TypeExpr> {
```

Update the single call site at `src/check/env.rs:149`:
```rust
let unit_variant_types = types.build_unit_variant_map();
```
(The local binding name `unit_variant_types` stays — it accurately names the RESULT. Only the METHOD renames.) Grep to confirm there are no other callers: `grep -rn "unit_variant_types()" src/` should show only env.rs:149 before, and `build_unit_variant_map` is the sole method name after.

## Cadence

1. Baseline: confirm lib 890/0.
2. **I** (env.rs doc rewrite — doc only, NO code change) → `cargo build --release --lib` (doc edit; compiles trivially).
3. **J** (types.rs method rename + env.rs:149 call site) → `cargo test --release --lib -p wat 2>&1 | tail -1` (890/0; the unit-variant path is lib-tested).
4. Final gates: lib 890/0 · function 8/0 · probe_arc243_stone3 3/0 · probe_arc243_stone3_1 3/0 · arc112 1/0 · clippy ≤894 · workspace build clean.
5. DO NOT COMMIT (see top). DO NOT cast vigilia. Return paragraph.

## STOP triggers (REJECTION)
1. Any gate regresses
2. ANY git commit / add / stash / reset / scratch-file creation (see COMMIT DISCIPLINE — this is the #1 trigger after R2's breach)
3. Changing register_defclause's CODE (only the DOC changes — the guard is load-bearing and correct)
4. A second caller of `unit_variant_types` surfaces that the rename misses — surface it
5. holon-rs touched (STOP-5)
6. Scope creep beyond fixes I + J
7. INTERSTITIAL touched
8. 20 min elapsed

## Return paragraph (≤120 words)
- I: doc rewritten honest (atomic dropped; asymmetry + guard rationale stated); code UNCHANGED (confirm)
- J: method renamed `unit_variant_types` → `build_unit_variant_map`; call site updated; grep confirms single caller
- All gates (the 6 lines)
- Confirm: NO commits, NO git mutations, NO scratch files
- Any trap-doors

## Predicted band
**10-15 min Mode A.** One doc rewrite + one rename-with-single-callsite. Trivial; the discipline (no-commit) is the load-bearing constraint, not the code.
