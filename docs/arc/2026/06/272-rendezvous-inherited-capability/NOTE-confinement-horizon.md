# NOTE — confinement horizon: capability-secure spawn ("locked down out of primitives")

> Banked 2026-06-16 (builder, capstone musing): *"can we launch vms configured such that they cannot
> access files or network? … can we make (not now, but perpetually 'soon') wat strongly locked down out
> of primitives if the user wants it that way?"* A **horizon**, not a now — the destination the arc-272
> ocap + comms-policy work walks toward. Captured so the next self finds it; **do not build the forcing
> function** ([[feedback_dont_build_the_forcing_function]]). It keeps the spawn/policy design honest +
> general.

## The vision

A user opts a spawned wat program into **strong confinement**: it cannot touch the filesystem or the
network — it holds ONLY the capabilities it was handed (an inherited `Peer'`, a passed `Address'`).
Built **out of primitives**, opt-in, default-off.

## Why today's work IS the foundation (not a detour)

Object-capability security IS the theory of confinement: deny all **ambient authority**, grant only
**handed** capabilities. A confined program can't reach files/network because it was never given the
*capability* to — and the kernel enforces the denial. Today we built the pieces:
- the **capability waist** (`wat-edn.cap`) — how a capability crosses a boundary (the thing a confined
  program may legitimately be handed);
- the **comms policy / powerbox** (`CommsPolicy`) — who may reach a program. **Confinement is the same
  powerbox generalized to ALL authority** (files, network, syscalls), not just peers: a
  `ConfinementPolicy` / capability-set the spawn consumes.

## The primitives (compose at the existing `clone3` seam)

- **No network** — `CLONE_NEWNET` (a fresh netns with zero interfaces → no network exists); or
  seccomp-deny `socket`.
- **No files** — empty **mount namespace** (`CLONE_NEWNS` + pivot to an empty root) and/or **Landlock**
  (the Linux filesystem-confinement LSM) and/or seccomp-deny `openat`. Strongest: no fs in the ns.
- **The floor** — `seccomp-bpf` (deny whole syscall classes), `no_new_privs`.
- The process tier ALREADY spawns via `clone3` (`src/process/clone.rs`); confinement = additional
  clone flags + a seccomp filter + a Landlock ruleset installed in the child-post-fork seam
  (`child_post_fork_init`), governed by a confinement policy on the spawn host opts.

## The spectrum (honest scope)

- **Confined process** (namespaces + seccomp + Landlock) — the natural fit for the `clone3` tier; what
  "locked down out of primitives" means here. THIS end first.
- **microVM** (gVisor / Firecracker / KVM) — true VM-grade isolation; a separate, heavier primitive.
  Name it; reach it only if a use case demands hardware-grade isolation over namespace confinement.

## Prior art (the same lineage)

**Capsicum** (FreeBSD capability mode — a process drops into a state where it may use ONLY capabilities
it already holds; the canonical ocap-confinement-on-a-real-OS); **seccomp-bpf** + **Landlock** (Linux);
**gVisor / Firecracker** (microVM confinement). All the object-capability family — the same waters 272
keeps landing in. (Pairs the REALIZATIONS ocap / narrow-waist / end-to-end synthesis — confinement is
ocap's *operational* face.)

## When it graduates

When a real "run untrusted wat safely" caller appears (the builder's anti-botnet / security-platform
work is the obvious one — [[user_career_anti_botnet]]), this becomes its own arc: `ConfinementPolicy`
on the spawn host + the clone-flag/seccomp/Landlock composition at the child seam + a probe that a
confined child genuinely cannot `open`/`socket`. Until then: banked, keeping the spawn surface general.
