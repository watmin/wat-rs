# NOTE — Remote as a class; the service/host lifecycle; "AWS on a single CPU" as a method

**Status: HORIZON (2026-06-23).** A grounded north-star captured from a four-thread design debate with the
builder, *before* remote is built. Nothing here is implemented beyond what arc 291 strikes 1–2 shipped
(`init`) and strikes 3–4 will ship (facet split, hibernate/resume). This NOTE exists so the vision survives
the next compaction. The remote tier itself stays the **deferred door** (don't build the forcing function
until a real remote caller surfaces — 272 discipline). What this NOTE settles is the **contracts**, so the
future isn't precluded.

## 1. Remote is not a tier — it is a generative family (transport × trust)

The three-loci law (`project_three_loci_one_interface`) is really *N*-loci-one-interface. The loci are
`{thread, process, **remote-family**}`. Each concrete remote is one `CommAddress`/`CommListener` impl + one
`CommsPolicy` trust rung. The interface — peer, `Address'`, `Handle`, `launch`, the admin/data split — is
**invariant** across the whole family. New members register without touching the core (the narrow waist,
272). The builder: *"there's an unknown amount of remote — a class of loci that all wrap 'not on the same
host'."* It is open by construction.

- **Loopback-TCP** is the degenerate first member: same-host *locality*, remote *mechanism*. Instructive
  because **TCP loopback carries no `SO_PEERCRED`** (a UDS-only credential), so it *cannot* use the
  process-tier kernel-vouched trust — it is forced onto the mTLS/cert path even on one machine. Proof that
  "remote = mechanism, not locality." Ideal **test vehicle**: exercise the whole mTLS remote `Locus` on one
  box before owning two.

## 2. The admin/data facet split, spanning hosts (the load-bearing decomplection)

Two capabilities, two authorities, by ocap construction (see `DESIGN.md` § administrative-capability split
+ the verified archaeology of the deleted first attempt):

- **`Handle`** = *management* authority only (`stop`/`hibernate`/`promote`). Held by the spawner. **It is
  NOT a backdoor to data ops.** To call `Get`/`Increment`, even the owner `connect'`s to the client
  `Address'` and authenticates against the client policy **like any other caller**. Management authority and
  usage authority are *separate capabilities* (the Principle of Least Authority — POLA: a caller holds only
  the authority its task needs — at its purest; stronger than AWS IAM).
- **`Address'`** (client facet) = *usage* authority. Either ocap-handed (internal) OR **public**
  (published endpoint + policy gate).

**Per-facet independent `(transport × trust)`.** The two listeners can live on different transports with
different policies. Example (builder's): a daemon's admin interface on `:31337` (mTLS, trusted orchestrator
only); a spawned service's client interface on `:443` (the service's chosen client policy — "maybe mTLS,
maybe not," e.g. a public HTTP proxy). The client-facet `CommsPolicy` is a **service-author / admin-tooling
choice**, not inherited from the admin facet.

**The published-vs-handed nuance (guard this).** A public client endpoint has a *published, well-known*
address. That looks like the retired `socket-address'`/connect-by-name (272 step 5, `4e473da1`) — but that
was killed because the address was treated as a **secret** (guessable name = false security). A public
service is the **legitimate inverse**: the address is openly published *on purpose*, and security lives
**entirely in the policy** (mTLS cert / token), never in the address's unguessability. Re-introducing a
well-known client address is fine **iff** the contract locates client-facet authority in the policy, not the
name. Do not let someone "fix" it back into the retired pattern.

## 3. `init` binds the data-plane socket (the in-locus doctrine, extended)

A `:443` client listener is a non-serializable runtime resource — exactly like an `LruCache` or DB handle.
So `init` (running in-locus, on the host that will serve) binds it, by the same arc-291 doctrine that builds
State in-locus. It is correctly **absent from the hibernate snapshot** (a resource, not data), so `resume`
re-binds it via `init`-like reconstruction. (Open design choice: bind in `init` vs in the remote `launch`.
Lean: `init`-as-in-locus-resource, with the service *declaring* the client-facet binding.)

## 4. The daemon — a per-host control-plane agent that is itself a service

The first concrete remote: a long-lived process on the far host whose **ops are administrative**
(`spawn-service X`, `teardown Y`). So it inherits the facet split for free — admin interface on `:31337`
(trusted orchestrator connects to spawn), and it spawns services that expose *their own* client interface on
`:443`. The daemon holds the Handle of each service it spawns. For the orchestrator to manage a spawned
service, either the daemon **delegates** that Handle back (ocap transfer over `:31337`) or the orchestrator
**proxies** management through the daemon. Both valid — a choice, not a missing primitive. (This is the
Kubernetes shape derived from first principles: orchestrator = control plane, the HA store = etcd-as-
consensus, the daemon = kubelet, a service = a pod. Genuinely ours: ocap-secured, typed contracts,
hibernate-migration native.)

## 5. The graceful lifecycle decomposes onto primitives we have (composition, not new mechanism)

The builder's shutdown dance, decomposed: admin issues `stop` (Handle channel) → the serve loop **removes
the client listener from its `select'` set** (stop accepting new) but **keeps open client peers in the set**
(drain) → when the last client peer closes, **send the `stop` resp = "shutdown complete"** (this *is*
strike-3's `stop → resp` — the resp is the drain-complete signal) → admin **deregisters from the LB** →
**spawns the replacement** (`start`/`init`) → **registers it**. The drain gets a deadline for free: an
`(after drain-timeout :force-close)` arm in the same `select'` (292). Graceful drain = a serve-loop
`draining` state in the `select'` set we already have. Understood; not written.

Full lifecycle → primitive map:

| lifecycle event | primitive | status |
|---|---|---|
| spawn + build runtime (incl. data socket) | `init` in-locus | ✅ shipped (strike 2) |
| management authority, no data backdoor | the facet split | 🔨 strike 3 |
| graceful drain shutdown | `stop → resp` + `select'`-drain + `after`-deadline | 🔨 strike 3 + ✅ 292 |
| migrate / replicate the soul | `hibernate`/`resume` (EDN snapshot) | 🔜 strike 4 |
| wire two services together | capability introduction (third-party) | ✅ 272 |
| role assignment / promote | `init` arg + an admin op (handler mutates `role`) | ✅ / 🔨 |
| LB bind / drain / rolling-replace | admin op + virtual-endpoint routing | ⚠️ the one open contract |
| the daemon | a service whose ops are spawn/teardown | ✅ composition |

"Coherence is the engine" (260) at lifecycle scale: once the primitives are right, the lifecycle falls out
as composition — which is why it *feels* understood. It is.

## 6. The worked example — rete-DDB, and the loopback oracle

The looming concrete case: a **database-as-a-service** (persistent rete, LMDB-esque) with DDB-like
semantics — a write-leader + secondary-writer + reader; reads load-balanced across all three; writes to the
leader, who cascades. Clients (e.g. Clojure shipping EDN) interface the client facet and need not know it is
wat-backed.

Contract walk-through (all settled or it is implementation):
- **role = leader/secondary/reader** → an `init` arg → a `State` field. Role-change (promote) = an admin op.
- **replication chain (leader → secondary → reader)** → capability introduction: the control plane hands
  each instance its downstream peer's `Address'` over a trusted channel (272 `#wat-edn.cap`).
- **replication payload** → the `hibernate` snapshot OR the fact-delta; 291's serializable State *is* the
  replication unit.
- **client is cross-language** → the wire *is* EDN (Clojure-native); the typed `Op`/`Reply` *is* the API
  schema (255 reflection can generate the client-contract doc).
- **routing** → writes = single virtual target (bound to current leader, rebound on failover); reads =
  balanced(all-3). The DDB case **resolves the session-model fork**: reads are stateless-fungible
  (per-request), writes are leader-pinned. The admin tooling owns the binding; the LB is dumb.
- **consistency** → a *contract parameter*, not pure implementation: `ConsistentRead: true/false` is a flag
  on the read op (strong → route to leader; eventual → any replica). *Achieving* it is implementation;
  *exposing* it is contract. (The one place "implementation problem" and "contract problem" blur.)

**The method — dual-impl turned on distributed systems.** Build the rete-DDB on **3 loopback processes**
(an outer wat program holding the 3 admin Handles + wiring the replication chain via cap-introduction; an
external wat program exercising the client read/write loop). This single-CPU build is the **correct ORACLE**;
the real multi-host deployment is the impl; the differential is *same EDN in → same EDN out*, because it is
the **same service code on a different `Locus`** (loci-invariance — the narrow waist makes scaling a config
swap). Nested oracles: the rete engine is already wat-oracle-vs-Rust-kernel (278 R1); now the *deployment*
is loopback-oracle-vs-multi-host. Builder: *"I can build durable distributed solutions as an oracle to ref
against at scale."* This is **why** the contract-vs-implementation cut works: the contracts are
loci-invariant, so proving them on loopback proves them for scale.

## 7. Honest bounds — contract (settle now) vs implementation (defer)

- **Contract (settled or settleable now):** the facet split; per-facet trust; `init`-binds-data-socket;
  capability-introduction topology; the EDN client contract; the consistency flag; graceful-drain via
  `stop→resp`. **One genuinely-open contract item:** the **virtual-endpoint + routing-mode** (single-target
  vs balanced) and how the admin tooling rebinds it.
- **Implementation (deferred — these are NOT contract problems):** consensus, partition tolerance,
  replication lag, split-brain, leader election, failover timing. They appear only when the network is real;
  the contract does not change when they do.
- **What the loopback oracle proves:** the happy path, topology, role routing, the facet split, cap
  introduction, replication correctness, the client EDN contract, **and crash-stop failures**.
- **What it CANNOT prove:** **partition** (leader alive-but-unreachable), consensus timing, split-brain.
  Those only appear with a real network. The oracle gives "is the logic correct when messages arrive"; not
  "what happens when they don't." Do not let the oracle's green imply partition-tolerance — it is deferred.

## 8. The genuinely-unwritten list (none are missing primitives)
- the remote `Locus` impl (`:443` TCP+TLS `CommAddress`/`CommListener` + mTLS `CommsPolicy`) — the deferred door
- where the data socket binds (`init` vs `launch`) — a design choice
- the drain serve-loop `draining` state — understood, unwritten
- the LB virtual-endpoint + routing-mode contract — the one open contract item
- delegation-vs-proxy for orchestrator→service admin authority — a choice, both supported

## Pairs
`DESIGN.md` (the admin-capability split + the deleted-first-attempt archaeology) · `REALIZATIONS.md` R1
(the prophecy) · `project_three_loci_one_interface` · `project_wat_is_spec_rust_is_impl` (dual-impl) ·
272 (ocap, capability introduction, narrow waist, `NOTE-remote-mtls-trust`) · 292 (`select'` + `after` =
the drain machinery) · 255 R4 (CEK / serializable-K — the deeper horizon under hibernate).
