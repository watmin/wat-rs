# DESIGN — 6c: a capability reconstructs ONLY over a lineage-verified channel

> ⛔ **SUPERSEDED as THE close by `DESIGN-STONE-step5-annihilate-the-name.md` (2026-06-16).** Builder:
> a deferred *security posture* is a violation — 6c.1 (the `PeerTrust` bit below) only refused
> *capabilities* over a euid-only connect; it left the **data** path open to same-uid name-squatting,
> which violates "only my peers." **Step 5 (annihilate guessable names → capability-only rendezvous) is
> the real close** and SUBSUMES 6c.1: with no guessable names, every connect targets an unguessable
> capability → the answerer is lineage-proven → there are no euid-only connect channels left, so the
> `PeerTrust` enum is unnecessary. **6c.2** (per-Address minter-pid stamped + verified — the
> belt-and-suspenders for the leak-then-rebind edge) survives as the accepted **deferral**. This doc is
> retained for the gate-map + the ZERO-MUTEX reasoning (still valid); read step 5 for the live plan.

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
