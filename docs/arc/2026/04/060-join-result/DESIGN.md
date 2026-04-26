# Arc 060 — `:wat::kernel::join-result` (spawn-thread death as data)

**Status:** opened 2026-04-26.
**Predecessor:** arc 058 (HashMap completion), arc 059 (Vec concat) — same small-arc shape, same builder direction ("if it's missing it shouldn't be").
**Consumer:** `holon-lab-trading` experiment 008 (Treasury service driver) hit a silent treasury-thread crash; the `assert-eq` failed, but the actual cause (`UnknownFunction(":wat::core::concat")`, which arc 059 fixes) stayed buried because `:wat::kernel::join` was never called before the assert. The diagnostic gap is real and orthogonal to the consumer's specific bug.

Builder direction (2026-04-26, mid-experiment 008 diagnosis):

> what diagnostics are we missing - we have a crashed thread?.. we
> can use the test's stdout,err here?... how is a crash silent?...

> i want to attack this another way... idk how yet.... but... i
> have the concat arc in motion... let's discuss the need to have
> stderr prints.... i don't like this its... a cheat.. an easy
> path... i don't like it... a better form exists... i don't know
> what it is

The substrate already CAPTURES spawn-thread panics in-band — the `Value::wat__kernel__ProgramHandle` is a one-shot `Result<Value, RuntimeError>` channel; panics propagate through `catch_unwind` into the channel; the captured `RuntimeError` flows out via `:wat::kernel::join`'s recv. The problem isn't capture; it's that today's `join` PANICS the calling thread on Err, leaving callers no in-band way to inspect "what killed my child."

This arc adds the in-band path. **No `eprintln!` cheat.** Death becomes data, routed through the same Result/match discipline the rest of the substrate uses.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::kernel::spawn :fn args...` → `ProgramHandle<R>` | shipped |
| `Value::wat__kernel__ProgramHandle` (one-shot result channel) | shipped |
| `:wat::kernel::join handle` → `R`-or-panic | shipped (THIS arc adds a sibling, doesn't change) |
| `:Result<T,E>` + `(Ok v)` + `(Err e)` | shipped |
| `(:wat::core::match)` on `Result<T,E>` | shipped (arc 048) |
| `(:wat::core::try result-expr)` (Rust's `?`) | shipped |

`:wat::kernel::join` stays as-is — it's the "I trust this thread; if it died, that's a bug worth panicking about" form. Same honest-naming principle as `:wat::test::assert-eq` (bit-identical) vs `:wat::test::assert-coincident` (substrate-tolerance). Both verbs are honest; consumer picks per call site.

## What's missing (this arc)

| Op | Signature |
|----|-----------|
| `:wat::kernel::ThreadDiedError` | enum (3 variants — Panic, RuntimeError, ChannelDisconnected) |
| `:wat::kernel::join-result` | `∀R. ProgramHandle<R> → Result<R, ThreadDiedError>` |

Two additions. Pure additions; non-breaking. Existing `join` callers unchanged.

---

## Decisions resolved

### Q1 — Non-breaking via sibling verb (not changing `join`)

Two reasons to keep `join` panicking and add `join-result` rather than retrofit:

1. **The verbs name different questions.** `join` says "I trust this thread; give me its value, panic if it failed." `join-result` says "tell me what happened, success or otherwise." Both are honest. Mirroring the Chapter 60 principle: when the assertion-shape doesn't exist, write it.

2. **Migration cost.** Every existing `join` site (Console smoke tests, CacheService smoke tests, RunDbService smoke tests, proof 002/003 pair files, future broker programs) doesn't have to change unless the author wants the in-band failure path. New code that NEEDS the failure path uses `join-result` directly.

This is the same shape as arcs 020/058 — pure additions, no breakage to existing callers.

### Q2 — `ThreadDiedError` shape: enum (Option 2 from the design discussion)

```scheme
(:wat::core::enum :wat::kernel::ThreadDiedError
  ;; The thread's eval panicked (catch_unwind caught it).
  (Panic (message :String))

  ;; The thread's eval returned :Err normally — the spawn function
  ;; itself was Result-typed and produced an Err.
  (RuntimeError (message :String))

  ;; The result channel disconnected without a value being sent —
  ;; rare; usually means the spawn machinery itself failed before
  ;; the spawned function could run.
  :ChannelDisconnected)
```

Three variants because supervisors / restart policies / debugging traces all want to discriminate cause. A panic deserves a different response than a graceful Err return. (E.g.: a supervisor might restart on Panic but not on RuntimeError, since RuntimeError represents a deliberate failure the function knew about.)

The two String fields aren't typed-error-objects on purpose — `RuntimeError` in wat-rs is already a Rust enum with display impl; we extract the formatted message into the wat-side String at the substrate boundary. Keeps the wat-side enum lightweight and not dependent on the full RuntimeError taxonomy.

### Q3 — Naming: `join-result` (not `try-join`, `wait`, or `await`)

`join-result` names what differs from `join`: it returns a `Result`. Verb-with-suffix matches Rust idioms (`try_into` etc.) and the existing wat surface (`first` returns `Option<T>` after arc 047; the suffix-naming convention already exists at the substrate).

`try-join` was considered but `try` already has substrate meaning (`:wat::core::try` — the `?` propagator). Reusing the prefix would confuse.

`wait` / `await` are too imperative-flavored — they don't say "this returns a Result you must handle."

### Q4 — The `Panic` variant carries a String, not a panic-payload

Rust's `std::thread::JoinHandle::join` returns `Result<T, Box<dyn Any + Send>>` where the error is the panic payload (could be anything panicker passed to `panic!`). At the wat boundary, we coerce to a `String` via `format!` on the panic value (same as Rust's default panic hook does). Loses some precision (downcast info), gains uniformity at the wat surface.

If a future caller surfaces a need for the structured panic payload, a future arc can add `:wat::kernel::PanicPayload` or similar. For v1 the formatted message is enough.

### Q5 — `ChannelDisconnected` semantics

This case fires when the spawned thread's stack unwound past `catch_unwind` somehow (substrate bug), OR the spawn machinery itself failed before the function ran (e.g., thread spawn returned an error). In practice it should never happen; if it does, it's a substrate bug worth investigating. Emit it as a distinct variant so consumers can distinguish "my function ran and died" from "the substrate ate my child."

### Q6 — `join-result` is single-use just like `join`

`ProgramHandle<R>` is a one-shot — both `join` and `join-result` consume it. Calling either twice on the same handle is an error (or returns immediately on the second call with whatever stale state the channel has — implementer's call). Same lifecycle as today's `join`.

---

## What ships

One slice. One commit. Mirrors arc 020 / arc 058 / arc 059 shape.

### `src/check.rs`

Two additions:

1. Register `:wat::kernel::ThreadDiedError` as a built-in enum type with three variants (mirrors the existing `:Result` registration).

2. Register `:wat::kernel::join-result`'s scheme:

```rust
env.register(
    ":wat::kernel::join-result".into(),
    TypeScheme {
        type_params: vec!["R".into()],
        params: vec![TypeExpr::Parametric {
            head: "wat::kernel::ProgramHandle".into(),
            args: vec![TypeExpr::Path(":R".into())],
        }],
        ret: TypeExpr::Parametric {
            head: "Result".into(),
            args: vec![
                TypeExpr::Path(":R".into()),
                TypeExpr::Path(":wat::kernel::ThreadDiedError".into()),
            ],
        },
    },
);
```

### `src/runtime.rs`

Dispatch arm + `eval_join_result`:

```rust
":wat::kernel::join-result" => eval_join_result(args, env),
```

```rust
fn eval_join_result(args: &[WatAST], env: &Environment) -> Result<Value, RuntimeError> {
    // (Same arity check + handle extraction as eval_join.)
    // Recv on the result channel:
    match handle_rx.recv() {
        Ok(Ok(value)) => Ok(Value::Result_Ok(Arc::new(value))),
        Ok(Err(runtime_err)) => Ok(Value::Result_Err(Arc::new(
            ThreadDiedError::RuntimeError(runtime_err.to_string()).into_value()
        ))),
        Err(_disconnected) => {
            // Catch_unwind caught a panic OR substrate bug; the channel
            // dropped without sending. Disambiguate via the panic-payload
            // store — wat-rs's catch_unwind handler stores the formatted
            // panic message in a thread-local before dropping the channel.
            // (Implementation detail; if no payload, fall through to
            // ChannelDisconnected.)
            let payload = take_thread_panic_payload();
            let err = match payload {
                Some(msg) => ThreadDiedError::Panic(msg),
                None => ThreadDiedError::ChannelDisconnected,
            };
            Ok(Value::Result_Err(Arc::new(err.into_value())))
        }
    }
}
```

(Pseudo-code; actual implementation depends on how wat-rs currently captures panics through catch_unwind — may need a thread-local panic-payload slot OR replacing the spawn body's catch_unwind to also send a structured error variant on the result channel.)

### Unit tests

5 tests (`tests/wat_join_result.rs`):

1. **Happy path.** Spawn a function that returns `42`; `join-result` returns `(Ok 42)`.
2. **Spawned function panics.** Spawn a function that calls `(:wat::test::fail "...")` or otherwise panics; `join-result` returns `(Err (Panic "..."))` with the message.
3. **Spawned function returns Err.** Define `fn -> :Result<i64,String>` that returns `(Err "bad")`; spawn + join-result returns `(Err (RuntimeError "bad"))`.
4. **Old `join` still panics on death.** Sanity check that the existing `join` is unchanged: spawn a panicking function; `(join handle)` panics the calling thread.
5. **Both verbs interoperable.** Sanity that `(join-result handle)` and `(join handle)` consume the same handle type.

### Doc

- `docs/arc/2026/04/060-join-result/INSCRIPTION.md` post-ship.
- `docs/CONVENTIONS.md` rubric: append `join-result` row under "kernel primitives", document the `join` vs `join-result` choice (parallel to `assert-eq` vs `assert-coincident`).
- `docs/USER-GUIDE.md`: add the entry under `:wat::kernel::*` section with example match form.

---

## Implementation sketch

Single slice, one PR. ~150 LOC + 5 tests. Mirrors arcs 058 / 059 small-arc shape.

```
src/check.rs:    +50 LOC  (enum registration + scheme)
src/runtime.rs:  +60 LOC  (eval_join_result + panic-payload capture if not already there)
tests/wat_join_result.rs: +90 LOC (5 tests)
docs/arc/.../INSCRIPTION.md:  post-ship
docs/CONVENTIONS.md:  +5 LOC
docs/USER-GUIDE.md:   +12 LOC
```

**Estimated cost:** ~215 LOC. **~2 hours** of focused work. Carries one substrate-uplift risk: panic-payload capture across `catch_unwind`. May already be present (the comment in `runtime.rs` mentions "If the thread panics before sending, the sender drops, and `join` reports the panic via `ChannelDisconnected`" — implementer to confirm whether the payload is already accessible or needs a capture path).

---

## What this arc does NOT add

- **Linked spawn / Erlang-style death notifications.** A different mechanism (caller is notified asynchronously when child dies). `join-result` is the synchronous shape; a future arc can add the async one if needed.
- **Supervisor primitives** (restart policies, escalation chains). Out of scope; build the supervisor as a wat program once `join-result` is in place.
- **Spawn cancellation** (kill a spawned thread from outside). Different concern; future arc when needed.
- **Structured panic payloads** (downcast info beyond the formatted message). Future arc when a caller needs it.
- **Removing `:wat::kernel::join`.** Both verbs stay; pick per call site.

---

## What this unblocks

- **`holon-lab-trading` experiment 008** — the test driver swaps `(kernel::join treas-driver)` for `(kernel::join-result treas-driver)` + match arm, and treasury-thread crashes surface in-band with the captured RuntimeError message instead of the test failing on a downstream `assert-eq` with no context.
- **Future supervisor programs** — wat-vm-level supervisors need `join-result` to discriminate Panic vs RuntimeError vs ChannelDisconnected for restart-policy decisions.
- **Test-side debugging discipline** — every multi-thread test (Console, CacheService, RunDbService, treasury, future broker programs) gets the option to surface spawn-thread crashes meaningfully rather than as a "join blocked forever" or "downstream assert failed" mystery.

PERSEVERARE.
