# Arc 194 — Orchestrated `:wat::kernel::exec` + wat-side worker/supervisor library

**Status:** stub opened 2026-05-13 per user direction. Captures the user-facing layer of the cooperative-migration architecture surfaced in arc 170 INTERSTITIAL conversation.
**Gates on:** arc 191 (bare exec-program) + arc 192 (state-preserving exec + SignalService).

## Motivation

User direction 2026-05-13 (after the architecture coalesced):
> *"so... we could signal ourselves and cascade it.. we explore something like (:wat::kernel::exec forms) who does all the things?.."*

This arc captures THE primitive the user actually writes — the high-level all-in-one `exec` that handles signal cascade + state collection + universe swap. Plus the wat-side library codifying the worker/supervisor pattern.

Arc 191's `exec-program` is bare. Arc 192's `exec-program-with-state` adds carry-overs manually. Arc 194's `exec` does **all the things** so user code stays clean.

## The user's view

```scheme
;; Setup
(:wat::kernel::spawn-with-state :name :user::data-worker :initial-state {})
(:wat::kernel::spawn-with-state :name :user::log-worker :initial-state [])

;; Time passes; workers accumulate state

;; Trigger reload
(:wat::kernel::exec next-program-ast)
;; ... user-level code never sees the dance; new universe's main starts
```

The dance the substrate runs under the hood:
1. Pre-validate new universe (freeze preview)
2. SIGEMT cascade — intra-process only; SignalService disseminates to all in-process threads
3. Collect state from compliant workers (registered via `spawn-with-state`)
4. Handle non-compliant threads per policy (strict by default — refuse if any plain-spawn threads alive)
5. Join all worker threads (confirm clean exit)
6. Freeze validated new world (reuse preview from step 1)
7. Lift carry-overs into new universe's prologue (mechanism: Gap H's lift, injected programmatically)
8. Services stay (OS-continuity)
9. Exec into new universe (never-return)
10. New universe's main inspects carry-overs; re-spawns workers via name resolution

## The three pieces

### 1. `:wat::kernel::exec` substrate primitive (the all-in-one)

```
:wat::kernel::exec
  (forms :wat::core::Vector<:wat::holon::Atom>)
  -> :wat::core::Result<:wat::core::never, :wat::kernel::ExecError>
```

Orchestrates the 10-step dance above. Internally uses arc 191's `exec-program` or arc 192's `exec-program-with-state` as the universe-swap primitive. Adds: signal cascade, state collection from compliant workers, join discipline, non-compliant policy enforcement.

### 2. `:wat::kernel::spawn-with-state` substrate primitive

```
:wat::kernel::spawn-with-state
  :name :wat::core::Keyword         ;; resolvable name (e.g., :user::data-worker)
  :initial-state :wat::holon::Atom  ;; starting state
```

Spawns a thread running the worker function looked up by `:name` in the current universe's defines. Registers the thread in a substrate-side registry keyed by `:name`. The worker function MUST be written using the `loop-with-shutdown` idiom (or equivalent — subscribe to SignalService; return state at safe-point on reload signal).

On reload signal:
- Worker function is responsible for state capture (via loop-with-shutdown safe-point check)
- Worker emits state through substrate-owned state-collection channel keyed by `:name`
- Substrate's `exec` orchestration reads from collection channel

On resume (new universe):
- Substrate looks up `:name` in new universe's defines
- Calls `(:user::data-worker carry-over-state)` to resume
- Same function shape both directions — fresh-start uses initial-state; resume uses carry-over

**Workers are referenced by NAME, not function reference.** The name resolves in whatever universe currently runs. This is the load-bearing piece that makes cross-universe resume possible.

### 3. Wat-side library: `wat::worker` + `wat::supervisor`

The library codifies the worker pattern + the supervisor pattern.

`wat::worker::loop-with-shutdown` — macro that hides the boilerplate:

```scheme
;; Without the macro
(:wat::core::defn :user::data-worker [state <- :MyState] -> :wat::core::nil
  (:wat::kernel::let [shutdown-rx (:wat::kernel::SignalService/subscribe :SIGEMT)]
    (:wat::core::loop [s state]
      (:wat::kernel::select
        [(:wat::kernel::Signal/ReloadRequested) <- shutdown-rx
          ;; Submit state + exit
          (:wat::kernel::submit-state s)]
        [work <- work-rx
          (:wat::core::recur (step s work))]))))

;; With the macro
(:wat::worker::loop-with-shutdown :user::data-worker [state <- :MyState]
  (work <- work-rx
    (:wat::core::recur (step state work))))
```

The macro:
- Subscribes to SignalService
- Wraps body in loop + select
- Auto-adds the shutdown-arm with submit-state
- User writes only the work-handling arm(s)

`wat::supervisor::*` — supervisor pattern (optional; users can wire by hand). Likely just helpers: spawn-and-track, broadcast-shutdown, collect-states, join-all, build-successor.

## Slice plan (rough)

### Slice 1 — `spawn-with-state` substrate primitive + registry

- Mints the spawn-with-state primitive
- Substrate-side registry of compliant threads (name → thread handle + state-collection channel)
- Initial integration with arc 192's SignalService

### Slice 2 — `:wat::kernel::exec` orchestrated primitive

- The all-in-one. Internally calls arc 192's exec-program-with-state.
- Implements the 10-step dance.
- Failure modes: ExecError variants for each step that can fail.

### Slice 3 — `wat::worker::loop-with-shutdown` macro

- Wat-side macro definition; lives in wat/kernel/ or similar
- Hides the loop+select+state-submit boilerplate
- Probably co-located with other worker idioms in a `wat::worker` namespace

### Slice 4 — `wat::supervisor::*` helpers (optional layer)

- For users who want explicit supervisor pattern (rather than `exec`'s automatic orchestration)
- Helpers: spawn-and-track, broadcast-shutdown, etc.
- Mostly for advanced cases; most users just use `exec`

### Slice 5 — REPL worked example (probably co-shipped with arc 191)

- Demonstrates `exec` end-to-end with workers
- The canonical demo: REPL has workers; user requests reload; workers cleanly drain + carry state; new REPL universe spawns; workers resume

### Slice 6 — INSCRIPTION + USER-GUIDE + cross-references

Closure paperwork.

## Open design questions

1. **Where does `exec` get called from?** Convention: main only. If `exec` is called from a non-main thread, the main thread doesn't know exec happened until it sees the universe swapped — probably surprising. Either:
   - **(a)** Compile-time check: `exec` is illegal outside `:user::main`'s call graph (analysis at check time)
   - **(b)** Runtime check: `exec` fails with `Err(ExecCalledFromNonMain)` if called outside main
   - **Recommendation:** (b) for now; (a) when the substrate has the analysis machinery.

2. **Non-compliant thread policy at `exec`.** Three options:
   - **Strict default:** `exec` returns `Err(NonCompliantThreadsAlive(names))` if any plain-spawn threads alive
   - **Timeout-then-kill:** wait T seconds; signal-kill survivors
   - **Wait-natural:** wait indefinitely
   - User-configurable via flag: `(:wat::kernel::exec forms :non-compliant :strict)` / `:timeout-then-kill` / `:wait-natural`
   - **Recommendation:** strict default; configurable for advanced cases.

3. **State submission protocol.** When a compliant worker submits state, does it go through:
   - **(a)** A substrate-owned channel hidden from user code (worker calls `(:wat::kernel::submit-state s)`)
   - **(b)** A user-visible channel the supervisor passes to spawn-with-state
   - **Recommendation:** (a) — keeps the channel discipline simple; substrate handles transport.

4. **What if `:name` doesn't resolve in the new universe?** A worker registered as `:user::data-worker` exists in the OLD universe; the user's new program AST might not define it. Three options:
   - **(a)** `exec` validates name resolution at pre-flight (slice 1's preview freeze includes a check)
   - **(b)** Resume-time error: worker can't be re-spawned; carry-over state is dropped; warning emitted
   - **(c)** Carry-over state is preserved in carry-overs map even without a resumer; new universe's main can inspect it manually
   - **Recommendation:** (a) at preflight + (c) as fallback if validator is permissive.

5. **Composition with arc 193's universe image.** Can `exec`'s state-collection mechanism feed into a `dump-image` instead of a new universe? I.e., "dump the universe AS IF we were exec'ing into nowhere, capturing all worker states into an image file." This is arc 193's slice 4 use case. Probably yes; the state-collection mechanism is reusable.

6. **Supervisor pattern vs `exec`-orchestrated.** The wat::supervisor library is OPTIONAL. The substrate's `exec` does the orchestration. When would a user want explicit supervisor instead?
   - Granular control (drain SOME workers, keep others)
   - Multi-stage rollout (start new universe; old + new coexist; eventually old drains)
   - Specialized rollback (if new universe misbehaves, kill it and resume old)
   - These are advanced cases. Most users use `exec`. Supervisor library is for when `exec`'s policy isn't enough.

7. **Generation tracking.** Should the substrate track WHICH universe-generation each thread belongs to? Useful for: ignoring stale signals (a worker in old universe receiving a SIGEMT meant for new universe), debugging, generation-aware policies. Probably yes; cheap to add; useful diagnostic.

## Cross-references

- Arc 170 INTERSTITIAL — the conversation that surfaced this architecture
- Arc 191 — bare exec-program (foundation)
- Arc 192 — state-preserving exec + SignalService (substrate plumbing)
- Arc 193 — universe image (orthogonal capability; may share state-collection mechanism)
- Memory `project_signal_cascade.md` — POSIX pgid + killpg discipline; cascade scope is intra-process for SIGEMT
- Memory `feedback_zero_mutex.md` — the Zero-Mutex doctrine that makes cooperative migration possible (threads communicate only via channels; no shared mutable state; each universe owns its TypeEnv)
- Erlang/OTP supervisor pattern — historical precedent for the worker-with-state + supervisor architecture
- Go's signal-channel idiom — precedent for select-on-signal pattern in loop-with-shutdown
- Gap H + Gap I-A — the lift + predicate machinery exec's carry-over injection reuses

## Why this matters

This is **the layer the user actually programs against.** Arc 191 + 192 + 193 are substrate; arc 194 is ergonomics. After 194 ships:

- Writing a hot-reloadable wat program is ROUTINE. User declares workers with `spawn-with-state`; writes worker functions using `loop-with-shutdown`; calls `exec` when ready to swap. The substrate handles everything else.
- The barrier to hot reload is exactly the same as the barrier to writing a worker function — basically zero.
- Operational tooling becomes: `pkill -EMT my-service` triggers reload. Workers checkpoint, carry state, new universe boots, workers resume. Zero downtime, zero user-side orchestration code.
- Service evolution becomes data-flow: receive new program AST → call `exec` → done.

Combined with arc 193's image dump: services can checkpoint to disk periodically, allowing crash recovery from the last image. Combined with arc 191's signed-exec: services can receive signed program updates over the network, verify them, exec into them. Combined with arc 192's typed carry-overs: services preserve typed state across reloads with the substrate's static guarantees intact.

This is Erlang/OTP-class capability with static typing per universe and image-style persistence. The substrate's design choices accumulate into this — none of these features were designed in isolation. The whole architecture composes.
