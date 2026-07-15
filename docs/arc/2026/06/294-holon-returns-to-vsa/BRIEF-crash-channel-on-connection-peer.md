# BRIEF — the "next slice": a crash channel on the CONNECTION peer, so a service's crash reason reaches its connect'd client

## Why (grounded — the builder's priority: "we absolutely need crash strings to propagate")
When a thread SERVICE crashes (e.g. mem-store' hitting `unknown function: :wat::query::mem-store'::Record` at
`wat/query/mem.wat:114`), its connect'd client's `recv'` reports only `#wat.runtime/MalformedForm: recv' failed:
peer closed / channel disconnected` — the REAL crash reason is captured but never propagates. That is a silent-
failure at the process/IPC boundary — the exact class this session has been extirpating, now at the wire.

**Grounded mechanism (do NOT re-investigate — it's confirmed in the code):**
- A **directly-spawned worker peer** (`spawn_thread_peer`, `src/kernel/spawn.rs:564`) HAS a crash channel
  (`crash_tx`/`crash_rx`): on a `catch_unwind` panic the worker sends the reason on `crash_tx` before it drops
  (`spawn.rs:16-20`). `recv'` (`eval_peer_recv_prime`, `runtime.rs:25797` — the `PeerRecvError::Crashed(reason)` arm)
  and `select'` (`runtime.rs:26347`, `classify_peer_death`) ALREADY surface `Crashed(reason)`. These WORK.
- A **connect'd client** holds a **connection peer with NO crash channel BY CONSTRUCTION** —
  `runtime.rs:26623` *"Bare Peer' has no crash channel (it is a connection peer, not a spawned worker)"*;
  `runtime.rs:20441` same. So a service crash reaches the client only as EOF → `Disconnected` → the reason is lost.
- This is exactly the deferred slice named at `spawn.rs:181-185`: *"no crash channel on `Peer`; adding one is the
  next slice."* We are completing it.

## The fix
Give the CONNECTION peer a live crash channel wired to the far-side service's crash reason, so a service crash
surfaces to the client's `recv'`/`select'` as `Crashed(reason)` (both already handle it — no change to their crash
arms). `recv'` on the connection peer must, on data-channel `Disconnected`, read this crash channel via the EXISTING
`classify_peer_death` (`spawn.rs:204`) — mirroring what `select'` already does at `runtime.rs:26347-26348` and what
`ProcessPeerBundle::recv` does for processes (`spawn.rs:322`, `classify_peer_error`).

The load-bearing wiring: a thread service's serve-loop crash (its `crash_tx` reason on death) must reach the
connect'd client's connection-peer crash channel. The connect'/serve handshake is where this is wired — the client's
connection peer's crash_rx must be fed by a crash_tx the serve-loop (or its death path) sends on before the
connection's data Sender drops.

## PROBE FIRST — MANDATORY (the reason this slice was deferred; the (C) flail's lesson)
Before ANY build, write a disconfirming probe at the ACTUAL failure shape and run it:
- A minimal thread SERVICE (a `defservice` or a `spawn_thread_peer` serve-loop) whose op body deliberately CRASHES
  (panics / calls an unknown fn) while a client is connect'd; the client `send'`s a request then `recv'`s.
- Prove the client's `recv'` can receive the SERVICE's crash REASON (not `Disconnected`) via a crash channel wired
  through `connect'`, and that the send-before-drop ORDERING holds (the reason is sent before the connection's data
  Sender drops — the same guarantee the process `err` channel relies on).
- **STOP + report if:** the reason can't be fanned from the single serve-loop crash to the connect'd client(s), or
  the send races the data-Sender drop, or the connect' handshake has no seam to carry a crash channel. That is why
  the authors deferred this slice — if it's genuinely racy/hard, surface the exact obstacle; do NOT hack a partial.
- Confirm the CURRENT behavior first (the client gets `Disconnected` today) so the probe fails on EXACTLY the gap.

## The build (only after the probe is green)
1. **Connection peer gains a crash channel.** The `kernel::peer` connection-peer type (the bare `Peer'` from
   `connect'`) gets a `crash: Receiver<String>` field like the worker `Thread` (`spawn.rs:684-690`).
2. **Wire it at connect'/serve.** `eval_connect_prime` (`runtime.rs:20678`) / `connect_as_value` /
   `wrap_connect_request` (`runtime.rs:20730`) mint the client peer with a crash_rx fed by a crash_tx the SERVICE's
   serve-loop sends its death reason on (fan to all connected clients if the serve-loop dies). Mirror the worker
   crash channel's send-before-drop discipline.
3. **`recv'` reads it.** In `eval_peer_recv_prime` (`runtime.rs:25797`), the connection-peer `Disconnected` branch
   consults the connection peer's crash channel via `classify_peer_death` → `Crashed(reason)` → surface the real
   reason (mirror the worker-peer arm already there + `select'`'s `26347` usage). `select'`'s connection-peer arm
   likewise (`runtime.rs:26623-26636`, the "bare connection peer, no crash channel" branches).
4. Process tier: verify parity — a connect'd client of a PROCESS service should already get the reason via the
   process `err` channel (`classify_peer_error`); if not, extend symmetrically.

## STOP triggers / constraints
- DO NOT change the worker-peer crash path (`spawn_thread_peer`) — it works. Only add the CONNECTION-peer crash.
- DO NOT weaken `recv'`/`select'`'s existing `Crashed` handling.
- If the probe shows the wiring is genuinely racy or architecturally blocked, STOP — this is a design escalation to
  the builder, not a hack.

## Gate (orchestrator re-runs ALL)
- `cargo build --release` clean.
- A NEW test: a crashing thread service → the connect'd client's `recv'` surfaces the REAL crash reason (the fn/span),
  NOT `ChannelDisconnected`. (Model on `tests/comms/probe_arc209_structured_peer_death.wat`.)
- The 4 surface-splice tests are UNRELATED here (that's (C)); but this fix will make (C)'s debugging trivial — note
  whether `smem_roundtrip`'s error now shows the real reason (it should surface `unknown function: …` instead of
  "channel disconnected").
- WHOLE FLOOR `cargo nextest run --release`: ZERO NEW failures vs baseline `5e8a6e6f` (~52). Report the set diff.

## Method
Build/test ONCE to a temp file, grep the FILE. Rebuild before running `target/release/wat` (binary staleness bit us).
A mid-edit diagnostic is a PHANTOM. Commit nothing.

## Report back
The PROBE result FIRST (is the wiring feasible? the ordering?), then the diff summary, the new-test result, the
whole-floor set diff, and any STOP hit.
