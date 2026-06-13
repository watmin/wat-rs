# EXPECTATIONS — Stone C0b.1 (written before the strike; the goalposts can't move)

## Mode prediction

- **Mode A — clean ship (~70%).** The Shadowdancer mirrors `eval_peer_pair_prime` for the
  three eval fns, mirrors `infer_peer_pair_prime` for the schemes, reuses `comms::thread::pair`
  + `sender_from_comms`/`receiver_from_comms`, and the probe goes green. ~60–110 min.
- **Mode B — small gap (~22%).** Likely shapes: the parametric type of the rendezvous
  (`Sender<Tuple<Receiver<S>,Sender<R>>>`) fights `infer_peer_pair_prime`'s shape and needs a
  looser scheme (note it); or the raw-half pack/unpack into a `Value` needs a specific
  `SenderInner::Comms`/`ReceiverInner::Comms` match the sketch under-specified. Surfaces; I
  decide; usually "use the correct inner accessor, ship, note the delta."
- **Mode C — STOP fires (~8%).** A `Peer'` end genuinely can't be wrapped on its owning thread
  from the packed half, or the one-way handshake genuinely can't carry the halves — a real
  design fork for the Inquisitor.

## Scorecard (Inquisitor re-runs each independently)

| # | what | command | expected |
|---|---|---|---|
| 1 | the connection round-trips | `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` | `1 passed` (returns i64 10) |
| 2 | no new nursery reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | only the 4 known (arc-255 ×2, undefined-builtin ×2) |
| 3 | wat-tests unbroken | `cargo test --release --test test 2>&1 \| tail -3` | 242 passed / 1 failed (`test_run_string_entry_direct`, pre-existing) |
| 4 | clean build + clippy | `cargo build --release` ; `cargo clippy` (touched files) | no errors, no new warnings on the new code |

Runtime: 60–110 min. If it returns under 40, that's over-specification data.

## Trap-doors (named so they can't surprise the SCORE)

- **The rendezvous type.** `Address'` carries `Tuple<Receiver<S>, Sender<R>>`. If the checker
  can't express that cleanly, the honest move is a looser scheme that still makes the probe
  green — note exactly what was relaxed; do not fake it green.
- **Depth-1 channel blocking.** `comms::thread::pair` is depth-1; `connect'` sends one
  connect-request (one slot) → fine for one client. If the probe wedges, it's a real ordering
  bug, not flake — surface it.
- **The unused `_admin` self-peer.** The service prog is `[Peer'<i64,i64>] -> nil` but uses the
  captured `Listener'`, not `self`. That's legal (self-peer not required to be used); if the
  checker rejects an unused self-peer, surface it.

## What "done" means

Probe green; nursery 4-known-only; wat-tests 242/1; build+clippy clean; the SCORE names the
`Listener'`/`Address'` representation chosen and any relaxed type-shape. No commit by the
Shadowdancer — the Inquisitor weighs against its own re-run, then commits.
