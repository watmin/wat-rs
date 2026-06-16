# DESIGN — Arc 272 step 6a: the child mints, the capability rides the lineage channel (lock-step)

> Opened 2026-06-16. Grounded against HEAD `3dc77c8c`. **Supersedes the fd-inheritance draft**
> (`DESIGN-STONE-6a-listener-inheritance.md`, retired): that added a listener arg to `spawn-program'`,
> breaking the constant `(spawn-program <host> <program>)` surface the defprotocol tooling exists to
> guarantee. This redraw is the four-questions + ZERO-MUTEX winner.

## The decision (four-questions, informed)

The end-state surface is **constant**: `(wat.spawn/spawn-program (host …) (program …))` — 2-arg for every
host (thread today; process/localhost-tcp/remote-mtls later). The listener rides in the **program**,
and the **shared-memory partition** decides *how the program gets it*:

- **thread (shared memory):** the program is a CLOSURE that **captures** the in-memory listener
  (shipped — spawn.wat:184).
- **process / remote (separate memory):** the program is FORMS that can't capture, so the child
  **autobinds its own listener** (step 2b, kernel-minted, no name) and **transmits its address back to
  the parent over the self-peer** — the lineage channel.

| | A — parent mints, child inherits the fd | **B — child mints, capability over the lineage channel** |
|---|---|---|
| Obvious? | NO — listener rides in *host* (process) vs *program* (thread): asymmetric | **YES** — listener rides in the *program* for both; 1-line delta from c0b3aii |
| Simple? | NO — install_listener + dup2→fd3 + extra_preserved + spawn_process_peer change + host optional/sum | **YES** — no fd machinery, no host change; just make `Address'` a wire value |
| Honest? | YES (272 "parent mints" verbatim) | YES — autobind = no name + lineage-only delivery = 272's *core*; deviates only from its wording |
| Good UX? | host gains a variant; keeps shipped start | uniform start (per-tier launch mints its own listener) |

**B wins.** And it's the same realization the partition principle and ZERO-MUTEX both point at.

## Why B is the ZERO-MUTEX shape (not just allowed — canonical)

`docs/ZERO-MUTEX.md`: *"synchronization IS the channel handoff"* (§ tier-3), *"mutual blocking IS the
synchronization"* (§ mini-TCP). B's handoff is exactly that: the child `(send' self addr)` after it has
`listen()`ed; the parent `(recv' svc)` **blocks until that send lands**, then `(connect' addr)`. No
sleep, no poll, no mutex, no race — the parent has **perfect knowledge** (the child is listening AND
where) the instant `recv'` returns. c0b3aii *already* runs this handshake (child `(send' self 1)` READY
→ parent `recv'`s before dialing); **B just makes the marker BE the capability** instead of a bare `1`.

This also dissolves A's last edge: A would *still* need the readiness handshake (the parent can't dial
a not-yet-listening child), so A = B's handshake **plus** fd-inheritance. B folds readiness + address
into one lock-step send.

## Mechanism (grounded)

- `Peer'<I,O>`: `send'` takes I, `recv'` returns O (check.rs:11050, 11091). `self-peer (:S :R) ->
  Peer'<S,R>`; the parent handle's `recv'` == the child self-peer's **S** (c0b3aii: S=i64).
- So the child's self-peer is `Peer'<Address'<S,R>, R2>`: it `send'`s the **`Address'`** (S); the parent
  `recv'`s the capability.
- **`Address'` becomes EDN-representable** (the one substrate change): its wire form is the `Vec<u8>`
  kernel name (`SocketAddress.name`, address.rs:131 — EDN encodes a byte vector natively; no base64),
  reconstructed on the far side via the existing `from_socket_name_bytes` (address.rs:241). What crosses
  is exactly "the kernel-assigned abstract name bytes" — the capability itself.

## Keeps `spawn-program'` 2-arg

`(spawn-program' (process) <forms>)` — unchanged, the deftest-hermetic' shape (259). The listener never
enters the spawn surface. The defservice `start` rework (6b) moves listener-minting *out* of `start`
into each tier's `Host/launch` (thread mints in-process + captures; process spawns a child that
autobinds + reports) so `start` stays host-agnostic over the constant surface — author-adjacent
`start'`/`launch'`, prove, prime-drop ([[feedback_author_adjacent_prime_drop]]).

## ⛔ BLOCKED ON the recv'/send' arrow-kill (pivot, 2026-06-16)

Building the probe surfaced an **enqueued arrow**: `recv'`'s `-> :T` ascription. We will NOT use it
(it's a non-return `->`, queued for the kill in arc 258's IO cluster — *"recv/send → mirror the prime
verbs"*, root: *"the type lives in the channel; `(recv chan)` should infer T from `chan : Channel<T>`;
the `-> :T` is a crutch from before channels carried their type"*). The 1-arg `(recv' svc)` is RED at
HEAD because the `spawn-program'` handle carries **fresh independent vars** (`Process'<I,O>`,
check.rs:10827), not the child's self-peer type — so `recv'` can't infer `Address'` from the channel.

**Per [[feedback_deferred_dep_becomes_necessary_block_and_build]] + [[feedback_reach_stumble_is_the_signal]]:
6a is PARKED; we pivot to kill the recv'/send' arrow (make the handle carry its type so recv'/send'
infer; drop the ascription), then circle back onto a green 1-arg probe.** Wrapping the spawn-program
arrow-kill by starting the next arrow-kill is the grain.

## The gate probe (RED at HEAD) — capability handoff, isolated

`tests/probe_arc272_6a_capability_handoff.rs` — minimal, NOT the full poll' loop (that's proven by
c0b3aii); isolates B's one new bit:
- child: `(listener' (process) :i64 :i64)` autobind → `Bound`; `self-peer` typed to carry `Address'`;
  `(send' self (Bound/address b))`; `accept'` the parent; round-trip n→n+100;
- parent: `(spawn-program' (process) <forms>)` → `svc`; `(recv' svc)` → the minted `Address'`;
  `(connect' addr)`; `send' 5`; `recv'` → 105.

RED at HEAD: `Address'` is a `RustOpaque` with no EDN wire form → `send'`/`recv'` cannot carry it across
the process pipe. GREEN once 6a-i makes `Address'` EDN-representable.

## Decomposition

- **6a-i — ✅ DONE** (`f35bcfb5`). `Address'` crosses as a portable `#wat-edn.cap/address [bytes]` tag
  (new `wat-edn.cap` namespace — see [NOTE-portable-capability-tags.md]); decode via
  `from_socket_name_bytes`. The gate probe (`probe_arc272_6a_capability_handoff`) is GREEN end-to-end —
  child autobinds (no name) → sends `Address'` over the self-peer → parent `recv'`s the capability (no
  `-> :T`, via 258.5a) → `connect'`s it → 5→105. lib 919/36, nursery 896/4 (zero-new). The CORE
  capability handoff is proven. (Depended on arc 258.5a — `connect'` infers — both shipped.)
- **6b** — `extend-type :wat::spawn::ProcessOpts :wat::spawn::Host` `launch` (child autobinds + reports;
  parent recv's the addr; returns the `Handle`) + unify `start` (listener-minting moves into per-tier
  `Host/launch`). Zero change to the constant `spawn-program'` surface. **Full design: see
  [DESIGN-STONE-6b-process-launch.md]** — grounded crux (the child universe = stdlib + spliced forms
  only, so the service code crosses as a defservice forms bundle [A1] and `state0` crosses over the
  lineage channel [B3]); `launch` returns `Launched<S,R>{handle,address}`; decomposed 6b-i (probe) /
  6b-ii (codegen) / 6b-iii (deftest process arm). IN FLIGHT.
- **6c** — post-spawn pid-trust (the mutual euid+pid both directions; `allow'` the client in).

Pairs [[project_rendezvous_inherited_capability]] + [[project_shared_memory_partition_hosting]]
+ ZERO-MUTEX (the handoff IS the synchronization) + [[feedback_author_adjacent_prime_drop]].
