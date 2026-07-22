# BRIEF — Stone 1 (foundation): relocate the timer to the CORRECT location (a unified `Peer'`)

> Substrate strike. Reference: `DESIGN-self-scheduling-defservices.md` § "✅ STATUS (2026-07-21e)".
> This is the FOUNDATION only — the settled design *above the multiplexer* (Outcome grow, leading-dash,
> selectables, superset) is Stone 2, a SEPARATE strike that rides on this. Do NOT build Stone 2 here.

## The work (one paragraph)

`(:wat::kernel::after peer-kind duration msg)` today builds a **tier-specific** `Thread'`/`Process'`
timer typed `Timer'<O>` — the WRONG location. The serve loop's `poll'` runs on the **unified `Peer'`**
(`PEER_TYPE_PATH`) and rejects anything else, so a timer cannot join it. Fix the location: make `after`
construct the timer as a **unified `Peer'<nil,O>`** (a real peer whose recv fires the `msg` once, then
EOFs), both tiers, so it drops into `poll'` **and** `select'` by construction. Then retire the now-
vestigial tier-open `Timer'` type + its fusion machinery. This is behavior-preserving for the 7 existing
`after` tests (they `select'` over timers; `select'` already accepts unified `Peer'`), and it unblocks a
timer joining a service's `clients`/`selectables` set — the capability the whole self-scheduling stone
needs.

## Read in order (the rooms — grounded 2026-07-21e)

1. `src/runtime.rs:27128-27275` — `eval_kernel_after`. The thread branch (`:27221-27248`) builds
   `kernel::peer::Thread{output: timer_rx,…}` → `THREAD_PEER_TYPE_PATH`; the process branch
   (`:27249-27274`) builds `ProcessSelectable::Timer(rx)` → `PROCESS_PEER_TYPE_PATH`. **This is what you
   rewrite** to build a unified `PEER_TYPE_PATH` peer instead.
2. `src/kernel/peer.rs:206-255` — the unified `Peer{tx,rx}` + its constructors `Peer::from_thread(tx:
   thread::Sender<Value>, rx: thread::Receiver<Value>)` (`:234`) and `Peer::from_socket(tx:
   process::Sender<String>, rx: process::Receiver<Value>)` (`:250`). **The target constructors.**
3. `src/comms/thread.rs:444` — `timer(dur, msg) -> Receiver<Value>` (maps DIRECTLY into `from_thread`'s
   `rx`). `src/comms/process.rs:1263` — `timer(dur, frame) -> io::Result<Receiver<String>>` (**MISMATCH**:
   `from_socket` wants `Receiver<Value>` — see STOP-2 / the adapter room #4).
4. `src/kernel/listener.rs:478-485` — `accept_as_value`: how an accepted socket connection becomes a
   `PEER_TYPE_PATH` peer. **Its `Peer::from_socket` call site shows how a socket `Receiver<Value>` (the
   decode-wrapped frame receiver) is obtained** — reuse that exact decode wrapper to adapt the process
   timer's `Receiver<String>` into the `Receiver<Value>` `from_socket` needs. (Grep the `from_socket`
   call sites + the `CommReceiver<Value>` decode wrapper for sockets.)
5. `src/check.rs:11696-11785` — `eval_kernel_after` checker; it returns `Timer'<O>` at `:11731`/`:11779`.
   **Change to `Peer'<nil, O>`** (head `wat::kernel::Peer'`, args `[:wat::core::nil, O]`).
6. `src/check.rs:12166-12175` (select' `Timer'` element arm), `:15467-15487` (the tier-open `Timer'`
   fusion), `:15531-15537` (`is_peer_tier_head`) — the arc-292 `Timer'` machinery that becomes
   VESTIGIAL once `after` returns `Peer'`. Retire it (see blast radius).
7. `wat-tests/timer-*.wat` (5) + `wat-tests/timer-family.wat` + `tests/services/probe_arc278_self_scheduling.wat`
   — the 7 `after` users (grep `kernel::after`). Migrate any `Timer'` type annotations to `Peer'<nil,_>`;
   a **wat-fix codemod** (`wat-scripts/fixes/`) handles a corpus form-change if the annotations are
   structural — do NOT hand-edit if there are several.

## Implementation sketch

- **Thread:** `Peer::from_thread(dead_tx, comms::thread::timer(std_dur, msg))` where `dead_tx` is a
  `thread::pair()` sender whose receiver is dropped (a timer has no input; the tx is never used). Wrap
  `make_rust_opaque(PEER_TYPE_PATH, Arc::new(ThreadOwnedCell::new(Some(peer))))`.
- **Process:** obtain a socket `dead_tx` the same shape `accept_as_value`'s peers use, and wrap the
  `process::timer` `Receiver<String>` in the socket decode wrapper → `Receiver<Value>`, then
  `Peer::from_socket(dead_tx, wrapped_rx)` → `make_rust_opaque(PEER_TYPE_PATH, …)`.
- **Checker:** `after` result type `Peer'<:wat::core::nil, O>`.
- **Retire `Timer'`:** delete the `Timer'` element arm in `infer_select_prime` + the fusion arms +
  `is_peer_tier_head` (or leave `is_peer_tier_head` if still referenced — grep first). Grep `Timer'`
  zero in `src/` when done (modulo comments/history).

## Blast radius

`src/runtime.rs` (eval_kernel_after), `src/check.rs` (after typing + Timer' retirement), possibly a small
adapter in `src/comms/process.rs` or `src/kernel/peer.rs` for the String→Value timer-frame decode, and 7
`wat-tests/`/`tests/` `.wat` files (via wat-fix if structural). NO change to `poll'` (`eval_poll_prime` /
`infer_poll_prime`), the serve loop, or `select'`'s Peer' handling.

## STOP triggers (halt + surface; do NOT improvise)

1. **STOP-1** — if a unified `Peer'` is NOT constructible from a one-shot timer receiver on a tier (the
   constructor won't take it, ownership/`Send` won't hold), STOP. Do not fake it with a Value-erasure or
   a `poll'` bridge. Report the exact blocker.
2. **STOP-2 (the known risk)** — the **process** timer yields `Receiver<String>` but `from_socket` wants
   `Receiver<Value>`. If you cannot find/reuse the socket decode wrapper (room #4) to adapt it cleanly,
   STOP and report — do not invent a parallel decode path.
3. **STOP-3** — if retiring `Timer'` cascades beyond the rooms above (a live non-timer consumer of the
   fusion), STOP and report; leaving `Timer'` as an alias is acceptable, silently keeping dead code is not.

## Done criteria (RED → GREEN, weighed by the ORCHESTRATOR's own re-run)

- **RED now → GREEN:** a disconfirming probe `wat-scripts/scratch-pad/probe-timer-as-peer.wat` — build a
  timer via `after`, drop it into a **real `poll'` set** (a minimal self-peer + listener + one real peer
  + the timer; copy the scaffold from an existing service/poll' test) and assert the timer's `msg` is
  delivered as a peer message, **BOTH tiers** (thread + process). This walks the REAL `poll'`/`Peer'`
  path — NOT `select'` (that adjacent path is the lesson that bit last run). Commit it.
- The **7 existing `after` tests stay green** (`select'` over `Peer'`-timers).
- `Timer'` grep-zero in `src/` (or an explained alias).
- **Floor:** `cargo nextest run --release` → 0 NEW failures (read the Summary line; the known
  `wat-cli sigterm…` flake is not a regression — confirm it passes isolated).
