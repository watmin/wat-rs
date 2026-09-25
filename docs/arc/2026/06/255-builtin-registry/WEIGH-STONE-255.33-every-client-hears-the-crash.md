# WEIGH — STONE 255.33: every connected client hears the crash — ACCEPTED

**Executor: grok via pulsare, commit `857ec35d5`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T07-40-07Z` | `6118 tests run: 6118 passed (9 slow), 22 skipped` |
| ⭐ the crash probe ×10 | `wat …probe_arc255_32_two_clients_see_the_crash.wat`, md5 of stdout | **10/10 byte-identical**. Every client line is the reasonless notice; the panic text appears **exactly twice**, on the two owner lines |
| can the cure block a dying process? | the listening socket's creation | **no**: `autobind_listener` creates the socket `SOCK_NONBLOCK` (`src/comms/process.rs:196`), so the accept loop ends on `WouldBlock`, and each accepted stream is set non-blocking before the write |

Taken from the SCORE: clippy 0; census `no STOP-8` (215, no rc changed); delta NEW 2 / RECOVERY 0; ledger 208.

## The cause, found by measurement: none of the brief's framings was exact

The idle process client was **never accepted**. A Unix `connect` completes into the listen backlog before
`accept`. `poll'` prefers a readable client, the handler panicked, and the process died with the connection
still queued. Its death **reset** that socket, giving `ECONNRESET` and so `io_uring read failed`. The trace
showed `broadcast peers=1` on process and `peers=2` on thread. **The thread tier has no backlog:** its
rendezvous `connect` returns only once `accept` has taken it.

## The cure

On a handler crash, after notifying every peer in `clients`, the dying service **drains its listen
backlog** (`SocketListener::notify_pending_best_effort`, `src/kernel/listener.rs` ~:306):

- it accepts until `WouldBlock`;
- it skips a peer the accept gate does not admit;
- it writes the same reasonless sentinel;
- it reads away unread inbound bytes, because a close with unread data is itself a reset that would
  discard the sentinel;
- it drops the socket.

Nothing on this path waits. The thread arm does nothing.

**The supervisor ruling now holds on every vantage, on both loci:** the server reaps with no reason; every
connected client hears a reasonless notice; the owner gets the reason.
