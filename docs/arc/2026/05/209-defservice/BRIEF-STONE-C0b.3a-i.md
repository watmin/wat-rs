# BRIEF — Stone C0b.3a-i: reactor listener-arm + poll-driven non-blocking accept

**Executor:** Shadowdancer (sonnet). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (verify with
`pwd` first; any `.claude/worktrees/` path is illegal — re-cd; use `git -C
/home/watmin/work/holon/wat-rs`). Design: `DESIGN-STONE-C0b.3a-i-reactor-listener-arm.md` (read it).

## The work in one paragraph

Teach the `comms::process::Select` reactor to watch a **listen fd** for an incoming connection (a
`PollAdd POLLIN` arm + a new `SelectOutcome::Listener`), then make `accept'` (process) **non-blocking
and poll-driven** so it can never block: `listener'` binds the `UnixListener` non-blocking; `accept'`
polls via a `Select` listener-arm then non-blocking-accepts (dogfooding the new capability). The
visible behavior of `accept'` is unchanged (blocks until a connection) — but it's now poll-driven and
deadlock-safe, and the reactor gains the listener-arm C0b.3a-ii's service loop will use.

## Read in order (the rooms)

1. `src/comms/mod.rs:778` `SelectOutcome<T>` — add the `Listener` variant here.
2. `src/comms/process.rs:722` `Select` struct + `:778` `new` + `:790` `recv` + `:813` `select()` —
   especially `:816` (empty-guard), `:840` (`arm_count`/`needed_capacity`), `:862`–`:934` (SQE push:
   broadcast token 0, data token i+1; CQE drain: broadcast wins). This is where the listener arm slots
   in.
3. `src/comms/process.rs:34`–`35` (the integration-test import of `Select`/`SelectOutcome`) +
   existing `Select` tests at `:261`,`:283` — the pattern for the new reactor unit test.
4. `src/runtime.rs` — the `listener'` (process) arm (binds the `UnixListener` — add
   `set_nonblocking(true)`) + the `accept'` (process) arm (`SocketListener'` opaque → `&UnixListener`
   → currently blocking `accept()`; rework to poll-then-accept) + `wrap_stream_as_socket_peer`
   (reuse). (Grep `SOCKET_LISTENER_TYPE_PATH` to find both arms.)
5. Grep `SelectOutcome::` across `src/` — every `match` on it must gain a `Listener` arm (the ripple).

## Implementation sketch (fill the shape; do not invent a different one)

**(A) `src/comms/mod.rs` — the variant.**
```rust
pub enum SelectOutcome<T> {
    Shutdown,
    Recv { index: ReceiverIndex, result: Result<T, RecvError> },
    /// Arc 209 C0b.3a-i — the registered listener arm fired (a connection is pending).
    /// The caller accepts (non-blocking) and wraps the new connection.
    Listener,
}
```

**(B) `src/comms/process.rs` — `Select` listener arm.**
- Field: `listener_fd: Option<std::os::fd::RawFd>` (init `None` in `new`).
- `pub fn listener(&mut self, fd: std::os::fd::RawFd) { self.listener_fd = Some(fd); }`
- In `select()`:
  - empty-guard (`:816`): `if self.receivers.is_empty() && current_broadcast_fd().is_none() &&
    self.listener_fd.is_none() { return Err(... "zero arms") }`.
  - `arm_count` (`:840`): `+ if self.listener_fd.is_some() { 1 } else { 0 }`.
  - SQE push (after the data-arm loop): if `Some(lfd) = self.listener_fd`, push
    `opcode::PollAdd::new(types::Fd(lfd), libc::POLLIN as u32).build().user_data(LISTENER_TOKEN)`
    with `const LISTENER_TOKEN: u64 = u64::MAX;` (outside broadcast 0 / data 1..=N).
  - CQE drain: add `let mut fired_listener = false;`; on `token == LISTENER_TOKEN { fired_listener =
    true; }`. After the broadcast-wins check and `first_data_arm`: if `first_data_arm` is `None` and
    `fired_listener`, return `Ok(SelectOutcome::Listener)`. (Priority: broadcast > data > listener.)
- Reactor unit test (`tests/comms/process.rs`, mirror `:261`):
```rust
#[test]
fn select_listener_arm_fires_on_pending_connection() {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
    let addr = SocketAddr::from_abstract_name(b"wat.arc209.c0b3ai.test").unwrap();
    let listener = UnixListener::bind_addr(&addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    let t = std::thread::spawn(move || { let _c = UnixStream::connect_addr(&addr).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50)); });
    let mut sel: Select<String> = Select::new();
    sel.listener(listener.as_raw_fd());
    match sel.select().expect("select") {
        SelectOutcome::Listener => { let _ = listener.accept().expect("accept the pending conn"); }
        other => panic!("expected Listener; got {:?}", other),
    }
    t.join().unwrap();
}
```
(If `SelectOutcome` isn't `Debug`, assert via `matches!` instead of `{:?}`.)

**(C) `src/runtime.rs` — non-blocking accept.**
- `listener'` (process) arm: after `UnixListener::bind_addr(...)`, add
  `listener.set_nonblocking(true).map_err(|e| /* MalformedForm "set_nonblocking: {e}" */)?;`
- `accept'` (process) arm: replace the blocking `listener.accept()` with:
```rust
let raw = listener.as_raw_fd();
let mut sel = crate::comms::process::Select::<String>::new();
sel.listener(raw);
loop {
    match sel.select().map_err(|e| /* MalformedForm "accept' select: {e}" */)? {
        crate::comms::SelectOutcome::Listener => match listener.accept() {
            Ok((stream, _)) => return wrap_stream_as_socket_peer(stream, list_span, OP),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue, // spurious; re-poll
            Err(e) => return Err(/* MalformedForm "accept on abstract UDS listener failed: {e}" */),
        },
        crate::comms::SelectOutcome::Shutdown =>
            return Err(/* MalformedForm "accept': interrupted by shutdown" */),
        crate::comms::SelectOutcome::Recv { .. } => unreachable!("accept' Select has no receivers"),
    }
}
```
(`listener` here is the `&UnixListener` downcast from the `SocketListener'` opaque — keep that.)

**(D) The ripple — `SelectOutcome::Listener` exhaustiveness.** Grep `SelectOutcome::` in `src/`; every
`match` gains a `Listener` arm. The thread-tier select' sites + brackets never produce `Listener` →
`SelectOutcome::Listener => unreachable!("thread/bracket Select has no listener arm")` (or a clean
error if the site prefers). The compiler will list every site — fix until `cargo build` is clean.

## Blast radius (bounded)

`src/comms/mod.rs`, `src/comms/process.rs`, `src/runtime.rs` (the two process arms), + the mechanical
`SelectOutcome::Listener` arms at existing match sites. NO new files except the reactor unit test in
`tests/comms/process.rs`. NO thread-tier behavior change (only the exhaustiveness arms). NO
`select'`-3arg process branch (C0b.3a-ii). NO `SO_PEERCRED`.

## STOP triggers (rejection criteria — ship nothing, report the gap)

1. **STOP-1:** if `PollAdd POLLIN` on a listening `UnixListener` fd does NOT fire when a connection is
   pending (the reactor unit test hangs/fails) — STOP, report; do not switch to `IORING_OP_ACCEPT`.
2. **STOP-2:** if adding `SelectOutcome::Listener` forces a change beyond adding match arms (e.g. a
   structural change to a match site's logic) — STOP, report which site.
3. **STOP-3:** if `set_nonblocking(true)` on the bound `UnixListener` is not reachable / breaks the
   c0b2c round-trip in a way the poll-then-accept rework can't resolve — STOP, report.

Rejection criteria, not permission to ship a workaround.

## The gate

`cargo build --release` clean (the `SelectOutcome::Listener` ripple — compile the FULL surface), then:
```
cargo test --release -p wat --test comms select_listener_arm_fires_on_pending_connection -- --test-threads=1
cargo test --release -p wat --test nursery probe_arc209_c0b2c -- --test-threads=1      # accept' poll-driven, still GREEN
cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener probe_arc209_c0b2b_socket_peer -- --test-threads=1
cargo test --release -p wat --test comms -- --test-threads=1                            # comms group intact
cargo test --release --workspace --no-run                                               # FULL surface compiles (the ripple)
```
Report the exact `test result:` line for each. Do NOT commit — the Inquisitor weighs and commits.
