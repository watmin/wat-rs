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

## The proof (here, on the 272 process tier)

A multi-process probe: parent spawns a child peer (in `only-my-peers`) — they comms, a capability flows
(6a green). A **non-peer** process (not in the lineage, or different euid) attempts to connect — **refused
at the gate** by the policy, not by an ad-hoc check. The refusal is the policy doing its job; the success
is the peer being recognized. (RED at HEAD: no unified policy exists — the checks are scattered and a
non-peer's refusal can't be attributed to a policy.)

## Decomposition (to draw as strikes)

1. **`CommsPolicy` + `only-my-peers`** — the predicate over `PeerCred` (Rust; the powerbox value).
2. **Consult it at the gates** — accept + connect call the policy instead of the inline euid/allow-set
   checks (the allow-set BECOMES the lineage set the policy reads; `allow'`/`deny'` mutate it).
3. **Prove** — the multi-process probe (peer comms; non-peer refused at the gate).
4. (later / v-next) **wat-expressible policy** — the predicate as a wat `fn(PeerCred) -> bool`, so a
   service declares its posture in wat. The full policy language.

Pairs [[project_rendezvous_inherited_capability]] + REALIZATIONS.md (ocap / powerbox prior art) +
[[feedback_bar_shockingly_well_written]] + the ZERO-MUTEX through-line (this is where "no mutex" arrives).
