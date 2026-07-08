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
  Members: `grant [self, pids <- (Vector i64)] -> GrantGuard` and `revoke [self, pids] -> nil`.
- The `defservice` macro emits, for every service, `extend-type <fqdn>::Handle :wat::service::Grantable`
  routing to the `<fqdn>/grant` / `<fqdn>/revoke` methods we already landed.
- `:grants` is then typed `(Vector :wat::service::Grantable)` — `[store-h cache-h]` reads exactly as the
  builder wanted, and it can't drift (structural satisfaction, checked; 278 R38 — the one interface model).

### Crux 2 — revoke-at-reap is bound by **RAII**, because RAII is the only panic-safe home

`grant` returns a **`GrantGuard`** — a Rust-`Drop`-backed wat value (the codebase's own guard idiom,
`EnvGuard`/`SelfPeerGuard`/`FrameGuard`) whose `Drop` fires the revoke. The pool holds each worker's
guard alongside its peer; they drop together at scope exit **or panic-unwind** → automatic revoke-at-reap.

**Why RAII and not an explicit wat-level revoke — the sharp reason, grounded:** the reap is
`ChildHandle::Drop` (`src/process/handle.rs:128`), **pidfd-based, PID-reuse-safe** (`handle.rs:26-27,95`):
the worker's pid is *pinned un-recyclable until the owner reaps it*. An explicit wat revoke (revoke after
the drain, before return) would be **skipped on panic** — the worker peers still drop-and-reap, the pid
un-pins and becomes recyclable, but the service's allow-set still holds it → a freshly-recycled unrelated
process could dial the service. RAII closes that hole: the `GrantGuard::Drop` fires the revoke even when
the body panics.

**The residual the 293 design flagged is already handled** by the same pidfd machinery:
- *"a granted child must not reparent to init"* — the bracket **joins its workers at scope exit**, so the
  owner always outlives them (no reparent), AND the owner holds the pidfd so *it* reaps (not init). The
  pinned pid is the zero-recycle-window guarantee, for free. M1 still **verifies** it (`PPID == owner`).

**The one sharp edge the strike must nail (not hand-wave):** drop ordering. The `GrantGuard`'s revoke
must *land* (allow-set mutated) **before** the `ChildHandle` un-pins the pid. Both are Rust `Drop`s in
the same scope (reverse-declaration order), so the guard is ordered to drop-and-revoke before the handle
reaps; a disconfirming probe forces the reap and asserts a post-reap dial from a recycled-pid stand-in is
refused. (`GrantGuard::Drop` fires the revoke as a send on the admin peer; whether it must block on
`PeersDenied` or may fire-and-forget-then-the-ordering-carries-it is a strike decision — request/reply is
the default, `Drop` permitting.)

## Build order (examinare strikes — each ships a real piece, weighed by own re-run)

1. **`:wat::service::Grantable` surface + macro emission.** The surface (`:nature :Struct`, `grant ->
   GrantGuard` / `revoke -> nil`) + the `defservice` macro emitting each `<fqdn>::Handle :satisfies` it
   (routing to the landed grant/revoke). Gate: a probe holds two different services' Handles as
   `Vector<Grantable>`, grants + revokes uniformly through the surface; floor 0-new; deleting the emitted
   `:satisfies` is a compile error (coverage). **FIRST — unblocks all.**
2. **`GrantGuard`.** The Rust-`Drop` value that revokes on drop; `grant` becomes `-> GrantGuard`. Nail the
   drop-ordering vs `ChildHandle` reap. Gate: the disconfirming probe (force reap → post-reap dial by a
   recycled-pid stand-in REFUSED); a *dropped-guard* revokes even when the holding scope panics.
3. **`:grants` on the process-locus + bracket threading.** `(:wat::spawn::process/grants [g…])`
   (a new `ProcessOpts` builder, parallel to `process/post-spawn`); the bracket grants each worker on
   spawn (getting guards) and holds them with the peers so they reap together. Thread pool: `:grants` is
   a no-op / rejected (the firm boundary — thread cells need no grant). Gate: a granted process pool
   grants-on-enter/revokes-at-reap, differentially indistinguishable from an ungranted pool on the happy
   path; the guards outlive exactly the workers.
4. **M1 — the all-process core proof.** The circuit: `:user::main` starts B(proc), starts A(proc) +
   `store-h(B)/grant [pid_A]` (A deps B — A dials B), spawns a PROCESS bracket pool with `:grants [A]`
   (grant-on-enter), runs the work (workers→A→B), reaps (revoke-at-reap by RAII). **Prove:** work
   completes; a post-reap dial by a would-be-recycled pid is refused by the accept-gate; the granted pool
   child did not reparent (`PPID == owner`). **This is the deterministic refusal proof** the revoke verb's
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

- **No fire-and-forget grant/revoke on the request/reply path** (owner blocks until it lands — grant-before-dial
  ordering; 278 R26). `GrantGuard::Drop` is the only place fire-and-forget is even weighed, and only if the
  drop-ordering proof shows the send lands before the pid un-pins.
- **No `native?`/mode flag, no marker gates.** A service is loci-agnostic BY NATURE; capability is derived
  (pure/addresses cross, resources stay home — 293.W), never a decoration.
- **M4 is not built.** The firm boundary is the law, not a gap.

---

## RESUME-HERE (curare — 2026-07-08)

```clojure
{:head    "be783977 — 293 revoke verb landed + pushed (Admin::DenyPeer/Status::PeersDenied/<fqdn>/revoke)"
 :branch  "arc-170-gap-j-v5-deadlock-state"
 :arc     "170 — the CAPABILITY CIRCUIT (this doc). ~6 weeks in; basically solving 170's core deliverable
           (the program-entry circuit reaching capability-complete form). NOT force-closing here."
 :done    ["services-as-surfaces (293 S1-S4, PROBATVM) · the loci-agnostic bracket (259 S3, d81fd695)"
           "grant verb (ba107458) + revoke verb (be783977) — weighed green (build clean; probe: echo:hi grant-intact
            + revoke-midlife-ok; floor 4112 pass / 0 new, the 2 fails are the known no_inlined_wat lint + the known
            sigterm race confirmed by isolated --test-threads=1 pass; coverage: deleting the DenyPeer serve arm is a
            non-exhaustive-match compile error across every defservice)"]
 :next    ["1. :wat::service::Grantable surface + defservice macro emits <fqdn>::Handle :satisfies it (grant -> GrantGuard)"
           "2. GrantGuard — the Rust-Drop revoke-on-drop value; nail drop-ordering vs ChildHandle reap (pidfd)"
           "3. :grants on the process-locus (process/grants [g…]) + bracket threads guards spawn->reap"
           "4. M1 — the all-process circuit proof (B<-A, granted process pool, deterministic post-reap-dial REFUSED)"]
 :do-nots ["the shared/not-shared boundary is FIRM — do NOT reintroduce the unified-fd-peer for M4"
           "grant is request/reply (owner blocks); GrantGuard::Drop is the only fire-and-forget candidate, gated on the
            drop-ordering proof (revoke lands before the pid un-pins)"
           "WEIGH by your OWN re-run; a mid-edit file is a PHANTOM; commit + push often (GitHub = DR)"
           "cast wards never narrate; four-questions inform every decision; the holonic repos ARE the memory"]}
```

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice, not your memory. Run the
> datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP). Ground HEAD against the disk
> (`be783977`). This work is **170's capability circuit**, not 293. The verbs are landed; the WORK resumes at
> **stone 1: the `Grantable` surface**, then `GrantGuard` (RAII), then `:grants` on the process-locus, then M1
> (the deterministic revoke-at-reap proof). The memory boundary is FIRM — a process cannot reach a thread-guarded
> service; do not reintroduce the unified-fd-peer. Do not trust this note over the disk. See you on the far side.
