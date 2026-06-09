# BRIEF — Stone 214 `1b-ii-α`: the Err channel becomes the 3rd io_uring arm

> **The kill:** fold the child's Err channel (fd 2) into the io_uring receiver as a
> 3rd `POLL_ADD` arm, so a crashed `:process` child's reason arrives through `recv`
> itself and `recv'` auto-raises it. This is the dogfood of the autoscaling TCO
> loop: 3 fds (`in`/`Ok`/`Err`) in a cap-4 ring. Slow is smooth; strike once, green.

## The work (one paragraph)

Today the `:process` peer's Err channel is a **separate plain `libc::pipe`** (the
child `dup2`s its fd 2 onto the write end; the parent drains the read end with
`ProcessPeerBundle::take_crash_reason`, a blocking `libc::read`). Make it the **3rd
`comms::process` channel** — symmetric with `in` and `Ok` — so the parent reads it
through the same cascade-aware io_uring loop. `ProcessPeerBundle` gains a
`recv()` that runs `comms::process::Select<String>` over **`[peer.output (Ok, idx 0),
err (Err, idx 1)]`**; the **arm index is the Ok/Err discriminant**. `recv'`
(runtime.rs) calls `bundle.recv()` and **auto-raises** the crash reason when the Err
arm fires (closing Q1 — the substrate raises on the user's behalf). The manual diag
pipe + `take_crash_reason` are **retired**; the 2 companion tests that used them
migrate onto the `recv'`-auto-raise behavior.

## Read in order (the rooms)

1. `src/comms/process.rs:809–960` — `Select<'a, T>` (`new` / `recv(&mut, &Receiver)`
   / `select() -> SelectOutcome::Recv{index, result}`). **Reused as-is — build
   nothing here.** Note `needed_capacity = arm_count.next_power_of_two().max(2)`:
   2 data arms + broadcast = 3 → cap-4 ring (the dogfood box). The Receiver
   `recv()`/2-arm loop is at `:308–358` for reference.
2. `src/kernel/spawn.rs:139–204` — `ProcessPeerBundle` (the `err_channel_r: OwnedFd`
   field + `take_crash_reason`). **Replace** `err_channel_r: OwnedFd` with
   `err: crate::comms::process::Receiver<String>`; **add** `ProcessPeerBundle::recv`
   (the Select); **delete** `take_crash_reason`.
3. `src/kernel/spawn.rs:448–501` — the channel setup: `in` + `Ok` pairs (`448–469`)
   and the **manual diag `libc::pipe`** (`471–501`). **Replace** the manual pipe with
   a 3rd `comms::process::pair::<String>()` → `(err_tx, err_rx)`.
4. `src/kernel/spawn.rs:523–555` — the child branch: the `dup2` block (`538–542`) and
   the `preserved` fd list (`550–551`). **Add** `dup2(err_tx.raw_fds()[0], 2)` and
   extend `preserved` with `err_tx.raw_fds()` so the close-sweep keeps them.
5. `src/kernel/spawn.rs:636–660` — the parent branch / bundle build. **Set**
   `bundle.err = err_rx`; drop the `diag_w` close / `diag_r` handling (now owned by
   the pair). `peer.output` is `pub(crate)` — `ProcessPeerBundle::recv` (same crate)
   borrows it for the Select.
6. `src/kernel/peer.rs:150–203` — `Process<I,O>` (`input`/`output`/`pidfd`). **DO NOT
   add an `err` field here** — `Select<T>` is homogeneous and `output: Receiver<O>`
   would not unify with `Receiver<String>` for generic `O`. The Err-Select lives on
   the **bundle** (always `Process<String,String>`), keeping `Process<I,O>` generic
   and untouched. `Process::recv` (`:192`) stays as-is for the test-only peers.
7. `src/runtime.rs` `eval_peer_recv_prime` `:process` arm (~`22614–22700`) — change
   `bundle.peer.recv()` → `bundle.recv()`; on the new `Crashed(reason)` map to a
   `RuntimeError { kind: MalformedForm { head: ":wat::kernel::recv'", reason } }`
   that **carries the crash reason text**; on `Disconnected` keep today's generic.
8. `tests/kernel/spawn_program_prime_process.rs:78–104, 322–499` — the `peer_recv`
   helper + the 2 `*_emits_diagnostic` tests. **Migrate** them off
   `peer_crash_reason`/`take_crash_reason`: assert the crash reason surfaces through
   `peer_recv`/`bundle.recv()` now. Delete the `peer_crash_reason` helper.
9. `tests/kernel/probe_arc214_alpha_crash_autoraise.rs` — **the load-bearing test.**
   Must flip RED→GREEN. Do not edit it.

## Implementation sketch (fill the skeleton; do not invent the shape)

```rust
// src/kernel/spawn.rs — the bundle
pub enum PeerRecvError { Disconnected, Crashed(String) }   // or in peer.rs; your call

pub struct ProcessPeerBundle {
    pub peer: Process<String, String>,
    pub(crate) err: crate::comms::process::Receiver<String>,  // was err_channel_r: OwnedFd
    pub _lifeline_w: OwnedFd,                                  // declaration order invariant HOLDS
}

impl ProcessPeerBundle {
    /// Result<T,E> wire: Select the Ok arm + the Err arm; index discriminates.
    pub fn recv(&self) -> Result<String, PeerRecvError> {
        use crate::comms::process::{Select, SelectOutcome};
        let mut sel = Select::new();
        let ok = sel.recv(&self.peer.output);   // idx 0
        let er = sel.recv(&self.err);           // idx 1
        match sel.select() {
            Ok(SelectOutcome::Recv { index, result }) if index == ok =>
                result.map_err(|_| PeerRecvError::Disconnected),
            Ok(SelectOutcome::Recv { index, result }) if index == er =>
                Err(PeerRecvError::Crashed(result.unwrap_or_default())),  // the #wat.kernel/ProcessPanics envelope
            _ => Err(PeerRecvError::Disconnected),   // shutdown / select err → honest disconnect
        }
    }
}
```
```rust
// src/kernel/spawn.rs — the 3rd channel (mirror the in/Ok pairs at :448–469)
let (err_tx, err_rx) = crate::comms::process::pair::<String>().map_err(/* … */)?;
// child branch: unsafe { libc::dup2(err_tx.raw_fds()[0], 2); }  preserved.extend(err_tx.raw_fds());
// parent: bundle { peer, err: err_rx, _lifeline_w }
```

## Blast radius (bounded)

`src/kernel/spawn.rs` (bundle + the 3rd pair + dup2 + parent build), `src/kernel/peer.rs`
(only IF you home `PeerRecvError` there — no field change), `src/runtime.rs`
(`eval_peer_recv_prime` `:process` arm only), `tests/kernel/spawn_program_prime_process.rs`
(2 tests + helpers). **No change to `comms/process.rs`** (Select reused), **no change to
`Process<I,O>`'s fields**, no new io_uring code.

## STOP triggers (halt + surface; do not improvise)

- **STOP-1:** if `emit_structured_exit` does NOT write the Err envelope as
  **newline-framed EDN** (what `comms::process::Receiver` decodes), STOP and surface
  it — the Err Receiver must decode the same frame format `println` writes. (Check
  `src/process/stdio.rs` `emit_structured_exit` + `src/process/mod.rs:66–69`.) Do not
  hack a parser around a mismatch.
- **STOP-2:** if `comms::process::pair`'s `Sender` write-fd cannot be `dup2`'d onto
  fd 2 + survive the close-sweep the way `output_tx` does onto fd 1 (`:540`), STOP —
  the symmetry with the Ok channel is the whole design; surface the asymmetry.
- **STOP-3:** if `Select` cannot borrow `&self.peer.output` (visibility / lifetime),
  STOP — do not duplicate the Receiver or reach through a clone.

## Prior comparable (copy for shape)

- `tests/kernel/spawn_program_prime_process.rs` (the whole file) — the `:process`
  spawn + send/recv harness; the 2 `*_emits_diagnostic` tests are exactly what you
  migrate. `src/kernel/spawn.rs:448–469` — the `in`/`Ok` `pair()` pattern the Err
  channel mirrors. The Receiver 2-arm loop (`comms/process.rs:308–358`) shows what
  the 3rd arm joins.

---

# EXPECTATIONS — independent scorecard (fixed before the strike)

| what | command | expected |
|---|---|---|
| the α probe flips GREEN | `setsid timeout 300 cargo test --release --test kernel probe_arc214_alpha_crash_autoraise -- --ignored --test-threads=1` | 1 passed (recv' raises `DivisionByZero`) |
| no kernel regression | `setsid timeout 600 cargo test --release --test kernel -- --ignored --test-threads=1` | all pass (9 incl. the probe; the 2 migrated diagnostic tests green via the new path) |
| lib builds clean | `cargo build --release` | Finished, no new errors |
| io_uring stays the only io-select | `grep -rn 'libc::poll' src/comms/process.rs` | unchanged (poll dies in ε, not here) — count not increased |
| no plain Err pipe left | `grep -rn 'take_crash_reason\|err_channel_r' src/` | 0 (retired) |

**Runtime prediction:** 25–45 min (channel symmetry is mechanical; the recv' error-mapping
+ 2 test migrations are the thought). **Trap-doors:** (1) STOP-1 frame-format mismatch is
the real risk — if the Err envelope isn't newline-EDN the Err Receiver silently never
decodes; the probe catches it (stays RED). (2) The bundle's field-drop order invariant
(`peer` before `_lifeline_w`) must survive the `err` field insertion — place `err`
between them or keep `peer` first / `_lifeline_w` last.

**The kill (orchestrator, after the strike):** re-run the probe + the full kernel suite
*myself*; read the diff myself; then the 25× soak (race disconfirmation on the io_uring
core) before commit. Commit on green only.
