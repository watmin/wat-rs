# DESIGN — a peer remembers its address

Builder: *"peer should remember its address - draw it"* — after rejecting the orchestrator's
"cannot redial" as a bad reason: *"this is like saying 'i cannot make a new tcp connection…
because… i don't know the ip address?'.. who is forgetting what when?"*

**Drawn 2026-09-14. NOT STRUCK.** Stone 1 of three; the chain is reasoned and the builder confirmed it:

```
1. a peer remembers its address        ← THIS STONE (enabling; src/-level)
2. DeadlineFired hands back a FRESH peer    = D2-(b), the class fix
3. the publisher's arm uses it              = D1-(1), trivial once (2) lands
```

## WHY — the answer to "who is forgetting what when"

Measured, and the builder's instinct was right:

```
who forgot?   SocketAddress::connect            src/kernel/address.rs:248
when?         at the moment of connecting — `self` IS the address
what?         Peer::from_socket(tx, rx) -> Self { tx, rx }    src/kernel/peer.rs:356
              Peer has exactly two fields. There is no address field to put it in.
```

The dialer holds the address, builds the peer, and drops it on the floor. **Nothing about the
transport prevents remembering it** — this is an omission, not a limit.

⭑ **And it is the unfinished half of a stone already struck.** `connections-are-reacquirable` opened
with exactly this defect one layer up:

> *"Every service takes an `Address` at `:init`, dials it once, keeps the `Peer`, and **discards the
> address**. So there is nothing to re-dial. And accordingly, **all 20 `RecvOutcome::Lost` arms … are
> `assertion-failed!`** — a lost connection kills the service."*

That stone taught **services** to remember. It left the **peer** forgetful, and every recovery path
since has had to carry an address alongside the peer by hand — which is why `circuit.wat:1000` can
redial (it has `Record/queue-addr`) and `call-by-deadline` cannot (it has only a `Peer`).

⛔ The live consequence is measured:
`FINDING-a-fired-deadline-desyncs-the-connection-forever.md` — a fired deadline abandons a frame and
returns the **same** peer, so every later reply on it is shifted, silently, and the generated client
method inherits it.

## ⛔ THE ONE CONTRACT DECISION

**`(:wat::kernel::dialed-from peer) -> (:wat::core::Option :- [(:wat::kernel::Address :- [I O])])`**

The `Option` **is** the answer to the asymmetry, expressed in the type rather than in a cause string.
Measured across all five peer constructors:

| site | what it is | dialable address |
|---|---|---|
| `src/kernel/address.rs:248` | `SocketAddress::connect` — **the DIALER** | ⭐ `Some` — `self` |
| `src/kernel/listener.rs:375` | `accept` — the server side | `None` |
| `src/process/verbs.rs:385` | the child's own self-peer | `None` |
| `src/runtime.rs:27397` | a dead sentinel peer | `None` |
| `Peer::from_thread` | thread tier | `None` — none exists |

⭑ **The name is `dialed-from`, deliberately not `peer-address`.** An accepted peer *has* a peer
address in the `getpeername()` sense and it is **not dialable** — the client never bound a listener on
it. Calling the accessor `peer-address` would make the honest answer (`None`) look like a missing
feature, and would invite someone to "fix" it by returning the client's autobind name. `dialed-from`
says exactly what it is and what it is not.

## Out of scope = REJECTED

- **Changing `call-by-deadline`.** That is stone 2 and the reason this one exists. This stone adds a
  memory and a reader; it changes no outcome and no behaviour.
- **Touching the publisher's arm.** Stone 3.
- **A `redial` primitive returning `ConnectOutcome`.** Tempting — one call instead of two — and
  rejected twice over: it would put dial policy in the kernel, and a non-dialed peer would have to
  answer `ConnectOutcome::Failed`, **overloading a variant whose documented meaning is a `peer_cred`
  read or socket-wrap io error** (`the-crash-surface-is-enumerated` §1). Two different causes in one
  variant is the collapse this campaign keeps removing.
- **A new `RedialOutcome` type.** A new enum for something `Option` already says exactly.
- **Making a stale peer *unusable* (linear/affine handles).** The excursus-002 wall is scope-based;
  invalidation-after-an-outcome is flow-sensitive and is a different, larger piece of work. ⭑ It is
  also **not needed**: stone 2 hands back a good peer rather than forbidding a bad one, which is
  constructive where linearity would be restrictive.

## Blast radius

```
src/kernel/peer.rs        +1 field, +1 accessor; from_socket / from_thread signatures
src/kernel/address.rs     :248 — pass `self` through
src/kernel/listener.rs    :375 — pass None, deliberately
src/process/verbs.rs      :385 — pass None
src/runtime.rs            :27397 — pass None; +the intrinsic
src/check.rs              the TypeScheme for the accessor (parametric — see trap-door 3)
.wat corpus               0 — nothing calls it yet. That is stone 2's job.
```

## Trap-doors named up front

1. ⛔⛔ **An ACCEPTED peer must NOT store the client's autobind name.** The accepted socket does have a
   remote name, and it is not dialable — the client bound no listener. Storing it produces a peer that
   **claims it can be re-dialed and cannot**: a painted brick, the exact defect
   `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md` is about.
   `None` at `listener.rs:375` is the whole correctness of this stone.
2. **Four of five constructors pass `None`, and each must be deliberate.** A default-`None` parameter
   that callers can forget is how the dialer silently stops remembering in six months. Prefer a
   signature that *requires* the decision at every site.
3. ⚠ **The accessor is parametric:** a `(Peer :- [I O])` yields an `(Address :- [I O])`. The type
   params are **erased at runtime** (`src/runtime.rs:16170`), so the value carries no evidence — the
   checker must thread them. If that cannot be expressed, STOP; a `dialed-from` that returns an
   untyped address pushes the mistake onto the caller.
4. **`Peer` is NOT a wire type** — verified this session (`PEER_TYPE_PATH` is absent from the
   capability registry's wire forms). An address field must not change that. An `Address` *is* portable
   (`address.rs:306`), so the temptation exists.
5. **Reading must not disturb.** `dialed-from` is an observation: the peer must still work afterwards,
   and nothing may be moved out of it.
6. **`cargo build --release` does not compile tests.** Use `cargo nextest run --release --no-run`.
