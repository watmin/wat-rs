# DESIGN — the comms policy: "only my peers can comms with me" (the powerbox, v4)

> Opened 2026-06-16. **The pivot** (builder: *"i say we do v4 now … only my peers can comms with me is
> the statement"*). The capability-decode gating (6a-i, `a69e14bd`) revealed that trust is a *policy*,
> not a flag. This stone lifts the three scattered hardcoded peer-credential checks into ONE consulted
> predicate — an object-capability **powerbox** (Miller) — and proves it on the 272 process tier.
> May graduate to its own arc; drafted here because it IS the 272 trust model formalized.

## The statement (the policy)

**I accept comms — a connection, a message, a reconstructed capability — only from a peer whose
kernel-verified identity (`SO_PEERCRED` {pid,uid,gid}) is in my trust set: my lineage peers running as
me.** "My peers" = `peer.uid == my_euid` **AND** `peer.pid ∈ my-lineage` (the pids I spawned, installed
via the post-spawn-fn — the 272 trust seam). Everything else is refused at the gate.

## Why this is one thing, not four (the unification)

Three checks exist today, hardcoded and scattered — and the cap-decode door makes an implicit fourth:

| gate | today (scattered) | source |
|---|---|---|
| **accept** (server vets client) | `SO_PEERCRED` read at accept + allow-set + euid | listener.rs:267 / runtime.rs accept arm |
| **connect** (client vets server) | `SO_PEERCRED` read + euid match (pid deferred) | address.rs:172-194 |
| **cap-decode** (reconstruct a capability) | the v1 door: coarse "trusted channel ⇒ reconstruct" | edn_shim::decode_trusted_wire |

**The decode door INHERITS the connection gate.** By the time you `recv'` a capability, the peer already
passed accept/connect — the channel is *already* policy-authorized, which is exactly why the door may
trust it. So the policy lives at **accept + connect** (where the peer's cred is verified); the door is a
beneficiary. v4 = lift the accept/connect euid+allow-set checks into ONE `CommsPolicy` predicate, consult
it at both gates. No new check — the *same* authority, expressed once.

## The abstraction (the powerbox)

A **`CommsPolicy`**: given a peer's verified `PeerCred` (and the action), return Allow / Deny.

```
only-my-peers(peer) := peer.uid == geteuid()  AND  peer.pid ∈ my-lineage-set
```

- **Policy #1 = `only-my-peers`** — the 272 trust model, now a value not a hardcode.
- The **policy LANGUAGE** is the set of expressible peer-authorization predicates: `any-of-my-user`
  (euid only), `these-pids`, `any-with-gid g`, `only-my-peers` (the strict lineage form), … The predicate
  is the language; `only-my-peers` is the first sentence. (Shaped to become wat-expressible — a
  `fn(PeerCred) -> bool` the substrate consults — but proven in Rust first.)
- This is precisely Miller's **powerbox**: a single mediator deciding which peers a process may obtain
  authority from. The cap-decode door (6a-i) is the first thing it mediates; accept/connect are the rest.

## Four questions

- **Obvious?** YES — "I talk only to my peers" is a one-sentence security posture; the policy *names* it.
- **Simple?** YES — replaces three ad-hoc credential checks with one predicate consulted at the gates;
  fewer concepts, not more (the decode door already inherits it).
- **Honest?** YES — the trust set is `SO_PEERCRED` (kernel-vouched, unforgeable) + lineage (the parent's
  perfect knowledge), not a guessable token. The mistake (talk to a stranger) has no path through the gate.
- **Good UX?** YES — flipping the posture (strict-lineage vs any-of-my-user vs a custom predicate) is one
  policy value, not a scattered edit across accept/connect/decode.

## The proof (here, on the 272 process tier) — SHIPPED, and where the honest ceiling sits

The proof set, all CI-portable + unprivileged (2026-06-16, `3d6357ed`):
- **predicate logic** — `capability::policy::tests` unit-prove both rungs: `OnlyMyPeers` admits a lineage
  peer, refuses a wrong-euid and a non-lineage pid; `AnyOfMyUser` admits my user at any pid, refuses a
  different euid. No IO — pure logic.
- **gate wiring (the refuse branch is real)** — `probe_arc209_c0b3bb_bounced::stranger_is_bounced`: a
  genuine separate process whose pid ∉ the allow-set is **bounced at the live accept gate**,
  multi-process — and post-v4 that refusal flows through `OnlyMyPeers.admits()`. The uid clause sits one
  `&&` away in the same `admits()`, on the same call path.

### Why there is NO cross-uid multi-process test (the dead-end, recorded so it is not re-walked)

The tempting next test — "a *different-uid* process refused at the live gate" — was **examined and
declined**. Two findings, grounded 2026-06-16:

1. **It is OS-impossible to stage unprivileged.** An unprivileged process cannot manufacture a foreign
   *kernel* uid: `unshare(CLONE_NEWUSER)` with no map makes a process `nobody` *to itself*, but its kuid
   is invariant — our init-ns gate reads its real kuid (probed: child sees 65534, gate reads 1000 →
   admitted). A genuine foreign kuid needs `newuidmap` + `/etc/subuid` (the rootless-container stack) or
   root — i.e. privilege at runtime, which breaks public-CI portability. **We do not use sudo for this.**
2. **Even if staged, it tests the kernel, not us (the decisive reason).** Its only delta over (predicate
   unit) ∘ (c0b3bb wiring) is "does `getsockopt(SO_PEERCRED)` report a foreign process's uid as foreign"
   — that is Linux's honesty, the **trusted axiom the whole ocap model stands on**, not our code. You do
   not test your axioms. The four-questions kill it on **Honest**: it would masquerade as proving our
   gate while proving the OS. (Same verdict retires the kuid-invariance probe as a permanent test — it
   too measures the kernel; it served once as exploration and is done.)

The connect-gate refuse branch (`!AnyOfMyUser.admits(...) → Err`) is a trivial early-return, identical in
shape to the accept refuse branch that `c0b3bb` exercises; it is correct by the predicate unit test + by
inspection. v4 is proven to the honest ceiling.

## Decomposition (to draw as strikes)

1. **`CommsPolicy` + `only-my-peers`** — the predicate over `PeerCred` (Rust; the powerbox value). ✅ DONE.
2. **Consult it at the gates** — accept + connect call the policy instead of the inline euid/allow-set
   checks (the allow-set BECOMES the lineage set the policy reads; `allow'`/`deny'` mutate it). ✅ DONE
   (`410af5e1` accept; `3d6357ed` connect → `AnyOfMyUser`, the honest connect-side rung until 6c).
3. **Prove** — ✅ DONE to the honest ceiling: predicate unit tests + `c0b3bb` gate-wiring (see *The proof*
   above). The cross-uid multi-process test is declined — OS-impossible unprivileged + tests the kernel.
4. (later / v-next) **wat-expressible policy** — the predicate as a wat `fn(PeerCred) -> bool`, so a
   service declares its posture in wat. The full policy language.

Pairs [[project_rendezvous_inherited_capability]] + REALIZATIONS.md (ocap / powerbox prior art) +
[[feedback_bar_shockingly_well_written]] + the ZERO-MUTEX through-line (this is where "no mutex" arrives).
