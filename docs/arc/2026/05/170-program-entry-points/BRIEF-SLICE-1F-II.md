# Arc 170 slice 1f-ii — BRIEF

**Substrate; opus.** Mints `:wat::kernel::StdOutService` + per-thread send-with-ack contract. Applies the registration pattern from slice 1f-i (documented in `src/services/mod.rs` rustdoc) — but with INVERTED data direction (threads → service via crossbeam channels) and an ACK channel for each send (mini-TCP discipline per arc 089 slice 5 + `wat/console.wat`).

**Reference docs (read first):**
- `src/services/mod.rs` — module-level rustdoc minted by slice 1f-i; documents the pattern (singleton + spawn_for_test + register/unregister + shutdown via control-pipe). Apply unchanged for the structure; the data-flow and multiplex specifics differ.
- `src/services/stdin.rs` — slice 1f-i's StdInService impl; mirror this for module organization, naming, drop semantics
- `wat/console.wat` — conceptual ancestor at the wat layer; arc 089 slice 5 added the ack channel (mini-TCP) so producers know their write completed
- `crates/wat-edn/src/writer.rs` — `write_keyword_body` (slice 1f-W) does the depth-aware comma↔underscore swap; `write` / `write_to` are the public API
- [`SCORE-SLICE-1F-W.md`](./SCORE-SLICE-1F-W.md) — slice 1f-W locked-in the wire encoding; `write` is the writer side
- [`SCORE-SLICE-1F-I.md`](./SCORE-SLICE-1F-I.md) — pattern proof + lessons learned
- [`BUILD-PLAN.md`](./BUILD-PLAN.md) §3 slice 1f-ii — scope + ship criteria

**Branch:** `arc-170-program-entry-points` (slice 1f-W shipped at `4278c4d`).

**Constraint:** STOP if any substrate primitive this BRIEF references doesn't exist or doesn't behave as cited — DON'T workaround. Surface as honest delta.

## Scope

### 1. New module `src/services/stdout.rs`

Mirror the structure of `src/services/stdin.rs` (slice 1f-i):
- `StdOutService` struct with worker handle
- `StdOutServiceHandle` public API
- `start_stdout_service()` singleton (OnceLock-stored)
- `StdOutService::spawn_for_test(output_fd: RawFd) -> StdOutServiceHandle`
- ControlMsg enum carrying Register / Unregister / Shutdown

Add to `src/services/mod.rs`: re-export the public API alongside `stdin`'s.

### 2. The contract — per-thread send-with-ack

**Public API (Rust):**

```rust
/// Start the StdOutService thread. Idempotent.
pub fn start_stdout_service() -> &'static StdOutServiceHandle;

pub struct StdOutServiceHandle { /* ... */ }

impl StdOutServiceHandle {
    /// Register a thread. Returns a (sender, completion_receiver) pair:
    ///   - thread sends (Arc<HolonAST>) to the service via the Sender
    ///   - thread blocks on AckReceiver to know the write completed
    /// — OR equivalent shape; document the actual API in module rustdoc
    pub fn register(&self, thread_id: ThreadId)
        -> Sender<(Arc<HolonAST>, AckSender)>;

    pub fn unregister(&self, thread_id: ThreadId);
}

/// Type alias for the ack channel side that producer threads block on.
/// Service signals () when the libc::write to fd 1 completes.
pub type AckSender = crossbeam_channel::Sender<()>;
```

(Names + types subject to refinement during implementation; follow 1f-i's lead and the rustdoc in `src/services/mod.rs`.)

### 3. Worker loop — crossbeam Select multiplex

UNLIKE slice 1f-i, the service does NOT use libc::poll. Multiplex is via `crossbeam_channel::Select` on:
- The control-Receiver (Register / Unregister / Shutdown)
- N per-thread message-Receivers (one per registered thread)

When a per-thread Receiver fires:
1. Receive `(Arc<HolonAST>, AckSender)`
2. Serialize HolonAST to EDN via `wat_edn::write_to(&Value::from(holon_ast), &mut buf)` (or equivalent)
3. Append `\n` to buf
4. `libc::write(fd, buf.as_ptr(), buf.len())` — single-writer guard since only this thread writes fd 1
5. Send `()` to AckSender — producer unblocks

When the control-Receiver fires:
- Register: add the thread's pipe-receiver to the select set; track in HashMap<ThreadId, Receiver>
- Unregister: remove from select set + map
- Shutdown: drain all pending messages? OR exit immediately? Document the choice in SCORE.

### 4. Single-writer guard on fd 1

Only the service worker calls `libc::write` on fd 1. Producer threads always go through the service. This is the doctrine that prevents interleaved writes.

For tests using `spawn_for_test(output_fd)`, the same guard holds — only the service-test instance writes the test fd.

### 5. Wire encoding via slice 1f-W

The serialization step (item 3.2 above) uses `wat_edn::write` / `write_to`. Per slice 1f-W (commit `4278c4d`), `write_keyword_body` does the depth-aware comma → underscore swap inside `<...>` substrings. Slice 1f-ii inherits this transformation transparently — just call `write` and the output is correct line-delimited EDN with proper wire encoding.

**Verify** in tests: write a parametric-keyword Atom (e.g., constructed from `:wat::core::HashMap<wat::core::String,wat::core::i64>`), assert the output bytes contain the underscore form (`HashMap<wat::core::String_wat::core::i64>`).

### 6. Rust integration tests in `tests/services_stdout.rs` (NEW)

Mirror the test shape from `tests/services_stdin.rs` (slice 1f-i):

- Row A — Module structure + start_stdout_service idempotent
- Row B — Service thread spawns + idles without panic
- Row C — Registration roundtrip (register returns Sender; unregister drops)
- Row D — Single-thread send + ack: register → send Atom + ack-receiver → assert ack received → assert bytes on test fd match expected EDN line
- Row E — EDN serialization correctness (not ack: actual bytes)
- Row F — Wire encoding applied (parametric keyword `<K,V>` → bytes with `<K_V>`)
- Row G — Multi-thread send: register N threads; each sends; assert all acks received; assert output bytes contain all N lines (per-thread ordering preserved within each thread; cross-thread ordering not guaranteed but each line is intact)
- Row H — Shutdown drains or doesn't (whatever the implementation chooses; test the documented behavior)
- Row I — fd ownership (caller retains; service uses but doesn't close)

Plus probes per honest-delta categories.

## Constraints

- **Don't write a workaround.** If the registration pattern from `src/services/mod.rs` rustdoc doesn't compose with crossbeam Select cleanly (e.g., needs a HashMap<ThreadId, Receiver> + dynamic Select rebuild on each register/unregister), surface the substrate friction; don't paper over.
- **Don't modify slice 1f-i.** StdInService stays. Slice 1f-ii is parallel infrastructure for the OTHER direction.
- **Don't modify slice 1f-W.** The wire encoding is settled. `wat_edn::write` does the right thing transparently.
- **Don't mint StdErrService.** That's slice 1f-iii.
- **Don't touch wat-cli.** Slice 1f-iv wires the services into wat-cli's startup.
- **Don't migrate Console-using tests.** Slice 3 sweeps.
- **Zero new Mutex / RwLock / CondVar.** Use `crossbeam_channel` + `OnceLock` + `AtomicBool` + libc syscalls.
- **No new dependencies.** Cargo.toml unchanged.
- **No TODOs in source.** FM 5.

## Substrate-grep citations

Verified to exist:

- `src/services/mod.rs` — pattern docs from slice 1f-i
- `src/services/stdin.rs` — sibling impl to mirror; OnceLock singleton + spawn_for_test + register/unregister
- `crates/wat-edn/src/writer.rs:144` — `write` / `write_to` public API
- `crates/wat-edn/src/value.rs` — `Value::Keyword(Keyword::ns(...))` constructors
- `crossbeam_channel::Select` — usage precedent at `src/runtime.rs:15347`
- `crossbeam_channel::Sender / Receiver / unbounded / bounded` — used throughout `src/runtime.rs`
- libc::write usage precedent — `src/spawn_process.rs:417`
- `wat/console.wat` arc 089 slice 5 ack pattern — read for the mini-TCP discipline shape
- HolonAST → wat_edn::Value bridge — verify via grep; slice 1f-i uses something similar in the parse direction; slice 1f-ii needs the WRITE direction

Any deviation: STOP, report, don't guess.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — Module structure | `src/services/stdout.rs` exists; `src/services/mod.rs` re-exports the API alongside stdin's | ✓ |
| B — `start_stdout_service` idempotent | second call returns same `&'static StdOutServiceHandle` | ✓ |
| C — Service thread spawns + idles | thread runs without panic; CPU near zero (Select blocks) | ✓ |
| D — Registration roundtrip | `handle.register(thread_id)` returns Sender; `handle.unregister(thread_id)` drops; multi-register works | ✓ |
| E — Single-thread send + ack | register → send (Arc<HolonAST>, ack_tx) → block on ack_rx → assert () received → assert bytes on test fd match expected EDN | ✓ |
| F — Wire encoding via slice 1f-W | parametric keyword Atom serializes with underscore form for commas inside `<>` | ✓ |
| G — Multi-thread send + ack | N=3+ threads register; each sends; each receives ack; output bytes contain all N+ lines; per-thread ordering preserved within each thread | ✓ |
| H — Shutdown semantics | drain-pending OR immediate-exit (whichever chosen); documented behavior tested | ✓ |
| I — fd ownership convention | service uses output_fd but does NOT close it; caller retains OwnedFd | ✓ |
| J — Single-writer guard | producer threads NEVER call libc::write directly; only the service worker writes fd 1 | ✓ |
| K — Rust integration tests green | `cargo test --release --test services_stdout` → all green | ✓ |
| L — Workspace doesn't regress | `cargo test --release --workspace --no-fail-fast` fail count is within ±5 of post-1f-W baseline (855) | ✓ |
| M — Honest deltas surfaced | per FM 5; no TODOs; no deferral language | ✓ |
| N — Zero new dependencies | `Cargo.toml` unchanged | ✓ |
| O — Foundation + slice 1e + 1f-i + 1f-W files untouched | `git diff 4278c4d..HEAD` shows only `src/services/stdout.rs`, `src/services/mod.rs` (re-export line), `tests/services_stdout.rs` (new) | ✓ |
| P — Pattern API documented for 1f-iii reuse | module-level rustdoc on `src/services/stdout.rs` documents the send-with-ack contract + how 1f-iii (StdErrService) can apply the same pattern with first-panic-wins semantics | ✓ |
| Q — Zero new Mutex/RwLock/CondVar | grep `src/services/stdout.rs` for these — zero hits | ✓ |

## Honest delta categories

Surface; don't workaround:

- **Dynamic Select rebuild on register/unregister** — `crossbeam_channel::Select` requires building the Select set BEFORE entering the select loop. Adding/removing receivers mid-loop typically requires breaking out, rebuilding, re-selecting. If this introduces awkward interleaving, surface for design discussion.
- **Ordering guarantees** — cross-thread ordering of writes is NOT guaranteed (Select picks readiness; producers race). Per-thread ordering IS guaranteed (single sender; FIFO crossbeam). Document this in module rustdoc + tests.
- **Ack channel cardinality** — one-shot per send (bounded(1) crossbeam) OR per-thread shared ack stream? One-shot is simpler; shared stream allows decoupled producer/ack. Pick one; document.
- **HolonAST → wat_edn::Value bridge** — slice 1f-i's parse direction has the inverse; slice 1f-ii needs `holon_ast → wat_edn::Value` for serialization. If the bridge function doesn't exist or has surprises, surface.
- **Shutdown drain vs immediate-exit** — design choice. Drain is friendlier (producers don't lose data); immediate-exit is faster (test cleanup). Pick one; document.
- **wat/console.wat ack pattern** — slice 1f-ii is the SUBSTRATE version of what wat/console.wat does at the wat layer. They might co-exist briefly (Console crossbeam service still operational; slice 3 migrates Console-using tests). Surface any conflicts.
- **FM 5 trap** — TODOs verboten.

## Predicted runtime

60-90 min opus. Pattern inheritance from 1f-i should keep this in the lower half. The novel pieces:
- Crossbeam Select with dynamic registration (vs libc::poll's static-fd self-pipe in 1f-i)
- Mini-TCP ack discipline
- HolonAST → wat_edn::Value serialization bridge

Hard cap: 180 min.

## What's next (orchestrator-side, post-slice-1f-ii)

When 1f-ii ships:
1. Score per EXPECTATIONS-SLICE-1F-II.md
2. Author SCORE-SLICE-1F-II.md
3. Atomic commit slice 1f-ii
4. Author BRIEF + EXPECTATIONS for slice 1f-iii (StdErrService) — applies the registration pattern from 1f-i + the send-with-ack contract from 1f-ii + first-panic-wins + libc::exit semantics
5. Spawn slice 1f-iii
