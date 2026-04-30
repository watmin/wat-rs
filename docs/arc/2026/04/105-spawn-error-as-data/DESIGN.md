# Arc 105 — `spawn-program` error-as-data + `ThreadDiedError/message` — DESIGN

**Status:** OPEN — drafted 2026-04-29 immediately after arc 104
sealed. Closes the deferral from arc 103b. Until this lands, the
substrate Rust `eval_kernel_run_sandboxed*` impls survive; once it
does, `wat/std/sandbox.wat` (already in source as scaffolding)
replaces them; `Vec<String>` exits the substrate boundary
permanently.

**Predecessor:** [arc 103b INSCRIPTION](../103-kernel-spawn/INSCRIPTION.md#slice-103b--partial-iowriter-close-shipped-sandbox-wat-scaffolded).

**Surfaced by:** mid-arc-103 conversation:

> "i never want to see Vec<String> ever again outside of tests —
> for real work we use real kernel pipes as the surface area of
> our programs"

Arcs 103a and 103c lived up to it for new code. Arc 103b documented
the gap that prevented the substrate's existing
`eval_kernel_run_sandboxed*` Rust impls from deleting: they absorb
startup / validation / panic failures into `RunResult.failure`, a
capability the wat-level `wat/std/sandbox.wat` couldn't replicate
without two substrate primitives changing shape. Arc 105 ships
those changes.

---

## What's wrong today

Two specific blockers from arc 103b's deferral note:

### Blocker 1 — `spawn-program` raises on startup errors

```scheme
(:wat::kernel::spawn-program
  (src   :String)
  (scope :Option<String>)
  -> :wat::kernel::Process)            ;; ← raises on startup failure
```

When `startup_from_source` fails (parse error, type error, config
error) or `validate_user_main_signature` fails, the dispatch arm
returns `Err(RuntimeError)`. That propagates as a panic up through
the eval pipeline; the wat caller cannot catch it.

The substrate Rust `eval_kernel_run_sandboxed` handles this by
catching the failure inside its own dispatch arm and synthesizing
a `RunResult { stdout: vec![], stderr: vec![], failure: Some(...) }`.
That capture happens AT the substrate. The wat-level helper in
`wat/std/sandbox.wat` cannot replicate the capture because spawn-
program raises before returning anything wat code can pattern-match
on.

**Fix:** spawn-program returns `:Result<:Process, :StartupError>`.
Failures become `(Err startup-error)` values; success becomes
`(Ok proc)`. Wat-level callers pattern-match.

### Blocker 2 — `ThreadDiedError` variants don't pattern-match cleanly from wat

When a forked / spawned program panics or runtime-errors mid-run,
`:wat::kernel::join-result` returns `Err(ThreadDiedError)`. The
ThreadDiedError enum has three variants:

```rust
pub enum ThreadDiedError {
    Panic(String),
    RuntimeError(String),
    ChannelDisconnected,
}
```

Pattern-matching this from wat hits a type-check bug — the matcher
mis-infers the scrutinee as `:Option<?>`. Surfaced during arc 103b's
sandbox.wat work; left deferred.

**Fix:** add a `:wat::kernel::ThreadDiedError/message` accessor
that extracts the carried String regardless of variant, returning
a generic message for `ChannelDisconnected`. wat callers don't
need to discriminate variants for the run-sandboxed use case
(they just want a message for `RunResult.failure.message`).

The wat-side type-checker bug for enum-variant scrutinizing is its
own future concern; arc 105 routes around it.

---

## What ships

### Slice 105a — `:wat::kernel::StartupError` + spawn-program returns Result

**`:wat::kernel::StartupError`** — new struct:

```scheme
(:wat::core::struct :wat::kernel::StartupError
  (message :String))
```

One field for now (the failure message). Distinct type identity so
wat code can't accidentally pass any String where StartupError is
expected. Extensible (could grow `kind`, `location`, etc.) without
breaking callers — but YAGNI; ship the minimum.

**spawn-program signature change:**

```scheme
;; before (arc 103a):
(:wat::kernel::spawn-program
  (src   :String)
  (scope :Option<String>)
  -> :wat::kernel::Process)

;; after (arc 105a):
(:wat::kernel::spawn-program
  (src   :String)
  (scope :Option<String>)
  -> :Result<:wat::kernel::Process, :wat::kernel::StartupError>)
```

Same change for `:wat::kernel::spawn-program-ast`.

**Implementation** (`src/spawn.rs::eval_kernel_spawn_program*`):

- Call `startup_from_source` / `startup_from_forms*`. On `Err`,
  return `Ok(Value::Result(Err(StartupError{message: format!("{}",e)})))`.
- Call `validate_user_main_signature`. On `Err`, return
  `Ok(Value::Result(Err(StartupError{message: ":user::main: ..."})))`.
- On success (validation passes), call the existing
  `spawn_with_world_inner` to get the Process struct value, wrap as
  `Ok(Value::Result(Ok(process)))`.

The dispatch arm itself never returns `Err(RuntimeError)` for these
cases anymore — only for genuine substrate-level failures (arity
mismatch, type mismatch on args).

**Caller migrations** (small surface):

- `tests/wat_arc103_spawn_program.rs` — 6 tests. Each binds the
  Process via pattern-match: `((Ok proc) ...) ((Err _) panic!)`.
- `wat-scripts/ping-pong.wat` — pattern-match before using proc.
- `wat-scripts/dispatch.wat` — pattern-match before using proc.
- `wat/std/sandbox.wat` — already scaffolded for the new shape;
  unbundle in slice 105c.

### Slice 105b — `:wat::kernel::ThreadDiedError/message` accessor

```scheme
(:wat::kernel::ThreadDiedError/message
  (err :wat::kernel::ThreadDiedError)
  -> :String)
```

Implementation: pattern-match the Rust `Value::Enum` variant on
`:wat::kernel::ThreadDiedError`:
- `Panic(msg)` → return `msg`
- `RuntimeError(msg)` → return `msg`
- `ChannelDisconnected` → return `"channel disconnected"`

Single Rust function in `src/runtime.rs` (or a new
`src/thread_died.rs`?). One scheme registration. One dispatch arm.

Why not fix the type-check bug that prevents wat-side enum
variant matching? That's a much bigger change to the type checker
(arc 055's recursive-pattern work needs extending to handle enum
scrutinees in this specific shape). The accessor is a one-function
fix; the type-checker fix is its own arc. Defer.

### Slice 105c — bundle `wat/std/sandbox.wat`; delete substrate Rust impls

The payoff. With slices 105a + 105b in hand:

1. Update `wat/std/sandbox.wat` to use the new spawn-program
   signature (Result handling) + ThreadDiedError/message accessor.
   Should be ~15 lines smaller than today's scaffold (no longer
   needs the "generic failure" stub).
2. Add `wat/std/sandbox.wat` to `STDLIB_FILES` in `src/stdlib.rs`.
   This makes wat-level `:wat::kernel::run-sandboxed` /
   `run-sandboxed-ast` available process-wide.
3. Delete from `src/runtime.rs`:
   - `":wat::kernel::run-sandboxed"` dispatch arm
   - `":wat::kernel::run-sandboxed-ast"` dispatch arm
4. Delete from `src/check.rs`:
   - schemes for both primitives
5. Delete from `src/sandbox.rs`:
   - `eval_kernel_run_sandboxed`
   - `eval_kernel_run_sandboxed_ast`
   - All helper functions (`build_run_result`, `failure_from_*`,
     `bytes_to_lines`) that no longer have callers
   - Keep: `resolve_sandbox_loader` (called by spawn.rs)
6. cargo test — every arc 007 / arc 027 / arc 031 test that goes
   through run-sandboxed should still pass, because the wat-level
   define has the same semantics.

`Vec<String>` exits the substrate's stdio boundary permanently.
The only place it survives is INSIDE the wat-level helper, where
collected output IS the assertion target — exactly the discipline
the user named.

### Slice 105d — INSCRIPTION + 058 row + USER-GUIDE update

Standard close-out:
- INSCRIPTION captures the deferral closure + the substrate
  shrinkage (line count drops in src/sandbox.rs)
- 058 changelog row
- USER-GUIDE §1 / §13 update (run-sandboxed signature unchanged
  from caller view; substrate-vs-wat-level note moves to
  past-tense)

---

## Three-question discipline

**Obvious?** Yes. Two specific blockers; two specific fixes; one
delete. Slice 105a turns spawn-program's failure raise into data
(matching wat's discipline elsewhere — `eval-edn!`, `eval-ast!`,
`edn::read` all return Results). Slice 105b adds a one-function
accessor that routes around a known checker limitation. Slice
105c is mechanical deletion.

**Simple?** Net code shrinks. Slice 105a +20 lines (the new
struct + Result wrapping). Slice 105b +15 lines (the accessor +
scheme + dispatch). Slice 105c -200+ lines (delete substrate
sandbox.rs body) +30 lines (the now-bundled wat/std/sandbox.wat
becomes the canonical implementation). Net: substrate gets
substantially smaller.

**Honest?** This was the open invariant arc 103 deferred. Arc 105
closes it. After this lands the arc 103-104 architecture is
complete: substrate primitives traffic in real pipes; failures
travel as data; `Vec<String>` survives only inside the wat-level
test convenience.

**Good UX?** No observable change for the shell user (`wat
<entry.wat>` still runs). Embedders using `wat::main!` /
`wat::test!` are unaffected. `wat::Harness` callers see the same
RunResult shape. The only callers that need migration are the 6
arc 103a tests + 2 wat-scripts that use spawn-program directly —
small surface, mechanical change.

---

## Slices

**105a — StartupError struct + spawn-program returns Result.**
Substrate Rust changes + scheme + 6 test migrations + 2 wat-script
migrations.

**105b — ThreadDiedError/message accessor.** One Rust function +
scheme + dispatch arm.

**105c — bundle sandbox.wat; delete substrate impls.** Update
sandbox.wat to use slice 105a + 105b primitives; bundle in
stdlib.rs; delete eval_kernel_run_sandboxed* + helpers.

**105d — INSCRIPTION + 058 row + USER-GUIDE update.**

---

## Open questions resolved upfront

1. **StartupError as struct vs enum vs typealias.** Struct with
   one `message :String` field. Type-distinct (a wat value of
   StartupError can't be confused with random String) and
   extensible (kind, location can grow as fields). Enum (with
   Parse/SignatureMismatch variants) was considered; rejected on
   YAGNI — current sandbox.rs doesn't discriminate either.

2. **Where the StartupError lives.** `:wat::kernel::StartupError`,
   sibling of `:wat::kernel::ThreadDiedError`. `register_builtin`
   in `src/types.rs` next to ThreadDiedError.

3. **ThreadDiedError accessor approach — fix checker vs route
   around.** Route around. Adding the `/message` accessor is one
   function; fixing the recursive-pattern checker for enum
   scrutinizing is a separate arc.

4. **What sandbox.wat does on StartupError.** Builds RunResult
   with empty stdout/stderr + Some(Failure { message: err.message
   }). Same shape today's substrate produces. The user surface is
   identical; the implementation shifts wat-ward.

5. **What sandbox.wat does on ThreadDiedError after spawn
   succeeded.** Drains stdout/stderr (might have partial output
   the program wrote before dying), then builds RunResult with
   captured lines + Some(Failure { message: ThreadDiedError/
   message err }). Substrate today does this in
   `failure_from_runtime_err` / `failure_from_panic_payload`.

6. **stdin pre-seeding in wat sandbox.wat.** Already in the
   scaffold: write the joined stdin lines to `proc.stdin`,
   close, drain. Unchanged.
