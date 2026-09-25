# SCORE — STONE 255.40: the Rust recv errors say what happened

Struck against draw `064862b91`. `RecvError::Malformed` is the bad message.
`RecvError::Failed` is the transport. No wat outcome variant was added.

## C3 — the builders

Each site is told apart by the value it already holds, not by reading the
string it would have built.

| site | source | now |
|---|---|---|
| `process.rs` `recv` / `recv_wire_raw` `read_into_acc` `Err(())` | the io_uring read | `Failed("io_uring read failed")` |
| `wait_for_data_or_cascade` poll SQE `Err(e)` (two sites) | io_uring submission | `Failed`, the submission error |
| `take_frame` `FrameScan::Malformed` | `next_complete_frame` returns this only from `from_utf8`'s `Err` | `Malformed` |
| `recv_wire_raw` `from_utf8` (two sites) and `decode_frame`'s `from_utf8` | `Utf8Error`, through `malformed_utf8` | `Malformed` |
| `decode_frame` `T::from_wire` | `WireError` | `Malformed` |

A completion whose `cqe.result() < 0` in `wait_for_data_or_cascade` is still
`Disconnected`. That source is an io result, and it was never `Failed`.
Moving it would change a close. It stays.

## What each consumer does with `Malformed`

| consumer | `Malformed` |
|---|---|
| `typed_recv` (`channel/transfer.rs`) | `RecvOutcome::DecodeError(reason)`, the outcome `Failed` already built |
| `recv'` socket and thread arms (`kernel/message.rs`) | `RecvOutcome.Lost`, the wat variant `Failed` already built |
| `poll'` process client, both the main arm and the re-poll arm | the existing `Err(_)` → `ServiceEvent.Closed`. `Failed` was already on that arm |
| `classify_peer_death` | `PeerDeath::Lost(reason)` |
| `classify_peer_error`, output channel | `PeerDeath::Lost(reason)`, without reading the crash channel, same as output `Failed` |
| `classify_peer_error`, crash channel after output EOF | `classify_peer_death` |
| `Thread::recv` (`kernel/peer.rs`) | the existing wildcard. The thread tier does not build `Failed` or `Malformed` |
| `listener.rs` | no match on `RecvError` |

## C4

Before: `classify_peer_death` mapped `Err(Failed)` to `Lost`.
`classify_peer_error`, after output EOF, mapped every crash-channel `Err`
except `Shutdown` to `Closed`.

Now `classify_peer_error`'s EOF arm calls `classify_peer_death`. A
crash-channel io failure is `Lost`. It is not a clean exit: the crash
channel's read failed, and that reason is the fact. `Malformed` on that
channel is `Lost` for the same reason.

That one fact changes the wat value built from it. Process `select'` maps
`PeerDeath::Lost` to `ServiceEvent.Lost` and `Closed` to `ServiceEvent.Closed`.
`ProcessPeerBundle::recv` maps `Lost` to `PeerRecvError::Crashed` and `Closed`
to `Disconnected`, and `recv'` maps those to `RecvOutcome.Lost` and
`RecvOutcome.Closed`. The variant set did not grow. A crash-channel io
failure that used to be reported as a clean close is now `Lost`.

## Rows

| row | result |
|---|---|
| non-UTF-8 frame (`take_frame` and `Receiver::recv`) | `Malformed("non-UTF-8 bytes in frame")` |
| `decode_frame` invalid UTF-8 | `Malformed("invalid UTF-8 in frame: …")` |
| `decode_frame` `from_wire` error | `Malformed("wire decode failed: no")` |
| io_uring read of a directory | `Failed("io_uring read failed")` |
| crash-channel `Err(Failed)` after output EOF, both classifiers | `Lost("io_uring read failed")` |

## Gates

Clippy `--all-targets --workspace -- -D warnings` exited 0.

Census `.census/2026-09-25T21-15-07Z.txt` against
`.census/2026-09-25T21-10-10Z.txt`: `census-diff: no STOP-8`. 0 rc flips.
2281 files, 215 nonzero.

Delta `.delta/2026-09-25T21-16-03Z`: NEW 2 / RECOVERY 0. The two NEW files
are `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Ledger 198. `the_heresy_ledger_matches_its_frozen_census` passed.

`.floor/2026-09-25T21-17-45Z`: `6132 tests run: 6132 passed (10 slow), 22 skipped`.
