# Arc 272 — rendezvous is an inherited capability, not a discovered name

> Opened 2026-06-15. A pivot, not a feature: the process-tier rendezvous is built on a **fixed,
> human-chosen abstract UDS name**, which is both **collidable** (EADDRINUSE) and **forgeable**
> (squat-first), and its "mTLS" is one-directional. This arc **annihilates the collision class
> structurally** (no shared fixed namespace can exist) and makes identity **mutual by construction**.
> Grounded against HEAD `41e01b8d`. The thread tier (arc-209 stone 4a, shipped) already embodies the
> doctrine; this arc brings the process tier to the same standard and supersedes the planned "4b".

## The flaw (what we annihilate)

`listener'(process)` binds a **fixed string** in the Linux *abstract* UDS namespace
(`"wat.arc209.c0b3bb.svc"`), and `connect'` dials it by that string (arc C0b.2d "connect-by-name").
The abstract namespace is **kernel-global per netns** — a single shared table of strings. Consequences:

- **Collidable.** Two binders of the same name → `EADDRINUSE`. This is the `c0b3bb_bounced`
  "flake": its two tests bind the same constant and race. Not flakiness — a shared global name
  colliding with itself.
- **Forgeable.** Any process can bind `"…svc"` *first* and the client dials the impostor. A fixed
  public name is squattable.
- **Half-mutual "mTLS".** SO_PEERCRED is read only at `accept` (runtime.rs:25005): the **server
  checks the client** (allow-set + euid, C0b.3b-b). The **client never checks the server**
  (`address.rs::connect` just `UnixStream::connect_addr` + wrap). The comment "SO_PEERCRED is local
  mTLS" (comms/process.rs:132) **overclaims** — it is local TLS in one direction.
- **Premature.** The whole name-discovery stack (`socket-address'`, connect-by-name, allow-set,
  SO_PEERCRED-as-defense) has **zero production consumers** — only the `c0b3bb`/`c0b3aii` tests
  exercise it; defservice has no process path yet. It was built ahead of a stranger-rendezvous use
  case **we do not have**. ([[feedback_dont_build_the_forcing_function]])

## The doctrine

**Every rendezvous we actually have is parent→child within one fork lineage.** The parent *mints*
the child and **declares the sync point at spawn (creation)**; the child receives its end **by
inheritance**. The parent has perfect knowledge (it forked the child; it knows the pid); the child
knows the parent (`getppid`). There is no third party and **nobody who can be misinformed**.

So the rendezvous is **an inherited capability, never a discovered name.** Thread tier: the captured
crossbeam `Sender` (you cannot forge a Sender — "the crossbeam handle IS the grant",
listener.rs:19084). Process tier: an inherited fd / a kernel-minted unguessable address (you cannot
forge an fd you did not inherit, nor dial a name you were never told). Same shape, two transports.

**The collision class is deleted, not guarded** — there is no fixed namespace to collide in, so
`EADDRINUSE` becomes *unreachable code*, not a handled error. The mistake has no constructor
(extirpare, top rung). And identity is **mutual by construction**: server checks client AND client
checks server. *Annihilate collisions — never deal with this problem again.*

## Grounded kernel facts (probe: `/tmp/peercred_probe.c`, run 2026-06-15)

| mechanism | name? | collision? | SO_PEERCRED returns |
|---|---|---|---|
| **socketpair** (inherited across fork) | none at all | impossible (no namespace) | **the creator** on *both* ends (parent pid on parent's *and* child's end) — useless for peer identity |
| **autobind listener** (`bind` empty → kernel assigns; accept/connect) | kernel-minted, unique, unguessable (`\0`+5 random bytes) | impossible (kernel guarantees uniqueness) | **the real peer** both ways — server sees client's real pid, client sees server's real pid |

The two requirements (no fixed name · mutual pid trust) do not conflict; they select the mechanism:

- **Point-to-point (self-peer, parent↔child):** `socketpair`, inherited. *No name whatsoever.*
  SO_PEERCRED is the wrong tool here (reports the creator) — and unnecessary: trust is the
  **lineage** (parent knows child via `fork()` return; child knows parent via `getppid`), which is
  stronger than a queryable cred. (This plumbing already exists — clone.rs pre-fork pipes, "the child
  inherits both ends", clone.rs:355.)
- **Accept-many (a defservice service):** **autobind**. The "name" becomes a kernel-minted, unique,
  unguessable token — no fixed/public/forgeable string — carried as a **capability on the `Handle`**
  (exactly like the thread tier's Sender). SO_PEERCRED delivers **true mutual peer-cred auth over UDS**
  (NOT mTLS — no certs/handshake/encryption): both ends read
  the other's real pid. **Do not forget: the client MUST check the server's pid/identity too** — the
  half we never built.

## Trust installation — the post-spawn-fn seam (the lineage trust root)

The same perfect knowledge as the rendezvous: **the parent learns every child's pid at fork**, and
the **post-spawn-fn** is the one moment it learns it — it runs OWNER-side in the parent after the fork
with `ProcessLaunch{pid}` (`spawn.wat:32`, `kernel/spawn.rs:349-350`). So the parent is the trust
ROOT, and the post-spawn-fn is where it INSTALLS the lineage trust (no discovery — it's told):

- Parent spawns the **server** S → its post-spawn-fn hands the parent `pid_S` (records "my server").
- Parent spawns a **client** C for S → C's post-spawn-fn (in the parent) calls `(allow' S-listener
  pid_C)` → S trusts C's pid. **Server→client pid trust, lineage-installed.** (`allow'`:
  `(Listener'<S,R>, i64) -> nil`, `runtime.rs:4770` / `listener.rs:272` — already built.)
- The parent hands C the capability (S's autobind addr) **and `pid_S`** via the `Handle` → C checks
  `server.pid == pid_S`. **Client→server pid trust** — the half step 3 is missing (euid-only);
  lands when the Handle carries the expected pid (step 6).

The capability bounds *who can reach you*; euid+pid (both ways, lineage-installed at post-spawn)
confirm *who they are*.

## What gets annihilated — and what SURVIVES re-grounded

**Annihilate (retire, do not patch) — the NAME:**
- `:wat::kernel::socket-address'` + connect-by-name (arc C0b.2d) — fixed-name discovery (forgeable,
  collidable).
- The fixed-name 2-arg `listener'(process)` arm (the LEGACY kept through step 2b).
- The "SO_PEERCRED is local mTLS" overclaiming comment (`listener.rs`) — it is mutual peer-cred over
  UDS, NOT TLS.

**KEEP, re-grounded — the lineage pid-trust** (builder correction, 2026-06-15): the allow-set +
`allow'`/`deny'` are NOT name-stranger-defense to delete — they are the **server's pid-trust half,
installed by the parent via the post-spawn-fn** (above). The rationale flips from *"bounce strangers
who guessed a public name"* to *"the parent declares the trusted lineage at spawn."* Same mechanism,
honest reason. (The birth-seed comment + the "stranger" framing get rewritten; the code stays.)

## Relation to host-parity

This **supersedes the planned stone 4b**. The process `extend-type :wat::spawn::ProcessOpts
:wat::spawn::Host` `launch` impl mints the rendezvous at spawn (the EXISTING inherited pipe-pair
self-peer for the control channel — step 4; autobind for the service listener), passes the child its
end by inheritance, and returns a `Handle`
whose `addr` is the capability — mirroring the thread tier. Zero edit to defservice `start` (it already
routes through `Host/launch`, arc-209 4a-iii `41e01b8d`).

## Decomposition (to draw as strikes)

1. **Kernel probe — DONE** (the C probe above grounds socketpair vs autobind SO_PEERCRED semantics).
2. **Autobind primitive + wire — DONE** (`4fa8f859` `comms::process::autobind_listener` + `5354c582`
   `listener'(process)` 3-arg autobind → `Bound<S,R>`; `SocketAddress.name`→`Vec<u8>`; the legacy
   2-arg named form kept as LEGACY for step 5).
3. **Mutual UDS peer-cred — DONE** (`2b451f2e` — `SocketAddress::connect` reads the server's peer_cred
   + refuses on euid mismatch, mirroring the accept gate; euid floor; pid-exact match deferred to
   step 6 when the expected pid threads via the Handle. NOT "mTLS").
4. **socketpair self-peer — GROUNDED: already satisfied by construction, no change.** The control
   self-peer is ALREADY the doctrine's point-to-point channel: the parent creates the pipes pre-fork
   (`kernel/spawn.rs:622+`, `dup2` onto fd0/fd1 at `:692`), the child inherits them and builds its
   self-peer from fd0/fd1 (`process/verbs.rs:391-408`, `Peer::from_socket`). No name, no discovery,
   lineage trust — built right in C0b.3a-0. The four-questions on the status quo find no failure
   mode; a *literal* socketpair offers nothing over the inherited pipe-pair (the probe showed
   SO_PEERCRED on a socketpair reports the creator, not the peer — useless), so converting would be
   pure churn + a new fd-plumbing failure surface for zero need. "socketpair" was an
   over-specification in this design ([[feedback_dont_build_the_forcing_function]] /
   [[feedback_curated_note_mechanism_must_be_grounded]] — four-Q the status quo first).
5. **Annihilate the name stack** — remove socket-address'/connect-by-name/allow-set + the overclaiming
   comment; migrate/retire the `c0b3bb`/`c0b3aii` tests onto the capability model (their flake dies
   with the namespace).
6. **Process Host/launch impl** — the `extend-type ProcessOpts Host` on the above (was "4b"); reuses
   the EXISTING inherited self-peer (step 4) + threads the expected server pid → enables step 3's
   pid-exact match.

Pairs [[project_shared_memory_partition_hosting]] + [[feedback_reach_stumble_is_the_signal]]
+ [[feedback_dont_build_the_forcing_function]] + [[feedback_no_magic_that_lets_llm_fake_correctness]]
(the overclaiming "mTLS" comment) + [[feedback_bar_shockingly_well_written]] (annihilate the class).
