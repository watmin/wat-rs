# BRIEF — Stone 259.S3.5a-0 — thread-peer crash-reason IPC fix (program-compliant)

## The flaw (grounded)

The unified `Peer` contract is violated on the thread transport. A PROCESS peer delivers its
crash reason over the pipe: `spawn_process_peer` allocates a 3rd `comms::process` Err channel,
the child dup2's `err_tx`→fd 2, and on a crash writes the reason there; `ProcessPeerBundle::recv`
(`spawn.rs:225-233`) reads the Err channel on Ok-EOF → `Crashed(reason)`, which `recv'`
(`runtime.rs:22419-22441`) surfaces. The THREAD peer does NOT: `spawn_thread_peer`
(`spawn.rs:455-458`) runs the body in `catch_unwind` and **discards** the result (`let _ =`);
the `Thread` struct (`peer.rs:66`) has no crash channel; `Thread::recv` (`peer.rs:96`) is just
`self.output.recv()`; and `recv'` (`runtime.rs:22382`) maps the disconnect with `|_|` to a
generic "peer closed / thread exited". **The failure message is silently lost on one transport.**

Proven by `tests/nursery/probe_arc259_thread_crash_reason` (RED): a thread peer crashing with
`BOOM-SENTINEL-9173` → `recv'` raises with the generic message, NOT the sentinel.

## The fix — mirror the process Err channel on the crossbeam tier

### 1. `src/kernel/peer.rs` — `Thread<I,O>` gets a crash channel + crash-aware `recv`

- Add a field: `pub(crate) crash: crate::comms::thread::Receiver<String>` (the crossbeam analog
  of `ProcessPeerBundle::err`).
- Change `Thread::recv` to mirror `ProcessPeerBundle::recv` (return `Result<O,
  crate::kernel::spawn::PeerRecvError>`):
  ```rust
  pub fn recv(&self) -> Result<O, crate::kernel::spawn::PeerRecvError> {
      use crate::kernel::spawn::PeerRecvError;
      match self.output.recv() {
          Ok(v) => Ok(v),
          Err(_) => match self.crash.recv() {       // blocks until crash_tx sends OR drops
              Ok(reason) => Err(PeerRecvError::Crashed(reason)),
              Err(_)     => Err(PeerRecvError::Disconnected),
          },
      }
  }
  ```
  (`drain_and_join` does NOT call `recv` — it drops input + joins — so it is unaffected.)

### 2. `src/kernel/spawn.rs` `spawn_thread_peer` — create the channel + send the reason on panic

- Create it: `let (crash_tx, crash_rx) = crate::comms::thread::pair::<String>();` — `crash_tx`
  moves into the worker closure (Sender<String> is Send), `crash_rx` stays for the parent Thread.
- Replace the discard at `spawn.rs:455-458`:
  ```rust
  let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
      apply_function(program_fn.clone(), vec![self_peer], &thread_sym, span.clone())
  }));
  // self_peer (output_tx) dropped here → the output channel EOFs.
  if let Err(payload) = outcome {
      // GENUINE panic (assert-failed! / Rust panic). Surface the reason over the crash
      // channel — the crossbeam analog of the process Err channel.
      let (message, _assertion) = crate::runtime::extract_panic_payload(payload);
      let _ = crash_tx.send(message);
  }
  // crash_tx dropped here → crash channel EOFs (reason buffered if it was sent).
  ```
- `Thread` construction (`spawn.rs:471`): add `crash: crash_rx`.

### 3. `src/runtime.rs` `eval_peer_recv_prime` (`22382`) — surface `Crashed` (mirror the process arm)

```rust
Some(peer) => peer.recv().map_err(|e| {
    use crate::kernel::spawn::PeerRecvError;
    let reason = match e {
        PeerRecvError::Crashed(crash_reason) => crash_reason,
        PeerRecvError::Disconnected => "recv failed: peer closed / thread exited".into(),
    };
    RuntimeError { span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm { head: OP.into(), reason } }.into()
}),
```

## STOP triggers (REJECTION — ship nothing, surface the issue)

- **STOP-1 (load-bearing correctness):** ONLY `Err(payload)` (a genuine panic) sends a crash
  reason. A clean drain — `Ok(Err(eval_break))`, the worker's own `recv'` raising because the
  parent dropped input during `drain_and_join` — is the NORMAL shutdown and MUST NOT send a
  reason (else every RAII reap / bracket runner drain reports as a crash). Do NOT match on
  `Ok(Err(..))`; only `Err(payload)`.
- **STOP-2:** do NOT touch `select'` (`runtime.rs:22727+`) — it reads the output receivers
  directly; surfacing crash reasons there is a separate follow (the cascade-abort message).
- **STOP-3:** do NOT change the process path; mirror it. If `crate::runtime::extract_panic_payload`
  isn't reachable as `pub(crate)`, surface it (don't reimplement the payload extraction).

## Watch-point
- Reason shape: sending the raw `message` (the assertion text) satisfies the contract + the probe.
  If the process `#wat.kernel/ProcessPanics` envelope helper is trivially reusable for full
  shape-parity, use it; else the raw message is acceptable — note which you used.

## Gate (run each, READ output, report REAL results — never chain a commit)
1. `cargo test --release -p wat --test nursery probe_arc259_thread_crash_reason -- --test-threads=1` → GREEN.
2. `cargo test --release -p wat --test nursery probe_arc259_brackets -- --test-threads=1` → still GREEN (drain/cascade intact).
3. `cargo test --release -p wat --test nursery probe_arc259_s2d_raii_hinge probe_arc259_bracket_runner -- --test-threads=1` → still GREEN (RAII drain-before-join intact).
4. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the known pre-existing reds (arc-255 reflection ×2, undefined-builtin ×2), zero new.
5. `cargo build --release` clean.

## Report back
- The diff (files + the four edits). Which reason-shape you used. The verbatim final line of each
  gate command. Any STOP hit. Do NOT commit.
