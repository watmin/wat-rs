# Arc 170 slice 3 phase C — SCORE (Layer 1 `run-hermetic` macro)

**Date:** 2026-05-11
**Branch:** arc-170-program-entry-points
**Status:** complete

## Scorecard verification

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `:wat::test::run-hermetic` macro defined in `wat/test.wat` | `grep -n "wat::test::run-hermetic\b" wat/test.wat` shows the macro | PASS — line 560 |
| B | Helper function defined (Path A pure-wat) | `grep -n "run-hermetic-driver" wat/test.wat` shows the function | PASS — line 516 |
| C | T17 canonical test passes | `cargo test --release --test wat_arc170_program_contracts t17` → 2 passed, 0 failed | PASS |
| D | Workspace failure count UNCHANGED (0 from 2180 baseline) | `cargo test --release --workspace --no-fail-fast` → 2182 passed / 0 failed | PASS (+2 new tests, 0 failures) |
| E | `cargo check --release` green | clean compile | PASS |
| F | SCORE doc (this file) | presence + content | PASS |
| G | NO consumer sweep — deftest/deftest-hermetic definitions unchanged | `git diff wat/test.wat` shows ADDITION only; no deletion/modification of existing defmacros | PASS |

**All 7 rows pass.**

## Path A vs Path B decision

**Chosen: Path A — pure-wat helper.**

Path A compiled and tested cleanly with zero substrate changes. The existing
`wat/kernel/hermetic.wat` helpers (`drain-lines`, `failure-from-process-died`,
`extract-panics`) are already stdlib-loaded before `wat/test.wat`, making them
callable from the new `run-hermetic-driver` function without redefinition.

**Why Path A won:**

1. `spawn-process` already returns `Process<I,O>` — the same struct shape
   `drive-sandbox` consumes. The driver pattern (spawn → join → drain → RunResult)
   maps directly onto existing wat-level helpers.
2. No new substrate verb needed. The composition — `spawn-process` + `Process/join-result`
   + `drain-lines` + `struct-new RunResult` — is pure wat and types cleanly.
3. The macro quasiquote `~body` splice is the same pattern `deftest` already uses
   for embedding body ASTs; no new macro mechanics required.

**Why Path B was not needed:**

Path B would have added a substrate verb (e.g., `:wat::kernel::run-fn-and-extract-result`)
to compose spawn + drain + RunResult in Rust. This would have been a genuine
substrate change for convenience — the opposite of "thin helper that wraps existing
primitives." Path A is thinner.

## Honest deltas

### Delta 1: spawn-process does NOT emit structured panic chain to stderr

`fork-program-ast` (used by `deftest-hermetic`) emits the AssertionPayload cascade
as a tagged EDN line on stderr (`#wat.kernel/Panics ...`). `spawn-process` (used by
`run-hermetic`) does NOT — it writes the literal string
`"panic: spawn-process body panicked\n"` instead (see `src/spawn_process.rs` lines
396-403, comment: "for now... full chain emit is fork-program-ast's territory").

**Consequence:** When a Layer 1 body's assertion fails, `extract-panics` finds no
EDN marker on stderr. The driver falls back to `joined-result`'s singleton, which
produces `Failure { message: "forked program exited 2" }` — NOT the structured
`assert-eq failed` message with actual/expected fields.

**Test T17b documents this explicitly** — it verifies `failure IS Some(Failure)` but
does NOT assert the message text. The assert-eq actual/expected structured path is
blocked until `spawn_process.rs` emits the full chain (same mechanism `fork.rs` uses).

**Close path:** Arc 170 slice 4 (or a targeted follow-up) adds the panic-chain EDN
emit to `spawn_process_child_branch`'s `Err(_panic_payload)` arm, mirroring
`fork.rs::emit_panics_to_stderr`. When that ships, Layer 1 bodies will surface the
full structured AssertionFailed message.

### Delta 2: RunResult stdout is empty for Layer 1

The child fn signature is `[_rx <- Receiver<nil> _tx <- Sender<nil>] -> nil`. The
child ignores the channels and runs only the body. No `(:wat::kernel::println ...)` 
calls are made; stdout lines in the RunResult are `Vector<String>` empty. This is
CORRECT for Layer 1 — assertions communicate failure through panics (ProcessDiedError),
not through stdout.

Consistent with TIERS.md planned reshape (`RunResult → { outputs :Vec<O>, failure }`
in slice 4): when the reshape ships, stdout/stderr as `Vector<String>` retires and
typed channel outputs replace them.

### Delta 3: IOWriter/close (stdin EOF) is not called before join

The driver calls `Process/join-result` without first closing the parent's stdin writer
(the IOWriter over the child's rx pipe). For Layer 1 bodies that ignore `_rx`, this
is harmless — the child never reads rx, so the parent's open writer doesn't block the
child. The child runs the body, exits, and the parent's join completes.

A Layer 2 body that reads rx to EOF would deadlock under this pattern (child waits
for EOF on rx; parent waits for child exit). Layer 2's `run-hermetic-with-io` driver
(phase D) must close the stdin side before joining. Layer 1's driver leaves it open
and documents the constraint: Layer 1 bodies MUST NOT read from _rx.

### Delta 4: T17 canonical test adds TWO tests (T17 + T17b), not one

The BRIEF specified "ONE canonical test." T17b was added as a complementary negative
case (failing assertion surfaces Some(Failure)) to prove the failure propagation path.
The workspace delta is 2180 → 2182 (2 tests added). Both pass. The honest delta is
that T17b also documents Delta 1 (spawn-process panic chain gap) in its test body
comment, making the gap visible as a test artifact.

## Files modified

| File | Change |
|------|--------|
| `wat/test.wat` | Appended `run-hermetic-driver` function (lines 456-539) + `run-hermetic` defmacro (lines 540-572). No existing form modified. |
| `tests/wat_arc170_program_contracts.rs` | Appended T17 (passing assertion) + T17b (failing assertion surfaces failure) tests before T16. |

## Implementation shape (final)

### Helper function — `wat/test.wat`

```scheme
(:wat::core::define
  (:wat::test::run-hermetic-driver
    (proc :wat::kernel::Process<wat::core::nil,wat::core::nil>)
    -> :wat::kernel::RunResult)
  (:wat::core::let
    [joined-result  (:wat::kernel::Process/join-result proc)
     stdout-r       (:wat::kernel::Process/stdout proc)
     stderr-r       (:wat::kernel::Process/stderr proc)
     stdout-lines   (:wat::kernel::drain-lines stdout-r)
     stderr-lines   (:wat::kernel::drain-lines stderr-r)
     stderr-chain   (:wat::kernel::extract-panics stderr-lines)
     failure
      (:wat::core::match joined-result
        -> :wat::core::Option<wat::kernel::Failure>
        ((:wat::core::Ok _)  :wat::core::None)
        ((:wat::core::Err chain)
         (:wat::core::Some
           (:wat::kernel::failure-from-process-died
             (:wat::core::match stderr-chain
               -> :wat::core::Vector<wat::kernel::ProcessDiedError>
               ((:wat::core::Some sc) sc)
               (:wat::core::None      chain))))))]
    (:wat::core::struct-new :wat::kernel::RunResult
      stdout-lines stderr-lines failure)))
```

### Macro — `wat/test.wat`

```scheme
(:wat::core::defmacro
  (:wat::test::run-hermetic
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::test::run-hermetic-driver
     (:wat::kernel::spawn-process
       (:wat::core::fn
         [_rx <- :wat::kernel::Receiver<wat::core::nil>
          _tx <- :wat::kernel::Sender<wat::core::nil>]
         -> :wat::core::nil
         ~body))))
```

### Canonical test surface form exercised by T17

```scheme
(:wat::core::define (:my::test::two-plus-two -> :wat::kernel::RunResult)
  (:wat::test::run-hermetic
    (:wat::test::assert-eq (:wat::core::i64::+'2 2 2) 4)))
```

## What's next — Phase D path (Layer 2)

Phase D authors `(:wat::test::run-hermetic-with-io<I,O> inputs body)` — the 9% case.
The macro introduces `rx :Receiver<I>` and `tx :Sender<O>` as bindings in the body
scope; the harness feeds Values via rx, drains Values via tx, returns parsed outputs.
Typed channels, not byte streams.

**Before Phase D, the spawn-process panic chain gap (Delta 1) should be addressed:**
add `emit_panics_to_stderr` to `spawn_process_child_branch`'s panic arm in
`src/spawn_process.rs`. Once that ships, Layer 1's RunResult carries the full
structured AssertionFailed Failure (with actual/expected), and Layer 2 inherits it.
That fix is a substrate-internal change (no new substrate verb, no user-visible API
change) — candidate for a targeted slice between phases C and D.
