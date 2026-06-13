# BRIEF — Stone C0b.2c: process `listener'`/`connect'`/`accept'` over abstract UDS

**Executor:** Shadowdancer (sonnet). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (verify
with `pwd` first; any path containing `.claude/worktrees/` is illegal — re-cd to the anchor; use
`git -C /home/watmin/work/holon/wat-rs` for git). Design: `DESIGN-STONE-C0b.2c-process-connection-verbs.md`
(read it). The RED gate is committed: `tests/nursery/probe_arc209_c0b2c_process_connection.rs`.

## The work in one paragraph

Fill the `(process)` arm of the three connection verbs. Today they are thread-tier only (crossbeam
rendezvous) and `(listener' (process) …)` is a clean C0b.2a check error. After this strike,
`(listener' (process) :S :R)` binds an abstract-namespace AF_UNIX socket and returns
`Tuple[SocketListener'<S,R>, SocketAddress'<S,R>]`; `(connect' addr)` over a `SocketAddress'`
returns a connected `SocketPeer'<S,R>`; `(accept' listener)` over a `SocketListener'` returns a
`SocketPeer'<R,S>`. `send'`/`recv'` already drive `SocketPeer'` (C0b.2b), so the committed probe
round-trips and turns GREEN. The thread tier is untouched.

## Read in order (the rooms)

1. `src/comms/process.rs:1051` — `socket_pair()`. The per-end logic (dup the fd, build
   Sender/Receiver with a fresh `IoUring::new(4)`) is what you factor out. The `pair()` fn just
   above (~`:998`) shows the Sender/Receiver field shapes.
2. `src/kernel/peer.rs:335` — `SocketPeer<I,O> { tx: Sender<I>, rx: Receiver<O> }`. The thing
   `connect'`/`accept'` wrap.
3. `src/kernel/spawn.rs:142` — `SOCKET_PEER_TYPE_PATH` + `SocketPeerCell` (`:149`). Add the two new
   consts here next to them.
4. `src/runtime.rs:17916` `eval_listener_prime`, `:17958` `eval_connect_prime`, `:18036`
   `wrap_connect_request`, `:18165` `eval_accept_prime`. The thread tier you extend. The dispatch
   call site is `:4562`–`4568` (note connect'/accept' already receive `env, sym`; listener' does not
   yet).
5. `src/runtime.rs:22723` + `:23018` — the existing `send'`/`recv'` `SOCKET_PEER_TYPE_PATH` arms
   (your wrapped peers flow straight into these). `:5632` `value_matches_type_by_name` — confirm the
   record `class_fqdn` format (bare, no leading colon, e.g. `"wat::spawn::ProcessOpts"`).
6. `src/check.rs:9897` `infer_listener_prime`, `:9941` `listener_tuple`, `:9952`
   `infer_connect_prime`, `:10008` `infer_accept_prime`. `:10332` `project_peer_io` already knows
   `wat::kernel::SocketPeer'` — no change needed there.
7. `src/rust_deps/marshal.rs:337` `make_rust_opaque<T: Any + Send + Sync>` + `:380`
   `downcast_ref_opaque<T: Any>`. The wrap/unwrap of the new opaques.
8. `tests/nursery/probe_arc209_c0b_uds_abstract_spike.rs` — the proven raw OS sequence
   (`from_abstract_name`/`bind_addr`/`connect_addr`/`accept`); mirror these calls.

## Implementation sketch (fill the shape; do not invent a different one)

**(A) `src/comms/process.rs` — factor the fd→(Sender,Receiver) helper, DRY `socket_pair` onto it.**

```rust
/// Wrap one connected socket fd as a (Sender<T>, Receiver<T>) pair.
/// write_fd = the fd; read_fd = a dup, so Sender and Receiver own independent
/// OwnedFd lifetimes. Per-Receiver IoUring::new(4) (same reactor as pair()/socket_pair()).
pub fn sender_receiver_from_fd<T: HolonRepresentable>(
    fd: OwnedFd,
) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let read_fd = fd.try_clone()
        .map_err(|e| std::io::Error::other(format!("dup for sender_receiver_from_fd failed: {}", e)))?;
    let receiver = Receiver {
        read_fd,
        accumulator: RefCell::new(Vec::new()),
        ring: RefCell::new(
            IoUring::new(4).map_err(|e| std::io::Error::other(
                format!("IoUring::new(4) failed at sender_receiver_from_fd: {}", e)))?,
        ),
        _phantom: PhantomData,
    };
    Ok((Sender { write_fd: fd, _phantom: PhantomData }, receiver))
}
```
Then `socket_pair` becomes: socketpair(2) → `let end_a = sender_receiver_from_fd(OwnedFd::from_raw_fd(sv[0]))?;`
`let end_b = sender_receiver_from_fd(OwnedFd::from_raw_fd(sv[1]))?;` `Ok((end_a, end_b))`. Keep the
SAFETY comments on the `from_raw_fd` calls. Confirm `socket_pair`'s own test still passes.

**(B) `src/kernel/spawn.rs` — two consts next to `SOCKET_PEER_TYPE_PATH`.**

```rust
pub const SOCKET_LISTENER_TYPE_PATH: &str = ":wat::kernel::SocketListener'";
pub const SOCKET_ADDRESS_TYPE_PATH:  &str = ":wat::kernel::SocketAddress'";
```
(No cell type alias — the payloads are a bare `std::os::unix::net::UnixListener` and a `String`,
both `Send + Sync`, wrapped directly by `make_rust_opaque`; no `ThreadOwnedCell`.)

**(C) `src/runtime.rs` — the runtime arms.**

A small shared helper near `eval_socket_pair_prime`:
```rust
/// Wrap a connected UnixStream as a SocketPeer' opaque (custody on this thread).
fn wrap_stream_as_socket_peer(stream: std::os::unix::net::UnixStream, span: &Span, op: &'static str)
    -> Result<Value, EvalBreak>
{
    use std::os::fd::OwnedFd;
    let (tx, rx) = crate::comms::process::sender_receiver_from_fd::<String>(OwnedFd::from(stream))
        .map_err(|e| RuntimeError { span: span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: op.into(), reason: format!("wrap socket stream failed: {}", e) } })?;
    use crate::kernel::peer::SocketPeer;
    use crate::kernel::spawn::SOCKET_PEER_TYPE_PATH;
    use crate::rust_deps::custodia::ThreadOwnedCell;
    use crate::rust_deps::marshal::make_rust_opaque;
    Ok(make_rust_opaque(SOCKET_PEER_TYPE_PATH,
        Arc::new(ThreadOwnedCell::new(Some(SocketPeer { tx, rx })))))
}
```

`eval_listener_prime` — grow the signature to `(args, list_span, env, sym)` and update the call site
`:4562` to `eval_listener_prime(args, list_span, env, sym)`. After the arity/keyword validation,
evaluate the host and dispatch:
```rust
let host_val = eval_inner(&args[0], env, sym)?.value_owned();
let is_process = matches!(&host_val,
    Value::wat__Record { class_fqdn, .. } | Value::wat__holon__Record { class_fqdn, .. }
        if class_fqdn.as_str() == "wat::spawn::ProcessOpts");
// thread path = the existing crossbeam rendezvous (unchanged), taken when !is_process.
```
For the process path: generate a unique abstract name, bind, return the two opaques:
```rust
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener};
use std::sync::atomic::{AtomicU64, Ordering};
static LISTENER_SEQ: AtomicU64 = AtomicU64::new(0);
let n = LISTENER_SEQ.fetch_add(1, Ordering::Relaxed);
let name = format!("wat.arc209.{}.{}", std::process::id(), n);
let sa = SocketAddr::from_abstract_name(name.as_bytes())
    .map_err(|e| /* MalformedForm "abstract addr: {e}" */)?;
let listener = UnixListener::bind_addr(&sa)
    .map_err(|e| /* MalformedForm "bind abstract UDS: {e}" */)?;
use crate::kernel::spawn::{SOCKET_LISTENER_TYPE_PATH, SOCKET_ADDRESS_TYPE_PATH};
use crate::rust_deps::marshal::make_rust_opaque;
Ok(Value::Tuple(Arc::new(vec![
    make_rust_opaque(SOCKET_LISTENER_TYPE_PATH, listener),
    make_rust_opaque(SOCKET_ADDRESS_TYPE_PATH, name),
])))
```

`eval_connect_prime` — after evaluating `args[0]`, add a `Value::RustOpaque(inner)` arm BEFORE the
thread `Value::wat__kernel__Sender` arm:
```rust
Value::RustOpaque(inner) if inner.type_path == SOCKET_ADDRESS_TYPE_PATH => {
    let name: &String = downcast_ref_opaque(&inner, SOCKET_ADDRESS_TYPE_PATH, OP, args[0].span().clone())?;
    let sa = SocketAddr::from_abstract_name(name.as_bytes()).map_err(...)?;
    let stream = UnixStream::connect_addr(&sa).map_err(...)?;
    return wrap_stream_as_socket_peer(stream, list_span, OP);
}
```
(`downcast_ref_opaque` needs `&RustOpaqueInner`; you have `inner: Arc<…>` from the match — pass
`inner.as_ref()` / `&inner`.) The existing Sender arm stays the thread path.

`eval_accept_prime` — same shape: add a `Value::RustOpaque(inner) if inner.type_path ==
SOCKET_LISTENER_TYPE_PATH` arm BEFORE the thread `Value::wat__kernel__Receiver` arm:
```rust
let listener: &UnixListener = downcast_ref_opaque(&inner, SOCKET_LISTENER_TYPE_PATH, OP, args[0].span().clone())?;
let (stream, _addr) = listener.accept().map_err(...)?;   // blocks until a connection (honest wire-wait)
return wrap_stream_as_socket_peer(stream, list_span, OP);
```

**(D) `src/check.rs` — the type arms.**

`socket_listener_tuple` helper next to `listener_tuple` (`:9941`):
```rust
fn socket_listener_tuple(s: TypeExpr, r: TypeExpr) -> TypeExpr {
    TypeExpr::Tuple(vec![
        TypeExpr::Parametric { head: "wat::kernel::SocketListener'".into(), args: vec![s.clone(), r.clone()] },
        TypeExpr::Parametric { head: "wat::kernel::SocketAddress'".into(),  args: vec![s, r] },
    ])
}
```
`infer_listener_prime` (`:9919` block): keep the host inference; replace the thread-only constraint
with a dispatch:
- `host_reduced == TypeExpr::Path(":wat::spawn::ThreadOpts")` → `listener_tuple(s, r)` (unchanged).
- `== TypeExpr::Path(":wat::spawn::ProcessOpts")` → `socket_listener_tuple(s, r)`.
- else → the existing `TypeMismatch` check error, but widen `expected` to name BOTH valid hosts:
  `"(:wat::spawn::thread) or (:wat::spawn::process)"`.

`infer_connect_prime` (`:9982` match): add a branch — `addr_reduced` is
`Parametric { head: "wat::kernel::SocketAddress'", args: [s, r] }` → return
`Parametric { head: "wat::kernel::SocketPeer'", args: vec![s, r] }`. Keep the `Address'` branch
(thread) and the error branch; widen `expected` to `"Address'<S,R> or SocketAddress'<S,R>"`.

`infer_accept_prime` (`:10038` match): add a branch — `listener_reduced` is
`Parametric { head: "wat::kernel::SocketListener'", args: [s, r] }` → return
`Parametric { head: "wat::kernel::SocketPeer'", args: vec![r, s] }` (FLIPPED, like the thread arm).
Keep `Listener'` (thread) + error; widen `expected` to `"Listener'<S,R> or SocketListener'<S,R>"`.

**(E) `tests/nursery/probe_arc209_c0b2a_listener_host_thread_only.rs` — supersede Test 1 (the honest delta).**

C0b.2a's first test, `listener_with_process_host_is_a_check_error_not_a_silent_thread` (`:30`),
asserts `(listener' (process) …)` is a CHECK ERROR — true when the process tier was unbuilt. C0b.2c
BUILDS it, so a process host is now VALID; that test will fail. This is a living nursery test (not an
immutable inscription) — update it in THIS strike. Rewrite it to assert the new truth:
```rust
/// C0b.2c BUILT the process connection tier — `(listener' (process) …)` now type-checks
/// (it was a C0b.2a check error while the tier was unbuilt; C0b.2c supersedes that by
/// building it, not rejecting it). The round-trip gate is probe_arc209_c0b2c.
#[test]
fn listener_with_process_host_now_type_checks() {
    let result = wat::freeze::startup_from_source(PROCESS_LISTENER, None, Arc::new(InMemoryLoader::new()));
    assert!(result.is_ok(),
        "(listener' (process) …) must type-check after C0b.2c built the process tier. got: {:?}",
        result.err());
}
```
Leave `listener_with_thread_host_still_type_checks` (`:48`) unchanged. Update the file's module-doc
header (`:1`–`:17`) to note C0b.2c closed the gap by building the tier (one or two lines; don't
rewrite the whole header).

## Blast radius (bounded)

`src/comms/process.rs`, `src/kernel/spawn.rs`, `src/runtime.rs`, `src/check.rs` only. NO new files
(the probe already exists). NO `ThreadOwnedCell` on the listener/address opaques. NO change to the
thread-tier code paths (only ADD `(process)` arms beside them). NO `select'` work, NO `SO_PEERCRED`,
NO well-known-name input (those are C0b.3a/3b/Stone C). Do NOT touch `project_peer_io` (already
SocketPeer'-aware).

## STOP triggers (rejection criteria — ship nothing, report the gap)

1. **STOP-1:** if `Sender`/`Receiver`/`IoUring`/`OwnedFd::try_clone` are not accessible from
   `comms::process` exactly as `socket_pair` uses them — STOP and report; do not restructure the
   module to reach them.
2. **STOP-2:** if `UnixStream` → `OwnedFd::from(stream)` does not compile on this toolchain — STOP
   and report (do NOT reach for `into_raw_fd` + `from_raw_fd` without confirming it's the equivalent;
   surface the toolchain fact).
3. **STOP-3:** if the host record's `class_fqdn` is NOT `"wat::spawn::ProcessOpts"` for
   `(:wat::spawn::process)` (read `value_matches_type_by_name:5632` + verify) — STOP and report the
   actual class string; do not guess a different match.
4. **STOP-4:** if `make_rust_opaque` rejects `UnixListener` or `String` on its `Send + Sync` bound —
   STOP and report; do not wrap them in an unnecessary cell to satisfy the bound.

These are REJECTION criteria, not permission to ship a workaround. If you cannot do it cleanly,
report the gap and ship nothing.

## The gate

`cargo build --release` clean, then:
```
cargo test --release -p wat --test nursery process_listener_connect_accept_round_trips_over_abstract_uds -- --test-threads=1
```
must PASS (it is RED at HEAD on the C0b.2a process-host check error; GREEN proves the round-trip).
Then confirm no regression:
```
cargo test --release -p wat --test nursery probe_arc209_c0b2b_socket_peer probe_arc209_c0b1b_select_listener probe_arc209_c0b1_thread_connection probe_arc209_c0b2a -- --test-threads=1
cargo test --release -p wat --lib comms::process -- --test-threads=1     # socket_pair unit test intact
```
Report the exact pass/fail line for each. Do NOT commit — the Inquisitor weighs and commits.
