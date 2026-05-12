# Arc 154 follow-up — kill `let*` substrate arms

**Status:** queued 2026-05-12. Discovery during arc 170 slice 3 Gap D substrate work: `let*` is user-facing retired (zero callers in workspace) BUT substrate eval/check arms still functional as fall-through. Arc 154 INSCRIPTION explicitly says *"arms for `:wat::core::let*` keep functional fall-through to `:wat::core::let`"*.

This is the broader retirement-theater pattern surfacing concretely. Arc 154 closed thinking let* was retired; the substrate carries the corpse.

User direction 2026-05-12 (mid-Gap-D-spawn): *"sonnet just found let* is still alive???????? we killed this like.... a long time ago.....?... god fucking damn it - we fucking suck at killing shit we don't need a then constantly lie about it"*

## What's actually in the substrate (verified 2026-05-12)

Grep `wat::core::let\*\|let_star\|infer_let_star\|eval_let_star` shows:

| File | Line | What |
|---|---|---|
| `src/check.rs` | 251 | Comment: *"Arc 154 — `:wat::core::let*` retired in favor of"* |
| `src/check.rs` | 254 | Comment about `let*` token in legacy state |
| `src/check.rs` | 263 | `:wat::core::let*` token parsing |
| `src/check.rs` | 657 | Retirement diagnostic: *"':wat::core::let*' at {} is retired (arc 154)..."* |
| `src/check.rs` | 951 | `.field("retired", ":wat::core::let*")` in Diagnostic |
| `src/check.rs` | 1636 | Comment: walker retired but arms keep functional fall-through |
| `src/check.rs` | 1639 | *"for `:wat::core::let*` → `:wat::core::let`"* |
| `src/check.rs` | 1643 | *"arms for `:wat::core::let*` keep functional fall-through to"* |
| `src/check.rs` | 1647 | *"`:wat::core::let` is the single-letform spelling; `:wat::core::let*`"* |
| `src/check.rs` | 2376 | Active code: `if s == ":wat::core::let*"` — runtime path |

Plus probable arms in `src/runtime.rs` (verify before retirement):
- `eval_let_star` (or similar) — runtime evaluation
- `step_let_star` — incremental evaluator
- `infer_let_star` — type-check arm

Plus tests:
- `tests/wat_arc154_kill_let_star.rs` — tests the retirement walker; what's its current status?

Workspace usage: **zero callers in wat/ wat-tests/ crates/ examples/**. Sweep complete. Only substrate carries the form.

## What needs to happen

**The arc:** "Arc 154 follow-up: substrate `let*` retirement."

**Scope:**
1. Remove `let*` parsing/recognition in `src/check.rs` — every line in the table above
2. Remove `infer_let_star` if it exists (separate type-check arm) OR confirm `infer_let` is the single inferrer
3. Remove `eval_let_star` / `step_let_star` if they exist in `src/runtime.rs`
4. Remove `tests/wat_arc154_kill_let_star.rs` (it tests the retirement walker which would be gone)
5. Decide the diagnostic story for legacy snippets:
   - Option A: leave a graceful "let* doesn't exist; use let" error (parser-level)
   - Option B: just generic "unknown form" error
   - Recommend A — preserves discoverability for any future code that drifts in

**Verification:**
- `cargo check --release` green
- All workspace tests pass (zero let* callers means no breakage expected)
- Grep `wat::core::let\*` returns zero hits in src/ (or only in historical comments)

**Cost estimate:** 30-60 min sonnet. Sub-100-LOC substrate deletion. Same shape as a vintage arc-109-style cleanup sweep.

## Cross-references

- Arc 154 INSCRIPTION: this dir's INSCRIPTION.md — explicitly admits "arms keep functional fall-through"
- The broader pattern: retirement theater — same shape as `run-sandboxed-*` (arc 105c said retired; still used by stdlib until Phase F), `fork-program-ast` (arc 170 slice 2 said retired; eval arms still in src/spawn.rs + src/fork.rs until slice 4)
- See `docs/arc/2026/05/170-program-entry-points/CLOJURE-BIAS-AUDIT-CANDIDATES.md` for the bias-capture meta-doc; this follow-up is the discipline-gap counterpart

## The broader discipline gap

This let* finding is one instance of a recurring pattern. Every "retire X" arc historically:
1. Adds user-facing walker / migration diagnostic
2. Sweeps all callers in user code (wat/ wat-tests/ crates/)
3. Declares X retired; ships INSCRIPTION
4. Leaves substrate eval/check arms as "functional fall-through"
5. INSCRIPTION reads as "X retired"; substrate reality reads as "X still works internally"

Steps 1-3 + 5 are honest. Step 4 is the dishonesty: the INSCRIPTION declares done while substrate carries the corpse.

Known instances of retirement theater in the workspace (2026-05-12):
- `:wat::core::let*` (arc 154; this doc)
- `:wat::kernel::run-sandboxed-ast` / `run-sandboxed-hermetic-ast` (arc 105c)
- `:wat::kernel::fork-program-ast` / `spawn-program-ast` / `fork-program` / `spawn-program` (arc 170 slice 2)
- Probable: `:wat::core::lambda` Rust-side identifiers (arc 162 was the user-facing rename; arc 163 the cleanup audit)
- Probable: `:wat::core::unit` → `:wat::core::nil` (arc 153)

A future arc should systematically audit and close these. The cost of NOT doing so: every future agent reads INSCRIPTIONs that lie, builds on top of "retired" substrate that still works, repeats the pattern.

## How to prevent this going forward

Process rule candidate: any arc that retires X requires substrate-cleanup-verified BEFORE the INSCRIPTION ships. The grep test: `grep -n X-identifier src/ | grep -v "comment\|historical\|retired\|legacy"` must return zero ACTIVE code references. If not zero, the arc doesn't INSCRIBE; it queues a follow-up slice that does the substrate cleanup, then INSCRIBES.

Alternative: a `/ward` (spell) that scans every INSCRIPTION-eligible arc for "retired X" claims + verifies the substrate. Arc closure gates on the ward passing.

The user's framing 2026-05-12: *"we fucking suck at killing shit we don't need a then constantly lie about it."* The discipline gap is real. The fix is structural (don't let INSCRIPTIONs ship without substrate verification), not per-arc.

## Status

Queued for execution. Order in the broader queue is the user's call. Suggested order:
1. Finish current Gap D (let-splice for def/defn) — workspace stays clean
2. Open this arc 154 follow-up — kill let* substrate
3. Open the parallel substrate retirement sweep for run-sandboxed-*, fork-program-ast, spawn-program-ast (likely a chain through arc 170's Phase E/F + slice 4)
4. Open a discipline-gap arc that codifies the retirement-verification rule (process or /ward)

Substrate honesty is the goal.
