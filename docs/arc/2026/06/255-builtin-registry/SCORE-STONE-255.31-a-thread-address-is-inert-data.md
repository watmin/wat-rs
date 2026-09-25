# SCORE — STONE 255.31: a thread address is inert data

Struck against draw `97fbb835c` (parent `aca395318`, the commit this brief names).
The draw added this brief and no code. The registry is gone. A decoded thread
address does not dial.

## What was deleted, and what the wire still carries

`register_rendezvous`, `resolve_rendezvous`, the `Mutex<HashMap<u64, Weak<…>>>`,
and the `AtomicU64` id counter are gone from `src/kernel/address.rs`. Grep of
that file for those names is empty.

Callers of the registry, before the delete: mint (`Address::from_thread`) and
trusted decode (`Address::from_thread_wire`). The only decode caller outside
the unit tests is `src/capability/registry.rs:289`. The echo row dials through
`connect`, not through the map. Nothing else needed a decoded thread address
to be dialable. Stop 1 did not fire.

Kept, because a `Status` or `PoolMsg` holding an address has to stay encodable:

- The record `:wat::kernel::ThreadAddressWire` with both i64 fields,
  `minter-pid` and `id` (`wat/spawn.wat`). The codec requires two fields.
  Dropping one would change the wire shape.
- Trusted-door encode and decode (`src/capability/registry.rs`). Decode still
  builds an `Address`. `tx` is always `None`.
- `thread_portable_form`, separate from `portable_form`, so `address-wire?`
  stays false.

`minter-pid` is `getpid()` at mint. `id` is stamped `0`. Nothing looks an id
up, so the counter had no remaining reader. `connect` does not branch on
either field.

## The sentence, and why `Rejected`

`connect` on any decoded copy — the minter's echo and a foreign process —
returns `ConnectFail::Rejected` with one sentence:

> a thread address is dialable only through the live value; one that crossed a wire is inert.

`Refused` is documented retryable ("the server may come up"). A wire copy
never becomes the live value, so that claim would be false. `Rejected` is the
not-retryable variant already in the vocabulary. `Failed` is an io error, and
this is not one. No outcome kind was added.

The live value (`tx: Some`, the address `listener'` minted) still dials. A
dropped listener on that live sender is still `ConnectFail::Refused` at
`address.rs:178`. That arm was not retargeted. The table below is the
measurement the builder asked for; this stone does not change it.

The expectations table says the child's dial "is refused". That is the
English verb. The variant is `Rejected`, for the reason above.

## Rows

Pre-stone words are `target/release/wat` as it stood at the draw (built
22:38, before this stone). `rc` is the next statement.

| row | pre | post |
|---|---|---|
| echoed thread address, and the copy the child dials | rc=0, stdout `["Rejected: thread address minted by process 2550253 dialed from process 2550423 — a thread address is dialable only inside its minting process" "Connected"]` | rc=0, both sides `Rejected` with the inert sentence |
| thread-locus defservice | rc=0, stdout `"echo:hi"` | rc=0, stdout `"echo:hi"` |
| bracket kwargs pool | rc=0, stdout `["echo:a" "echo:b" "echo:c"]` | rc=0, same stdout |
| pure payload on a thread peer | rc=0, stdout `7` | rc=0, stdout `7` |
| struct on a thread peer | rc=3, `:wat::core::fn`, `:p30::S` | rc=3, same wall, path now `.wat.bad` |

Post, the echo row's stdout is:

```
["Rejected: a thread address is dialable only through the live value; one that crossed a wire is inert." "Rejected: a thread address is dialable only through the live value; one that crossed a wire is inert."]
```

`git mv` renamed `tests/kernel/probe_arc255_30_struct_on_thread_peer.wat` to
`.wat.bad`. The four drivers that call `startup_from_file` on it use the new
path. The `.wat.bad` gate saw it (the floor's shards passed): startup returns
`Err`, so the file needs no exemption rune.

## Can a dropped thread listener come back?

No. `CrossbeamListener` holds the rendezvous `Receiver` (`listener.rs:104`).
The live `Address` holds `Arc<Sender>` of that same channel. Dropping the
listener drops the receiver. A later `send` on that sender returns
`Disconnected` for good: nothing installs a new receiver on an existing
crossbeam sender. `listener'` mints a new pair and a new address. Retrying
`connect` on the old address hits `address.rs:178` again.

The listener side of that same drop is `AcceptFail::Closed` (`listener.rs:129`),
a terminal. The dialer side is `ConnectFail::Refused`, whose document says
retryable. On a thread, that claim is false.

## Outcome vocabulary, measured and not changed

Producing lines only. A locus with no line does not construct that variant.
Claims are the words on the variant (`wat/kernel/outcomes.wat`,
`wat/spawn.wat`, and the Rust docs on `ConnectFail` / `AcceptFail`).

### `ConnectOutcome` and `ConnectFail`

`connect_as_value` (`address.rs:406-410`) maps `Ok` → `Connected`, and each
`ConnectFail` arm onto the same-named outcome. The lines below are where the
`ConnectFail` is built. `Connected` has no `ConnectFail` arm.

| variant | thread | process | claim | thread | process |
|---|---|---|---|---|---|
| `Connected` | live sender, rendezvous send `Ok` (`address.rs:183`) | UDS connect plus `OnlyThisPeer` admits (`address.rs:288`) | dialed and admitted | true of that dial | true of that dial |
| `Refused` / `ConnectFail::Refused` | rendezvous send `Disconnected` (`address.rs:178`) | `UnixStream::connect_addr` error (`address.rs:235`) | retryable; the server may come up | false. The sender stays disconnected. See above | the abstract name would answer if something bound those exact bytes again. `listener'` autobinds a new name and does not rebind this one, so a new listener does not revive this address |
| `Rejected` / `ConnectFail::Rejected` | decoded wire copy (`address.rs:142`) | `OnlyThisPeer` mismatch (`address.rs:267`) | not retryable | true. The wire copy never grows a channel. The doc's "wrong process" is the socket gloss: the minter's own echo is inert too | true of this dial: the answerer was not the minter. A later dial is a new attempt |
| `Failed` / `ConnectFail::Failed` | not produced | `peer_cred` (`address.rs:252`), socket wrap (`address.rs:282`) | io error, structured cause, not labeled retryable | — | true: those arms are io failures |

### `AcceptFail`

Maps to `AcceptOutcome::Closed` and `AcceptOutcome::Failed`. `Accepted` is the
`Ok(peer)` path, not an `AcceptFail`.

| variant | thread | process | claim | thread | process |
|---|---|---|---|---|---|
| `Closed` | rendezvous `Disconnected` or `Shutdown` (`listener.rs:129-131`) | reactor `Shutdown` (`listener.rs:393`) | rendezvous shut down or address dropped; clean terminal | true for `Disconnected` (the receiver is gone). False for `Shutdown`: a stop is not a drop, and this arm folds both into `Closed` | this arm is reactor shutdown, not "the address was dropped". A failed `accept()` is `Failed` instead |
| `Failed` | rendezvous decode error (`listener.rs:136`) | select io (`listener.rs:330`), `peer_cred` (`351`), wrap (`371`), `accept` io (`384`) | decode / select / peer-cred / wrap, with a cause | the arm is a decode error on an in-memory connect-request, which the doc names | true: those arms are io failures |

### `RecvOutcome`

Built in `eval_peer_recv_prime` (`src/kernel/message.rs`). `Stopped` is
`recv_outcome_shutdown` (`outcome.rs:96`), whose wat name is `Stopped`.

| variant | thread | process | claim | thread | process |
|---|---|---|---|---|---|
| `Message` | `Thread'` `469`; `Peer'` `630` (`recv_outcome_from_decoded`) | `Process'` `530`; socket `Peer'` `591` | a real message | true when the recv returned a value that is not `Reply::Failed` | same |
| `Closed` | cell `None` `467`; `Disconnected` `477`; `Peer'` wildcard `643` | cell `None` `521`; `Disconnected` `538`; socket `Peer'` wildcard `624` | genuine clean EOF; the only reason-free terminal | true for `None` and `Disconnected`. The `Peer'` wildcard is the same arm the socket side uses for `FrameTooLarge` | true for `None` and `Disconnected`. False for `FrameTooLarge`: `message.rs:620` says an over-budget frame is not a clean close, and the wildcard still reports `Closed` |
| `Stopped` | `Thread'` `480`; `Peer'` `642` | `Process'` `541`; socket `Peer'` `618` | nothing died, nothing closed; peer alive, channel open | true: `RecvError::Shutdown` is a stop | true, same arm |
| `Lost` | `Thread'` crash `473`; `Peer'` `Failed` `632`, `PeerCrashed` `636`; `Reply::Failed` via `from_decoded` | decode failure `531` and `595`; crash `536` and `609`; `Failed` `603` | abnormal loss. The carrier type is `LociDiedError` | true for a crash. A `Reply::Failed` is a live peer's refusal reported as a death | true for a crash. False as "died" for an EDN decode failure or `Reply::Failed`: the peer can still be alive. `ServiceEvent` has `Malformed` for that fact; `RecvOutcome` does not |

### `SendOutcome`

Built in `eval_peer_send_prime`. `send_outcome_from_error` (`outcome.rs:166`)
sends `Shutdown` to `Stopped` and `Disconnected`/`Failed` to `Lost`.

| variant | thread | process | claim | thread | process |
|---|---|---|---|---|---|
| `Sent` | `Thread'` `202`; `Peer'` `297` | `Process'` `239`; socket `Peer'` `290` | delivered | true on `Ok` | true on `Ok` |
| `Closed` | cell `None`: `Thread'` `200`, unified `Peer'` `282` | cell `None`: `Process'` `236`, unified `Peer'` `282` | this handle was already closed | true. Terminal for the handle | true, same |
| `Stopped` | `from_error` on `SendError::Shutdown` (`203`, `298`) | same (`240`, `291`) | stop while the write was parked; peer alive, channel open | true of a stop | true of a stop |
| `Lost` | `Disconnected` or `Failed` (`203`, `298`) | same (`240`, `291`) | disconnected mid-send; terminal | true. A dropped receiver does not come back. This variant does not claim retryable | true for that connection. The fd is gone |

### `ServiceEvent`

`poll'` and `select'` in `src/kernel/message.rs`. The process `poll'` re-poll
(spurious `POLLIN`) repeats the client arms at `2049` (`Message`), `2059`
(`Malformed`), `2074` (`Rejected`), `2085` (`Closed`). Its self-peer arm
(`2020`) is `Shutdown` for both `Ok` and `Err`, so an admin payload on that
rare path is reported as `Shutdown`.

| variant | thread | process | claim | thread | process |
|---|---|---|---|---|---|
| `Shutdown` | `poll'` self-peer `Err` `1617`; `select'` on `Thread'` when the classifier says stop `872` | `poll'` self-peer `Err` `1801`; re-poll index 0 always `2020`; `select'` on `Process'` `1029` | owner dropped the handle | true of the `poll'` `Err` arm. The `select'` arm is a stop, which the comment at `864` says is not a per-peer death; the variant still says shutdown | true of `1801`. False on the re-poll: `2020` does not look at `Ok(msg)`. The `select'` arm is the same stop-as-shutdown fold |
| `Admin` | `poll'` self-peer `Ok` `1611` | `poll'` self-peer `Ok` `1794`. Not on the re-poll | owner sent an admin op | true | true on the main arm |
| `Connection` | `poll'` listener `1638` | `poll'` accept `1956` | a client dialed and was admitted | true | true. A stranger is dropped inside the loop and does not become an event |
| `Message` | `poll'` `1651`; `select'` `Thread'` `839`; peers-only `select'` `1195` | `poll'` `1839` (re-poll `2049`); `select'` `Process'` `1062`; peers-only `1313` | a real message from `peers[idx]` | true on `Ok` | true on `Ok` |
| `Closed` | `poll'` any client `Err` `1667` (no crash channel); `select'` `Thread'` classifier `Closed` `857`; peers-only any `Err` `1207` | `poll'` wildcard `1886` (re-poll `2085`); `select'` classifier `1021`, timer `1037`; peers-only any `Err` `1326` | clean close of that peer | `select'` classifier `Closed` is the clean EOF. `poll'` and peers-only `select'` have no crash channel, so every error, including one that was not clean, is `Closed` | classifier `Closed` and a timer EOF are clean. The `poll'` wildcard also covers `PeerCrashed`, `Shutdown`, and `Failed` (`1880`). Peers-only `select'` reports `FrameTooLarge` as `Closed`; `poll'` reports that same error as `Rejected` |
| `Lost` | `select'` on `Thread'` when the crash channel has a reason `851`. Not on `poll'` (the comment at `1659` says so). Not on peers-only `select'` | `select'` on `Process'` `1010`. Not on `poll'`. Not on peers-only `select'` | abnormal loss, with a cause | true when the classifier says `Lost` | true when the classifier says `Lost` |
| `Malformed` | not produced. A thread peer does not decode a wire | `poll'` decode failure `1846` (re-poll `2059`) | peer still alive; the message did not decode | — | true: the serve loop replies and keeps serving |
| `Rejected` | not produced. Crossbeam has no frame budget | `poll'` `FrameTooLarge` `1870` (re-poll `2074`) | over the service's frame budget; that client is evicted; the service continues | — | the variant is built for `FrameTooLarge` on `poll'`. Peers-only `select'` does not build it |

No variant above was retargeted.

## Census, floor, ledger

Pre census `.census/2026-09-25T06-04-10Z.txt`: 2274 files, 216 non-zero.
The struct row was the extra non-zero (`rc` 1).

Post `.census/2026-09-25T06-22-35Z.txt`: 2273 files, 215 non-zero. The only
path difference is `tests/kernel/probe_arc255_30_struct_on_thread_peer.wat`
leaving the `*.wat` set. No other file changed `rc`. `census.sh --diff`:
no STOP-8.

Floor `.floor/2026-09-25T06-15-55Z`: `Summary [ 330.896s] 6116 tests run: 6116 passed (9 slow), 22 skipped`. Exit 0.

Clippy: `cargo clippy --all-targets --workspace -- -D warnings`, exit 0.

Delta `.delta/2026-09-25T06-23-36Z`: NEW 2 / RECOVERY 0. The two NEW files are
the same pair as 255.30 (`probe-c1-clean-surface.wat`, `wat/holon/Ngram.wat`).

Ledger: `the_heresy_ledger_matches_its_frozen_census` passed. `LEDGER_TOTAL`
is still 208.
