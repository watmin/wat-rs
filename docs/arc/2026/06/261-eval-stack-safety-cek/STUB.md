# Arc 261 — Eval stack-safety (the recursion concern) + the CEK question

**Status:** STUB — captured 2026-06-14, not yet scoped into stones. Open question, not a
decision. Author: surfaced while running `fix-macro-param-types` over the real stdlib corpus
(arc 251 maturity-line work).

---

## The finding (what surfaced)

The tree-walking interpreter (`src/runtime.rs`) evaluates by **native Rust recursion**: each
nested wat form is another `eval_*` call on the Rust stack. There is **no `stack_size` anywhere
in the eval path** (grep `src/`: the only `stack_size` is in `process/clone.rs`, unrelated) —
wat programs run on whatever stack the caller provides:
- the **main thread** gives ~8 MB,
- a **cargo test thread** gives ~2 MB (the default for spawned test threads).

This bit us concretely. Running `fix-macro-param-types` (a *pure-wat* recursive codemod, `wat/fix.wat`)
over the corpus:
- on a 2 MB test thread → **stack overflow / SIGABRT** on the larger files (`core.wat`, `Record.wat`).
- on an 8 MB thread → **completes cleanly** across the whole corpus (largest file = 430 lines).
- on 512 MB → also clean (confirmed it is *depth*, not an infinite loop).

So: the migration is correct; the overflow was the **2 MB test-thread stack**, and 8 MB covers the
current corpus comfortably.

### The class, named honestly

The interpreter's recursion depth is bounded by the **native stack size**, and each `eval` frame
is *heavy* (the `eval_list` match is enormous), so even modest wat recursion (~20–40 deep over a
few-hundred-line file) can approach a 2 MB budget. Two compounding factors:

1. **Native-stack-bounded recursion.** Any sufficiently deep wat program (a deeply nested form, a
   long non-tail-recursive walk over a big collection) will overflow *some* stack size. 8 MB is
   not a guarantee — it is a current-corpus headroom.
2. **No proper tail calls.** wat's idiom is plain recursion; `wat/core.wat` + `wat/fix.wat` have
   **no `reduce` / `loop` / `recur`** (grep-confirmed). `fix.wat`'s walks recurse with `concat`
   *after* the recursive call — **not** in tail position — so they grow the stack linearly in input
   size. `fix-text` (shipped `a528147d`) shares this shape. (The service loops do TCO-`recur`, so
   *some* TCO exists, but it is not the universal eval discipline.)

This is a **wat-wide trait**, not a defect specific to `fix-macro-param-types`. It did not block the
arc-251 corpus sweep (run on ≥8 MB), but it is a real ceiling worth a root-level answer.

---

## The cheap patches (the lower rungs — what we could do WITHOUT CEK)

If we only wanted to lift the immediate ceiling, climbing the extirpare ladder partway:

- **Run wat on a generous, explicit stack.** Give the interpreter entry point a fixed large stack
  (e.g. spawn eval on a 64–256 MB thread) instead of inheriting 2/8 MB. A *convention/config* rung —
  pushes the ceiling up, does not remove the class.
- **Make the hot walks tail-recursive / add `reduce`.** Rewrite `fix.wat`'s collectors in
  accumulator-passing tail form and add an iterative `reduce`/`fold` primitive the interpreter
  evaluates as a Rust loop. A *per-site* fix — removes the depth for *those* walks, not the class.

Both are real options and far cheaper than CEK. Neither makes deep recursion *structurally* safe.

---

## The CEK question (the builder's open call)

> *"i was planning on moving us to CEK — i'm not sure if that'll make things better or worse."*

A **CEK machine** (Control · Environment · Kontinuation) defunctionalizes the interpreter: instead
of recursive `eval` calls, an explicit loop steps a triple `(control-expr, env, continuation)`, with
the **continuation living on the heap**. The native Rust stack stops growing with wat nesting.

**What CEK does for THIS concern — better, structurally:**
- **The stack-overflow class becomes unrepresentable.** There is no native recursion in eval, so wat
  recursion depth is bounded by *heap*, not the 2/8 MB native stack. This is the **top rung** of the
  ladder for this failure class — not "push the ceiling up," but "there is no ceiling of this kind."
- **Proper tail calls fall out for free.** A tail call just *replaces* the control without pushing a
  continuation frame → TCO becomes the default, not a special `recur`. The `concat`-after-recurse
  shape in `fix.wat` would still build a big *heap* continuation, but it would not crash.
- **Pausable / resumable / metered eval falls out too.** A CEK state *is* a serializable snapshot of
  a running computation — you pause by holding `(C,E,K)` and resume by stepping it. This is precisely
  what the **metered-eval / wat-mcp "verification market"** vision needs (see memory
  `project_metered_eval_verification_market`): pay-to-continue, pausable remote eval is trivial on a
  CEK machine and effectively impossible on a native-recursive one. **If CEK is coming for metered
  eval anyway, the recursion concern is corroborating evidence, not the sole driver.**

**What CEK costs — where it could be "worse":**
- **It is a rewrite of the entire eval core.** `runtime.rs` is enormous; every special form and every
  builtin must be re-expressed in CEK step-style. Large surface, high risk, long arc.
- **Per-step overhead.** Heap-allocated continuations + an explicit dispatch loop can be slower than
  fast native-stack recursion for shallow code, unless carefully optimized (continuation pooling,
  defunctionalized K as an enum, etc.).
- **Debuggability shifts.** Native Rust backtraces through eval are gone; you debug an explicit
  continuation stack instead (arguably *better* for wat-level introspection, different for us).

### Honest preliminary read (to be challenged, not trusted)

CEK **does** resolve the recursion/stack class at the top rung *and* gives TCO + pausability for
free. So for *correctness and capability* it is strictly better on this axis. The "better or worse"
tension is **not** about whether it fixes the recursion — it does — but about **cost vs. driver**:

- If we value the recursion fix **alone**, CEK is over-kill: the two cheap patches above clear the
  current corpus for a fraction of the effort.
- If CEK is **already wanted** for metered/pausable eval (the verification-market arc) and/or proper
  TCO as a language guarantee, then the recursion concern is one more reason to do it, and it
  dissolves this whole class as a side effect.

**The decision is therefore really: "is CEK on the roadmap for its OWN reasons (metered eval, TCO)?"**
If yes → this arc folds into that migration and the recursion ceiling is a non-issue. If no → patch
the ceiling cheaply (big-stack entry + tail-recursive `fix.wat` walks + `reduce`) and revisit CEK
when a second driver appears.

---

## ⚠️ SHIPPED STOPGAP — `rune:exigere(attested-arc)` (2026-06-14) — DELETE WHEN THIS ARC LANDS

The cheap "big-stack entry" rung is **live now**, on purpose and visibly, so the self-hosted
fix-wat migration runner works over the whole corpus today (the largest stdlib file, `wat/test.wat`,
SIGSEGV'd the forked child at the default 8MB `RLIMIT_STACK`).

- **Site:** `crates/wat-cli/src/lib.rs`, just before `fork_program_from_source` — a `setrlimit`
  raising `RLIMIT_STACK` soft to 1 GiB (or the hard cap) before the fork; the child inherits it.
- **Marker:** `rune:exigere(attested-arc)` citing this arc. It is the standing, grep-able reminder:
  `grep -rn "rune:exigere(attested-arc)" crates/wat-cli` surfaces it; it MUST NOT be forgotten.
- **What it does / does not do:** RAISES the ceiling (~1 GiB); does NOT remove the class. A
  pathologically large file would still overflow. Only the forked-program (CLI) path is covered;
  in-process eval is unchanged.
- **Retirement condition:** when this arc lands the structural fix (CEK — no native eval
  recursion), **delete the `setrlimit` block and its rune.** The rune retires with the arc, per
  `docs/CONFORMARE.md` `rune:exigere(attested-arc)` discipline.

This stopgap is the lower rung of the ladder below, made real. It does not change the arc's goal;
it buys time without hiding the debt.

## Open questions to resolve before scoping stones

1. Is CEK already committed for metered/pausable eval? (cross-ref `project_metered_eval_verification_market`)
2. What is the realistic perf delta of a CEK step-loop vs. the current native recursion on the hot
   paths (encode/eval benchmarks)?
3. If we patch cheaply first: which `fix.wat` walks need tail-recursive rewrites, and does adding an
   iterative `reduce` primitive cover them?
4. Does a big-stack eval entry point have any interaction with the spawn/clone3 stack handling
   (`process/clone.rs`) or the comms reactor threads?

## Cross-references

- The trigger: arc 251 `fix-macro-param-types` corpus sweep (the maturity-line work).
- `wat/fix.wat` — the non-tail recursive walks (`fix-text`, `fix-macro-param-types` collectors).
- Memory: `project_metered_eval_verification_market` — the strongest "CEK anyway" signal.
