# DESIGN — STONE 293.W.2e: `address-wire?` — the Address answers "is this shared memory?"

## Why this stone, and why it is THIS stone first

A process may never dial a thread. Only a thread may dial a thread. A thread may dial a process.
The one question is **is this shared memory?**

That question already has two wat mouths, both on the **wrong unit** for a dial:

| verb | unit | true / Some | false / None |
|---|---|---|---|
| `peer-process` | lineage handle | wire process (there is a pid to signal) | thread |
| `peer-wire?` | connection peer | `send` would encode | shared memory, never encodes |

The unit you **dial** is an **Address**. Arc 209 C0b.2e-iii unified the runtime entity
(`Address { inner: Box<dyn CommAddress> }`) — correct. It also erased the fact from the
*type* and never minted a wat projection. `Address::portable_form()` already measures it
(`Some` = `SocketAddressWire`, `None` = crossbeam rendezvous) and is not a verb.

Live MCP, 2026-08-16: `(bracket/map (process) … :gamma hg)` with `hg` a thread-started
handle shipped `Setup` to a process runner. The child died decoding
`#wat-edn.opaque/RustOpaque`. Illegal program. The compiler could not raise: Address is
one type, `/start` returns one Handle, `Coords` is a "pure" record of those Addresses.

**This stone mints the missing mouth only.** It does not yet put the fact in the type,
does not change `/start`, does not make the compiler raise. Those are the next stones
(2f+). The measurement must be nameable before the type can carry it.

## What it delivers

`(:wat::kernel::address-wire? addr) -> :wat::core::bool`

- **true** — this address is a wire (`portable_form()` is `Some`). A process may hold it
  and dial it.
- **false** — this address is shared memory (crossbeam). Only a thread in that address
  space may dial it.
- Not an Address → `TypeMismatch` (same shape as `peer-wire?` on a non-peer).
- Wrong arity → `ArityMismatch` expected 1.
- PURE PROJECTION. No effect. No connect. No encode.

The algorithm is one line the runtime already has:

```text
addr.portable_form().is_some()
```

## The error contract (one surface decision)

The verb names the **axis**, not the locus: `address-wire?`, never `address-process?` /
`address-thread?`. Same wording as `peer-wire?` (DESIGN-STONE-the-client-validates-locally
STOP-3: "branch on whether there is a wire", never `locus == process`).

A thread address is not an error. `false` is a legal answer.

## Rooms

- `src/kernel/address.rs:306-317` — `portable_form` (the measurement).
- `src/runtime.rs:6764` / `31520-31567` — `peer-wire?` dispatch + eval (the twin to copy).
- `src/runtime.rs:25727-25753` — `eval_connect_prime` Address downcast (the type-path to reuse).
- `src/check.rs:4117` / `10801-10841` — `infer_peer_wire` (check twin: 1 arg, unify, return bool).
- `src/kernel/spawn.rs:152` — `ADDRESS_TYPE_PATH`.

## Out of scope — REJECTED (not deferred)

- Changing `Address<S,R>` to `Address<S,R,T>`. That is 2f.
- Making `/start` a defclause / parametric Handle. That is 2f.
- Making `Coords` / process `Setup` / `connect` a compile error. That is 2f.
- Overloading `peer-wire?` onto Address (`peer-wire?` downcasts `PEER_TYPE_PATH` only).
- Hoisting into the 255 builtin registry. A dispatch arm is enough; 255 is its own carve.
- A new `Value` variant. Address stays `RustOpaque` under `ADDRESS_TYPE_PATH`.

## Probe contract

`tests/comms/probe_arc293_W2e_address_wire.{rs,wat}`

- Mint a thread `Bound/address` and a process `Bound/address`.
- `(:probe::compute)` returns `[false true]`.
- **RED at HEAD:** startup/`UnknownFunction` on `:wat::kernel::address-wire?`.
- **GREEN after:** `[false true]`.

Negative control (keepable): a non-Address argument is a `TypeMismatch` naming
`Address<S,R>` — a `.wat.bad` sibling, same idiom as other kernel probes.

## Calibration

Mirror **293.W.2d**'s SCORE shape. This stone is smaller (one verb, no cascade).
Band: 15–35 min. STOP at 50 min if the rider is inventing a type parameter.
