# BRIEF — STONE: the eval surface faces `:wat::WatAST`; the compiler names every heretic

> **Builder, 2026-09-04:** *"we are attacking abuse and misuse of holon-ast — I find this
> unacceptable — the enemy is hidden in our ranks… strike the heresy where they stand — **the
> compiler identifies the heretics immediately** — they will not survive us"*
>
> **THE ORCHESTRATOR HAS ALREADY MADE THE TYPE CHANGE. The tree is RED BY CONSTRUCTION and that is
> the brief.** `docs/SUBSTRATE-AS-TEACHER.md` governs: the failures are the worklist, the fail-count
> is the progress meter, and each error names its own next site. **Do not revert. Do not stash.**

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Never operate on a path containing
`.claude/worktrees/`. Do not commit, push, stash, or revert. Do not run the full floor.

## What is already done, and why the tree is red

`src/types.rs` — three field types retyped from `:wat::holon::HolonAST` to `:wat::WatAST`:

```
:wat::eval::WalkStep     Skip.terminal
:wat::eval::StepResult   StepTerminal   ← StepNext was ALREADY :wat::WatAST; it is the precedent
:wat::eval::StepResult   AlreadyTerminal
```

⚠ **rustc reports ZERO errors** — `types.rs` declares wat types as DATA (`TypeExpr::Path` strings),
so the Rust compiler cannot see this. **wat's own checker is the compiler that finds them**, at
startup, with a located message per site:

```
:wat::eval::WalkStep::Skip: parameter #1 expects :wat::WatAST; got :wat::holon::HolonAST
```

## The worklist — 17 reds, from the compiler, not a grep

`cargo nextest run --release -E 'binary_id(wat)'` → **1164 passed, 17 failed**:

```
step_holon_constructor_atom     step_arith_right_descent      step_arith_left_descent
step_arith_single_redex         step_if_branch_true           step_if_branch_false
step_holon_constructor_bind     step_holon_thermometer        step_let_substitute
step_if_cond_reduces            step_let_peel_first           step_match_scrutinee_reduces
step_match_canonical            step_user_function_call       step_round_trip_agrees_with_eval_ast
step_tail_recursion_terminates_under_bound                    walk_w3_skip_short_circuits
```

Sixteen flow through ONE shared driver — `step_to_terminal_prelude()` / `step_drive_to_terminal()`
in `src/runtime.rs`. Fix the driver and most fall together.

## ★★★ THE BLOCKER A PRIOR RIDER REPORTED IS NOT A BLOCKER — measured

That rider reported the driver's `Err` arm packs its error as `(:wat::holon::leaf (struct-field e 1))`
so success and failure share one HolonAST-typed return, and concluded *"there is no equivalent
wat-source-level primitive to wrap an arbitrary runtime string as a WatAST leaf — a design question,
not a mechanical swap."*

**Measured this session: `(:wat::holon::to-wat (:wat::holon::leaf x))` does exactly that.** Both verbs
are registered. It `--check`s clean, runs, and **satisfies a `:wat::WatAST`-typed parameter** —
verified with a probe. Use it.

⚠ It is a wart, not a blocker: building a holon to immediately convert it. Say so in a comment where
you use it and name the follow-up — a `:wat::core::`-native WatAST leaf constructor. **Do not mint
one in this stone.**

## The Rust-side pattern-matchers

Several of the 16 (`step_holon_constructor_bind`, `_bundle`, `step_holon_thermometer`, and others)
match the terminal in RUST as `HolonAST::Bind(..)` / `Bundle(..)` / `Thermometer{..}`. Those become
`WatAST` shape-matches. ⛔ **Each such test's ASSERTION MUST NOT WEAKEN** — if a test proved a
specific composition was reached, the rewritten form must still prove it. If you cannot preserve an
assertion's strength, STOP and name the test.

## STOP triggers

- **STOP-1** — an assertion that cannot be preserved at equal strength. Name it; do not weaken it.
- **STOP-2** — do not touch `:wat::holon::BundleResult`, `:wat::holon::Holons`,
  `:wat::holon::Reckoner/new-discrete`, `wat/holon*.wat`, `wat/test.wat`'s `assert-coincident`, or
  `wat/cache.wat`'s `hologram-svc`. **All are VSA and CORRECT** — `hologram-svc` is the SIMILARITY
  cache and is legitimately HolonAST-keyed. The orchestrator mis-filed it once; do not repeat that.
- **STOP-3** — do not mint new verbs. `to-wat ∘ leaf` is the tool.
- **STOP-4** — the fail-count must fall monotonically. If a fix RAISES it, stop and report both
  counts; that means the rule being applied is wrong, not that more sweeping is needed.
- **STOP-5** — 9 golden `<HolonAST>` string literals (8 in `src/runtime.rs`, 1 in
  `tests/value/wat_arc221b_keyword_dispatcher_completeness.rs`) describe these types. They go stale
  with this change — update them, and report the count you actually found.

## Verification

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::types)'
cargo nextest run --release -E 'binary_id(wat::value)'
cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'
cargo clippy --release --all-targets -- -D warnings
```

Report the fail-count waterfall (17 → … → 0), each STOP-1 candidate if any, the golden-literal count,
and anything that surprised you.
