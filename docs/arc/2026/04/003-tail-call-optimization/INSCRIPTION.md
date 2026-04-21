# Arc 003 — Tail-Call Optimization — INSCRIPTION

**Status:** shipped 2026-04-20.
**Design:** [`DESIGN.md`](./DESIGN.md) — the planning and reference notes.
**This file:** completion marker. What landed, where, why.

The code led, the spec followed: same *inscription* pattern the
058 proposal batch adopted for 058-033-try. The arc directory has
a DESIGN.md that was the planning document and this INSCRIPTION.md
that describes the shipped contract as it actually exists in the
code. If DESIGN.md and INSCRIPTION.md disagree, INSCRIPTION.md
wins — it's the honest record.

---

## What shipped

**Stage 1 — named defines** (`32e918b`, `tcp-stage-1`):

- New `RuntimeError::TailCall { func: Arc<Function>, args: Vec<Value> }`
  variant. Internal control-flow signal alongside `TryPropagate`.
- New `eval_tail` sibling of `eval` plus four tail-aware helpers:
  `eval_if_tail`, `eval_match_tail`, `eval_let_tail`,
  `eval_let_star_tail`. Each duplicates its non-tail twin's
  validation but dispatches the body via `eval_tail` instead of
  `eval`.
- `apply_function` wraps its body in a `loop` that catches
  `TailCall`, reassigns `cur_func` / `cur_args`, and re-iterates
  without growing the Rust stack.
- Signature change: `apply_function` now takes `Arc<Function>`
  instead of `&Function` (cheap clone required for the trampoline
  reassignment). All in-crate call sites updated; `freeze.rs`'s
  `invoke_user_main` also updated.
- User-function call detection keys on
  `sym.functions.contains_key(head)` — the `define`-registered
  universe.

**Stage 2 — lambda values** (`9089867`, `tcp-stage-2`):

- Three detection paths in `eval_tail`:
  1. Keyword head resolving in `sym.functions` (Stage 1 scope).
  2. Bare-symbol head resolving to `Value::wat__core__lambda` in
     env — lambda-valued params, let-bound lambdas.
  3. Inline-lambda-literal head `((lambda ...) args)` — eval the
     head, if it's a lambda emit `TailCall`.
- All three paths funnel through a new `emit_tail_call` helper.
- No changes to the trampoline itself — lambda's `closed_env`
  already traveled through the signal and the re-iteration's
  builder correctly used the lambda's closed env as parent.

## Tests

`tests/wat_tco.rs` — 11 cases covering:

- Self-recursion via `if` at 1M depth (the canonical benchmark).
- Self-recursion via `match` at 100k (the Console/loop shape).
- Mutual recursion between two named defines at 100k each way.
- Tail call inside a `let*` body.
- Non-tail recursion still correct at modest depth (confirms
  TCO isn't over-applied).
- `try` + `TailCall` coexist on happy + error paths.
- Let-bound lambda tail call (Stage 2 detection).
- Inline-lambda-literal tail call (Stage 2 detection).
- Named define tail-calls a lambda param (Stage 2 via env lookup).
- Lambda's `closed_env` survives the thread boundary
  (closure capture + spawn + tail-call composition).
- Named/lambda alternation at 100k via inline lambda literal
  (exercises both stages per iteration).

## What it unlocks

`:wat::std::program::Console/loop`,
`:wat::std::program::Cache/loop-step`, and every future
`gen_server`-shaped driver now run in constant Rust stack. The
wat source was already written in tail-recursive shape; the
evaluator now recognizes it.

Downstream: arc 004's stream stdlib (`map`, `filter`, `chunks`
workers) all use tail-recursive internal shapes. Without TCO they
would have a stack-depth ceiling per pipeline; with TCO each stage
runs indefinitely.

## Lessons captured

1. **Mature interpreted languages require TCO.** Scheme's R*RS
   specs mandate it; Erlang's BEAM has `call_only`. Rust itself
   doesn't have TCO, but the language we're hosting (wat) needs
   it, so we implemented it in the evaluator. This is the
   pattern: hosting a language doesn't mean inheriting all its
   rules from the host.

2. **Trampolining is structurally simple once named.** One new
   `RuntimeError` variant, one new `eval_tail` function, one
   `loop` wrapper around `apply_function`. The complexity is in
   identifying tail positions — that's the `eval_tail` branching
   and the sibling tail-aware helpers.

3. **Closure capture survives spawn.** Rust's `Send`/`Sync`
   derivation on `Value` + `Function` + `Environment` made
   cross-thread lambda passage "just work" at compile time — no
   runtime check needed. The type system's default derivation did
   the proving. This is what `docs/ZERO-MUTEX.md` names as
   Rust-contributes-the-guarantee.

## Open questions (from the DESIGN, mostly resolved)

- **try × TailCall interaction.** Resolved. Both are internal
  `RuntimeError` variants caught at `apply_function`'s loop;
  `TryPropagate` converts to `Ok(Result(Err))`, `TailCall`
  reassigns and continues. Loop order confirmed by tests.
- **Lambda tail calls across closure boundaries.** Resolved in
  Stage 2. The trampoline's `closed_env.clone().unwrap_or_default()`
  branch handles both lambdas (closed env as parent) and defines
  (fresh root).
- **Mutual recursion between let-bound lambdas.** Not covered.
  Requires letrec-style binding that wat's let* doesn't provide
  (RHS evaluation in the prefix scope; later names can't be
  forward-referenced). Documented in the Stage 2 test file as an
  intentional out-of-scope.
- **stacker fallback for non-tail recursion.** Not adopted. TCO
  handles the Console/loop-style case that motivated the arc;
  non-tail depth-bomb is a separate concern and its own slice if
  it ever becomes real.
- **Observability trace.** Not added; build-time feature flag
  deferred. Can ship when someone's debugging a TCO issue.

## Pointers to FOUNDATION / 058

The 058 proposal batch didn't explicitly propose TCO — it's
implementation-level, below the algebra/stdlib surface. If
FOUNDATION.md wants an entry, the natural place is the
"Programs are userland" section noting that `gen_server`-shaped
programs run in constant stack by virtue of the evaluator's
trampoline.

---

**Arc 003 — complete.**
