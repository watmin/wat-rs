# BRIEF — Stone 259.S2a: the ThreadProg self-peer model (unified `Peer'`)

**The work, in one paragraph.** Add a pipes-only `Peer<S,R>` peer type and make the
`:thread` tier of `spawn-program'` hand the program its own `Peer'` self-peer ONCE
(replacing the platform apply-loop), so a thread program `[self <- :wat::kernel::Peer'<S,R>] -> nil`
drives its channel with `send'`/`recv'` directly. Teach `send'`/`recv'` (eval + check)
the new `Peer'` head with the SAME uniform projection as `Thread'`. The committed
disconfirming probe flips from CHECK-RED to GREEN.

**Read in order (the rooms):**
1. `tests/nursery/probe_arc259_s2a_thread_self_peer.rs` — the GREEN target. Its prog
   `(fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :nil (send' self (recv' self)))`
   echoes through its self-peer; the parent drives via `send'`/`recv'`/`close'` on the
   returned `Thread'`. Make it pass.
2. `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S2a.md` — the pinned contract
   (the uniform projection table, the owner-thread invariant, the five touches).
3. `src/kernel/peer.rs:59-110` — `Thread<I,O>` struct + impl. `Peer<S,R>` is its
   mirror **minus the `JoinHandle`** (pipes only): fields `tx: comms::thread::Sender<S>`,
   `rx: comms::thread::Receiver<R>`; methods `send(&self, S)->Result<(),SendError<S>>`
   (→ `tx.send`) and `recv(&self)->Result<R,RecvError>` (→ `rx.recv`). No `close`/`join`.
4. `src/kernel/spawn.rs:98-119` — `ThreadPeerCell` alias + `THREAD_PEER_TYPE_PATH` (:115).
   Add `PEER_TYPE_PATH = ":wat::kernel::Peer'"` + a `PeerCell = Arc<ThreadOwnedCell<Option<Peer<Value,Value>>>>` alias.
5. `src/kernel/spawn.rs:335-412` — `spawn_thread_peer`: the apply-loop closure (357-390).
   Rewrite it (see sketch). Parent side (the `Thread'` peer it returns) is UNCHANGED.
6. `src/runtime.rs:22074` (`eval_peer_send_prime`, `Thread'` arm 22091-22127) +
   `src/runtime.rs:22187` (`eval_peer_recv_prime`, `Thread'` arm 22248-22285) — add a
   third arm for `PEER_TYPE_PATH`, mirroring the `Thread'` arm exactly (Value
   pass-through; `send'`→`tx.send`, `recv'`→`rx.recv`; same `Option`/`with_ref`/
   use-after-close shape).
7. `src/check.rs:9963` — the `send'`/`recv'` peer-head match guard
   `if (head == "wat::kernel::Thread'" || head == "wat::kernel::Process'")`. Add
   `|| head == "wat::kernel::Peer'"`. The uniform projection (send'→arg0, recv'→arg1)
   then applies to `Peer'` identically — no other check change.

**Implementation sketch (spawn_thread_peer closure — the heart):**
```rust
// input_tx/input_rx = parent→worker ; output_tx/output_rx = worker→parent (as today)
let thread_sym = sym.clone();
let span = list_span.clone();
let join_handle = std::thread::Builder::new()
    .name(format!("wat-thread-peer::{}", fn_name))
    .spawn(move || {
        // OWNER-THREAD INVARIANT: build the Peer opaque INSIDE this closure, so the
        // ThreadOwnedCell's owner-thread == this spawned thread (where the prog runs).
        // Worker is Peer'<O,I>: tx = output_tx (worker→parent), rx = input_rx (parent→worker).
        let self_peer = crate::rust_deps::marshal::make_rust_opaque(
            PEER_TYPE_PATH,
            std::sync::Arc::new(crate::rust_deps::custodia::ThreadOwnedCell::new(Some(
                Peer { tx: output_tx, rx: input_rx },
            ))),
        );
        // Hand it over ONCE — no apply-loop. The prog owns its body; result (nil) ignored.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply_function(program_fn.clone(), vec![self_peer], &thread_sym, span.clone())
        }));
    })
    .map_err(/* same MalformedForm as today */)?;
// Parent peer UNCHANGED: Thread { input: input_tx, output: output_rx, join: join_handle } → Thread' opaque
```

**Blast radius:** `src/kernel/peer.rs`, `src/kernel/spawn.rs`, `src/runtime.rs` (the two
verb fns only), `src/check.rs` (the one head-match guard). No new wat files; no parser
changes; the `:process` tier untouched; the parent `Thread'` path untouched.

**STOP triggers (halt + report; do not work around):**
- **STOP-1 (owner-thread):** the `Peer'` opaque MUST be constructed *inside* the spawned
  closure. If a path constructs it on the parent thread, the `ThreadOwnedCell` owner-guard
  panics at runtime — keep construction inside the closure.
- **STOP-2 (projection):** if `Peer'` needs a *different* check-side projection than the
  uniform `Thread'` one (send'→arg0 / recv'→arg1), STOP and report — the DESIGN says it is
  identical; a divergence means the contract is wrong, not the code.
- **STOP-3:** if making the probe green requires touching the `:process` tier, `close'`,
  `select'`, or the parent `Thread'` path, STOP and report — S2a's scope is the `:thread`
  worker self-peer only.

**Done = green:**
- `cargo test --release -p wat --test nursery probe_arc259_s2a` → the probe passes (42).
- `cargo build --release` clean.
- `cargo test --release -p wat --test nursery probe_arc214_stone46aii_peer_verbs` still
  green (the `Thread'`/`Process'` arms unregressed).

**Mirror for shape:** the `Thread'` arms in `eval_peer_send_prime`/`eval_peer_recv_prime`
are the exact pattern the `Peer'` arms copy. The `spawn_thread_peer` test at
`src/kernel/spawn.rs:587` shows the opaque/downcast/`with_ref` idiom.
