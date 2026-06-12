# BRIEF — Stone 259.S3.5a (hermetic) — `deftest-hermetic'` / `run-hermetic'` (the forms sibling)

## The work

Add `run-hermetic'` + `deftest-hermetic'` to `wat/test.wat` — the PROCESS-tier siblings of the
just-shipped `deftest'`/`run-thread'` (`ace1ab3a`). Same caller (`spawn-program'` + `recv'`),
different body PACKAGING: a thread shares memory → ships a CLOSURE (`deftest'`); a process has
SEPARATE memory → ships FORMS (program over the wire). **This forms interface is the SHARED one
with the future `deftest-remote`** ("separate memory" = same-host-process OR remote-host) — build
it remote-ready: do NOT special-case "process" in a way that would block a `(remote)` host later.
PURE WAT; no Rust.

## CWD discipline (FIRST, every git/build)
Anchor `/home/watmin/work/holon/wat-rs`. `pwd` first; ignore any `.claude/worktrees/` path. Do NOT
commit — the orchestrator weighs and commits.

## The grounded shape

The legacy `run-hermetic` (`wat/test.wat`) wraps the body as forms for `spawn-process`:
```
`(:wat::test::run-hermetic-driver
   (:wat::kernel::spawn-process
     (:wat::core::forms
       (:wat::core::defn :user::main [] -> :wat::core::nil ~body))))
```
The PIPE-MODEL sibling uses `spawn-program'` (process) + `recv'`, and the child `println`s a
pass-marker (the process analog of the thread body's `send' self 0`):

```
(:wat::core::defmacro :wat::test::run-hermetic'
  [body <- :AST<wat::core::nil>]
  -> :AST<wat::core::nil>
  `(:wat::core::let
     [p (:wat::kernel::spawn-program' (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::do ~body (:wat::kernel::println 0)))))
      _ (:wat::kernel::recv' p)]
     (:wat::core::struct-new :wat::kernel::RunResult
       (:wat::core::Vector :wat::core::String)
       (:wat::core::Vector :wat::core::String)
       :wat::core::None)))

(:wat::core::defmacro :wat::test::deftest-hermetic'
  [name    <- :AST<wat::core::nil>
   prelude <- :AST<wat::core::nil>
   body    <- :AST<wat::core::nil>]
  -> :AST<wat::core::nil>
  `(:wat::core::do
     ~@prelude
     (:wat::core::defn ~name [] -> :wat::test::TestResult (:wat::test::run-hermetic' ~body))))
```

- **Pass:** the child runs `:user::main` (body, then `println 0`), the parent `recv'`s 0 → returns
  a clean RunResult.
- **Fail:** a failing assertion crashes the child → the reason travels over the process Err channel
  (fd 2) → `recv'` raises with it (the process tier already surfaces crashes).

## Unknowns to resolve against the build (the process pipe path is less-trodden than thread)
1. **The pass-marker.** Does the child's `(:wat::kernel::println 0)` reach the parent's `recv'`
   (EDN over fd 1)? If `println` is the wrong verb, or the marker needs a different shape/type, find
   the working one from the existing process tests (counter-service-process / spawn-program' process
   probes) — do NOT fake it.
2. **One-shot main.** `spawn-program'` (process) expects a forms-server. Confirm a `:user::main`
   that runs once, `println`s, and returns/exits works with `recv'` (vs a readln-loop server). If it
   needs a different program shape, surface it.
3. **recv' type / coercion.** `recv'` on a process peer decodes EDN → Value; the 0 marker decodes to
   i64. If a `-> :T` ascription is needed, note it.

## STOP triggers (REJECTION — ship nothing, surface the exact error)
- **STOP-1:** keep the FORMS interface (program over the wire) — it is the shared interface with
  `deftest-remote`. Do not block a future `(remote)` host.
- **STOP-2:** do NOT edit the probe (`probe_arc259_deftest_hermetic_prime`) or the thread
  `deftest'`/`run-thread'` macros.
- **STOP-3:** if the process pipe path (println-marker + recv') genuinely doesn't work and the only
  way to pass is `Process/join-result`, STOP and surface it — that's a design fork for the
  orchestrator (the pipe model is the goal; join-result is the fallback only if the pipe can't carry
  the marker).

## Gate (run each, READ output, report REAL results — do NOT chain a commit)
1. `cargo test --release -p wat --test nursery probe_arc259_deftest_hermetic_prime -- --test-threads=1` → 2 GREEN.
2. `cargo test --release -p wat --test nursery probe_arc259_deftest_prime -- --test-threads=1` → still 2 GREEN (thread sibling unaffected).
3. `cargo test --release -p wat --test nursery -- --test-threads=1` → only known reds (arc-255 ×2, undefined-builtin ×2), zero new.
4. `cargo test --release --test test 2>&1 | tail -3` → 237/1 (the 1 = `test_run_string_entry_direct`, pre-existing).
5. `cargo build --release` clean.

## Report back
- The two macros as written. Which unknowns you resolved + how. Verbatim final line of each gate.
  Any STOP hit + the verbatim error. Do NOT commit.
