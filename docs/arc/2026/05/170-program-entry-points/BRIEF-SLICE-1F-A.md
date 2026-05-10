# Arc 170 slice 1f-α — BRIEF

**Substrate; opus.** Mint three substrate primitives that look up
per-thread channel handles from a new thread-local data structure
(`ThreadIO`) and apply mini-TCP block-on-completion discipline.
Slice 1f-β / 1f-γ / 1f-δ haven't shipped yet — these primitives
must work standalone (in tests with hand-populated ThreadIO) AND
return a clean diagnostic when ThreadIO is unpopulated.

Architecture lock-in: see REALIZATIONS-SLICE-1.md passes 15 + 16
and BUILD-PLAN.md § Slice 1f.

## Mission

Mint:

```
(:wat::kernel::println v)  -> :wat::core::nil
(:wat::kernel::eprintln v) -> :wat::core::nil
(:wat::kernel::readln)     -> :wat::holon::HolonAST
```

`println` / `eprintln` are polymorphic in `v` (any wat value).
The substrate already has `value_to_edn_with`
(`src/edn_shim.rs:954`) — invoke it internally to convert any
`Value` into a String for transmission. The caller never has to
think about EDN.

Each primitive:
1. Looks up the calling thread's `ThreadIO` from `thread_local!`
2. If unpopulated → returns `RuntimeError::ServiceNotRunning`
   with a clear diagnostic
3. If populated → executes its mini-TCP cycle:
   - **println**: serialize v → send String on stdout req-tx
     → block on stdout ack-rx → return `Value::Nil`
   - **eprintln**: same shape, stderr pair
   - **readln**: send `()` on stdin req-tx → block on stdin
     reply-rx → return the received HolonAST

Block-on-completion is mandatory. Every send is paired with a
recv. No fire-and-forget.

## Substrate edits

### 1. New struct `ThreadIO` + thread-local (in `src/runtime.rs` or a new `src/thread_io.rs` if cleaner)

```rust
use crossbeam::channel::{Receiver, Sender};
use std::sync::Arc;
use holon::HolonAST;

/// Per-thread channel handles used by `:wat::kernel::println` /
/// `eprintln` / `readln`. Populated by `:wat::kernel::spawn-thread`
/// (slice 1f-γ); for slice 1f-α, populated by tests via a setter
/// helper.
///
/// All six channel ends are owned (not Arc'd) — the thread that
/// owns the ThreadIO IS the thread that uses these channels.
/// crossbeam's Sender/Receiver are Send by themselves; the
/// thread_local! cell ensures only one thread accesses any given
/// ThreadIO.
pub struct ThreadIO {
    pub stdout_req_tx: Sender<String>,
    pub stdout_ack_rx: Receiver<()>,
    pub stderr_req_tx: Sender<String>,
    pub stderr_ack_rx: Receiver<()>,
    pub stdin_req_tx: Sender<()>,
    pub stdin_reply_rx: Receiver<Arc<HolonAST>>,
}

thread_local! {
    static THREAD_IO: std::cell::RefCell<Option<ThreadIO>>
        = std::cell::RefCell::new(None);
}

/// Slice 1f-γ will call this from spawn-thread's substrate
/// primitive. Slice 1f-α tests call this directly to populate
/// the per-test ThreadIO.
pub fn install_thread_io(io: ThreadIO) {
    THREAD_IO.with(|cell| { *cell.borrow_mut() = Some(io); });
}

/// Slice 1f-γ will call this when reaping a thread.
pub fn uninstall_thread_io() -> Option<ThreadIO> {
    THREAD_IO.with(|cell| cell.borrow_mut().take())
}

/// Internal accessor used by the three eval arms. Returns
/// `RuntimeError::ServiceNotRunning` if unpopulated.
fn with_thread_io<F, T>(op: &'static str, f: F) -> Result<T, RuntimeError>
where
    F: FnOnce(&ThreadIO) -> Result<T, RuntimeError>,
{
    THREAD_IO.with(|cell| {
        match &*cell.borrow() {
            Some(io) => f(io),
            None => Err(RuntimeError::ServiceNotRunning {
                op: op.into(),
                span: Span::unknown(),
            }),
        }
    })
}
```

### 2. New `RuntimeError` variant

Add to the existing `pub enum RuntimeError` in `src/runtime.rs`:

```rust
ServiceNotRunning {
    op: String,           // ":wat::kernel::println" etc.
    span: Span,
},
```

Display impl: *"`{op}` called before stdio services running. The
runtime spawns these services at process start (slice 1f-δ); when
called from a hand-spawned context (e.g., a test), the test must
populate ThreadIO via `install_thread_io` before invoking."*

### 3. Three eval arms — model on `eval_edn_write` (`src/edn_shim.rs:72`)

```rust
/// `(:wat::kernel::println v)` → `:wat::core::nil`. Serialize v
/// to compact EDN; send through this thread's stdout req-tx;
/// block on ack-rx; return nil.
pub fn eval_kernel_println(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::println";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write(&edn);
    with_thread_io(OP, |io| {
        io.stdout_req_tx.send(line).map_err(|_| RuntimeError::ChannelDisconnected {
            op: OP.into(),
            span: Span::unknown(),
        })?;
        io.stdout_ack_rx.recv().map_err(|_| RuntimeError::ChannelDisconnected {
            op: OP.into(),
            span: Span::unknown(),
        })?;
        Ok(Value::Nil)
    })
}

// eprintln symmetric — uses stderr_req_tx + stderr_ack_rx
// readln symmetric — sends () on stdin_req_tx, recvs Arc<HolonAST>
//   on stdin_reply_rx, returns Value-wrapped HolonAST
```

If `RuntimeError::ChannelDisconnected` doesn't exist yet, add it
(model on existing send/recv error variants per arc 111).

### 4. Type-check arm registrations in `src/check.rs`

Three new entries in the appropriate registration table:

```
":wat::kernel::println"  → fn(:T) -> :wat::core::nil   (T: any)
":wat::kernel::eprintln" → fn(:T) -> :wat::core::nil   (T: any)
":wat::kernel::readln"   → fn() -> :wat::holon::HolonAST
```

Find the existing pattern by searching for how `:wat::edn::write`
is registered (it has the same any-T input shape).

## Test fixture: `tests/wat_arc170_slice_1f_alpha_helpers.rs`

Build on the harness style of
`tests/wat_arc170_slice_1e_user_main_nil.rs` (most-recent slice
1e test).

### Required tests

| Row | Test | What it verifies |
|-----|------|------------------|
| A | `println_unpopulated_returns_service_not_running` | `(:wat::kernel::println 42)` without ThreadIO → `Err(RuntimeError::ServiceNotRunning)` with op=":wat::kernel::println" |
| B | `eprintln_unpopulated_returns_service_not_running` | same shape for eprintln |
| C | `readln_unpopulated_returns_service_not_running` | same for readln |
| D | `println_populated_sends_serialized_string` | install ThreadIO with stdout pair set up via `crossbeam::channel::bounded(1)`; spawn a tester thread that recv's on stdout-req-rx + acks via stdout-ack-tx; `(:wat::kernel::println 42)` from main thread; assert tester received "42"; assert println returned `Value::Nil` |
| E | `eprintln_populated_sends_serialized_string` | same shape, stderr pair, value `"hello"` → assert tester received `"\"hello\""` (EDN-quoted) |
| F | `readln_populated_returns_received_form` | install ThreadIO with stdin pair set up; spawn tester that recv's on stdin-req-rx + sends back a HolonAST via stdin-reply-tx; `(:wat::kernel::readln)` from main thread; assert returned value matches expected HolonAST |
| G | `println_polymorphic_value_types` | exercise `println` with i64, String, bool, tuple, struct — each serializes via `value_to_edn_with` correctly; tester reads each EDN line and asserts content |
| H | `type_check_println_accepts_any_T` | parse `(:wat::core::define (:test::p (v :wat::core::i64) -> :wat::core::nil) (:wat::kernel::println v))`; assert no type-check errors |
| I | `type_check_eprintln_accepts_any_T` | same shape for eprintln |
| J | `type_check_readln_returns_holonast` | parse `(:wat::core::define (:test::r -> :wat::holon::HolonAST) (:wat::kernel::readln))`; assert no type-check errors; assert return type inferred as `:wat::holon::HolonAST` |

### Test helper pattern

Each "populated" test follows this skeleton:

```rust
fn run_with_thread_io<F, T>(io: ThreadIO, body: F) -> T
where F: FnOnce() -> T {
    install_thread_io(io);
    let result = body();
    let _ = uninstall_thread_io();
    result
}

#[test]
fn row_d_println_populated_sends_serialized_string() {
    let (out_req_tx, out_req_rx) = bounded::<String>(1);
    let (out_ack_tx, out_ack_rx) = bounded::<()>(1);
    let (err_req_tx, _err_req_rx) = bounded::<String>(1);
    let (_err_ack_tx, err_ack_rx) = bounded::<()>(1);
    let (stdin_req_tx, _stdin_req_rx) = bounded::<()>(1);
    let (_stdin_reply_tx, stdin_reply_rx) = bounded::<Arc<HolonAST>>(1);

    let io = ThreadIO {
        stdout_req_tx: out_req_tx,
        stdout_ack_rx: out_ack_rx,
        stderr_req_tx: err_req_tx,
        stderr_ack_rx: err_ack_rx,
        stdin_req_tx,
        stdin_reply_rx,
    };

    // tester thread plays "service" role — receives the string,
    // immediately acks
    let tester = std::thread::spawn(move || {
        let line = out_req_rx.recv().unwrap();
        out_ack_tx.send(()).unwrap();
        line
    });

    let result = run_with_thread_io(io, || {
        eval_wat_string("(:wat::kernel::println 42)")
    });

    assert_eq!(result.unwrap(), Value::Nil);
    let received = tester.join().unwrap();
    assert_eq!(received, "42");
}
```

## What to NOT do

- **No service implementations** — slice 1f-α does NOT ship
  `wat/kernel/services/*.wat`. Those are slice 1f-β.
- **No spawn-thread integration** — slice 1f-α does NOT modify
  `:wat::kernel::spawn-thread`. That's slice 1f-γ.
- **No wat-cli boot integration** — slice 1f-α does NOT touch
  `crates/wat-cli/`. That's slice 1f-δ.
- **No Console retirement** — Console retires in slice 1f-ε.
  Don't touch Console-using tests.
- **No new dependencies** — Cargo.toml unchanged. crossbeam +
  std::sync::Arc are already in scope.

## Constraints

- `cargo check --release` must be green at slice end.
- New tests in `tests/wat_arc170_slice_1f_alpha_helpers.rs` must
  pass.
- Workspace cargo test fail count must not regress beyond ±5
  from baseline (slice 1f-α is parallel infrastructure;
  existing tests don't touch the new primitives).
- Zero new Mutex / RwLock / CondVar (per ZERO-MUTEX.md). The
  `RefCell<Option<ThreadIO>>` inside `thread_local!` is the
  per-thread interior-mutability primitive — same shape as
  existing `thread_local!` usages in `src/runtime.rs`.

## Substrate-grep citations (verify before committing)

Each of these MUST exist for the BRIEF to be achievable:

- `value_to_edn_with` at `src/edn_shim.rs:954` ✓
- `eval_edn_write` pattern at `src/edn_shim.rs:72` ✓ (template
  for the three new eval arms — same shape: extract one arg,
  serialize via value_to_edn, return)
- `require_one_arg` at `src/edn_shim.rs:108` ✓ (helper for
  arity check)
- existing `thread_local!` at `src/runtime.rs:14304` ✓ (one
  precedent in this codebase; the new ThreadIO uses the same
  pattern)
- `wat_edn::write(&edn)` ✓ (substrate already calls it from
  `eval_edn_write`)
- `holon::HolonAST` ✓ (universally available via `use holon`)
- crossbeam `Sender` / `Receiver` ✓ (used throughout the
  substrate)
- `RuntimeError` enum at `src/runtime.rs` ✓ (extend with
  `ServiceNotRunning` + maybe `ChannelDisconnected` if not
  already present)
- `Span::unknown()` ✓ (used throughout for synthesized errors)

## Ship criteria (mapped to scorecard)

See `EXPECTATIONS-SLICE-1F-A.md` for the row-by-row scorecard.
At a glance:

- All three primitives evaluate cleanly (populated + unpopulated
  paths)
- Type-check arms register correctly + accept the right shapes
- 10 test rows pass in `tests/wat_arc170_slice_1f_alpha_helpers.rs`
- Workspace fail count unchanged within ±5 band
- Zero new dependencies; zero new Mutex
- `cargo check --release` green; `cargo test --release --test
  wat_arc170_slice_1f_alpha_helpers` green

## Honest delta categories — surface, don't work-around

- **`Arc<HolonAST>` vs raw `HolonAST` on the stdin reply channel.**
  Slice 1f-i precedent used `Arc<HolonAST>` (avoids cloning large
  ASTs across the channel). If `value_to_edn_with` or
  `Value::HolonAST` expects a different ownership shape, surface
  the friction.

- **`Value::Nil` constructor name.** If the actual variant is
  `Value::Unit` or similar, use whatever the substrate defines.
  Don't invent a new variant.

- **Test crossbeam imports.** If crossbeam isn't in dev-deps,
  surface — the test fixture imports it.

- **eval-arm registration site.** If the registration in
  `src/runtime.rs` happens via macro vs match arm vs registry
  table, follow the existing convention. The pattern for
  `:wat::edn::write` is the canonical reference.

- **Type-check arm registration site.** Same — find the
  `:wat::edn::write` registration in `src/check.rs` and mirror it.

If any of these surface as substantive substrate friction
(scope expansion required), STOP and surface — don't expand the
slice unilaterally.

## Reference

- DESIGN.md (passes 1-13, then 15 + 16 lock-in)
- REALIZATIONS-SLICE-1.md § Pass 15 + § Pass 16
- BUILD-PLAN.md § Slice 1f-α
- ZERO-MUTEX.md § Tier 3 + § Mini-TCP
- SERVICE-PROGRAMS.md § The lockstep
- src/edn_shim.rs (the value_to_edn family — your serialization
  primitives)
- src/runtime.rs:14304 (existing thread_local! precedent)
- tests/wat_arc170_slice_1e_user_main_nil.rs (most-recent arc
  170 test fixture style)
