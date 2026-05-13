# Arc 170 — Shutdown-aware channels (substrate-imposed)

**Date:** 2026-05-13
**Surfaced by:** Stone C leak diagnosis. Empirical: hermetic-forked children orphaned to init, all threads at `futex_do_wait` on crossbeam channels, never woken because parent died without dropping senders.

## The question

> *"How do we close channels on crash?"*

A blocked `recv()` wakes on (a) message arriving, or (b) all senders dropped (Disconnected). Crash isn't a message. Signal handlers can't drop things (not async-signal-safe). Pure-crossbeam has no native shutdown semantics. The substrate currently has SIGTERM → `KERNEL_SIGTERM` atomic → `(:wat::kernel::stopped?)` polling, but polling threads only — blocked recvs never see the flag.

## Four-questions verdict

Pattern enforcement: **impose, not follow.**

The substrate's doctrine is to make violation structurally impossible. Precedents:
- ZERO-MUTEX (substrate provides primitives that prevent Mutex)
- arc 110 silent-kernel-comm illegal (walker)
- arc 117 / 126 / Gap K (structural deadlock detection)
- arc 132 time-limit default

User services don't NEED to be shutdown-aware. They ARE shutdown-aware because the only channels they can create are shutdown-aware.

## The shape

Single substrate site enforces shutdown for all callers:

1. **Global `SHUTDOWN_RX`** initialized at substrate startup. `OnceLock<Arc<Receiver<()>>>`. The corresponding `SHUTDOWN_TX` lives in another `OnceLock`.

2. **Substrate's `recv` primitive** internally multiplexes:
   ```rust
   fn typed_recv(receiver, ...) -> RecvOutcome {
       match receiver {
           ReceiverInner::Crossbeam(rx) => {
               crossbeam::select! {
                   recv(rx) -> msg => msg-based-outcome,
                   recv(SHUTDOWN_RX.get()) -> _ => RecvOutcome::Shutdown,
               }
           }
           ReceiverInner::PipeFd(reader) => { /* same multiplex via os pipe + crossbeam */ }
       }
   }
   ```

3. **SIGTERM/SIGINT handlers** (async-signal-safe path):
   ```rust
   extern "C" fn sigterm_handler(_: c_int) {
       KERNEL_SIGTERM.store(true, SeqCst);
       SHUTDOWN_PIPE_WRITE_FD.store(/* wake the shutdown worker */);
   }
   ```
   A dedicated shutdown-worker thread reads from the pipe; on wake, drops `SHUTDOWN_TX` (lock-free atomic swap). The drop is normal-context (async-signal-safe). All `recv`s waiting on `SHUTDOWN_RX` (cloned from the dropped `_TX`) wake with `Disconnected`.

4. **`RecvError` gains a `Shutdown` variant.** Recv returns `Result<Option<T>, RecvError::Shutdown>` on the multiplex-fires-shutdown branch. Arc 110 Result/expect panics with diagnostic naming shutdown specifically. Distinguishable from `Disconnected` (partner-dropped).

5. **Child fork branch sets `PR_SET_PDEATHSIG(SIGTERM)`** (substrate side, in `spawn_process_child_branch` after `setpgid`). When parent dies for any reason, kernel delivers SIGTERM to child → substrate's handler fires → shutdown cascade → all blocked recvs in child wake → panic chain → child dies cleanly with diagnostic.

## User-side UX implications

**Existing service code, unchanged:**

```scheme
;; A service drives a control + data select loop.
;; The existing arc 110 pattern — recv into match — works as-is.
(:wat::core::define
  (:my::service/loop
    (data-rx :wat::kernel::Receiver<wat::core::String>)
    (control-rx :wat::kernel::Receiver<wat::core::String>)
    -> :wat::core::nil)
  (:wat::core::let
    [maybe (:wat::kernel::recv data-rx)]
    (:wat::core::match maybe -> :wat::core::nil
      ;; The Ok-arm pattern: got a value, do work, recurse.
      ((:wat::core::Ok (:wat::core::Some msg)) ...)
      ;; The Ok-None: data-rx EOF (partner cleanly closed).
      ((:wat::core::Ok :wat::core::None) :wat::core::nil)
      ;; NEW: Err arm carries Shutdown OR Disconnected; both terminal.
      ;; Result/expect-style panic preserves diagnostic.
      ((:wat::core::Err e)
        (:wat::kernel::assertion-failed!
          (:wat::core::string::concat "service shutdown: " (... render e))
          :wat::core::None :wat::core::None)))))
```

**What changed at user level:** **nothing.** The wat code already has to handle `Err` on recv per arc 110. The new `RecvError::Shutdown` variant is just one more reason for `Err`. Existing `Result/expect` panics on it without modification. Existing `match` with explicit `Err _` arm wildcards covers it.

**What changed at substrate level:** the `Rust` impl of recv. ONE site. Adds the select on `SHUTDOWN_RX`. All wat-level recv calls now multiplex transparently.

## Demonstrability

The user asked: "how do we demonstrate correctness?"

Two layers:

**(1) Structurally** — there's no API to create a non-shutdown-aware channel. The substrate's only channel-creation primitive (`:wat::kernel::Channel/new` and friends) returns a Receiver wrapped in `ReceiverInner::Crossbeam` or `ReceiverInner::PipeFd`. Both go through the recv multiplex. Demonstrated by the **absence** of an alternative API.

**(2) Behaviorally** — a deftest that proves shutdown reaches a blocked service:

```scheme
(:wat::test::deftest :wat-tests::shutdown::test-blocked-recv-wakes
  ()
  ;; Spawn a service whose entire job is to recv forever.
  ;; Send SIGTERM mid-block. Service must panic + die within budget.
  (:wat::core::let
    [pair (:wat::kernel::Channel/new<wat::core::String>)
     data-rx (:wat::core::second pair)
     thread
       (:wat::kernel::spawn-thread
         (:wat::core::fn [data-rx <- ...] -> :wat::core::nil
           ;; Blocked here. Process-wide shutdown must wake this.
           (:wat::core::Result/expect -> :wat::core::String
             (:wat::kernel::recv data-rx)
             "blocked recv must wake on shutdown signal")))]
    ;; Trigger SIGTERM on self.
    (:wat::kernel::send-signal :SIGTERM)
    ;; Service thread must panic — join-result returns Err.
    (:wat::core::match (:wat::kernel::Thread/join-result thread)
      ;; ...
      )))
```

A pass means the cascade works end-to-end at the user-visible boundary. The deftest's existing budget assertion catches the regression case (shutdown doesn't reach → service blocks → budget exceeded → test fails).

## Why this satisfies the four questions

1. **Obvious?** ✓ — Drop senders to wake recvs. Recv multiplex on shutdown is the implementation. RecvError::Shutdown names what happened.
2. **Simple?** ✓ — ONE substrate site (recv impl). No user-side changes. Substrate-internal shutdown worker handles async-signal-safety.
3. **Honest?** ✓ — recv honestly says "this channel is over because process is shutting down." Distinguishable from "partner dropped" via separate RecvError variant.
4. **Good UX?** ✓ — User services don't change. Demonstrable structurally (no escape API) AND behaviorally (deftest probes).

## What this does NOT do

- Does not panic the wat program from within the signal handler (async-signal-unsafe).
- Does not block during shutdown — recv wakes immediately; panic chain runs in normal thread context.
- Does not require services to be re-written with select-on-shutdown explicitly — the multiplexing lives BELOW the user-visible `recv` boundary.
- Does not introduce wall-clock timeouts anywhere (still pure lock-step).
- Does not introduce Mutex — `OnceLock`s for the global shutdown signal pair are atomic.

## Implementation order

Suggested slice sequence (independently verifiable):

1. **Slice S1:** mint `SHUTDOWN_TX` / `SHUTDOWN_RX` global lock-free signal infrastructure + shutdown-worker thread + RecvError::Shutdown variant. No callers yet.
2. **Slice S2:** wire `typed_recv` PipeFd path to multiplex on SHUTDOWN_RX. Behavioral probe: send-signal triggers blocked PipeFd recv to wake.
3. **Slice S3:** wire `typed_recv` Crossbeam path. Behavioral probe: same for tier-1 channels.
4. **Slice S4:** set `PR_SET_PDEATHSIG(SIGTERM)` in `spawn_process_child_branch` and the fork-program branch. Behavioral probe: orphan-the-child test.
5. **Slice S5:** demonstrate end-to-end. Probe with deftest + verify zero leaked processes after a stability-100 run.

## Cross-references

- `wat/kernel/services/stdin.wat` — existing service pattern (the model)
- `src/typed_channel.rs::typed_recv` — the site to modify
- `src/runtime.rs:KERNEL_SIGTERM` — existing signal infrastructure
- `src/fork.rs:109+` — existing signal handler installation
- `feedback_silent_disconnect_hang` — the discipline we're closing
- `feedback_no_speculation` — measured the leak empirically (futex_do_wait at every thread)
- `project_signal_cascade` — wat-rs's pgid+killpg discipline (this complements, doesn't replace)

## Empirical proof of the gap (2026-05-13)

Standalone Rust binary (`/tmp/shutdown_gap_proof.rs`, 50 lines, no wat involved):
1. Install SIGTERM handler that sets atomic flag (mirrors `src/fork.rs:113`).
2. Spawn worker thread that blocks on `crossbeam::Receiver::recv()`.
3. Keep `Sender` alive so channel doesn't disconnect.
4. Send SIGTERM to self via `libc::raise`.
5. Wait 1s for worker to wake.

**Result:**
```
SIGTERM handler fired:  true
Worker recv woke:       false
Elapsed:                1.007663018s

GAP CONFIRMED: signal handler fired, but blocked recv did NOT wake.
```

Confirms the substrate-level gap empirically. Crossbeam `recv()` is in kernel
futex_wait; signal handler runs briefly, returns to kernel, futex_wait resumes.
Atomic flag is set but the worker never returns from `recv()`. Only dropping
the `Sender` wakes it.

## User decisions (2026-05-13)

- **`RecvError::Shutdown` dedicated variant** (not folded into `Disconnected`).
  Honest naming of distinct events; users can pattern-match either variant
  specifically OR wildcard with `Result/expect` per arc 110.
- **PR_SET_PDEATHSIG approved.** No mandatory-or-optional ambiguity — same
  status as existing kernel-process settings (setpgid, dup2, signal handlers).
  Just an integration we missed in `spawn_process_child_branch` and `fork.rs`.

## Status

DESIGN with user approval. Slices ready to brief.
