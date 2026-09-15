# BRIEF — a peer remembers its address

**Read `DESIGN.md` beside this first.** It carries the one contract decision, five constructor sites
with their answers, three affirmatively rejected alternatives, and six trap-doors — the first of which
is the whole correctness of the stone.

## The work, in one paragraph

`Peer` has two fields and neither is an address, so `SocketAddress::connect` builds a peer and drops
the address it dialed with. Give `Peer` a third field holding the address it was **dialed from**, set
it at the one constructor that has one, pass `None` deliberately at the other four, and expose it as
`(:wat::kernel::dialed-from peer) -> (Option :- [(Address :- [I O])])`. Nothing calls it yet; this
stone adds a memory and a reader and changes no behaviour.

## The rooms

| where | why you are going there |
|---|---|
| `src/kernel/peer.rs:265` `pub struct Peer` | two fields today (`tx`, `rx`). Yours is the third. |
| `src/kernel/peer.rs:356` `from_socket` · `:340` `from_thread` | the constructors whose signatures gain the decision. |
| `src/kernel/address.rs:248` | ⭐ THE ONE SITE WITH AN ADDRESS — inside `SocketAddress::connect`, where `self` *is* it. |
| `src/kernel/listener.rs:375` | ⛔ `accept`. Pass `None`. Trap-door 1 — read it before you touch this line. |
| `src/process/verbs.rs:385` · `src/runtime.rs:27397` | the self-peer and the dead sentinel. `None`, deliberately. |
| `src/check.rs:4676` + `infer_recv_prime` | ⭑ THE PROVEN SHAPE for threading a `Peer`'s type param into a result type. `recv`, `recv-by-deadline` and `close` all do it; write `infer_dialed_from` beside them. |
| `the-rope-can-be-looked-at/` (this campaign) | ⭑ THE CLOSEST EXEMPLAR — `lineage-status`, a **non-consuming peek** at a handle, added three stones ago. `dialed-from` is the same species: an observation that must leave its subject working. Copy its shape, its doc rune and its test placement. |
| `src/kernel/address.rs:289` `pub struct Address` · `:302` `from_socket_name_bytes` | what you are storing. `SocketAddress { name: Vec<u8>, minter_pid: i32 }` — check whether `Clone` is derived; it is trivially cloneable if not. |
| `src/kernel/address.rs:306` | ⚠ an `Address` IS portable across the wire. `Peer` is NOT a wire type — keep it that way (trap-door 4). |

## Implementation sketch

```rust
pub struct Peer {
    pub(crate) tx: PeerTx,
    pub(crate) rx: Box<dyn CommReceiver<Value> + Send>,
    /// The address this peer was DIALED FROM, when it was dialed at all.
    /// `None` for an accepted peer (the remote bound no listener — see the DESIGN's
    /// trap-door 1), a self-peer, a dead sentinel, and every thread-tier peer.
    pub(crate) dialed_from: Option<SocketAddress>,
}
```

```wat
;; a non-consuming observation — the peer still works afterwards
(:wat::kernel::dialed-from peer)   ;; -> (Option :- [(Address :- [I O])])
```

⭑ **Prefer a signature that forces the decision at every construction site** over a defaulted
parameter (trap-door 2). If `from_socket` takes the option as an argument, no site can silently
forget — which is the failure this stone is repairing one layer down.

## Blast radius

`src/kernel/peer.rs`, `src/kernel/address.rs`, `src/kernel/listener.rs`, `src/process/verbs.rs`,
`src/runtime.rs`, `src/check.rs`, plus a probe. **`.wat` corpus: 0** — nothing calls it yet.

## STOP triggers

1. ⛔ **STOP-1 — if an accepted peer can be made to report `Some`**, STOP. The accepted socket has a
   remote name and it is **not dialable**; storing it yields a peer that claims it can be re-dialed and
   cannot. That is a painted brick and it is the one outcome this stone must not produce.
2. **STOP-2 — if the parametric return type cannot be expressed**, STOP and report what the checker
   does instead. An untyped `Address` out of `dialed-from` pushes the mistake to the caller.
   `infer_recv_prime` is the precedent that says this is possible.
3. **STOP-3 — if reading the address disturbs the peer** (moves out of it, invalidates a channel,
   changes a later `recv`), STOP. This is a peek, like `lineage-status`.
4. **STOP-4 — if adding the field makes `Peer` serializable or wire-crossing**, STOP. An `Address` is
   portable; a `Peer` must not become so as a side effect.
5. **STOP-5 — do NOT touch `call-by-deadline`, the generated client method, or the publisher.** Those
   are stones 2 and 3. This stone's `.wat` diff is expected to be **zero**.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- ⭑ **The round trip (the deliverable):** dial a service, read `dialed-from`, `connect` the address it
  returns, and use the **fresh** peer successfully — then use the **original** peer successfully too.
  Both must work; the observation steals nothing.
- ⭑ **The negative control that matters most:** an **accepted** peer (server side, from `accept`)
  reports `None`. Also assert `None` for a thread-tier peer. A `Some` anywhere but a dial is STOP-1.
- ⭑ **A `Peer` is still not a wire type** — assert it, do not assume it.

## Shape to copy

`docs/excursus/2026/08/001-sns-sqs/the-rope-can-be-looked-at/` — the non-consuming peek this campaign
added for lineages. Same species, same discipline: observe, do not consume, and say in the doc what the
`None` means.
