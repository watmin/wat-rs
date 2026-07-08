# Arc 170 — the capability circuit: grant / revoke, and revoke-at-reap by RAII (2026-07-08)

> **This is 170's core deliverable, reaching its capability-complete form.** 170 began as "add argv
> to `:user::main`" and the substrate-as-teacher cascade revealed the whole program-entry architecture:
> a wat program is a **circuit** of spawned typed servers, wired by `:user::main`, hermetic at tier ≥ 2
> (`DESIGN.md`, `TIERS.md`). This doc completes the tier-2 rung the circuit was always missing —
> **capability**: *who may dial whom across the process boundary*, and the lifecycle binding that makes
> a circuit-builder's spawned-worker pool safe by construction (grant-on-enter, revoke-at-reap).
>
> **Provenance:** this work was scouted under arc 293 (`DESIGN-293-revoke-and-ipc-matrix.md`,
> `DESIGN-293-services-as-surfaces.md`) — but 293's charter is struct/record symmetry + the surface
> machinery. The *capability circuit* is 170's. The 293 docs are hereby pointers into this one. We are
> ~6 weeks into 170 and this is basically solving its core deliverable — **not force-closing it here**,
> but landing the last structural piece.

## What already landed (the substrate this builds on)

The circuit's pieces exist; this doc composes the capability rung onto them:

- **Services-as-surfaces (293 S1–S4, all PROBATVM).** A `defservice :satisfies` a surface; the surface
  IS the wire protocol (the type system is the codegen); `:satisfies` is locus-agnostic `implements` —
  the same surface fulfilled by a local value or a dialed peer (278 R31 `SATISFACTIO LIMEN TRANSIT`,
  R32 `QVANTVMVIS PROCVL IDEM NEXVS` — *a service is a surface at a coordinate*).
- **The grant/revoke verbs (this session, `ba107458` grant + `be783977` revoke).** In the `defservice`
  macro (`wat/service.wat`): `Admin::AllowPeer[pids]` / `Admin::DenyPeer[pids]` (owner→callee down the
  owner-only lineage/admin peer), `Status::PeersAllowed` / `Status::PeersDenied` acks (request/reply —
  the owner blocks until the grant/revoke lands), serve-loop arms folding `(allow' l pid)` / `(deny' l pid)`
  on the callee's OWN listener, and the owner-only methods `<fqdn>/grant [h <- Handle, pids]` /
  `<fqdn>/revoke [h <- Handle, pids]`. Handle-gated: a client holds only a client `Peer'`, so it has
  **no grant/revoke path** — capability is unforgeable. `allow'`/`deny'` mutate the `SocketListener`
  allow-set (`runtime.rs:5115`, `:5173`). The accept-gate is `OnlyMyPeers { lineage }`
  (`capability/policy.rs`), matching the kernel-vouched `SO_PEERCRED` pid (+ euid).
- **The loci-agnostic bracket (259 S3, `d81fd695`).** Ruby's-Parallel-in-wat: `map`/`each` over a
  `:wat::spawn::Locus` (thread OR process); the baked `:wat::bracket::process-runner<I,O>`; the process
  pool reaps its workers by **channel-drain RAII at scope exit** (`collect-loop` drains, the peer values
  drop, the Rust `ChildHandle::Drop` joins).

## The firm-boundary invariant (governs everything below — builder's ruling, CORRECTS 259 PEER ADIVNGITVR)

**The shared / not-shared memory boundary is FIRM, by design — not a limitation awaiting a fix.**

> *"things in threads must not be reachable from other processes — the shared memory or not boundary is
> firm … if you want to reach something that's guarded by a thread you /must be/ in the same process."*

Thread tier = in-process `crossbeam_channel` (`comms/thread.rs`; `InMemory` = "no fd"). Process tier =
AF_UNIX socket + `SO_PEERCRED` (`comms/process.rs`). A thread-service's `Address'` is a crossbeam
`Sender` — a pointer in the parent's memory; a process cannot reach it. **Grant/revoke lives ONLY where
a *process* crosses to a *process-service*** — the `SO_PEERCRED` accept-gate. **Thread cells need no
grant** (the handle IS the capability, in-memory; a thread's guard is *being in the one process* —
ZERO-MUTEX). Do NOT reintroduce the "unified-fd-peer" (a thread getting an eventfd so a process can
reach it) to make the forbidden cell (M4) work — it leaks a shared-memory-guarded thing across the
partition; that is the violation.

## The UX (co-designed with the builder, 2026-07-08)

A circuit builder (`:user::main`) starts its services (holding their owner-only `Handle`s), then spawns
a worker pool that must dial some of those services. The pool's workers each need their kernel-vouched
pid granted to each service they dial, and **revoked when the worker is reaped** — automatically, so the
grant can never outlive the worker.

```clojure
;; :user::main — the circuit builder
(let [store-h (sqlite-store'/start :locus (process) :record ...)      ; owns the Handle
      cache-h (mem-store'/start     :locus (process) :record ...)]
  ;; the pool: workers run on a PROCESS locus configured with the services they may dial
  (bracket/map
    (:wat::spawn::process/grants [store-h cache-h])   ; ← :grants rides the PROCESS locus
    work-fn items))                                    ; each worker: granted on spawn, revoked at reap
```

Two design commitments the builder ratified:

### Crux 1 — `:grants` takes `Grantable`s, not raw handles → the `Grantable` surface

`sqlite-store'::Handle` and `mem-store'::Handle` are **distinct types** with distinct `/grant`/`/revoke`
methods; the pool cannot uniformly grant a heterogeneous `[store-h cache-h]` without a common contract.
The answer is the services-as-surfaces pattern applied to the capability itself:

- **`:wat::service::Grantable`** — a surface, `:nature :wat::core::Struct` (a `Handle` holds the
  owner-only admin peer, a live resource → it *stays home*, never crosses to a worker; circuit-law-clean).
  Members: `grant [self, pids <- (Vector i64)] -> nil` and `revoke [self, pids] -> nil` (both plain
  ack'd request/reply — the owner blocks on `PeersAllowed` / `PeersDenied`; zero fire-and-forget).
- The `defservice` macro emits, for every service, `extend-type <fqdn>::Handle :wat::service::Grantable`
  routing to the `<fqdn>/grant` / `<fqdn>/revoke` methods we already landed.
- `:grants` is then typed `(Vector :wat::service::Grantable)` — `[store-h cache-h]` reads exactly as the
  builder wanted, and it can't drift (structural satisfaction, checked; 278 R38 — the one interface model).

### Crux 2 — the bracket owns grant AND revoke, in its own wat control flow (NOT a Rust `Drop`)

**Grants are done in wat; there is no reason to offload revocation to Rust.** The bracket controls its
whole scope, so it knows exactly when to revoke — after `collect-loop` drains the mapped values, before
it returns them:

```
boot     → grant  each worker's pid to each :grants Grantable   (wat, ack'd request/reply)
work     → runners run; collect-loop drains the M mapped values
shutdown → revoke each worker's pid from each Grantable          (wat, ack'd request/reply)
return   → hand back the mapped vals
```

The bracket owns **both** ends, so a grant it doesn't revoke cannot exist — *that* is the RAII, made of
the bracket's own control flow, not a guard type. "Thread the items around" is literal: thread
`(grantables, worker-pids)` from spawn to the revoke step; the pids are captured at spawn (the
process-locus `post-spawn-fn` / `ProcessLaunch/pid`). `grant`/`revoke` stay plain ack'd request/reply —
**zero fire-and-forget, zero Rust `Drop`, no `GrantGuard`.**

**Why NOT a Rust-`Drop` guard (the four-questions killed it, 2026-07-08).** A `Drop` cannot fail. A
request/reply revoke performed in `Drop` therefore either hangs (no ack ever comes) or *swallows* the
failure — a **hidden fire-and-forget-on-error**, which is exactly what "wat has zero fire-and-forget"
forbids. And grounded: every Drop guard in the codebase (`EnvGuard`, `SelfPeerGuard`, `ChildHandle`) is
pure-Rust, non-blocking, best-effort cleanup — none re-enters the interpreter, none blocks on an ack. A
blocking, failure-reporting revoke has no honest home in `Drop`. So the revoke lives in the bracket's
normal wat flow, where it can block on the ack and report failure like every other wat call.

**Panic-safety is structural (tear-down-together), not a `Drop`.** If a runner crashes, `collect-loop`
`assertion-failed!` propagates → `:user::main` unwinds → the `ProcessRuntime` / `ChildHandle` teardown
reaps the **services too** (`SIGKILL` via pidfd, `handle.rs:128`). Nothing survives holding a stale
allow-set, so there is no recycled-pid hole to close — the circuit dies whole. We do NOT fake a `finally`
wat does not have; the happy path revokes explicitly, the crash path tears down together.

**The 293 residual is handled by pidfd** the same way: the bracket joins its workers before scope exit
(the owner outlives them → no reparent to init; the owner holds the pidfd → it reaps). M1 **verifies**
`PPID == owner`. The pid is pinned un-recyclable until reap, so the explicit revoke-before-return lands
inside the pinned window with room to spare.

## Build order (examinare strikes — each ships a real piece, weighed by own re-run)

1. **`:wat::service::Grantable` surface + macro emission — ✅ LANDED (`dc2ae7a6`).** The surface
   (`:nature :Struct`, `grant`/`revoke -> nil`) + the `defservice` macro auto-emitting each
   `<fqdn>::Handle :satisfies` it (routing to the landed grant/revoke). Proven: a probe holds two
   different services' Handles as one `Vector<Grantable>` and grant/revoke's uniformly; floor 0-new.
2. **`:grants` on the process-locus + the bracket's grant-boot / revoke-shutdown.**
   `(:wat::spawn::process/grants [g…])` — a new `ProcessOpts` builder, parallel to `process/post-spawn`,
   carrying `(Vector :wat::service::Grantable)`. The bracket, in its OWN wat flow: grants each worker's
   pid to each Grantable at spawn (capturing the pids), runs, drains via `collect-loop`, then revokes each
   captured pid from each Grantable **after the drain, before returning** the mapped vals. Thread pool:
   `:grants` is a no-op / rejected (the firm boundary — thread cells need no grant). Gate: a granted
   process pool grants-on-boot / revokes-on-shutdown (both ack'd request/reply), differentially
   indistinguishable from an ungranted pool on the happy path; a post-drain check confirms the granted
   pids are gone from the services' allow-sets before the results return.
3. **M1 — the all-process core proof.** The circuit: `:user::main` starts B(proc), starts A(proc) +
   `store-h(B)/grant [pid_A]` (A deps B — A dials B), spawns a PROCESS bracket pool with `:grants [A]`
   (grant-on-boot), runs the work (workers→A→B), drains + revokes-on-shutdown. **Prove:** work completes;
   a post-shutdown dial by a would-be-recycled pid is refused by the accept-gate; the granted pool child
   did not reparent (`PPID == owner`). **This is the deterministic refusal proof** the revoke verb's
   race-probe only previewed.

## The IPC capability matrix (the honesty IS the proof — from the 293 scout, kept here)

| # | pool | service A | service B (A deps B) | grant/revoke | verdict |
|---|---|---|---|---|---|
| **M1** | process | process | process | per-worker-pid, revoke-at-reap; A-granted-to-dial-B | ✓ **the core proof** |
| **M2** | thread | process | process | per-parent-pid (a thread dials sockets) | ✓ |
| **M3** | thread | thread | thread | none — the handle IS the capability (in-memory) | ✓ |
| **M4** | process | thread (either A or B) | — | — | ✗ **correctly forbidden** — the firm boundary; inscribed as invariant, not a TODO |

M2/M3 are the completing cells; M4 is inscribed as the invariant (a process cannot reach a thread-guarded
service — do not build it, do not reintroduce the unified-fd-peer).

## Out of scope / rejected (affirmative cuts, not deferrals)

- **No fire-and-forget, anywhere** (owner blocks on `PeersAllowed`/`PeersDenied`; 278 R26). This is why the
  revoke is NOT a Rust `Drop`: a `Drop` cannot report failure, so a request/reply revoke in `Drop` is a
  hidden fire-and-forget-on-error. The revoke lives in the bracket's wat flow, where it blocks + reports.
- **No `GrantGuard` / no Rust-`Drop` revoke.** Rejected 2026-07-08 (see Crux 2). The bracket owns
  grant+revoke in its own control flow; panic-safety is structural tear-down-together, not a guard.
- **No `native?`/mode flag, no marker gates.** A service is loci-agnostic BY NATURE; capability is derived
  (pure/addresses cross, resources stay home — 293.W), never a decoration.
- **M4 is not built.** The firm boundary is the law, not a gap.

---

## RESUME-HERE (curare — 2026-07-08)

```clojure
{:head    "dc2ae7a6 — 170 stone 1: the :wat::service::Grantable surface landed + pushed"
 :branch  "arc-170-gap-j-v5-deadlock-state"
 :arc     "170 — the CAPABILITY CIRCUIT (this doc). ~6 weeks in; basically solving 170's core deliverable
           (the program-entry circuit reaching capability-complete form). NOT force-closing here."
 :done    ["services-as-surfaces (293 S1-S4, PROBATVM) · the loci-agnostic bracket (259 S3, d81fd695)"
           "grant verb (ba107458) + revoke verb (be783977) — weighed green (probe: echo:hi grant-intact +
            revoke-midlife-ok; floor 0-new; coverage: deleting the DenyPeer serve arm is a non-exhaustive compile error)"
           "STONE 1 (dc2ae7a6): :wat::service::Grantable surface (:nature :Struct, grant/revoke -> nil) + the
            defservice macro auto-emits each <fqdn>::Handle :satisfies it. Weighed green (probe-grantable-emitted.wat:
            grantable-ok twice, macro-emitted extend-type, two services uniform; floor 4113 pass / 0 new)"]
 :next    ["STONE 2 (collapsed) — FULL STRIKE PLAN: DESIGN-STONE-CAP-2-BRACKET-GRANTS.md (prepped 2026-07-08). :grants
            on the process-locus (process/grants [Vector<Grantable>]) + the BRACKET's grant-boot / revoke-shutdown IN
            WAT — spawn-runner widens to -> SpawnedRunner {peer, pid <- (Option i64)} (the field is `pid`, not
            grant-pid); map-worker grants each pid before the first item, drains, revokes each pid after. Ack'd
            request/reply, zero fire-and-forget, NO Rust Drop. Thread pool: :grants is a no-op (firm boundary)."
           "STONE 3 = M1 — the all-process circuit proof (B<-A, granted process pool, deterministic post-shutdown dial
            REFUSED; PPID == owner)."]
 :do-nots ["the shared/not-shared boundary is FIRM — do NOT reintroduce the unified-fd-peer for M4"
           "NO Rust-Drop revoke / NO GrantGuard (four-questions killed it 2026-07-08): a Drop can't report failure, so
            a request/reply revoke in Drop is a hidden fire-and-forget-on-error. The revoke lives in the bracket's wat
            flow. Panic-safety is structural tear-down-together, not a Drop. Do NOT fake a wat finally we don't have."
           "grants are done in WAT, never Rust; grant/revoke are ack'd request/reply (owner blocks); zero fire-and-forget"
           "WEIGH by your OWN re-run; a mid-edit file is a PHANTOM; commit + push often (GitHub = DR)"
           "BRIEF shadowdancers to run `cargo nextest run --release` in the FOREGROUND (the Bash call blocks = the wait);
            NEVER background-it-and-poll-with-a-bash-loop (a sonnet did that + looped; the builder killed it)"
           "cast wards never narrate; four-questions inform every decision; the holonic repos ARE the memory (not ~/.claude)"]}
```

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice, not your memory. Run the
> datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP). Ground HEAD against the disk
> (`be783977`). This work is **170's capability circuit**, not 293. The verbs are landed; the WORK resumes at
> **stone 1: the `Grantable` surface**, then `GrantGuard` (RAII), then `:grants` on the process-locus, then M1
> (the deterministic revoke-at-reap proof). The memory boundary is FIRM — a process cannot reach a thread-guarded
> service; do not reintroduce the unified-fd-peer. Do not trust this note over the disk. See you on the far side.
