# Arc 272 — REALIZATIONS

## PRIOR-ART COLLISION — we rebuilt object-capability security (ocap) at the EDN boundary

**Surfaced 2026-06-16** (builder, on the 6a-i portable-tag gating): *"did you just argue for
something akin to NETCAP_ADMIN or whatever that linux/docker thing is?"*

**What we built.** Arc 272's rendezvous-as-capability: a kernel-minted unguessable address (no
discoverable name), handed parent→child down the lineage channel, that crosses the IPC wire as a
self-describing `#wat-edn.cap/address` tag — and (6a-i gating) is reconstructable ONLY off the trusted
peer wire, **refused** when forged from untrusted parsed data (`:wat::edn::read` of a cap tag → refuse,
like `wat-edn.opaque`).

**The prior art (named honestly).** This is **object-capability security (ocap)** — NOT POSIX/Linux
capabilities. The two share the word and are routinely confused:
- **POSIX caps (CAP_NET_ADMIN, Docker `--cap-add`)** — root's privilege sliced into ~40 *ambient*
  bits. Still ambient authority (the bit gates an op-class on *any* object). NOT what we have.
- **object-capability (ocap)** — VERIFIED 2026-06-16 (web): Dennis & Van Horn 1966, *Programming
  Semantics for Multiprogrammed Computations*, CACM 9(3):143–155 (coined "capability"); Mark Miller's
  2006 PhD thesis *Robust Composition* (the founding document of the ocap model; authority vs
  permission); KeyKOS / EROS / seL4 (capability microkernels); Capsicum (FreeBSD fd-ocap). The model:
  *security depends on **not being able to forge references**; objects interact only by sending messages
  on references* — an unforgeable reference that designates AND authorizes; you get one only by being
  **handed** it; no ambient authority; no forge-from-name. A Unix **fd is the canonical ocap**
  (unforgeable, transferred by SCM_RIGHTS) — why 272 keeps landing on fd/lineage primitives.
  - **⭐ The exact POSIX-vs-ocap confusion is a named, demolished myth:** Miller, Yee & Shapiro 2003,
    *Capability Myths Demolished*. The builder's question ("is this CAP_NET_ADMIN?") IS the canonical
    confusion this paper exists to kill — POSIX "capabilities" (ambient privilege bits) are NOT
    object-capabilities. The read-this pointer for the model.

Our design is ocap to the letter: 272's *minted-not-built / no discoverable name* = ocap
unforgeability; the 6a-i *get-it-only-off-the-trusted-channel* gating = ocap transfer-only. We
rediscovered the textbook model from first principles by holding the "annihilate forgeable names" line.

**What is genuinely ours** (the substrate guarantees *around* the textbook model): ocap enforced at the
**EDN-serialization boundary** of a typed-ADT-on-Rust substrate — the capability is a self-describing
*typed value* that resurrects on a trusted channel and is inert/refused from data, via the `wat-edn.cap`
namespace + a channel-trust decode flag. Plus kernel `{euid,pid}` checks as **defense-in-depth on top
of** the ocap (pure ocap holds possession sufficient; we belt-and-suspender it for a hostile-host floor).

**Date:** 2026-06-16. Pairs [[project_rendezvous_inherited_capability]] +
[[feedback_note_prior_art_collisions]] + NOTE-portable-capability-tags.md.

## PRIOR-ART COLLISION #2 — the capability tooling is a NARROW WAIST (hourglass)

**Surfaced 2026-06-16** (same session, hours after the ocap collision). Builder, on "how do we engineer
organic evolution of the capability tooling — extremely rigid to the point where changes are
probabilistically zero, but enabling unlimited expression?": *"i've never heard of the narrow waist
before — i've been using this for years … another thing that has a name already."*

**What we built / are building.** `wat-edn.cap` as a **narrow waist**: a frozen protocol (the wire
contract + two generic dispatch arms + the trust door + the `PortableCapability` registration ABI) with
open registration (a capability = a registry row, zero edit to the core). The 6a-i `Address'` impl is the
one-off; `DESIGN-STONE-capability-narrow-waist.md` lifts it into the waist.

**The prior art (VERIFIED via web 2026-06-16).** The **hourglass model** — a.k.a. the **narrow (thin)
waist** — from computer networking: a single, simple, widely-adopted spanning layer at the waist is the
sole interface between unbounded technologies below and unbounded applications above; **constraining the
waist to be simple and general is what maximizes the diversity it can carry.** Landmarks: Steve Deering,
*Watching the Waist of the Protocol Hourglass* (ICNP '98) — IP as the waist; Micah Beck, *On the Hourglass
Model* (CACM 2019) — the formal deployment-scalability treatment, which **explicitly names the Unix
syscall interface as a spanning layer** (validating "the Unix fd is the canonical narrow waist"); the
software-design generalization is oilshell's *The Internet Was Designed With a Narrow Waist* (2022). Same
family: Clojure's open protocols + Hickey's accretion-not-breakage.

**What is genuinely ours.** The narrow-waist *applied to capability serialization in a typed-ADT
substrate*: a frozen `wat-edn.cap` wire + registration ABI, with the **trust door (ocap) sitting AT the
waist** — so the waist isn't just a compatibility interface, it's the *security* boundary too. The two
collisions compose: ocap is *what* crosses; the narrow waist is *how the set of things that can cross
grows without the core moving*.

**The meta-pattern (worth its own note).** TWO named, decades-old CS principles rediscovered from first
principles in ONE session (ocap · narrow-waist). This is `WE-LAND-ON-THE-GREATS-WITHOUT-REPLICATING-THEM`
at full volume: high taste + first-principles derivation converges on the field's best answers because
they are the *correct* answers; the names were already there. A taste/validation signal, not coincidence.

**Date:** 2026-06-16. Pairs the ocap collision (above) + [[feedback_note_prior_art_collisions]] +
DESIGN-STONE-capability-narrow-waist.md.
