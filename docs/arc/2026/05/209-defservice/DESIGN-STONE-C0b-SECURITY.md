# Stone C0b — security addendum: the same-host lock (`SO_PEERCRED` allow-set)

> **Settled & LOCKED 2026-06-12.** The same-host (thread + process) security model for the
> defservice connection layer. This is what we build. Remote (mTLS) is the deferred north
> star — same grant *shape*, different identity *source* — built when `:remote` arrives.
> Connection mechanism: [`DESIGN-STONE-C0b-host-parametric-connection.md`](./DESIGN-STONE-C0b-host-parametric-connection.md).

## The model — visible socket, closed service

**No namespace isolation.** The socket is visible on the box; any process can `connect()`
and reach the accept path. We do **not** wall it off. The service **refuses to serve** anyone
it didn't authorize. Open door, closed service — a public port with authenticated access.

## The grant — admin authorizes a pid; the kernel is the unforgeable witness

The admin **already knows** its children's pids: `clone3` / the spawn *returns* the pid. It
does not *ask* who its children are — it *spawned* them, so it *holds* the pid. So a grant is
a fact the admin already has, enforced by one socket option:

- **Provision = "authorize pid X."** The admin spawned the client, holds its pid, and tells
  the service — over the trusted admin channel — that pid X may be served. The service adds X
  to an **allow-set**. No token minted, no secret distributed, no `/proc` walked, no `unshare`.
- **Enforcement = one `getsockopt(SO_PEERCRED)` at accept.** The kernel hands the service the
  connector's real `{pid, uid, gid}` — captured at connect, unforgeable, no handshake, **no
  `/proc`**. The service checks `pid ∈ allow-set`. A stranger reports *its* pid, not in the set
  → **refused, dropped, nothing returned.** It saw the door; it got no answer.
- **Deprovision = "revoke pid X."** Remove from the allow-set, drop the connection.

Two cheap, `/proc`-free layers from that one `getsockopt`:

- `ucred.uid == mine` — **coarse**: only processes running as me are even considered.
- `ucred.pid ∈ allow-set` — **precise**: only the ones I *provisioned* are served.

**Pid-reuse — the one real gap, and its close.** A provisioned child exits, its pid is reused
by a stranger who then connects. Closed by the admin's existing **reap discipline**
(RAII drain+join): the admin prunes X from the allow-set the moment it reaps the child —
before the pid can be reused-and-trusted. The window is "exited but not yet reaped," and the
admin reaps its own children promptly because it manages their whole lifecycle.

## Capability vs. identity — the tier duality (why it's not a token)

A capability is an *unforgeable proof you may use a thing.* Its form changes with the memory
boundary; its meaning does not:

| tier | the capability is… | unforgeable because… |
|---|---|---|
| **thread** (shared memory) | a **handle** (a `Value` reference) | you hold the reference or you don't |
| **process** (separate memory, same host) | a **`SO_PEERCRED` pid in the allow-set** | the **kernel** vouches for the pid; the admin authorized it |
| **remote** (separate memory, other host) — *later* | a **cert identity** (mTLS) | a **CA** vouches for the cert |

Same idea — *who vouched for this peer, and do I trust the voucher?* — with three vouchers:
you-hold-the-reference (thread), the kernel watched it get born (process), your CA signed it
(remote). **`SO_PEERCRED` is local mTLS**; the bearer token is avoided entirely, because the
substrate beneath the socket *is* the credential.

## What we proved on disk (this session)

- **UDS abstract namespace works** — `tests/nursery/probe_arc209_c0b_uds_abstract_spike.rs`
  (`c01dc077`): listen/accept/connect/round-trip on an in-memory socket, no fs entry.
- **Names are enumerable** — a "secret" abstract name shows in `/proc/net/unix` + `ss -x`.
  *Do not design around name secrecy.* Addresses are public, always.
- **netns-without-root works** — `unshare --user --net --map-root-user` isolates totally
  (host's 1391 unix sockets → 1 in the fresh ns), no sudo. **Proven available, deliberately
  NOT our mechanism** — we chose the visible-socket + allow-set model instead.
- **`SO_PEERCRED` works** — the kernel reports the peer's real `{pid, uid, gid}`, unforgeable,
  no `/proc`. The grant is decided from it.
- **pidns family check works** (`pid == 0` for a non-namespace peer) — proven, **NOT used**;
  the allow-set is `/proc`-free *without* a pidns, so we don't need one.

## What we build now (thread + process, together)

`listener'` / `accept'` / `connect'` (host-parametric: thread = crossbeam rendezvous,
process = abstract UDS) + the `SO_PEERCRED` allow-set grant on the process tier (the thread
tier's handle *is* its grant). Remote = the same allow-list shape over mTLS cert identity,
deferred to `:remote`.
