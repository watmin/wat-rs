# DESIGN — 6c: a capability reconstructs ONLY over a lineage-verified channel

> ⛔⛔ **CORRECTED 2026-06-16 (vigilatum round 6 — the live plan is 6c.2 below).** An earlier banner here
> claimed *step 5 subsumes 6c.1 because every connect targets an "unguessable" capability → the answerer
> is lineage-proven.* **That premise is FALSE.** vigilatum round-6 circumspicere + an empirical probe
> proved Linux autobind names are `%05x` = **2²⁰ ≈ 1M** (randomized-start on modern kernels, but a small,
> brute-forceable space — NOT a secret). So name-unguessability was never load-bearing, and step 5 did
> NOT actually close the connect leg. **The real close is 6c.2, and the builder's perfect-knowledge
> insight makes it clean and REQUIRED (not deferred):**
>
> In the capability-only world (post step 5) you only ever dial a *handed* capability, and the minter
> knows its own `getpid` at mint time → **the minter stamps its own pid INTO the `Address'`**; the pid
> rides the capability *by value* over the lineage channel (NO process-global set, NO mutex — my old
> objection was bogus). The connect gate then becomes **symmetric** with accept:
> `connect admits iff answerer.uid == my_euid AND answerer.pid == address.minter_pid`.
> - live minter → pid matches → admit; **death-then-rebind attacker** → different pid → **refused**
>   (the edge that name-secrecy could never close); cross-uid → euid floor; self-connect → you stamped
>   your own pid → matches.
> - This makes the connect-leg cap-decode safe **by construction** → it genuinely SUBSUMES 6c.1 (the
>   `PeerTrust` bit below) for the RIGHT reason (a pid-verified channel), not a false one.
>
> The accept gate (uid + pid∈lineage) was always correct + enforced — no live exploit existed; the round-6
> finding was a FALSE SHIPPED CLAIM, not a hole. The fix is 6c.2 + retract every "unguessable / kernel-
> minted-random ⇒ lineage-proven" claim (policy.rs:38-40, address.rs:165-170, comms/process.rs:179, the
> step5 design line 24, the recv'/select' comments runtime.rs:23842/24300) and re-anchor on: *the SO_PEERCRED
> uid+pid checks are the security; the autobind name is an exclusive-bind rendezvous token, not a secret.*
> Also exigere round-6 L1: policy.rs:13-14 + :26 name unbuilt rungs (`these-gids`, the wat predicate) as
> future-work with no arc tracker — drop them / keep present-tense only. The `PeerTrust` design below is
> retained as the conceptual seam (now realized as the pid check); the live strike is **6c.2 (THE STRIKE)**
> appended at the end of this doc.

---


> Opened 2026-06-16. **Surfaced by the vigilatum, not planned** — round-3 circumspicere L1 on
> `src/capability/`: recv' (runtime.rs:23980) decodes through the trusted capability door
> *unconditionally*, and its comment claims *"bytes from a lineage peer"* — but the **process connect**
> gate (address.rs:209) admits at the weaker `AnyOfMyUser` rung (euid only, no pid/lineage). So a recv'
> over a connect-established channel reconstructs **live capabilities** off a channel that proved only
> "same user." The door's premise is false on the connect leg; the policy.rs claim *"authority flows
> only along the spawn lineage, never to a stranger"* overreaches for the system as shipped. This stone
> makes the premise **structurally true** — and it BLOCKS the `src/capability/` vigilatum stamp (the
> home's central claim is unenforced until this lands). The honest discipline: no stamp over a live L1.

## The statement

**A capability (`#wat-edn.cap/*`) is reconstructed ONLY over a `Peer` whose channel passed a
lineage check** (kernel-verified `{uid, pid∈lineage}`, or an in-process handle that IS the grant). A
channel that passed only the euid floor (a process `connect'` to an un-pinned address) decodes via the
**cap-refusing** path — its recv' yields ordinary values, never a live capability.

This makes recv's own comment honest: "bytes from a lineage peer" stops being an assumption and becomes
an invariant the type enforces.

## Why a bit on the Peer (not a process-global set, not a name purge)

The naive close — "give the client a lineage set of pids it spawned and check the server's pid against
it" — is a **ZERO-MUTEX trap**: that set is process-wide mutable state shared across a program's threads,
so it can't be a `ThreadOwnedCell` (single-thread-owned) and the only fallback is a `Mutex` — banned.

The honest shape is to carry the trust **on the channel itself**, set once by the gate that built it:

```
enum PeerTrust { Lineage, EuidOnly }
```

- It travels **by value** with the `Peer` — no shared state, no lock (the ZERO-MUTEX through-line).
- Every gate already KNOWS, at construction, whether it verified lineage. It just wasn't recorded.

## The contract decision (pinned)

`Peer` gains one field `trust: PeerTrust`, set at construction by each of the five gates
(grounded sites):

| gate | site | trust | why |
|---|---|---|---|
| spawn handle (parent↔child) | spawn.rs:526 | **Lineage** | inherited by fork; cannot be obtained except by spawning |
| self-peer (→ supervisor) | process/verbs.rs:408 | **Lineage** | the program's inherited link; lineage by construction |
| thread connect / accept | address.rs:88 / listener.rs:241 | **Lineage** | the crossbeam handle IS the grant (in-process, unforgeable) |
| **process accept'** | listener.rs:376 | **Lineage** | passed `OnlyMyPeers` (euid + pid∈allow-set) |
| **process connect'** | address.rs:209 | **EuidOnly** | passed only `AnyOfMyUser` (euid floor; no expected pid) |

recv' (runtime.rs:23980) and select' (24438) — both hold the peer they read from — branch on it:
**`Lineage` → `decode_trusted_wire` (the capability door); `EuidOnly` → the cap-refusing read.**

## Four questions

- **Obvious?** YES — a channel either proved lineage or only euid; caps cross only the former. One field, one branch.
- **Simple?** YES — no new concept; it makes the EXISTING "trusted wire" premise real. The bit rides the Peer by value (no global, no lock).
- **Honest?** YES — the door's "lineage peer" claim and policy.rs's "never to a stranger" become **structurally enforced** for the capability path; a euid-only channel can no longer mint a capability. The mistake has no constructor.
- **Good UX?** YES — automatic; a service author cannot accidentally reconstruct a capability off an unauthenticated channel. No caller discipline.

## The proof (RED at HEAD → GREEN after)

A multi-process probe (sibling of c0b3bb): a same-uid process connects to a listener by a well-known
name and is handed a `#wat-edn.cap/address` tag over the connection.
- **RED at HEAD:** the client's recv' over the connect-leg (euid-only) channel reconstructs a live
  `Address'` capability — authority crossed a euid-only channel.
- **GREEN after:** the same recv' over the `EuidOnly` peer takes the refusing path → the cap tag does
  NOT reconstruct (an error, or an opaque non-capability). The capability never crosses.

Regression guards (must stay green): 6a (caps cross the **handle** = Lineage → still reconstruct);
c0b3bb (accept' = Lineage → served); c0b2c (self-connect; data only, no caps).

## Decomposition

1. **6c.1 — THE CLOSE (this stone).** `PeerTrust` field + the five gate assignments + the recv'/select'
   decode branch + the RED probe. Closes the L1; unblocks the vigilatum. **Breaks nothing**: no current
   flow reconstructs a capability over a process `connect'` peer (6a's caps cross the handle).
2. **6c.2 — THE ENABLE (named follow-on, NOT now).** Let a *pid-verified* connect carry capabilities:
   the `Address'` carries the minter's `getpid` (stamped at autobind/`Bound`), `connect'` checks the
   answered server's `SO_PEERCRED` pid == that → upgrades the peer to `Lineage`. Deferred —
   *don't build the forcing function*: no caller needs cap-over-connect yet ([[feedback_dont_build_the_forcing_function]]).
3. **step 5 (orthogonal).** Annihilating guessable `socket-address'` names is a separate simplification;
   it is NOT required for this close (an un-pinned channel simply refuses caps).

## The bar

The capability home's central claim — "a capability reconstructs only off the trusted door" — becomes
**true on every leg**, enforced by a value the type carries, not by a comment. Then the vigilatum
re-casts and converges. Pairs [[project_rendezvous_inherited_capability]] +
[[feedback_vended_primitives_never_deadlock]] (the ZERO-MUTEX shape) + REALIZATIONS.md (ocap) +
[[feedback_dont_test_the_substrates_honesty]] (the proof tests OUR gate-routing, not the kernel).

---

## 6c.2 — THE STRIKE (the live plan; redrawn 2026-06-16 round 6)

**Contract decision (pinned):** the connect gate verifies `answerer.uid == my_euid AND answerer.pid ==
the minter pid carried by the dialed `Address'`.` The minter pid is stamped at autobind (perfect
knowledge: the minter's own `getpid`), travels with the capability by value (no global set, no mutex),
and is checked against the kernel-vouched `SO_PEERCRED` pid of the answerer.

**Files / seams (grounded):**
- `src/kernel/address.rs` — `SocketAddress` gains a `minter_pid: i32` (or `Option<i32>`; prefer
  always-present since every autobind has one). `connect_admits` (addr.rs:224) becomes
  `OnlyMyPeers`-shaped: `answerer.uid==euid AND answerer.pid==self.minter_pid` (NOT `AnyOfMyUser`). The
  connect gate (addr.rs:183) passes the dialed address's `minter_pid`.
- `src/runtime.rs` `eval_listener_prime` autobind arm (~18752) — stamp `libc::getpid()` into the
  `Address::from_socket_name_bytes(...)` result (extend the ctor or set the field).
- `src/capability/registry.rs` `address_codec` — the `wat-edn.cap/address` wire body carries name bytes
  **+** the minter pid (e.g. `OwnedValue::Vector([pid, ...name bytes])` or a 2-field tagged shape).
  Encode `portable_name_bytes` + pid; decode reconstructs both. Update `address_decode_rejects_*` tests +
  add the distinctness already there.
- `Address::from_socket_name_bytes` + `portable_name_bytes` signatures extend to carry the pid.
- 6a handoff (`probe_arc272_6a_capability_handoff`, `c0b3bb_bounced` served leg) — the child autobinds
  (its pid stamped), sends the `Address'` up; the parent connect verifies answerer.pid==child pid. Should
  stay green (parent dials the live child). The c0b3bb stranger leg: the stranger rebinds → different pid
  → connect (by the owner) would now ALSO bounce on pid — but that test bounces at ACCEPT; keep it.

**RED probe:** a same-uid process binds a DIFFERENT autobind address than the one stamped, hand the client
an `Address'` whose `minter_pid` ≠ the actual answerer's pid → connect REFUSES (today it would admit on
euid alone). GREEN after 6c.2.

**Then:** retract the false "unguessable" claims (listed in the corrected banner above) + the exigere
future-rungs prose; re-cast the `src/capability/` vigilatum → converges (the round-6 L1 was the false
claim; 6c.2 + the retraction close it) → **stamp** the vigilatum in `src/capability/mod.rs`.

**Why this is the bar (not euid-only-resignation):** both gates do uid+pid; the connect leg is genuinely
lineage-verified; name-secrecy is irrelevant because the PID is checked, not because we gave up on
same-uid. Perfect knowledge (the minter knows its own pid) makes it mutex-free. Pairs
[[feedback_dont_test_the_substrates_honesty]] (the pid check is OUR code; don't multi-process-test the
kernel's cred honesty) + [[project_rendezvous_inherited_capability]].
