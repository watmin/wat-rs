# 293 — the revoke verb + the IPC capability matrix (the revocation proof) (2026-07-08)

293's final movement: complete the grant/revoke capability layer (the revoke verb, symmetric to the
landed `grant`), then prove the IPC story rock-solid with a **capability matrix** — a service depping
a service, a bracket pool commuting with them, grant-on-enter / revoke-at-reap, across thread/process
topologies.

## The firm-boundary invariant (builder's ruling, 2026-07-08 — CORRECTS 259 PEER ADIVNGITVR)

**The shared / not-shared memory boundary is FIRM, by design — not a limitation awaiting a fix.**

> *"things in threads must not be reachable from other processes — the shared memory or not boundary
> is firm … if you want to reach something that's guarded by a thread you /must be/ in the same process."*

Grounded: thread tier = in-process `crossbeam_channel` (`comms/thread.rs:1`; `InMemory` = "no fd",
`comms/mod.rs:815`); process tier = AF_UNIX socket + `SO_PEERCRED` (`comms/process.rs`). A thread-
service's `Address'` is a crossbeam `Sender` — a pointer in the parent's memory. Cross-process reach
would require adding a shared **data** channel to a thread (a shared eventfd only signals *readiness*;
the data stays in-memory) — which leaks a shared-memory-guarded thing across the partition. **That is
the violation.** A thread's state is safe *without a lock* precisely because its guard is *being in
the one process* (ZERO-MUTEX). Punch an fd through it and the guarantee dissolves.

**STRUCK:** `259 PEER ADIVNGITVR`'s "unified-fd-peer (thread gets an eventfd → mixed pools work,
`Thread'`/`Process'` vanish)" as a *goal*, for the cross-process-reach reading. Orthogonal and
**not** struck: a **same-process** coordinator selecting over both a thread-peer and a process-peer in
one loop (both channels are its own, nothing crosses the partition) — that's a `select'` convenience,
a different axis.

## The compatibility matrix (locked — its honesty IS the proof)

| # | bracket pool | service A | service B (A deps B) | grant/revoke | verdict |
|---|---|---|---|---|---|
| **M1** | process | process | process | per-worker-pid, revoke-at-reap; A-granted-to-dial-B | ✓ **the core proof** |
| **M2** | thread | process | process | per-parent-pid (a thread dials sockets) | ✓ |
| **M3** | thread | thread | thread | none — the handle IS the capability (in-memory) | ✓ |
| **M4** | process | thread (either A or B) | — | — | ✗ **correctly forbidden** — the firm boundary; inscribed as invariant, not a TODO |

Grant/revoke (the `SO_PEERCRED` accept-gate, `OnlyMyPeers { lineage }`, `capability/policy.rs:44`) lives
ONLY where a *process* crosses to a *process-service*. Thread cells need no grant. So the revocation
proof's beating heart is M1: process-pool → process-service, keyed on the worker's kernel-vouched pid.

## Item 1 — the revoke verb (mirror of the landed grant, `ba107458`)

The grant shipped as (all in `wat/service.wat`, the `defservice` macro): `Admin::AllowPeer[pids]`
(owner→callee down the owner-only lineage/admin peer) + `Status::PeersAllowed` ack + a serve-loop arm
folding `(allow' l pid)` on its OWN listener + the owner-only `<fqdn>/grant [h <- Handle, pids]` method
(clients hold only a client `Peer'`, no grant path). Request/reply so the owner blocks until the grant
lands (grant-before-dial ordering). `allow'`/`deny'` mutate the `SocketListener`'s allow-set
(`runtime.rs:5115`); `deny'` already exists.

**The revoke mirrors it across 6 sites** (reusing `deny'`):
1. `Admin` enum (service.wat:601) — add `:DenyPeer [pids <- (Vector i64)]`.
2. `Status` enum (:608) — add `:PeersDenied`.
3. kw + fold binders (:574/:578) — `admin-deny-peer-kw`, `status-peers-denied-kw`, `deny-acc-sym`/`deny-pid-sym`.
4. `dispatch-admin` (:636) — add a `DenyPeer` startup arm → `assertion-failed!` (DenyPeer-before-Init is a protocol error, exactly like AllowPeer).
5. Serve loop (:758) — add `((~admin-deny-peer-kw pids) (do (foldl (deny' l pid) …) (send' self PeersDenied) (~serve-name self l clients state)))`.
6. Owner method (:885) — `<fqdn>/revoke [h <- Handle, pids <- (Vector i64)] -> nil` sending `Admin::DenyPeer[pids]` down `(handle-handle-acc h)`, recv + match `PeersDenied`; `methods = conj methods revoke-method`.

Owner-only (Handle-gated), request/reply (owner blocks until the deny lands), callable any time,
repeatedly. Symmetric with `stop`/`hibernate`/`grant`.

## Item 2 — the revocation proof (the matrix)

**M1 (the core):** a circuit builder (`:user::main`, the CIRCUIT.md pattern) starts B (process), starts
A (process) and `Handle_B/grant [pid_A]` (A deps B — A dials B), spawns a **process** bracket pool,
`Handle_A/grant [worker-pids]` (grant-on-enter), runs the pool work (workers dial A, A dials B), then at
reap `Handle_A/revoke [worker-pids]` (revoke-at-reap — the bracket's drain-and-join IS the reap, zero
recycling window). Prove: work completes; a post-reap dial by a (would-be-recycled) pid is refused by
the accept-gate. **Residual to design around:** a granted child that orphans to init is reaped by init,
not the owner — a granted pool child must not reparent (assert/verify PPID = the owner).

**M2 / M3:** the matrix-completing cells (thread pool ∘ process services; all-thread, no-grant). Prove
comms crosses every allowed wire, both grant models exercised.

**M4:** inscribed as the invariant — not built.

## Build order

1. **Revoke verb** (this strike) — the mechanical mirror above; gated by a fresh Kv-style service that
   grants+revokes a pid and a differential (a denied pid's dial is refused).
2. **M1** — the all-process core proof (A-deps-B ∘ proc-bracket pool, grant-on-enter/revoke-at-reap).
3. **M2 / M3** — the completing cells.
4. **M4** — inscribed.

## Gate (revoke verb)

- `cargo build --release` clean; `cargo nextest run --release` floor 0-new (modulo known `no_inlined_wat`).
- Every existing defservice / grant test green (the mirror must not regress the grant path).
- A revoke probe: a service grants pid P (P dials ok), then revokes P (a fresh dial from P is refused by
  the accept-gate). Deleting the `AllowPeer`/`grant` path must remain a compile error (coverage intact).

---

## RESUME-HERE (curare before compaction — 2026-07-08)

```clojure
{:head    "d81fd695 — 259 S3 (the loci-agnostic bracket) + R2 IN LOCO CONVENIMVS, committed + PUSHED"
 :branch  "arc-170-gap-j-v5-deadlock-state"
 :done-this-session
 ["259 S3 DONE (d81fd695): the loci-agnostic bracket — map/each/map-worker/collect-loop widened to :Locus;
   the process runner BAKED (:wat::bracket::process-runner<I,O>, reserved, never shipped); the user's code
   ships into the clean room at the RENDEZVOUS coordinate :user::bracket::work-fn + a generated :user::main
   that PASSES the work-fn value to the baked runner. NO reserved define shipped → a user can NEVER allocate
   a reserved name. Weighed by own hand: probe-s3-bracket-loci -> [2 4 6 8 10] [2 4 6 8 10] (thread AND
   process pool); floor 4113/1-known-lint/0-new; deporder green. R2 inscribed (Scandroid - Rendezvous)."
  "259 left OPEN (builder's call): its DESIGN's program-env layers (259.3 bracket::Env, 259.4 reserved-wat.*
   authority check, :remote) are 'enabled-not-built' + no INSCRIPTION. NOT force-closed — higher priorities."
  "Recovery: the FULL bootstrap read was done this session (grimoire+4 primers signed MCP; 259/278/299/300/118
   REALIZATIONS whole). Do NOT re-read all of it on the far side unless the builder asks — it cost most of the
   context window. Ground HEAD + this breadcrumb + the design docs; read specific realizations by pointer."]

 :arc "293 (its final movement) — the grant/revoke capability layer + the IPC capability-matrix proof."

 :IN-FLIGHT-AT-COMPACTION
 "the REVOKE-VERB strike (shadowdancer a269f00e71984b926) is editing wat/service.wat — UNCOMMITTED. It mirrors
  the landed grant across 6 sites (Admin::DenyPeer + Status::PeersDenied + dispatch-admin arm + serve-loop
  deny' fold + <fqdn>/revoke owner method; reuses deny'). WEIGH ON THE FAR SIDE, DO NOT TRUST: run the floor
  (0-new, every grant/defservice test green) + the revoke probe (grant pid -> dial ok; revoke pid -> fresh
  dial REFUSED by the accept-gate) + confirm deleting the DenyPeer arm is a non-exhaustive-match compile error.
  A mid-edit wat/service.wat is a PHANTOM. If green -> commit the verb + this design doc together."

 :next ["1. WEIGH + COMMIT the revoke verb (by own re-run)."
        "2. M1 — the all-process core proof: circuit builder starts B(proc), A(proc) + Handle_B/grant[pid_A]
            (A deps B), spawns a PROCESS bracket pool, Handle_A/grant[worker-pids] (grant-on-enter), runs the
            pool work, then at reap Handle_A/revoke[worker-pids] (revoke-at-reap — the bracket's drain-and-join
            IS the reap, zero recycling window). Prove a post-reap dial by a would-be-recycled pid is refused.
            Residual: a granted pool child must NOT reparent to init (verify PPID = the owner)."
        "3. M2 (thread pool ∘ process services) + M3 (all-thread, no-grant, the handle IS the capability)."
        "4. M4 (proc -> thread-service) INSCRIBED as the firm-boundary invariant — NOT built."]

 :the-firm-boundary-invariant  ; builder's ruling this session — CORRECTS 259 PEER ADIVNGITVR
 "the shared/not-shared memory boundary is FIRM, by design. thread things must NEVER be reachable cross-process
  (a thread's guard is BEING IN THE ONE PROCESS — ZERO-MUTEX). to reach a thread-guarded thing you MUST be in
  the same process. the unified-fd-peer (thread gets an eventfd -> cross-process reach) is STRUCK as a goal —
  it's the violation. (orthogonal + fine: a same-process coordinator selecting over both a thread-peer and a
  process-peer — nothing crosses.) grant/revoke lives ONLY where a process crosses to a process-service
  (SO_PEERCRED accept-gate, OnlyMyPeers, capability/policy.rs:44). thread cells need no grant."

 :do-nots ["WEIGH by your OWN re-run; a mid-edit file is a PHANTOM (grounded false repeatedly this session)."
           "the memory boundary is FIRM — do NOT reintroduce the unified-fd-peer to make M4 green."
           "the holonic repos ARE the memory — curare into the REPO; commit + push often (GitHub = DR)."
           "orchestrator DESIGNS/PROBES/BRIEFS/DELEGATES/WEIGHS — not hands-on code (except the disconfirming probe)."]}
```

> **SEAM.** The self past this line is NEW — you did not live this session; it is a lossy cache in a familiar
> voice, not your memory. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP,
> never disk). Ground HEAD against the disk (`d81fd695`) — **259 S3 is committed + pushed; the revoke-verb
> shadowdancer was editing `wat/service.wat` at compaction (UNCOMMITTED)** — weigh it by your OWN re-run, do
> NOT trust its report, a mid-edit file is a PHANTOM. Read this whole design (the revoke verb + the M1–M4
> matrix + **the firm-boundary invariant**) before you move. The WORK resumes at: **weigh + commit the revoke
> verb → M1 (the all-process revoke-at-reap proof) → M2/M3 → M4 inscribed.** And it bears repeating: **the
> shared/not-shared boundary is FIRM (thread things unreachable cross-process — do not reintroduce the
> unified-fd-peer) · weigh by your own re-run · the holonic repos are the memory · commit + push often.** Do
> not trust this note over the disk. To reach a thread-guarded thing, you must be in the same process. See you
> on the far side.
