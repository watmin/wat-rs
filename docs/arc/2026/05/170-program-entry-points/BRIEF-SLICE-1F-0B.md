# Arc 170 slice 1f-0b — BRIEF

**Substrate; opus.** Retroactively reshape what slice 1f-α
shipped at `fcaf600`. ThreadIO's per-thread channel halves
change from `Sender<()>` (stdin) + `Sender<String>` (stdout/
stderr) to `Sender<Event>` per-service. The eval arms
internally construct the appropriate Event variant before
sending. The caller-facing primitive surface
(`(:wat::kernel::println v)` etc.) is unchanged — only the
internal channel-payload types and the eval-arm bodies change.

Per pass 18: this slice concretizes the Rust-side Event types
that slice 1f-β-i's wat-side service implementations will
mirror.

Per user direction 2026-05-10: *"we fix what we break once the
idealized shape is realized... they are us and we are fixing
our patterns."* No backward-compat shims; the 10 test rows in
`tests/wat_arc170_slice_1f_alpha_helpers.rs` migrate to the
new shape in this same slice.

## Mission

### 1. Mint three Rust enums (location: `src/thread_io.rs`)

Mirror the wat-side Event enums per pass 18:

```rust
/// Per-pass-18 control-plane + data-plane union. Sent on the
/// stdout req-tx; consumed by the wat-side StdOutService.
#[derive(Debug, Clone)]
pub enum StdOutServiceEvent {
    /// Caller's println rendered an EDN line; service writes
    /// it to fd 1 and acks.
    Write { line: String },
    /// Runtime registers a thread; service stores
    /// `(thread_id → (data_rx, ack_tx))` in its routing table.
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdOutServiceEvent>,
        ack_tx: Sender<()>,
    },
    /// Runtime reaps a thread; service drops the routing entry.
    Remove { thread_id: ThreadId },
}

/// Mirror of [`StdOutServiceEvent`] for fd 2.
#[derive(Debug, Clone)]
pub enum StdErrServiceEvent {
    Write { line: String },
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdErrServiceEvent>,
        ack_tx: Sender<()>,
    },
    Remove { thread_id: ThreadId },
}

/// Stdin's data variant is unit (the "give me next form"
/// request); the parsed HolonAST comes back via the reply-tx.
#[derive(Debug, Clone)]
pub enum StdInServiceEvent {
    /// Caller's readln signals "next form please."
    Read,
    /// Runtime registers a thread; service stores
    /// `(thread_id → (data_rx, reply_tx))` in its routing table.
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdInServiceEvent>,
        reply_tx: Sender<Arc<HolonAST>>,
    },
    Remove { thread_id: ThreadId },
}
```

**ThreadId representation:** mint as a typealias to whatever
the wat side picks (likely `i64` for monotonic counter
populated by slice 1f-γ). For slice 1f-0b's scope: define as
`pub type ThreadId = i64;` (matches the wat-side
`:wat::kernel::ThreadId` typealias-to-i64 decision; surface as
honest delta if newtype reads cleaner).

### 2. Reshape `ThreadIO` (location: `src/thread_io.rs:44`)

**Before** (as shipped at fcaf600):

```rust
pub struct ThreadIO {
    pub stdout_req_tx:  Sender<String>,
    pub stdout_ack_rx:  Receiver<()>,
    pub stderr_req_tx:  Sender<String>,
    pub stderr_ack_rx:  Receiver<()>,
    pub stdin_req_tx:   Sender<()>,
    pub stdin_reply_rx: Receiver<Arc<HolonAST>>,
}
```

**After:**

```rust
pub struct ThreadIO {
    pub stdout_tx:      Sender<StdOutServiceEvent>,
    pub stdout_ack_rx:  Receiver<()>,
    pub stderr_tx:      Sender<StdErrServiceEvent>,
    pub stderr_ack_rx:  Receiver<()>,
    pub stdin_tx:       Sender<StdInServiceEvent>,
    pub stdin_reply_rx: Receiver<Arc<HolonAST>>,
}
```

Rationale for field-name change (`stdout_req_tx` →
`stdout_tx`): "req-tx" leaked the old framing where the channel
carried a primitive request type. With the Event payload, the
channel carries arbitrary Event variants (data + control). The
shorter name reads honestly. (Surface as honest delta if you
elect a different rename; the BRIEF prefers `stdout_tx`.)

### 3. Reshape the three eval arms

**`eval_kernel_println`** at `src/thread_io.rs:130`:

```rust
// Before:
io.stdout_req_tx.send(line)...

// After:
io.stdout_tx.send(StdOutServiceEvent::Write { line })...
```

Same shape for `eval_kernel_eprintln` (uses `StdErrServiceEvent::Write`).

**`eval_kernel_readln`** at `src/thread_io.rs:188`:

```rust
// Before:
io.stdin_req_tx.send(())...

// After:
io.stdin_tx.send(StdInServiceEvent::Read)...
```

The recv side (`stdout_ack_rx.recv()`, `stderr_ack_rx.recv()`,
`stdin_reply_rx.recv()`) is unchanged — the reply types
weren't part of the Event reshape.

### 4. Migrate the 10 test fixture rows

File: `tests/wat_arc170_slice_1f_alpha_helpers.rs`

The fixture builds a `ThreadIO` and spawns tester threads that
play "service" roles. The migration:

- Constructor builds the three channel pairs with the new
  Event-typed senders
- Tester threads `recv()` Event variants instead of bare
  String / unit
- Match the Event variant to confirm the expected operation
  (`Write { line }` for stdout/stderr; `Read` for stdin)

Example sketch for row D (`row_d_println_populated_sends_serialized_string`):

```rust
// Before:
let (stdout_req_tx, stdout_req_rx) = bounded::<String>(1);
// ...
let tester = std::thread::spawn(move || {
    let line = stdout_req_rx.recv().unwrap();
    stdout_ack_tx.send(()).unwrap();
    line
});

// After:
let (stdout_tx, stdout_rx) = bounded::<StdOutServiceEvent>(1);
// ...
let tester = std::thread::spawn(move || {
    let event = stdout_rx.recv().unwrap();
    let line = match event {
        StdOutServiceEvent::Write { line } => line,
        _ => panic!("expected Write variant"),
    };
    stdout_ack_tx.send(()).unwrap();
    line
});
```

Apply the analogous shape to row E (eprintln) and row F (readln
— matches on `StdInServiceEvent::Read`).

Rows A / B / C / H / I / J don't exercise the channel payload
(they test the unpopulated path + type-check arms) — they
don't need migration beyond the ThreadIO constructor's new
field types.

Row G (`row_g_println_polymorphic_value_types`) iterates
println calls with different value types — same one-line
match-and-extract pattern as row D.

### 5. Add to `src/lib.rs` exports

Re-export the new types so the wat-side service implementations
(slice 1f-β-i and following) + the runtime orchestrator (slice
1f-γ) can reference them. After `pub mod thread_io;`:

```rust
pub use thread_io::{
    install_thread_io, uninstall_thread_io,
    StdInServiceEvent, StdOutServiceEvent, StdErrServiceEvent,
    ThreadId, ThreadIO,
};
```

(The existing `pub mod thread_io;` already grants access via
the path; the re-export is a convenience for consumers.)

## What to NOT do

- **No wat-side service implementations.** Those are slice 1f-β-i
  / ii / iii. This slice ONLY reshapes the Rust caller side
  + its test fixture.
- **No type-check arm changes.** `:wat::kernel::println` /
  `eprintln` / `readln` keep their existing TypeSchemes (∀T. T → nil
  for println/eprintln; () → :wat::holon::HolonAST for readln).
  The change is internal-only.
- **No runtime orchestrator changes.** Slice 1f-γ is later.
- **No deftest macro changes.** Those are slice 1f-0a-ii (or
  whatever the rot-fix slice ends up named).
- **No new Mutex / RwLock / CondVar.** Per ZERO-MUTEX.md
  discipline.
- **No new dependencies.** crossbeam_channel, std::sync::Arc,
  holon::HolonAST are already in scope.

## Substrate-grep citations (verify before committing)

- `src/thread_io.rs:44` — current `ThreadIO` struct
- `src/thread_io.rs:130` — `eval_kernel_println`
- `src/thread_io.rs:159` — `eval_kernel_eprintln`
- `src/thread_io.rs:188` — `eval_kernel_readln`
- `src/thread_io.rs:75` — `install_thread_io`
- `src/thread_io.rs:87` — `uninstall_thread_io`
- `src/lib.rs:94` — `pub mod thread_io;`
- `tests/wat_arc170_slice_1f_alpha_helpers.rs` — the 10 test
  rows to migrate
- `docs/arc/2026/05/170-program-entry-points/REALIZATIONS-SLICE-1.md`
  § Pass 18 — the locked Event enum shapes
- `docs/arc/2026/05/170-program-entry-points/BUILD-PLAN.md` §
  Slice 1f protocol — Event enum quoted in BUILD-PLAN

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — Three Event enums minted | grep finds `pub enum StdInServiceEvent`, `StdOutServiceEvent`, `StdErrServiceEvent` in `src/thread_io.rs` | ✓ |
| B — ThreadId typealias | grep finds `pub type ThreadId = i64;` (or newtype if elected) | ✓ |
| C — ThreadIO struct fields reshaped | stdout_tx, stderr_tx, stdin_tx fields carry Event-typed Senders | ✓ |
| D — `eval_kernel_println` constructs Write variant | grep finds `StdOutServiceEvent::Write { line }` in the eval arm | ✓ |
| E — `eval_kernel_eprintln` constructs Write variant | grep finds `StdErrServiceEvent::Write { line }` | ✓ |
| F — `eval_kernel_readln` constructs Read variant | grep finds `StdInServiceEvent::Read` | ✓ |
| G — `src/lib.rs` re-exports Event enums | grep finds Event types in `pub use thread_io::...` | ✓ |
| H — All 10 test rows in `tests/wat_arc170_slice_1f_alpha_helpers.rs` pass | `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 10/10 | ✓ |
| I — `cargo check --release` green | no compile errors | ✓ |
| J — Workspace within ±5 band | post-1f-0b: 1328 passed / 854 failed ±5 (no new regressions beyond slice 1f-α's 10 tests being green) | ✓ |
| K — Zero new dependencies | Cargo.toml unchanged | ✓ |
| L — Zero new Mutex / RwLock / CondVar | grep returns 0 hits in modified files | ✓ |
| M — Type-check arms unchanged | grep `src/check.rs` for `:wat::kernel::println` registration; verify the TypeScheme is unchanged | ✓ |
| N — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **`stdout_tx` vs `stdout_req_tx` field name** — BRIEF prefers
  shorter; if friction surfaces (e.g., a different name reads
  more honest given the Event payload), surface
- **ThreadId representation** — typealias to i64 vs newtype.
  Surface the call.
- **`#[derive(Clone)]` on Event enums** — needed for
  test fixtures that hand-construct Add variants? Or unused?
  Surface if it doesn't serialize cleanly
- **`#[derive(Debug)]`** — required for clean test-assertion
  diagnostics; should be standard but surface if friction
- **`Arc<HolonAST>` ownership on stdin reply** — same as slice
  1f-α; unchanged in this slice but worth verifying it composes
  with the Event::Add payload (which embeds a `Sender<Arc<HolonAST>>`)
- **Module location of the Event enums** — `src/thread_io.rs`
  is the natural home (alongside ThreadIO). If they want their
  own file or sub-module, surface

## Predicted runtime

60-90 min opus. Mostly mechanical (struct + enum mints, eval-arm
edits, test fixture migrations) but the design choices on
naming + the migration of 10 test rows warrant opus-tier
judgment. The pattern is fully specified by the BRIEF + pass 18.

**Hard cap:** 180 min (3 hours). Wakeup scheduled.

## Reference

- DESIGN.md (passes 1-18)
- REALIZATIONS-SLICE-1.md § Pass 18 (the locked Event protocol)
- BUILD-PLAN.md § Slice 1f-0b (the spec this slice fulfills)
- `src/thread_io.rs` (the file to reshape)
- `tests/wat_arc170_slice_1f_alpha_helpers.rs` (the 10 test
  rows to migrate)
- SCORE-SLICE-1F-A.md (slice 1f-α's calibration; this slice
  builds upon)
