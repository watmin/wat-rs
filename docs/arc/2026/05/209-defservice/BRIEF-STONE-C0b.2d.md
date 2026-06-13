# BRIEF — Stone C0b.2d: connect-by-name (well-known names)

**Executor:** Shadowdancer (sonnet). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (verify with
`pwd`; any `.claude/worktrees/` path is illegal — re-cd; use `git -C /home/watmin/work/holon/wat-rs`).
Design: `DESIGN-STONE-C0b.2d-connect-by-name.md` (read it). RED gate committed:
`tests/probe_arc209_c0b2d_named_cross_process.rs`.

## The work in one paragraph

Make cross-process services addressable by a shared **name**. Add `(:wat::kernel::socket-address'
name :S :R) -> SocketAddress'<S,R>` (construct a typed address from a `String` name). Change
`listener'` (process) to **bind a given address** (`(listener' (process) addr)`) instead of minting
one — retiring the C0b.2c mint-and-return. `connect'`/`accept'` are UNCHANGED (they already consume
`SocketAddress'`; it now comes from `socket-address'`). After this, two processes that name the same
string rendezvous on the same abstract UDS — the committed cross-process gate (5→105 across the
boundary) turns GREEN.

## Read in order (the rooms)

1. `src/runtime.rs` `eval_socket_pair_prime` (grep it) — the template for the new `socket-address'`
   verb (validate type-keyword args, `make_rust_opaque(SOCKET_ADDRESS_TYPE_PATH, …)`).
2. `src/runtime.rs` `eval_listener_prime` (grep `fn eval_listener_prime`) — the process arm currently
   mints (`LISTENER_SEQ` + auto-name + returns a Tuple of two opaques). You change it to bind a given
   addr. The dispatch evaluates `args[0]` (host) and currently requires exactly 3 args.
3. `src/runtime.rs` `eval_connect_prime` — the `SocketAddress'` arm (`SOCKET_ADDRESS_TYPE_PATH`,
   downcast `&String` → `connect_addr`). UNCHANGED — read it to see the opaque shape `socket-address'`
   must produce (`make_rust_opaque(SOCKET_ADDRESS_TYPE_PATH, name_string)`).
4. `src/check.rs` `infer_socket_pair_prime` (`:9852`-ish) + `parse_peer_pair_type_arg` — the template
   for `infer_socket_address_prime`. `infer_listener_prime` (`:9941`) process arm +
   `socket_listener_tuple` (`:10000`) — you change the process arm to bind-addr → `SocketListener'`.
   `infer_connect_prime` `SocketAddress'` branch (`:10046`) — UNCHANGED.
5. `src/runtime.rs` + `src/check.rs` dispatch sites for `:wat::kernel::socket-pair'` /
   `:wat::kernel::listener'` — add the `:wat::kernel::socket-address'` entry beside `socket-pair'`.
6. `tests/nursery/probe_arc209_c0b2c_process_connection.rs` — update to the named form (in-strike
   supersession of the mint form).

## Implementation sketch (fill the shape; do not invent a different one)

**(A) `socket-address'` verb — `src/runtime.rs` (beside `eval_socket_pair_prime`) + dispatch.**
```rust
fn eval_socket_address_prime(args, list_span, env, sym) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::socket-address'";
    if args.len() != 3 { /* ArityMismatch expected 3 (name :S :R) */ }
    // args[0] = name (a String value); args[1],args[2] = type keywords (checker-only).
    for i in [1usize, 2usize] { if !matches!(args[i], WatAST::Keyword(_,_)) { /* MalformedForm */ } }
    let name = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),  // confirm the String value accessor in this codebase
        other => return Err(/* TypeMismatch: name must be a String */),
    };
    Ok(crate::rust_deps::marshal::make_rust_opaque(
        crate::kernel::spawn::SOCKET_ADDRESS_TYPE_PATH, name))
}
```
Dispatch arm beside `socket-pair'`: `":wat::kernel::socket-address'" => eval_socket_address_prime(args, list_span, env, sym),`.
Check (`infer_socket_address_prime`, beside `infer_socket_pair_prime`): `args[0]` must be `String`
(infer it, unify/expect `:wat::core::String`); `args[1],args[2]` via `parse_peer_pair_type_arg` →
return `TypeExpr::Parametric { head: "wat::kernel::SocketAddress'".into(), args: vec![s, r] }`. Add
the infer-dispatch entry beside `socket-pair'`.

**(B) `listener'` (process) binds a given addr — `eval_listener_prime` + `infer_listener_prime`.**
- `eval_listener_prime`: keep evaluating `args[0]` (host) for the tier dispatch. ThreadOpts → the
  existing 3-arg `(host :S :R)` mint path, UNCHANGED. ProcessOpts → now expect **2 args** `(host
  addr)`: eval `args[1]` → `SocketAddress'` opaque → `downcast_ref_opaque::<String>` the name →
  `UnixListener::bind_addr(from_abstract_name(name))` + `set_nonblocking(true)` (C0b.3a-i invariant) →
  return `make_rust_opaque(SOCKET_LISTENER_TYPE_PATH, listener)` (JUST the listener — no tuple, no
  SocketAddress' return). DELETE the `LISTENER_SEQ` mint + the auto-name + the SocketAddress' return.
  Per-tier arity: validate 3 args when thread, 2 when process (check host first, then arity).
- `infer_listener_prime`: ThreadOpts arm → `listener_tuple(s,r)` (unchanged). ProcessOpts arm → expect
  `args[1]` reduces to `SocketAddress'<S,R>`; return `SocketListener'<S,R>` (a single Parametric, NOT
  the tuple). Retire `socket_listener_tuple` if now unused (grep first). Per-tier arity in the arity
  guard.

**(C) `connect'` / `accept'` — UNCHANGED.** Confirm `connect'`'s `SocketAddress'` arm still consumes
the opaque `socket-address'` produces (same `SOCKET_ADDRESS_TYPE_PATH`, payload `String`).

**(D) Update the c0b2c probe** (`tests/nursery/probe_arc209_c0b2c_process_connection.rs`) to the named
form — same-process, but via `socket-address'` + `listener'(process addr)`:
```
[addr     (:wat::kernel::socket-address' "wat.arc209.c0b2c.svc" :wat::core::i64 :wat::core::i64)
 listener (:wat::kernel::listener' (:wat::spawn::process) addr)
 client   (:wat::kernel::connect' addr)
 server   (:wat::kernel::accept' listener)
 _ (:wat::kernel::send' client 5) got (:wat::kernel::recv' server)
 _ (:wat::kernel::send' server (:wat::core::+ got 10)) reply (:wat::kernel::recv' client)]
 reply]   ;; still 15
```
Update the module-doc header to note the mint form was superseded by `socket-address'`.

## Blast radius (bounded)

`src/runtime.rs` (`socket-address'` + `eval_listener_prime` process arm) + `src/check.rs`
(`infer_socket_address_prime` + `infer_listener_prime` process arm) + the two dispatch sites + the
c0b2c probe + the new c0b2d probe (already committed). `connect'`/`accept'` UNCHANGED. Thread-tier
`listener'` UNCHANGED. NO `select'` work. NO `SO_PEERCRED`. Keep `SOCKET_ADDRESS_TYPE_PATH` (reused).

## STOP triggers (rejection criteria — ship nothing, report the gap)

1. **STOP-1:** if the `String` value accessor in `eval_socket_address_prime` isn't `Value::String` as
   sketched — STOP, report the actual variant; do not guess.
2. **STOP-2:** if making `eval_listener_prime` per-tier-arity (3 thread / 2 process) requires a
   structural change beyond an arity branch after the host dispatch — STOP, report.
3. **STOP-3:** if `connect'`'s `SocketAddress'` arm does NOT consume what `socket-address'` produces
   (opaque mismatch) — STOP, report; do not add a parallel path.

## The gate

`cargo build --release` clean, then the cross-process gate GREEN:
```
cargo test --release -p wat --test probe_arc209_c0b2d_named_cross_process
```
must PASS (5→105 across the boundary). Then no regression:
```
cargo test --release -p wat --test nursery probe_arc209_c0b2c -- --test-threads=1     # updated to named, GREEN
cargo test --release -p wat --test nursery probe_arc209_c0b2b_socket_peer probe_arc209_c0b1b_select_listener -- --test-threads=1
cargo test --release -p wat --test probe_arc209_c0b3a0_self_peer
cargo test --release -p wat --test nursery -- --test-threads=1                         # 895 passed / 4 failed (known baseline)
cargo test --release --workspace --no-run                                              # full surface compiles
```
Report the exact `test result:` line for each. Do NOT commit — the Inquisitor weighs and commits.
