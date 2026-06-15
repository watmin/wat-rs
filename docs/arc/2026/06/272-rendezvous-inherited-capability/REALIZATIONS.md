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
