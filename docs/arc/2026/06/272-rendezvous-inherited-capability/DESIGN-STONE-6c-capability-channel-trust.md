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

## 6c.2 — THE STRIKE (the live plan; contract PINNED 2026-06-15, grounded against `32e2e9d6`)

> ⛔ **WIRE SHAPE SUPERSEDED 2026-06-15 → see `BRIEF-STONE-6c.2-D1.md`.** The "2-element positional
> vector" wire (decision #4 below) was rejected: a heterogeneous `(i64, bytes)` product modeled as a
> homogeneous vector/map is ill-typed in wat (maps/vectors are `HashMap<K,V>`/`Vector<T>` — single
> value type; `:Any` banned). The honest shape is a **record** (struct = non-wire/opaque; record =
> EDN/wire — `wat/spawn.wat:116`). Since 234.7a/b now make records round-trip, the address's portable
> form is a **registered base record** `:wat::kernel::SocketAddressWire {minter-pid, name}`, and the cap
> codec reuses the ONE general record encode/decode (threading `types` into the `CapCodec` signature —
> a one-time waist evolution) — no hand-build, no divergence. Wire:
> `#wat-edn.cap/address #wat.kernel/SocketAddressWire {:minter-pid 4242 :name [1 2 3 4 5]}`. Decisions
> 1-3 + 7 below STAND (the pid gate, `OnlyThisPeer`, `AnyOfMyUser` annihilation, retractions);
> decisions 4-6 (the positional wire / `from_socket_name_bytes` / `portable_form` shapes) are restated
> in the D1 brief. The four-questions favored D1 over a codec hand-build on Simple + Honest
> (single-source-of-truth, same discipline as 234.7b).

**Contract decision (pinned):** the connect gate verifies `answerer.uid == my_euid AND answerer.pid ==
the minter pid carried by the dialed `Address'`.` The minter pid is stamped at autobind (perfect
knowledge: the minter's own `getpid`), travels with the capability by value (no global set, no mutex),
and is checked against the kernel-vouched `SO_PEERCRED` pid of the answerer.

### The seven pinned decisions (the orchestrator's crawl, settled)

1. **A new policy rung `OnlyThisPeer { pid: i32 }`** (`src/capability/policy.rs`) — `admits` iff
   `peer.uid == my_euid && peer.pid == pid`. The connect side has perfect knowledge of *exactly one*
   expected pid (the minter), not a *set* — so the cardinality-honest shape is its own rung, not
   `OnlyMyPeers{lineage:{minter}}` (a singleton-set fiction) nor `AnyOfMyUser` (the dropped-pid floor).
   The module doc already invites this: "adding a rung extends the policy language; the `admits`
   contract never changes" (the narrow-waist law). Add a parallel unit test.
2. **`AnyOfMyUser` is ANNIHILATED** (extirpare — not bypassed). Grounded: its *sole* consumer is
   `connect_admits` (address.rs:184/226). With the minter pid stamped, the euid-only posture has no
   honest consumer left → remove the variant, its `admits` arm, its doc paragraph, and its unit test
   `any_of_my_user_admits_my_user_at_any_pid_and_refuses_other_users`. The enum is left with two live
   rungs: `OnlyMyPeers{lineage}` (accept gate) and `OnlyThisPeer{pid}` (connect gate).
3. **`SocketAddress` gains `minter_pid: i32`** (always present — every autobind stamps one; not
   `Option`, there is no address without a minter). `ThreadAddress` is untouched (in-process, no
   peer-cred gate, no portable form).
4. **Wire shape (PINNED):** the `wat-edn.cap/address` body becomes a 2-element vector
   `OwnedValue::Vector([OwnedValue::Integer(minter_pid), OwnedValue::Vector([name bytes…])])` —
   `(pid, name-bytes-vector)`. Decode validates: outer is a `Vector` of len exactly 2; elem 0 is an
   `Integer` in `i32` range; elem 1 is the byte-vector (the existing empty / over-long / `0..=255`
   checks move onto elem 1). The pid being a distinct element (not folded into the byte stream) keeps
   the existing byte-range validation honest.
5. **`from_socket_name_bytes(name: Vec<u8>, minter_pid: i32)`** — both call sites pass the pid: the
   autobind arm stamps `libc::getpid()` (runtime.rs:18697); decode passes the wire pid (registry.rs:138).
6. **`portable_name_bytes` → `portable_form() -> Option<(i32, Vec<u8>)>`** returning `(minter_pid, name)`
   — one downcast yields both fields (they always cross together or not at all). `Some` only for
   `SocketAddress`; `None` for `ThreadAddress` (no portable form).
7. **`connect_admits(server, euid, minter_pid)`** → `CommsPolicy::OnlyThisPeer{pid: minter_pid}
   .admits(server, euid)`. `SocketAddress::connect` passes `self.minter_pid`.

### The disconfirming probe — OUR gate logic, never the kernel's honesty

`tests/probe_arc272_6c2_pid_gate.rs` exercises the **public** `CommsPolicy::OnlyThisPeer` rung with
**synthesized** `PeerCred` values (no socket, no fork, no privilege): exact-pid same-uid → admitted;
**same-uid wrong-pid → REFUSED** (the death-then-rebind edge); right-pid wrong-uid → refused (the floor).
**RED at HEAD** because the `OnlyThisPeer` variant does not exist — the compile failure names exactly the
gap: *the policy language cannot express "only this one pid" today.*

⛔ **No multi-process IO probe.** The earlier draft proposed a same-uid forked process with a mismatched
stamped pid → connect refuses. That is REJECTED per [[feedback_dont_test_the_substrates_honesty]]: it
would need privileged staging and would prove the kernel reports `SO_PEERCRED` pid honestly — an axiom,
not our code. The pid *comparison* is ours and is proven pure, at the unit level. The live wiring
(connect passes `self.minter_pid`; the wire round-trips the pid) is covered by inspection + the
GREEN-after regression guards below.

### Regression guards (must stay green)

- `probe_arc272_6a_capability_handoff` — the parent dials the **live** child; answerer pid == the child's
  stamped minter pid → admitted. The codec now carries the pid across the wire, so the parent decodes
  the child's pid. Stays GREEN.
- `address.rs` connect-gate unit test — rewritten for the 3-arg `connect_admits`: exact pid admitted,
  wrong-pid-same-uid refused, wrong-uid refused.
- `registry.rs` codec tests — a pid round-trip (encode→decode carries pid + name); the empty / over-long
  rejections updated to the 2-element wire shape.

### Retractions (the false "unguessable ⇒ lineage-proven" claim) + exigere L1

Re-anchor every site on: *the `SO_PEERCRED` uid+pid checks ARE the security; the autobind name is an
exclusive-bind rendezvous token, not a secret.*
- `policy.rs` — the `AnyOfMyUser` doc block goes with the variant; the module-doc `these-gids` /
  wat-predicate "future rungs" prose (the exigere round-6 L1, policy.rs:12-14/25-26) rewritten
  present-tense to the two rungs that EXIST.
- `address.rs:163-172` (the connect comment) + `connect_admits` doc — the pid IS checked now.
- `runtime.rs` autobind arm (18636/18663 "unguessable") + recv'/select' decode comments
  (23845/24307) — re-anchor on the pid-verified channel, not name-secrecy.
- `comms/process.rs:178` — soften "unguessable abstract name" → "kernel-minted, exclusive-bind, not a
  chosen name" (the collision/squat-freedom claim is TRUE; only the secrecy implication is dropped).
- `DESIGN-STONE-step5-annihilate-the-name.md:24` — the false "unguessable" line.

**Then:** re-cast the `src/capability/` vigilatum → converges (round-6 L1 was the false claim; 6c.2 +
the retraction close it) → **stamp** the vigilatum in `src/capability/mod.rs`.

**Why this is the bar (not euid-only-resignation):** both gates do uid+pid; the connect leg is genuinely
lineage-verified; name-secrecy is irrelevant because the PID is checked, not because we gave up on
same-uid. Perfect knowledge (the minter knows its own pid) makes it mutex-free. Pairs
[[feedback_dont_test_the_substrates_honesty]] (the pid check is OUR code; don't multi-process-test the
kernel's cred honesty) + [[project_rendezvous_inherited_capability]].
