# FINDING — B3: a thread address can be data; the self-peer is a type, not a mechanism

**Measured 2026-09-25** in an isolated worktree at `c5585f1f8`. Nothing landed. The orchestrator re-read
the floor Summary lines and the raw timing pairs. The prototype diff and the probes ride beside this file
in `probes-b3/`, as `.txt` so no loader gate reads them.

## The question

Builder: services are locus-agnostic, so **clients and servers pass only data**. The escape-hatch finding
showed the only resources crossing thread peers are thread-tier `Address`es:

- in the lineage's `Status.Started`: 2461 sends, 35 services;
- in `PoolMsg.Setup`: 3 sends.

**B3** proposes that a thread address itself become data: a portable **capability** like the process
tier's `SocketAddressWire`.

## Part 1 — B3 is feasible, measured, at no measurable cost

**The prototype** (+295/−4 across 3 files):

- `defrecord :wat::kernel::ThreadAddressWire [minter-pid nonce id]`;
- a process-global **weak** registry of listener rendezvous. It is weak so that dropping the last
  `Address` still closes the listener, preserving today's lifetime;
- a per-process `(pid, random nonce)` stamp;
- encoding through the **same trusted door** as the socket tier, while general decode still refuses it;
- `connect` on a decoded address: `Rejected` from another process instance, `Refused` if the id is
  dangling.

**Floors** (worktree `.floor/`):

- baseline `6094 passed`;
- prototype `6097 passed`, after one red caused by the prototype's own test assertions was fixed and the
  whole floor re-run.

**Every thread-tier payload now round-trips as data.** A probe pushed every `send'` payload across the
floor through the real process-wire encoder, then `decode_trusted_wire`, then re-encoding: **2592 OK,
identical, 0 errors.** 2464 of them were thread tier: exactly the finding's 2461 + 3.

**End to end:** a thread address shipped to a child process is dialed there and `Rejected` ("dialable
only inside its minting process"), while the echoed copy connects in the parent. **Before B3 the child
died:** `unsupported substrate tag ##wat.kernel/Address`.

**Cost.** The thread tier still does not serialise. Values pass through crossbeam, and encoding happens
only at a process wire. Six interleaved runs each (`probes-b3/timing-ab.txt`):

| | base | B3 |
|---|---|---|
| launch cycle (start, handshake, connect, one echo, drop) | 831–873 µs | 826–858 µs |
| steady round trip | 113.8–115.9 µs | 112.8–117.0 µs |

**No difference outside noise.**

## Part 2 — the self-peer: the builder's critique lands on the TYPE, not the mechanism

⭐ **At runtime a thread self-peer is an ordinary `Peer`.** `spawn_thread_peer` (`src/kernel/spawn.rs`
~:670–842) hands the child `Peer::from_thread(…)` under `PEER_TYPE_PATH`, the same runtime type `connect`
returns. **`ThreadSelfPeer` exists only in the checker, as a purity exemption.**

What the spawn actually provides is **not on the child's channel**:

- channels that exist before the child runs (no rendezvous);
- a crash channel, which gives a structured `Lost` reason;
- a join handle;
- the readiness barrier.

| thing | replaceable by a Peer the child gets from `connect`? |
|---|---|
| the data channel | **Y**: same runtime type |
| the `ThreadSelfPeer` checker head | **Y, once thread addresses are data.** Its only measured runtime customers are those 2464 sends |
| `derive Peer ThreadSelfPeer` | **Y**: it goes away with the head |
| `poll`'s owner-EOF → `Shutdown` | **Y** |
| crash reason, join, pre-wiring, readiness | **N.** A connect model deadlocks the owner in `accept` when the child dies before dialing (`accept` is Closed only when every Sender is gone), and it loses the crash reason. Rebuilding those needs an accept-or-crash wait plus crash-channel pairing, which **re-creates the self-peer** |

**So:** under B3, the self-peer can be typed as a plain pure `(Peer :- [S R])`, with the hatch and the
derive deleted, **while the pre-wired spawn stays.** The owner still holds `Thread'`/`Process'` for crash
and join. (Not measured: a floor with `ThreadSelfPeer` removed. The nearest measurement is the
escape-hatch finding's closed-hatch floor, where 3 fixtures change.)

**A Shared address made pure:** flipping that one purity arm reddens **exactly one fixture** (2 tests):
`probe_arc255_25_transport_family_address_shared_field.wat.bad`.

## Design questions for the builder, each tied to its measurement

1. **"Portable" and "dialable from another locus" are two properties.** `address-wire?` reads
   `portable_form().is_some()` as "a process may dial this", so B3 had to add `thread_portable_form`
   beside it. Should both survive, or should one door answer both through the `Transport` marker?
2. **Should a Shared `Address` be pure?** One fixture changes. (The generic-enum purity hole is
   independent of this and stands.)
3. ⛔ **Is a random token enough authentication on the thread tier?** The trusted door rebuilds any
   registered capability tag from lineage bytes, and a thread listener has no accept gate. A process
   child could hand-write a `ThreadAddressWire` and the parent would dial it if the 64-bit id + 64-bit
   nonce matched. The socket tier gets a **kernel-vouched** pid instead.
4. **ZERO-MUTEX:** the prototype's registry uses a `Mutex`. Alternatives: a lock-free map, or no registry
   (one-way portability, where a decoded thread address is always `Rejected`/`Refused`).
5. **The dangling-id outcome:** measured `Refused`, which is labelled *retryable*, but that id can never
   return.
6. **Keep the pre-wired spawn and delete only the `ThreadSelfPeer` type** (the measurement's reading), or
   go to the connect model (which needs the two new primitives above)?
7. **Is the lineage under the client/server rule?** B3 makes its payloads data. It still carries crash,
   join and readiness, which no connection carries.
