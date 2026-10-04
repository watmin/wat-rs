# Lowered bodies — a shim beside the interpreter, rete's way: partial first

## Why

the-little-wat bootstraps its compiler by running `elf/compile.wat` on this interpreter. That stage 0 takes 40+ minutes.
A 60-second `perf` sample of a live stage 0 (2026-10-03, the shared binary at `e67f9da99`, 238,952 samples) spends:
- ~13% in SipHash. The name-resolution arc's FxHash change already takes most of this out.
- ~15% in interpretive dispatch: `eval_tail`, `eval_inner`, `eval_list`, `apply_function`.
- 5.2% in `Environment::lookup`.
- 5.8% in malloc/free.
- 2.7% in `dispatch_keyword_head_value`, 2.7% in `Value` clones, and 2.3% in dropping `Provenance`.

Most of that spread is the cost of interpreting a body by walking its syntax and naming every local by string. The
builder (2026-10-03): *"maybe we just add a path like rete's compiled dag?... not exact but close... just a shim to speed
things up"*, and the branch must stay the simplest it can be to merge into main: **additive**.

## The shape — what already exists, and where the shim attaches

- **One door for user calls.** `apply_function` (`src/runtime.rs:11149`) is where every user-function call converges.
  It checks arity, builds a call environment by binding parameters by name, then runs `eval_tail(body_ast, &call_env,
  sym)` inside a trampoline. A tail call comes back as `EvalSignal::TailCall` and the loop continues without recursing.
- **The value door.** `#[wat_intrinsic]` generates, from one declaration, both the AST door the interpreter uses and a
  VALUE door, `fn(&[Value], &Span) -> Result<Value, EvalBreak>`. `:wat::core::apply` reaches the value door through
  `dispatch_substrate_impl` (`src/runtime.rs:6593`), which looks the entry up in the registry, takes `value_handler`,
  and guards arity. A lowered body calls builtins through that door, so no builtin is implemented twice.
- **The precedent.** `src/rete/expr_ir.rs` lowers to an `Expr` DAG: `Slot(u16)`, `Let` over slots, `If`, `Match`,
  `Call`. Its rule is "`lower()` is total or it refuses". This shim takes the shape but not the code. Rete's operators
  are its own value-level table (`apply_core_kind`), and the shim calls the registry's value doors instead. Nothing
  under `src/rete/` changes.

## The contract — unobservable to wat programs

Same values, same output, same errors, same tail-call behaviour (mutual tail calls to 10,000,000). A function the
lowering cannot take whole stays interpreted, exactly as today.

## The rows

- **L0 — the census, and nothing else.** Add `WAT_LOWER_CENSUS=1`. At exit it prints, per user function, how often it
  was applied and whether its body would lower. If not, it gives the FIRST form that refuses (its head, or its node
  kind). It also prints a ranked table of refusal reasons, weighted by application count. Run it on the three
  wat-rs benches and the floor's wat programs. The orchestrator will run it on stage 0; that table decides what L2
  covers.
- **L1 — the shim, minimal.** A new module, `src/lower/`, holds everything; its only other footprint is one `mod` line
  and the hook in `apply_function`.
  - **What lowers.** A body lowers when it consists only of:
    - literals;
    - parameters and `let`-bound locals, read by SLOT;
    - `if`;
    - `let` with plain symbol binders;
    - calls to user functions by name (`SymbolTable::functions`), in tail position or not;
    - calls to intrinsics whose registry entry has a `value_handler` and `@Purity Pure`.

    Anything else refuses, and the function is interpreted.
  - **Lowering happens once per function, on first application.** The result is cached in a side table keyed by the
    `Arc<Function>` pointer, so `Function` gains no field. Refusals are cached as well.
  - **The hook.** In `apply_function`, where the body would run: if the function has a lowered program, run it with
    `cur_args` in slots. It must return exactly what `eval_tail` returns: `Ok(v)`, or a `TailCall` signal for a user
    call in tail position, which the existing trampoline already handles. Arity checks and frames stay as they are.
  - **Switches.** `WAT_LOWER=0` turns the shim off. `WAT_LOWER=check` runs the lowered program AND the interpreter on
    every lowered call and stops with a named error on any difference. Pure-only lowering is what makes running twice
    safe.
- **L2 — coverage from the census.** The census ranks the forms that block lowering. Add support for them, in that
  order, until stage 0's census (from the orchestrator) shows most applications lowered. `match`, `cond` and record
  field access are the likely first rows; the census decides.

## Gates

1. Baseline: the floor at the current `the-little-wat` HEAD, 5,405 of 5,405.
2. After each of L1 and L2:
   - the floor three ways: `WAT_LOWER=0`, the default, and `WAT_LOWER=check`. All green, the same 5,405.
   - The three benches print the same checksums with the shim on and off, and the instruction counts are reported
     both ways.
3. `WAT_LOWER=check` reports zero differences across the floor.

## STOP triggers

- **STOP-1** — a user-function call reaches a body by a path other than `apply_function`. List the paths: the hook
  covers only that door.
- **STOP-2** — an intrinsic's value door disagrees with its AST door on the same values. That is a defect in wat-rs,
  not in the shim. Report it with a reproduction.
- **STOP-3** — making the shim work needs a change outside `src/lower/`, the `mod` line and the `apply_function`
  hook, for example a new field, a signature change, or an edit to `eval_tail`. Name it before making it: merge cost
  is the builder's constraint.
- **STOP-4** — `eval-step!` (excursus 004's stepping) observes a lowered body differently. Report it. Stepping may have
  to force the interpreter.

## Method

- **Where you work.** Your clone `/var/tmp/wat-rs-names`, with its own `target/`, on branch `the-little-wat`. This is
  wat-rs territory only; the-little-wat's repository is not part of this arc. The orchestrator measures stage 0.
- **Running and checking.**
  - `timeout -s KILL` on every run.
  - Never read an exit code through a pipe.
  - Never re-run a red.
  - Assert every text replacement.
  - Floor logs go under `/var/tmp/wat-rs-names-logs/`.
- **Benchmarks.** `taskset -c 2`, three runs, instructions and cycles reported separately.
- **Committing.** Commit each green row (`feat(lowered-bodies): …`) and push it after the commit.
- **Write `SCORE.md` here AS YOU GO.**

**Prediction (the orchestrator's, before the strike):** L1 alone lowers the leaf arithmetic and recursion helpers and
moves the benches 10–30%. Stage 0 moves only after L2 covers `match` and record access, which the census will show
dominate `compile.wat`.
