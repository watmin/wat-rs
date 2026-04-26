# wat-rs arc 060 — `:wat::kernel::join-result` — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~1.5 hours
of focused work.

Builder direction (2026-04-26, mid-experiment 008 diagnosis):

> "what diagnostics are we missing - we have a crashed thread?.. we
> can use the test's stdout,err here?... how is a crash silent?..."

> "i don't like this its... a cheat.. an easy path... i don't like
> it... a better form exists... i don't know what it is"

The "better form": death as data. Spawn-thread crashes route
through the same Result/match discipline the rest of the
substrate uses. No `eprintln!` cheat.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/runtime.rs` — `enum SpawnOutcome` (3 variants); `ProgramHandle` channel type changes from `Result<Value, RuntimeError>` to `SpawnOutcome`; spawn body wrapped in `catch_unwind`; `format_panic_payload` helper handling `&str` / `String` / `AssertionPayload` downcasts; `eval_kernel_join` updated to handle the new SpawnOutcome variants (panic surfaces as `RuntimeError::ChannelDisconnected` carrying the captured message in the op string); `eval_kernel_join_result` added (death-as-data path); 3 thread_died_error_* helpers building the ThreadDiedError enum value variants. `src/types.rs` — `:wat::kernel::ThreadDiedError` registered as built-in enum (3 variants). `src/check.rs` — `:wat::kernel::join-result` type scheme. `docs/USER-GUIDE.md` — Spawning-programs section gains the join-vs-join-result framing + match-form example; surface table row added. | ~250 Rust + ~25 doc | 6 new (happy path, captures Panic, captures RuntimeError, legacy join still propagates panic via op-string, refuses non-handle, arity mismatch) | shipped |

**wat-rs unit-test count: 637 → 643. +6. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release` (workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### `SpawnOutcome` channel-payload enum

The pre-arc-060 channel carried `Result<Value, RuntimeError>`. A
spawn-thread panic was un-captured: the thread unwound, the sender
dropped before send, and `recv` got a `Disconnected` error that
`join` mapped to a `ChannelDisconnected` RuntimeError with no
message detail. The panic payload (the `&str` / `String` /
`AssertionPayload` the unwind carried) was lost to stderr.

Arc 060 changes the channel to a 3-state `SpawnOutcome` enum:

```rust
pub enum SpawnOutcome {
    Ok(Value),                  // thread returned a Value
    RuntimeErr(RuntimeError),   // thread returned Err normally
    Panic(String),              // thread panicked; payload captured
}
```

The spawn body wraps the `apply_function` call in `std::panic::catch_unwind`:

```rust
let outcome = match std::panic::catch_unwind(...) {
    Ok(Ok(v))    => SpawnOutcome::Ok(v),
    Ok(Err(e))   => SpawnOutcome::RuntimeErr(e),
    Err(payload) => SpawnOutcome::Panic(format_panic_payload(&payload)),
};
let _ = tx.send(outcome);
```

`AssertUnwindSafe` is honest here — `thread_sym` and `arg_values`
are owned by the closure; we don't share them with the caller
after a panic. `format_panic_payload` downcasts `&str`, `String`,
and the substrate's `AssertionPayload` (the structured shape
`:wat::kernel::assertion-failed!` panics with) to a String.

### Two verbs, same channel — non-breaking

`:wat::kernel::join` keeps its "I trust this thread" semantic:
spawn-thread death surfaces as a `RuntimeError` to the caller. The
panic case now carries the captured message in the op string
(`":wat::kernel::join (spawned thread panicked: <message>)"`)
where pre-arc-060 it just said "spawned thread panicked before
yielding a result." Existing call sites unchanged; they get richer
diagnostic-on-failure for free.

`:wat::kernel::join-result` is the Story-2 / death-as-data path.
The spawn-thread's outcome becomes a wat-side `Result<R,
ThreadDiedError>` the caller matches on. Three Err variants
discriminate cause:

- `Panic(message)` — the spawned function unwound; the captured
  payload rides as a String.
- `RuntimeError(message)` — the spawned function returned `Err`
  from a Result-typed eval path (deliberate failure).
- `ChannelDisconnected` — substrate bug; should never fire under
  the catch_unwind wrap; emitted as a distinct variant so
  consumers can tell "my function ran and died" from "the
  substrate ate my child."

The two verbs share the same channel + the same handle type. New
code that wants in-band failure handling reaches for `join-result`
directly; existing code keeps `join` and gets the better error
message.

### Why the verbs name different questions

This is the same shape as arc 057's `assert-eq` vs
`assert-coincident` recognition: the substrate provides both
verbs; the consumer picks per call site based on what failure
means at THAT point.

`join` says "this thread should not fail; if it does, that's a
bug worth halting on." Use it in tests where spawn-thread death
genuinely means something is broken in the program-under-test.

`join-result` says "the spawned thread's outcome IS the data I'm
operating on." Use it in supervisors, restart-policy code,
test harnesses that want to discriminate cause, and any setup
where the calling thread needs to keep running regardless of
what the spawn-thread did.

### `AssertionPayload` downcast

The substrate's `:wat::kernel::assertion-failed!` primitive panics
with a structured `AssertionPayload` (defined in
`src/assertion.rs`) carrying `message`, optional `actual` /
`expected`, `location`, and call-stack `frames`. `format_panic_payload`
downcasts to that type and extracts the message field. Without
this arm, assertion-failures would surface as
`"panic with non-string payload"` — accurate but useless. The
arm makes the substrate's own assertion failures readable on the
join-result path the same way they're readable in the sandbox's
RunResult.failures.

---

## What this unblocks

- **`holon-lab-trading` experiment 008.** The Treasury service's
  test driver swaps `(:wat::kernel::join treas-driver)` for
  `(:wat::kernel::join-result treas-driver)` plus a match arm and
  treasury-thread crashes surface in-band with the captured panic
  or RuntimeError message instead of the test failing on a
  downstream `assert-eq` with no context.
- **Future supervisor programs** — wat-vm-level supervisors need
  `join-result` to discriminate Panic vs RuntimeError vs
  ChannelDisconnected for restart-policy decisions.
- **Test-side debugging discipline** — every multi-thread test
  (Console, CacheService, RunDbService, treasury, future broker
  programs) gets the option to surface spawn-thread crashes
  meaningfully rather than as a "join blocked forever" or
  "downstream assert failed" mystery.
- **Even the legacy `join` callers** — the panic message now
  rides in the RuntimeError op string, so a surprise spawn-thread
  panic surfaces with diagnostic content instead of just
  "ChannelDisconnected".

---

## What this arc deliberately did NOT add

Reproduced from DESIGN's "What this arc does NOT add":

- **Linked spawn / Erlang-style death notifications.** Different
  mechanism (caller is notified asynchronously when child dies).
  `join-result` is the synchronous shape; future arc can add the
  async one if needed.
- **Supervisor primitives** (restart policies, escalation chains).
  Out of scope; build the supervisor as a wat program once
  `join-result` is in place.
- **Spawn cancellation** (kill a spawned thread from outside).
  Different concern; future arc when needed.
- **Structured panic payloads** (downcast info beyond the
  formatted message). Future arc when a caller surfaces real need.
- **Removing `:wat::kernel::join`.** Both verbs stay; pick per
  call site.

---

## The thread

- **2026-04-26 (mid-experiment 008)** — treasury-thread crash
  surfaces silently; the test fails on a downstream `assert-eq`
  with no context.
- **2026-04-26 (DESIGN)** — proofs lane drafts the arc; sibling-
  verb shape (parallel to arc 057's assert-coincident) settled.
- **2026-04-26 (this session)** — slice 1 lands in one commit:
  SpawnOutcome plumbing + `eval_kernel_join_result` + ThreadDiedError
  enum registration + 6 inline tests + USER-GUIDE rows + this
  INSCRIPTION.
- **Next** — Treasury experiment swaps `join` → `join-result` at
  its diagnostic call sites; the silent-crash diagnostic gap
  closes.

PERSEVERARE.
