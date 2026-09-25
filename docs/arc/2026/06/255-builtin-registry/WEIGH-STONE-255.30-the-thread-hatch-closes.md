# WEIGH — STONE 255.30: the thread escape hatch closes — ACCEPTED

**Executor: grok via pulsare, commit `fda59f3e9`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T05-38-03Z` | `6116 tests run: 6116 passed (9 slow), 22 skipped` |
| a struct on a thread self-peer | `wat tests/kernel/probe_arc255_30_struct_on_thread_peer.wat` | **rc=3**, the §7 wall (pre-stone rc=0, per SCORE) |
| a pure payload · a thread-locus defservice · the kwargs bracket pool | `wat …` | rc=0 · rc=0 · rc=0 |
| `:wat::kernel::ThreadSelfPeer` in live code | grep, comments excluded | 3 hits, **all strings inside recorded migrations** (a tool is never its own input) |

Taken from the SCORE without re-running:

- clippy 0; delta NEW 2 / RECOVERY 0;
- the ledger shrank **211 → 208**;
- census `no STOP-8`, with no existing file changing rc;
- two floors, the first red fixed at its cause and not re-run to green;
- the pre-wired spawn (crash reason, join, readiness) untouched, with its rows green.

## What landed — the three-stone ruling is complete

⭐ **`ThreadSelfPeer` is gone; a self-peer is a plain `(Peer :- [S R])`.** The codemod
`thread-self-peer-to-peer.wat` (with replay fixture) deleted the `derive` and respelled 69 files. The §7
wall now also runs:

- at the thread-spawn producer;
- on a concrete `Peer` fn parameter (`infer_fn`, which catches the user's `spawn-peer` fn literal);
- on `after`'s message type.

Its remedy reads: *"A resource belongs in `:ephemeral` state, never on a channel."* The three hatch
fixtures are negative rows now.

With 255.28 (purity sees through generics) and 255.29 (a thread address is data), **the builder's ruling
holds structurally: only data crosses a comm, on every locus.**

## Findings, carried

1. ⚠ **The negative row is named `.wat`, not `.wat.bad`.** `probe_arc255_30_struct_on_thread_peer.wat` is
   rc 3 by design, but the `.wat.bad` gate never sees it, and it moved the census's non-zero count
   **215 → 216**. It should be renamed `.wat.bad`, with its driver updated. **The census baseline is 216
   until then.**
2. ⚠ **A send-time purity check refused the stdlib, on an open type variable.** Checking every `send`
   payload refused `wat/bracket.wat:735`'s `(PoolMsg.Setup :- [:D _])`, so it was taken back off. An
   **open** type variable should be decided at instantiation, not read as impure. **Unmeasured:** which
   path read it impure (255.28's substitution, or an unknown-path arm), and whether that path mis-reads
   other type variables. **For its own look.**
3. **The hole the stone did not close:** a `Process` spawn leaves `I`/`O` as fresh variables. The first
   `send` binds them, and nothing re-checks them. That is a producer-time check on an unbound type.
   **A future stone**, probably tied to finding 2.
