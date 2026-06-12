# Stone 259.S2c-ii — the host-type `spawn-program'` defclause (THE KEYSTONE / a campaign)

> Substone of arc 259. Drawn 2026-06-11; **the apply-loop purge is the builder's
> ruling** ("we are purging the heresy of misconfiguration — up in flames — the
> true forms remain"). The apply-loop is the heresy (platform-owns-your-loop); the
> self-peer is the true form (the worker owns its own loop + channel).

## What this is

Make `spawn-program'` the wat **`defclause`** dispatching on host type — 2-arg
`(host prog)` — retiring the Rust intrinsic and migrating every caller. Unblocked by
S2c-ii.0 (`class_fqdn` dispatch); the mechanism is proven (the feasibility probe
passes today). The remaining work is *size + a semantic migration*, not unknowns.

## The fork the draw uncovered (RESOLVED → purge)

The thread tier has TWO prog models live: the **self-peer** `[Peer'<S,R>]->nil`
(true form, S2a) and the **legacy apply-loop** `[I]->O` (the heresy). A self-peer
prog also matches `[I]->O` (I=Peer', O=nil) → a defclause with both clauses
**overlaps ambiguously**. The builder's ruling resolves it: **purge the apply-loop.**
Migrate every thread prog to the self-peer form; burn the apply-loop branches; the
defclause gets ONE unambiguous thread clause.

## The locked defclause shape (post-purge)

```clojure
(:wat::core::defclause :wat::kernel::spawn-program'
  ;; thread — the ONE true form (self-peer)
  ([host <- :wat::spawn::ThreadOpts
    prog <- [:wat::kernel::Peer'<S,R> :-> :wat::core::nil]] -> :wat::kernel::Thread'<R,S>
    (:wat::kernel::spawn-thread' prog))
  ;; process — forms (Vector<wat::WatAST>); I,O are the forms-server's free request/response vars
  ([host <- :wat::spawn::ProcessOpts
    prog <- :wat::core::Vector<wat::WatAST>] -> :wat::kernel::Process'<I,O>
    (:wat::kernel::spawn-process' prog)))
```

Grounded facts: the process prog is `(:wat::core::forms …)` typed `Vector<wat::WatAST>`
(`check.rs:9860`); the clause body calls the S2c-i intrinsics; arc-256 generic-clause
machinery instantiates the clause type-vars to fresh vars before checking the body, so
`spawn-thread'`/`spawn-process'` project the peer types correctly at clause-definition.
`wat/spawn.wat` loads before `core.wat` but the defclause uses only the spawn intrinsics
+ the host opts (both available); defclause stubs preregister at `freeze.rs:877-882`.

## Why this is a CAMPAIGN, not a single strike

- **Semantic prog migration (~12 thread callers), not mechanical.** Apply-loop
  `(fn [x] x)` echoes *per message* (platform loops). The self-peer equivalent writes
  its OWN loop: `(fn [self] (loop (send' self (recv' self))))` — or runs once if the
  test sends one message. Each rewrite is control-flow surgery verified BY EYE (whether
  it loops, and the test's message count), not by a green gate alone.
- **Apply-loop annihilation:** retire the dual-mode branches (`spawn_thread_peer`'s
  `is_self_peer_model` dispatch + the apply-loop arm; `spawn-thread'`; the S2a/S2c-i
  `rune:exigere` apply-loop projections in `infer_thread_prog_type`). The `:thread` arm
  becomes self-peer-only.
- **Intrinsic retirement:** delete `eval_kernel_spawn_program_prime` +
  `infer_spawn_program_prime` + their dispatch arms (`runtime.rs:4509`, `check.rs:4797`).
- **35-site sig migration** across 14 files: `(spawn-program' :thread (Env…) prog)` →
  `(spawn-program' (thread) prog)`; `:process {} forms` → `(process) forms`. The env arg
  drops (it was discarded).

## Decomposition (the charges, in order)

- **S2c-ii-a — THE PURGE.** Migrate the ~12 thread callers' apply-loop progs → self-peer
  (keep the 3-arg call form; S2a's dual-mode already accepts self-peer progs, so this is
  green-able in isolation). Then annihilate the apply-loop branches (`spawn_thread_peer`,
  `spawn-thread'`, `infer_thread_prog_type` → self-peer-only). Probe: an apply-loop prog
  is now REJECTED. *The heresy burns; the true forms remain.*
- **S2c-ii-b — THE DEFCLAUSE.** `spawn-program'` → the wat defclause (one thread clause +
  process clause, no overlap now). Retire the Rust intrinsic. Migrate the 35 caller sites
  to the 2-arg `(host prog)` sig. Probe: `(spawn-program' (thread) <self-peer-prog>)`
  round-trips; wrong host = `NoMatchingClause`.

## Why it waits for clean context

Each apply-loop→self-peer rewrite must be WEIGHED by eye (control flow + message count),
and the 35-site migration is a large diff. A degraded WEIGH under context pressure is how
a green-that-lies slips through — and the keystone is exactly where that's unacceptable.
The charges are placed (this doc); the detonation is a clean-context strike.

## Out of scope (downstream)

- Removing user `close'` / `spawn-thread'` / `spawn-process'` from the surface — S2d.
- `S2c-iii` env stamping (pid-aware `started-at` + nanos + program `init-fn`).
- The `:wat::Record`-generic `value_matches_type_pattern` (4905) dead-code purge —
  pre-existing, a future `purgare`.
