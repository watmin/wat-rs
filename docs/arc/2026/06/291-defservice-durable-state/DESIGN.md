# Arc 291 — defservice durable state: `init` / `stop` / `hibernate` / `resume`

**Status:** SCOPED (2026-06-22). Surfaced by arc 290's lru migration: the cache's
state is a thread-owned, non-serializable Rust `LruCache`, which defservice today
**cannot host** because it ships the State value over the wire (down as `state0`,
back on `:Stop`). The HORIZON note
(`272/NOTE-service-final-state-return.md`) already recorded the constraint:
*"a service's return value IS its final state; the symmetry forces state to be
wire-serializable."* This arc removes that forcing.

## The realization

defservice conflates two different things — **the State** and **what crosses the
wire**. gen_server keeps them separate. There are exactly THREE concepts:

| concept | must be EDN? | always present? | role |
|---|---|---|---|
| **`resp`** (return value) | yes | yes | the caller's answer from any op, incl. stop |
| **State** | **no — any type** | yes | the live thing; lives only in-locus |
| **snapshot** (hibernate/resume) | **only if State is EDN** | **only then** | durable hibernation + migration |

The wire only ever carries: **init-args in**, **resp out** (both always EDN), and
— *when the type allows* — a **Snapshot** for hibernate/resume. The live State
never crosses the wire by accident.

gen_server has three core callbacks; defservice built two:

| gen_server | defservice today | this arc |
|---|---|---|
| `handle_call/3 → {reply,R,S}` | ✅ `Outcome::Reply` | unchanged |
| `terminate` (+ state back) | ✅ `:Stop` ships `StopResponse[state]` | **split** → `stop → resp` |
| `init/1 → {ok,State}` | ❌ missing | **add** the `init` callback |
| (hibernation) | ❌ ships raw state always | **add** type-gated `hibernate`/`resume` |

## The lifecycle (target)

```
fresh:        start(locus, init-args) ─→ [running] ─→ stop ─→ resp        ;; ANY state type
hibernation:  start(locus, init-args) ─→ [running] ─→ hibernate ─→ Snapshot
                                                                  │ (EDN; persist anywhere)
                              later / another process: resume(locus, Snapshot) ─→ [running]
```

- **`init`** — a callback `(init-args → State)` that runs **in the service's locus**
  (thread: in the spawned thread; process: child-side, after it `recv'`s the EDN
  args it already receives). Builds the live State — including non-serializable
  resources (LruCache, sockets, DB handles) — where it lives. `start` takes
  **EDN `init-args`**, not a pre-built `state0`.
- **`stop`** — terminates, returns a serializable **`resp`** (may be `nil`). The
  return value is **decoupled from the State**. This is how a caller gets a final
  value regardless of what the State is made of.
- **`hibernate`** — terminates and hands the State out **as a Snapshot** (the State,
  EDN-encoded). Emitted/valid **only when State is EdnRepresentable**.
- **`resume`** — the dual of `start`: a **fresh spawn** whose initial State is the
  **deserialized Snapshot**, **bypassing `init`** (a snapshot-able state is pure
  data — no resources to rebuild). `resume : snapshot :: start : init-args`. NOT
  injection into a live service (no hot state-swap).

**The headline:** hibernate → kill the holding process → `resume` in a new process
→ **the service cannot tell the difference**. Service code is byte-identical across
`start` and `resume`; it only ever observes "I hold State X and I'm serving." That
makes a defservice location- and time-independent: durable actors + transparent
process migration, achieved purely by the EDN-only-on-the-wire discipline.

## The corollary: the remote locus × hibernate/resume = live migration (across HOSTS)

defservice is locus-agnostic by law (it never names thread/process/remote;
`DESIGN-STONE-6b-ii-beta-IDEALIZED.md`). The remote locus is the **perpetually-
deferred** door (`wat/spawn.wat:16` — *"⛔ THE REMOTE DOOR IS PERPETUALLY AWAITING
ITS KEY"*; remote = one new `CommAddress`/TLS impl + one `CommsPolicy` rung, zero
defservice edit). A **Snapshot is pure EDN** and `resume` is **locus-parametric**,
so the same EDN bytes travel identically over shared memory, a pipe, or an mTLS
socket. Therefore:

```
hibernate on host A → EDN Snapshot → (mTLS) → resume((remote B), snapshot) on host B
```

is the SAME operation as `resume((thread), snapshot)` — only the `Locus` impl
differs. The service is alive on B and cannot know it moved. **Cross-host live
migration falls out for free** the day the remote locus exists — *because* the
narrow waist was held shut until now. `init`/`hibernate`/`resume` IS the key the
remote door was waiting for.

**Standing discipline (do NOT violate in this arc):** arc 291 does **not** build
remote (`feedback_dont_build_the_forcing_function` — build it when a remote caller
surfaces). Arc 291's obligation is to be **remote-ready by construction**:
- `hibernate` produces a **pure EDN** Snapshot — no transport, no locus, no opaque
  handle baked in.
- `resume` is **strictly locus-parametric** — it takes `(locus, snapshot)` and never
  inspects or special-cases the transport.
- the snapshot path never touches `CommAddress`/TLS/pid — those join later as a
  Locus impl, not as an edit to snapshot/resume.

If those hold, remote migration is a drop-in `Locus` later, with zero edit to
`hibernate`/`resume`/the macro.

## Sub-strikes (sequenced)

0. **DESIGN** (this doc) — the contract.
1. **RED probe** — a counter that `start`s, increments, `hibernate`s, has its
   process killed, then `resume`s in a fresh process and continues — asserting the
   resumed value. RED at HEAD (no init/hibernate/resume exists). Commit before build.
2. **`init` callback** (THE KEYSTONE, unblocks arc 290) — add `:init [args] <body→State>`
   to the defservice macro; `start [locus init-args]` runs it in-locus on BOTH the
   thread launch (`wat/spawn.wat` ThreadOpts) and the process launch (child runs init
   after `recv'`ing the EDN args). After this lands, the lru/holon-lru cache migration
   (arc 290 Class A) is unblocked — non-serializable state is constructible in-locus.
3. **`stop → resp` decouple** — split today's `:Stop[state:S reply:R]` /
   `StopResponse[state]` into `stop → resp` (always EDN, no raw state). Preserve
   "return value IS final state" *for EDN state* by letting the author project the
   state into `resp`; non-EDN services return an honest serializable summary (or nil).
4. **`hibernate` / `resume`** (the durable-actor capability) — type-gated on
   `EdnRepresentable`. **RISK / PROBE:** there is no wat-level `EdnRepresentable`
   protocol to bound on today (only the Rust trait in `edn_shim.rs`). Decide the gate:
   - (a) **type-checker gate** (preferred) — `hibernate`/`resume` are always emitted,
     but their bodies EDN-encode/decode the State; if EDN-encode of a non-EDN type is
     a **compile error**, the gate is automatic (calling `cache/hibernate` won't
     type-check). **Probe first:** does encoding / sending a non-EDN value (e.g. an
     `LruCache`) over a peer fail at **compile** time or **runtime**? That answer
     decides whether the gate is clean (compile) or needs a protocol bound.
   - (b) **macro-time gate** — emit `hibernate`/`resume` only when all state-field
     types are EDN-able. Harder (macro lacks type info); fallback only.

## Out of scope (affirmative cuts)
- Hot state-swap / live-injection into a running service (different, hairier op).
- Snapshot persistence format / storage — Snapshot is just EDN; persistence is the caller's.
- Generic `<K,V>` defservice — orthogonal (arc 290 stays monomorphic).

## The administrative-capability split (strikes 3–4) — owner-only `stop`/`hibernate`

**Added 2026-06-23 (builder): *"how can we restrict this such that stop/hibernate are 'administrative'
APIs — only the thing who creates the service instance can call them?"* This amends strikes 3–4 below;
the prior contract stands, this layers authority on it.**

**The gap today** (grounded — `service.wat:407-440`, `575-600`): `stop` is folded into the **client**
`Op` enum, and the generated `stop` method takes `[c <- client-peer-ty]` — the peer from
`connect'(Handle/addr)`. So **any holder of the dial-address can stop the service.** Ambient authority.
Hibernate would inherit this if folded the same way.

**The cure — two capabilities, two authorities, by construction (ocap / POLA, 272's law):**

| capability | who holds it | confers | analog |
|---|---|---|---|
| **`Address'`** (dial-address, handed out via `Handle/addr`) | clients | data-plane ops (Get/Increment) | S3 `GetObject` · a scoped client grant |
| **`Handle`** (returned by `start`) | the creator only | control-plane ops (`stop`/`hibernate`) | the bucket owner's IAM role · the Erlang supervisor |

**Mechanism:** the administrative ops come **off the client `Op` enum** onto a separate surface reachable
**only through the Handle**. `start` mints *two* listener/address pairs — the public client address
(exposed via `Handle/addr`) and a **private admin address kept inside the Handle, never exposed**. The
serve loop `select'`s over both listeners; `stop`/`hibernate` are **Handle methods** travelling the admin
channel. A client holding only the dial-address *cannot name the admin door* — it's a different,
unforgeable, never-handed capability. **Not a runtime permission check — authority is possession of an
unforgeable reference** (272: no ambient authority, no forge-from-name). Crosses the process/remote wire
free: `Address'` is already a portable `#wat-edn.cap` capability (272 6a).

**Two properties that fall out:**
- **Delegation composes, explicitly.** Admin authority *is* a capability, so the owner can hand it to a
  supervisor/deploy-tool by transferring it — explicit transfer, never ambient.
- **It is "AWS on a CPU" exactly** — data-plane (`Address'`) vs control-plane (`Handle`); and the Erlang
  supervisor (children don't stop each other; the owner owns the lifecycle).

**Where it lands:**
- **strike 3 (`stop → resp`)** absorbs this: `stop` moves off the client `Op` enum onto the Handle admin
  surface (it is already "reshape stop," so the authority split rides along).
- **strike 4 (`hibernate`/`resume`)** is then born owner-only: `hibernate` is a Handle method; `resume` is
  `start`-family (the caller creates the instance and receives the Handle), owner by construction.
- **`init`** (strike 2, shipped `d5d71766`) — unaffected; `init` is inherently the creator's (part of `start`).

### Lessons from the deleted first attempt (verified archaeology, 2026-06-23)

The builder asked for the history of the *first* attempt at restricted-admin tooling, deleted, so we
don't repeat its mistakes. Found + verified against git/disk:

**What was attempted (arc 203, May 2026):** a hand-rolled per-service **Admin/Client capability split** —
an `Admin` `struct-restricted` holding a `server-id` UUID **secret-witness** + the admin channel; a
`Client/User` struct carrying that UUID; `:admin {Provision/Deprovision/Stop}` vs `:user {Get/Increment}`
sections; a `Wire Admin|User` enum multiplexing both planes over one stream. Commits `26c92981`,
`e7aa671b`, `b1fed2be`, `cd6f2617`.

**Why it was deleted — the load-bearing lesson** (`DESIGN-REGROUNDED-2026-06-12.md:17-20`, verbatim on disk):
> *"Admin existed for one job: PERMISSIONS … The substrate now answers that directly, per tier — thread =
> you hold the handle; process = your pid is in my SO_PEERCRED allow-set (kernel-vouched); remote = your
> cert chains to my CA (mTLS). A hand-rolled permission system on top of a real one is redundant ceremony."*

So `Admin`/`User` caps, `Provision`/`Deprovision`, the `server-id` witness, and `Wire` multiplexing **all
collapsed**. Further deletions: the restricted FORMS `struct-restricted`/`def-restricted`/`defn-restricted`
HARD CUT into `{:restricted-to}` metadata-maps (arc 241 Stones 241.8 `f6cb564f` / 241.14 `839cf9e6`,
retirement table); `socket-address'` guessable-name rendezvous annihilated (272 step 5 `4e473da1`);
`AnyOfMyUser` (euid-only connect gate) annihilated + the "autobind names are unguessable" premise retracted
(272 6c.2 `ed633891` — autobind is `%05x` = 2²⁰, brute-forceable, NOT a secret).

**THE PRIMARY LESSON → the contract for this attempt:** do **not** hand-roll a permission system (no
`server-id` witness, no `Provision`/`Deprovision`, no token). Lean on the substrate's **per-tier authority
that already exists and survives** (`src/capability/policy.rs`, `src/comms/process.rs`): thread =
**handle-possession**; process = **`SO_PEERCRED` allow-set + `OnlyThisPeer{pid}`**; remote = **mTLS**. The
two-capability split (`Handle` vs `Address'`) provides ONLY the admin/data **separation** (the ocap facet);
the substrate provides the per-tier **WHO**. They compose — the facet says *which door*, the substrate auth
says *who may walk through it*. This is why the second attempt is better: the real auth now exists, so the
ceremony the first attempt hand-rolled is replaced by leaning on it.

**The avoid-list (verified mistakes that could recur — guard each):**
1. **No Mutex on the Handle / admin listener.** A Handle is single-owner by construction → `ThreadOwnedCell`
   / plain ownership, never `Mutex` (the arc-209 `Mutex<HashSet>` → `ThreadOwnedCell` lesson, `06bfdf92` —
   a Mutex lies about a single-owner access pattern).
2. **Never claim the admin address is "unguessable."** Autobind is `%05x` (2²⁰). Security = the admin
   address is **never exposed** (only the client address via `Handle/addr`) + **pid-stamped**, NOT secrecy.
3. **Never add a `Handle/admin-addr` accessor or leak the admin address.** Exposure collapses the
   by-construction guarantee back to a runtime check — the exact forge-from-name failure 272 step 5 killed.
4. **Do not multiplex admin+data onto one listener with a runtime "which kind of message" check.** That
   reintroduces ambient authority (a dispatch decision instead of possession-of-address). Two listeners.
5. **Pid-stamp the admin address at mint** (`OnlyThisPeer{minter_pid}`, 272 6c.2 `ed633891`) so the
   process-tier admin connect gate is symmetric with accept — else it falls back to the annihilated
   `AnyOfMyUser`.

(Status: PROPOSED contract for strikes 3–4 — the two-address Handle split, leaning on substrate per-tier
auth, guarded by the avoid-list. To be confirmed before the strike-3 draw.)

## Done = the gate
The RED probe (sub-strike 1) goes GREEN: counter hibernate → process-kill → resume →
continue, across processes, asserted. `service-locus-parity.wat` still green. And the
arc 290 cache migration compiles against the new `init` surface (non-serializable
state hosted, no `Option`/`ensure-cache` hack).
