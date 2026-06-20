# NOTE — the overlay/COW read path (⑥) + the distributed-service horizon (forward, NOT-NOW)

Two things, captured live (2026-06-20). Part 1 is the near-term read-path model for ⑥ (the persistent-WM
service) — it will fold into ⑥'s DESIGN-STONE when that stone is drawn. Part 2 is a **forward horizon the
builder asked to record but explicitly is NOT building now** — its only claim on the present is an
*evolvability constraint*: build ⑥ so this can grow out of it without a rewrite.

---

## Part 1 — the read path is a copy-on-write overlay (near-term, ⑥)

The engine is value-semantics (`Session → Session'`, R1/R5) over persistent collections (rpds HAMTs, stones
0a/0b). That makes speculative reads an **overlay / copy-on-write** layer, not a mutation:

- **The base session is the read-only lower layer.** It is an immutable value; nothing a read does can touch
  it.
- **A what-if fire is a writable upper layer.** `fire` over the base returns a new root whose new nodes
  *point into* the shared base. Read what it derived, then **discard it** — the base is byte-identical
  afterward. The discard frees only the new paths (structural sharing).

The builder's image: **Iron Man building the Mark I in the cave** — the armor (overlay) is built from the
cave (base) and you step in and out of it, the cave untouched. With one way it is *better* than the cave:
sharing is a non-destructive read, so the base's "materials" are never consumed — you can build **N armors
from the same untouched cave at once**, and stack them (a what-if on a what-if on the base). The result is a
canonical session with a *fan* of independently-discardable overlays.

This gives **snapshot isolation with no MVCC and no locks**: immutability is the untouchable base, persistence
makes each overlay cheap, and the lockstep single-actor (ZERO-MUTEX) sequences the writes.

The ⑥ read path, concretely:
- `query` — read-through the base; no overlay; `state' == state`.
- `:what-if [extra-facts]` / `:try-rules [rs]` — first-class messages: fire over a shared snapshot, read the
  derivation, drop the overlay. The base (the service's committed `Session`) is never mutated. This is the
  AWS triage loop from R5, made a service verb.

(plain `query`, value-semantics, persistence, the lockstep actor = **built**; `:what-if` as a service message
= part of ⑥, **unbuilt** — zero new substrate, pure composition.)

---

## Part 2 — HORIZON (forward, NOT-NOW): an eventually-consistent distributed service, like DynamoDB

> **Status: a thought to hold, not work to schedule.** The builder, verbatim intent: *"we're not building it
> now… but we need these kinds of deployments to be evolvable into."* This is **not an arc**; it is a horizon
> recorded so the present design does not foreclose it. Do not open an arc for this from this note.

The same primitives (value-semantics `Session`, the `{facts,rules}` snapshot blob from R5, the overlay/COW
ref, the lockstep actor, spawn-programs over UDS/process) compose into a leader-based replicated service:

**Write path (leader-coordinated, durable-before-ack):**
1. The **write leader** does the work and updates its local ref as a **revertable overlay** (the COW stage —
   commit-or-revert, never a destructive in-place write).
2. It notifies its **first peer**, which performs a **blocking (synchronous) write**. Only after that peer
   acks does the leader **commit** its staged ref and **return to the client**. So an acked write is never a
   single point of failure on the leader — **replication-on-write guarantees ≥1 durable replica beyond the
   leader exists at ack time.**
3. The leader then replicates the same write to the **remaining peers asynchronously** (eventual-consistency
   fan-out).

**Read path (tunable consistency):**
- **Eventually-consistent reads** off any replica's struct (cheap, scalable, may lag).
- **Strong/linearizable reads** from the **write leader**, which is single-threaded *while writes are
  happening* (the lockstep actor) — so a leader read sees a consistent, committed view.

**Topology:** the leader and peers are **spawn-programs** communicating over the existing rendezvous channels
(arc 214 / C0b: `select'`/`poll'` over UDS/process). The lockstep blocking rendezvous *is* the synchronous
replication transport.

**Prior-art collisions (noted honestly, not claimed novel):** this is primary-backup / chain-replication in
shape; the sync-one-then-async-rest + tunable strong-vs-eventual reads is DynamoDB's model (the builder named
ddb); strong-reads-from-the-leader is the leader-lease/Raft-read pattern. The derivation here is from our own
primitives; the collisions are recorded per practice, not as borrowings.

---

## Part 3 — the evolvability constraint on ⑥ (the ONLY present-tense obligation)

Build ⑥ as a single-node `defservice` holding a `Session`, but shaped so the distributed version is an
*evolution*, not a rewrite. Concretely, do not foreclose:
- **Writes as stage→commit**, not destructive mutation. ⑥'s write handlers should advance the ref via the
  COW overlay and *commit*, so "commit" can later become "commit after first-peer ack." (Falls out of value
  semantics — just don't design against it.)
- **The replication unit already exists**: the `{facts, rules}` snapshot blob (R5) ships over the wire as EDN;
  deltas are inserts/retracts. Keep writes expressible as a replayable op + a snapshot, not as opaque
  in-memory state.
- **Strong-read = leader-read** is the lockstep actor we already have; **eventual-read = replica-read** is a
  reader holding an older immutable `Session` value (RCU-style, free from persistence). Don't bake an
  assumption that there is only one reader of one mutable store.

**The one genuinely-new leg** the distributed version needs and we do NOT have: **cross-host transport**
(TCP/TLS). ⑥ deliberately dropped HTTPS/TCP (single-node, UDS/process only). The horizon is the place that
transport earns its arc — when, and only when, the builder decides to build it. Everything else above is
composition of what exists.

---

## Part 4 — the crystallized service architecture (HORIZON; the target the interface must aim at)

> Derived across a long design dialogue (2026-06-20, the "inquisitor" thread). Still **not-now, not-an-arc.**
> Its present-tense claim is only the **interface constraint** at the end (the four properties ⑥ must have).

### The store is one abstraction with two substrates (NOT a rewrite)
rpds and LMDB are the **same abstraction** — a persistent versioned tree + an atomic root = *an atom holding
an immutable map*. LMDB's commit (dual-meta-page flip) *is* the atomic root swap; an LMDB read-txn *is* a
point-in-time frozen ref; a what-if overlay layers in-memory on top of either base. They differ only in
substrate: rpds = heap pointers / refcount / in-proc / ephemeral; LMDB(/`redb`, pure-Rust) = mmap offsets /
reader-table / cross-proc / durable. So the design is: **one persistent-store `defprotocol`** (`get`,
`insert→new-version`, `snapshot→frozen-ref`, `commit→atomic-swap`); the rete engine written against it; **two
backends**. The opt-in *is* the backend choice; the client interface never changes.
- **rpds backend** — in-memory, single-proc, fastest, ephemeral. *(⑥ as scoped.)*
- **LMDB/redb backend** — shared mmap (one physical copy per host = the memory win), cross-proc, durable, and
  **larger-than-RAM** (it's an mmap'd file; the kernel demand-pages the hot working set into RAM and evicts
  cold; capacity is bounded by disk + address space, not RAM). This turns the engine into an *actual durable
  deductive database*. Honest caveats: fast only for the hot working set (cold access = page-fault/disk
  latency); the engine must read the on-disk structure directly (materializing into rpds pulls it all into
  RAM, defeating it); COW write amplification under sustained writes. **Two offerings, one codebase:**
  ephemeral rete-as-a-service and persistent/durable rete-as-a-service.
- **Overlays stay in-memory (transient rpds) on top of the base snapshot** — speculation never hits disk.
  Durable shared base + ephemeral overlay.
- **LMDB gives single-node crash-consistency WITHOUT a separate WAL** (COW B+tree + atomic meta-flip). So the
  journal (below) earns its place for **replication + cross-host recovery**, not basic single-node durability.

### Replication = snapshot + journal (the universal pattern)
Full **snapshot** + incremental **journal** side-by-side = checkpoint+WAL = Postgres (base backup + WAL) /
Raft (snapshot + log) / DDB. **Bootstrap a fresh/recovering host:** install the latest snapshot → replay the
journal from that point → catch up to live → join. That's Raft state-transfer / Postgres restore-and-replay.
The journal is also what a consensus protocol (Raft) replicates — so journal-now evolves *directly* into
distributed-strong-consistency-later.

### Topology, the deployment shape
- N **single-threaded** procs per host, each pinned to a core (Redis's model — single-threaded execution,
  scale by instances-per-core, not threads). Single-threaded **dissolves the shared-atom problem** (no
  concurrent shared read → no atom → stays shared-nothing within a proc).
- Procs on a host share the host's **LMDB** (one physical copy; mmap). Reader procs = concurrent MVCC
  readers; the writer host = **1 active writer + 1 hot standby** (LMDB is single-writer per environment).
- **N procs sharing one accept handle** (the prefork model): a parent binds the listener and spawns N
  siblings sharing that listen fd; each calls `accept()`; the **kernel** load-balances connections across
  them and the kernel accept-backlog *is* the bounded work queue (no mutex, no user-space queue). Primitives
  largely exist: fd-pass-at-spawn (`process/verbs.rs:625`), `listen_fd` exposed (`kernel/listener.rs:59`),
  non-blocking accept. Missing = the *pattern*, not the primitives. (Cross-running-proc fd handoff would need
  `SCM_RIGHTS`, which we lack — but inherit-at-spawn sidesteps it.)

### Concurrent sessions + the request layer
- **Keyspace tuple** `(account, [group], session, [partition])` = multi-tenancy (DDB's `(table, key)`) AND
  the write-concurrency knob: each partition with its own LMDB environment = its own writer → concurrent
  writes across partitions; the granularity chosen *is* the concurrency granularity.
- **Write side = a partition ROUTER** (hash key → that partition's single-writer owner; NOT round-robin —
  single-writer forbids it). DDB's request router. **Read side = an LB** round-robin over replicas (eventual)
  + a direct path to the writer (strong). Router for writes, LB for reads — that distinction *is* the
  consistency story.

### Auth = capability-scoped keyspace
The keyspace tuple is the **authorization unit**: a grant scopes which `(account, group)` → sessions, or which
partition, a client can reach ("table access" vs "partition access" = grant granularity). Enforcement
substrate already exists in pieces: `SO_PEERCRED` + the arc-272 rendezvous capability (ocap: who-are-you at
connect) + `:restricted-to` (namespace whitelist).

### THE present-tense obligation (the only thing this note asks of now)
This whole architecture is the **"perpetually distant deployment"** — the same stance as spawn-program's
`:remote` locus (arc 259): the interface *reserves and targets* the distributed shape without building it. So
**⑥'s client interface must have four properties** so the entire DDB-shaped future is a backend/transport
swap with clients untouched:
1. **Tuple-keyed, location-independent addressing** — `(account, session/partition)`, so a key can later
   resolve to a remote owner.
2. **Consistency-explicit reads** — a read declares `strong | eventual`, so a router can later choose
   writer-vs-replica.
3. **Store-protocol-backed** — rpds now, LMDB later, swapped underneath.
4. **Transport-agnostic** — UDS now; router/LB/TCP later.

Build ⑥ on the rpds backend with an interface carrying those four, and the reactor, the LMDB backend, the
replication chain, the router/LB, concurrent sessions, and the datalog second-offering are all **additive
behind the same interface**. Nothing above is scheduled; the four properties are.

### Second offering (own future arc): datalog-as-a-service
Arc 287 (WorkQuery v2) — a Datalog query surface over the same rete kernel ("query = a transient rule; rete
*is* semi-naive datalog"). One kernel, two services. Tracked there, not here.

---

## Part 5 — the product thesis: eBPF for application rules (+ the three-boundary safety model)

Because registered filter/accumulator fns are **pure ∧ deterministic ∧ typed** (the 6a fence), users can
register their *own* rules and fold fns on a session and the service just runs them — **users program their
own network/anomaly policy, with their data, on our compute.** This is structurally **eBPF**: a user supplies
a program, a **verifier proves it safe**, and the trusted context runs untrusted code at line rate. eBPF's
verifier ⇒ our **purity fence**; the kernel ⇒ our **rete engine**. holon's origin (line-rate anomaly
detection) and rules-engine-as-a-service are the *same machine*: customers bring policy (their p99/stddev
thresholds, their custom folds), ship data as facts, the engine runs their logic correctly at scale.

### ⛔ The resource-safety caveat (load-bearing — do NOT ship without it)
**Purity buys EFFECT-safety for free (no IO, no exfiltration, no mutation, no ambient reads) — the scary part.
It buys ZERO resource-safety.** A pure, deterministic fn can still:
- **loop forever** (purity says nothing about termination — the halting problem),
- **allocate enormous memory**,
- **be arbitrarily expensive** (O(n³)+).

On the single-threaded-per-core model, one tenant's unbounded fn **blocks the whole core's loop** — a
trivial multi-tenant DoS. eBPF is instructive *exactly here*: its verifier is **stricter** than ours — it
*also* proves **termination (bounded loops) + memory bounds**, which our purity fence does not. **The gap
between our fence and eBPF's verifier IS the resource-safety requirement.** Running strangers' code on shared
infra needs **all three boundaries**:
- **Effect isolation** — the purity fence (`is_pure_expr`). *Built.*
- **Data isolation** — capability/keyspace scoping (a tenant fn sees only its session's facts;
  `SO_PEERCRED` + arc-272 capability + `:restricted-to`). *Designed.*
- **Resource isolation** — a **fuel / instruction budget + memory quota** on every user fn. ***Not built —
  REQUIRED before any untrusted multi-tenant execution.***

### Decision (2026-06-20): STATIC totality fence — eBPF's road, not runtime fuel
Resource-safety is a **compile-time / at-registration guarantee**, not a runtime abort. Solve it eBPF-style:
**restrict user fns to a total (bounded) fragment and reject the rest at registration.** A static
instruction-count guarantee for *arbitrary* fns is impossible (= the halting problem); eBPF doesn't solve
halting, it **rejects what it can't prove terminates** — so do we.

- **`total?` = a THIRD axis on the 6a fence** (`pure? ∧ deterministic? ∧ total?`), same structural walk. 6a's
  transitive walk is *already cycle-safe* (tracks `seen` to not loop the checker on recursion); `pure?`/`det?`
  *tolerate* recursion, **`total?` flips it: cycle detected (recursion) → REJECT.** The recursion detector
  already exists. If wat's only iteration is bounded combinators (`map`/`foldl`/`range` over finite
  collections — `for` killed arc 249, threading is map-based; CONFIRM no `while`/unbounded-loop primitive),
  bounded iteration is bounded by construction → nothing else to reject. So `total?` is nearly free given 6a.
- **Imposed on our own tooling NOW** (all fenced user fns: `where`/`:test`/custom accumulators) → the remote
  deployment inherits it for free (the "perpetually distant" pattern). Filters/accumulators are *naturally*
  total (bounded folds over the finite gather) — the restriction costs users ~nothing, and it's the
  project's ethos (make the illegal state *uncompilable*, not caught at runtime). **Warrant: a whole rete
  already shipped in eBPF's bounded fragment at XDP line-rate — the bounded road is proven sufficient for
  rules.**
- **Guarantee + the one knob:** `total?` ⇒ termination + cost **O(gather-size)**. eBPF's bound is a *constant*
  (loops bounded by a constant); ours is bounded by N (the gather). For a hard *constant* ceiling, also **cap
  the gather size** per accumulate. O(N)-over-finite-N is already a real bound; the cap is the DoS backstop
  for a huge-but-finite fold (add if/when a workload needs it).
- **Dynamic / runtime fuel: REJECTED (not deferred).** It only buys running a Turing-complete user language
  with a runtime abort — no use case here (filters/accumulators are bounded by nature), and real costs
  (partial work on abort, head-of-line block until fuel drains, fuzzier reasoning, worse failure mode).
  Zero gain, real problems.
- **CEK is orthogonal** — it remains for its own job (stack-safety + hibernation, arc 261), NOT mandated for
  resource-safety. The earlier "fuel needs a stepped/CEK evaluator" framing was wrong; the static fence needs
  no evaluator change at all.

So the *verifier* = the 6a fence with the `total?` axis (+ optional gather cap), all at registration.

### Edge-hardening (HORIZON — the standard multi-tenant checklist; NOT near-term)
The fire/compute threat is closed (metered fire + `total?` + add-and-fire transaction). An adversarial sweep
of a malicious *local* user (⇒ covers all remote) leaves **5 edge mitigations — all known-problem /
known-solution, all at the surfaces *around* the fire, none novel.** They matter **only for untrusted
multi-tenant** (the distributed-service future), **not** for the near-term engine or the lint-rule first-app.
Recorded so the threat model is complete; **not promoted to current work.**
1. **Fact/field SIZE cap** — the closure-cap counts tokens, not bytes; one huge fact (a 1 GB string) blows
   memory at low count. Per-fact/field byte limit (cf. DDB 400 KB/item). *Data layer.*
2. **Per-tenant rate limit + fair scheduling** — the metered fire bounds each request, not the *rate*; a
   tenant sending bounded-but-constant add-and-fires monopolizes the single thread. *Scheduling layer.*
   (Inversion: **harder locally** [shared proc] than remotely [router shards tenants] — solving local
   over-solves remote.)
3. **Connection caps + I/O read timeouts** — slow-loris / fd exhaustion (`mora`: time via the wire).
   *Transport layer.*
4. **Verifier cost bound** — a giant rule set / deeply-nested AST attacks the fence itself; AST-size /
   rule-count caps at registration (eBPF's verifier has explicit complexity limits). *Verifier layer.*
5. **Hash-flooding resistance** — attacker-crafted colliding join keys degrade the keyed join within budget;
   seeded/SipHash for attacker-controlled keys. *(Verify: does `rpds` `_sync` map use a seeded hasher?)*
   *Algorithmic-complexity layer.*
- **(persistent-only, out of ephemeral scope):** per-tenant **storage quota** (valid large batches fill
  disk on the LMDB backend; ephemeral dies with the proc).

**Add-and-fire = the transactional write contract:** `add-and-fire(mutations)` applies the batch (inserts +
retracts) as a candidate overlay on the committed session, fires it metered, then **commits on success /
discards on ResourceExceeded** — so a malicious batch *never commits* (the committed state is always
last-good; the fire is the gate; rollback is free via value-semantics). Batch size = a **performance** knob
(~1k default, tunable; our per-fact cost is an append, not DDB's cross-partition write); the metered fire is
the **safety** floor, independent of batch size. Reads (query/what-if) stay on committed snapshots. This is
LMDB's read-txn + write-txn model, and ⑥'s write interface.
