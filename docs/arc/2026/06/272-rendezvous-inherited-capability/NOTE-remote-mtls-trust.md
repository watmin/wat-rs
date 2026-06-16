# NOTE (HORIZON) — remote trust is mTLS: the SSL channel IS the network's vouching

> Captured 2026-06-16, builder: *"i want mtls for remote trust — the ssl channel is the trust for
> networks."* FORWARD-LABELED design intent: PINNED decision, UNBUILT (its own arc — built when a
> remote caller surfaces; don't-build-the-forcing-function until then).

## The symmetry (the whole idea)

**The channel vouches the peer's identity; the powerbox decides if that vouched identity is authorized.**
SO_PEERCRED and mTLS are two *credential sources* for the SAME decision:

| | who vouches | the credential | the policy rung (today / future) |
|---|---|---|---|
| **Local** (UDS) | the **kernel** (`SO_PEERCRED`, both ends local) | `{pid, uid}` | `OnlyMyPeers{lineage}` (accept) / `OnlyThisPeer{pid}` (connect) |
| **Remote** (TCP+TLS) | the **TLS handshake** (mutual cert auth) | the verified peer **cert identity** (subject / SPKI pin) | a NEW rung — `OnlyTheseIdentities{set}` / `OnlyThisIdentity{pinned}` |

`SO_PEERCRED` is the kernel-local mechanism; **mTLS is its network analog** — the TLS layer does the
identity vouching that the kernel can't do across a host boundary.

## Why this is a clean fit (not a rewrite)

1. **The data layer is already done + transport-blind.** Typed EDN records (base + holon) round-trip
   over a pure bytes pipe (234.7a/b + 258.5b-ii: encode-in-eval → `String` → fd → decode-in-eval). A
   TLS stream is just another byte fd — the record wire rides it unchanged. Nothing in the data path
   is UDS-specific.
2. **The powerbox is the frozen waist.** `CommsPolicy::admits(credential, my_identity) -> bool` never
   changes; a remote transport ADDS a rung that consults the TLS-verified identity instead of
   `SO_PEERCRED`. (The narrow-waist law — same as `OnlyThisPeer` was added in 6c.2.)
3. **`CommAddress` is the open transport trait.** Its doc already reserves this: *"Any remote
   CommAddress impl — organic future addition (a new impl, zero central edit; `remote` is
   perpetually-awaiting-definition)."* A TLS address (`host:port` + expected peer identity / trust
   anchor) is a new `CommAddress` impl: `connect` does the TLS handshake, verifies the peer cert, and
   feeds the verified identity to the policy gate — exactly mirroring how the UDS `connect` reads
   `SO_PEERCRED` and feeds `connect_admits`.

So remote = **one new `CommAddress` impl (TLS) + one new `CommsPolicy` rung (verified-identity)**. The
wire, the codecs, the ocap trust-door, the capability records — all unchanged.

## Forward / undecided (confirm when the arc opens)

- **Identity model**: cert subject vs SPKI pin vs SAN; one accepted CA vs a pinned peer set.
- **Provisioning**: where certs come from (a local CA, rotation, the `:remote` opts shape — still
  perpetually-awaiting-definition, on purpose).
- **The rung's exact name + shape** (`OnlyTheseIdentities` is a placeholder).
- **Handshake wiring**: rustls vs native-tls; where the verified identity surfaces to the gate.
- **Capability semantics over remote**: an `Address'` minted on host A dialed from host B — the
  minter-pid check (6c.2) is host-local; remote needs the identity check to be THE gate (pid is
  meaningless across hosts). The cap record (`SocketAddressWire`) may need a remote sibling carrying
  an identity instead of a pid — or a unified `AddressWire` whose gate field is transport-tagged.

## Bar

Don't build until a remote caller surfaces ([[feedback_dont_build_the_forcing_function]]). The DECISION
is pinned (mTLS = remote trust); the architecture seam (open `CommAddress` + the powerbox rung) is
proven by 6c.2's local close. Pairs the confinement-arc horizon + [[project_rendezvous_inherited_capability]].
