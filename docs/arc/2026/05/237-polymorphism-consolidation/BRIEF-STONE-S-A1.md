# BRIEF — Stone S-A1 — `assignable` choke point (subtyping at the arg boundary)

**Status:** READY TO SPAWN. `model: "sonnet"`.
**Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (verify with `pwd` first; reject any
path containing `.claude/worktrees/`; use `git -C` if needed).

## What to do

Teach the type checker's argument-acceptance to consult the is-a hierarchy S-A
shipped. Today a value typed `:my::Circle` is REJECTED at a `[v <- :wat::Record]`
parameter (records are nominal; `unify(:my::Circle, :wat::Record)` → Err). S-A1
makes Liskov substitution work at the call-arg boundary: **a subtype is accepted
where its supertype is wanted.**

**Two mechanical changes, `src/check.rs` ONLY:**

1. **Mint `fn assignable`** (module-level, place it right after `fn unify`,
   ~line 14780 region). Directional-subtype-FIRST (mutation-free), then ordinary
   `unify` — copy verbatim:

   ```rust
   /// Arg-boundary acceptance: is `actual` assignable to `expected`?
   /// Liskov — a subtype is accepted where its supertype is wanted. Checks the
   /// `typesub` hierarchy FIRST (mutation-free; only concrete distinct paths with a
   /// registered edge), then falls through to ordinary unification (behaviour
   /// unchanged for every other pair). Peels each side exactly as `unify` does at
   /// its head (line ~14633): `reduce(&walk(x, subst), subst, types)`.
   fn assignable(
       actual: &TypeExpr,
       expected: &TypeExpr,
       subst: &mut Subst,
       types: &TypeEnv,
   ) -> bool {
       let a = reduce(&walk(actual, subst), subst, types);
       let e = reduce(&walk(expected, subst), subst, types);
       if let (TypeExpr::Path(ap), TypeExpr::Path(ep)) = (&a, &e) {
           if ap != ep && crate::types::is_subtype(ap, ep, types) {
               return true;
           }
       }
       unify(actual, expected, subst, types).is_ok()
   }
   ```

2. **Reroute the 8 call-arg boundary sites** from
   `unify(<actual>, <expected>, <subst>, env.types()).is_err()` to
   `!assignable(<actual>, <expected>, <subst>, env.types())` — keeping each site's
   EXISTING borrow form on the first two args (some are `&arg_ty`, some `arg_ty`;
   some `&expected`, some `expected`):

   - **6386** — single-arg call (`callee: k`, param `#1`).
   - **7025**, **7079** — multi-arg call (`callee: k`, param `#{i+1}`).
   - **7213** — value-head application (`callee: "(value head)"`).
   - **10256**, **10365** — 236.2-harvested single-arg (`callee: callee`, param `arg`).
   - **12044** — multi-arg (`callee: callee_label`).
   - **6867** — defclause clause-MATCH. Different surrounding code: the `.is_err()`
     branch does `all_match = false; continue 'outer` (NOT an error push). Same
     reroute: `if !assignable(arg_ty, expected_ty, &mut clause_subst, env.types()) {`.
     This makes a defclause whose clause param is `:wat::Record` MATCH a subtype value.

   (Line numbers HEAD-current at `6df86955`; check.rs drifts — re-locate each by the
   `unify(...&arg_ty...|arg_ty...)` + the `TypeMismatch{callee,param}` / clause-match
   shape shown above. There are EXACTLY these 8. **Leave 14049 / 14099 untouched** —
   arc-146 Dispatch, retiring in 237.7.)

Make `tests/probe_arc237_sA1_assignable.rs` go **6/6** (3 currently RED:
probe_01/02/05; 3 green guards: 03/04/06 must STAY green).

This is **`src/check.rs` only**. NO `Record.wat` (no constructor-return flip — not
needed; a `[c <- :my::Circle]` annotation already yields a subtype-typed value). NO
`types.rs` (`is_subtype` shipped in S-A). NO `runtime.rs`. NO holon-rs (STOP-5).

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-A1-assignable.md`
   — **especially the bottom section "POST-B.2 SCOPE CORRECTION — GROUNDED"** (it is
   authoritative; the body above it is the older inert-mechanism framing — the grounded
   section supersedes it: no constructor flip, wat-surface probe, the exact reroute set).
2. `tests/probe_arc237_sA1_assignable.rs` — **LOAD-BEARING** 6 contracts.
3. `src/check.rs` lines **14621–14791** (`unify` + its head peel `reduce(&walk(...))`
   at 14633-14634; `walk` at 14780) and **14953** (`reduce`) — the helpers `assignable`
   reuses. Confirm `reduce`, `walk`, `crate::types::is_subtype` are all in scope.
4. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-A.md` — the
   predecessor (it shipped `is_subtype` + `typesub` + the seeded roots that `assignable`
   now consults). Mirror its SCORE shape.

## Discipline

- `src/check.rs` ONLY. If you find yourself editing any other file → STOP.
- `assignable` checks subtype FIRST, mutation-free, THEN unify. Do NOT add a directional
  arm INSIDE `unify` (that would spray subtyping into return-position + symmetric uses —
  the symmetric-leak class S-A's design explicitly rejected). Keep it a wrapper.
- Match each reroute site's existing borrow form; don't change the `TypeMismatch{...}`
  push bodies or the 6867 `all_match`/`continue` control flow — only the condition.

## STOP triggers (REJECTION — not permission to defer)

1. Compile errors not traced to a probe contract.
2. Lib baseline drops below **827** for ANY reason.
3. 70 min elapsed (STOP-3); 95 min (STOP-4 hard kill).
4. Any file other than `src/check.rs` touched (STOP — esp. Record.wat / a constructor
   flip / runtime.rs / types.rs / holon-rs).
5. Probe doesn't reach 6/6.
6. Any records-thread predecessor probe regresses (S-A 10/10, S-B.1 6/6, S-B.2 5/5).
7. You feel the urge to flip a constructor return, touch a holon flavor, or add a
   directional arm inside `unify` — STOP; none are in scope.

## Regression suite (re-run all; expect green)

```
cargo test --release --test probe_arc237_sA1_assignable        # 6/6 (the target)
cargo test --release --lib -p wat                              # >= 827, 0 failed
cargo test --release --test probe_arc237_sA_hierarchy          # 10/10
cargo test --release --test probe_arc237_sB1_recordtype        # 6/6
cargo test --release --test probe_arc237_sB2_defrecord_recordtype  # 5/5
```

## FM 2-bis evidence

`tests/probe_arc237_sA1_assignable.rs` (committed `6df86955`). Pre-stone: probe_01
(single-arg subtype) + probe_02 (multi-arg subtype) + probe_05 (transitive) FAIL;
probe_03 (directional rejection) + probe_04 (exact-match) + probe_06 (no-edge) pass.
Post-stone: 6/6.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-A1.md` (NEW). Mirror
SCORE-STONE-S-A: scorecard (compile clean; **S-A1 probe 6/6 LOAD-BEARING**; lib 827;
the records-thread regression suite green; check.rs only) → the `assignable` fn + the
8 reroutes (list the line each landed at) → honest deltas → working tree. DO NOT commit
(orchestrator commits).

## Calibration

`assignable` (~12 lines) + 8 one-line condition reroutes, all in check.rs. Baseline-
preserving by construction (assignable diverges from unify ONLY on distinct concrete
paths with a registered record edge — no existing arg pair is). **Target band: 25–45
min Mode A; 70 STOP-3; 95 STOP-4. Cascade: check.rs ONLY, 0 forced files.** Per
`feedback_stone_briefs_cite_prior_score`: SCORE-STONE-S-A is the shape.
