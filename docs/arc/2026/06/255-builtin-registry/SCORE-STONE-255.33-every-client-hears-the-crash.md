# SCORE — STONE 255.33: every connected client hears the crash

Struck against draw `f4300a4eb` (parent `3ed90abeb`, the commit this brief names).
The draw added this brief and no code. The idle process client now hears the
same reasonless notice as the others. No outcome variant was renamed.

## The cause

A file trace, taken before the cure and removed before this commit. The
process child logged `broadcast peers=1` and one `try_send ok`. The parent's
read of the idle socket logged `uring-read errno=104`, which is `ECONNRESET`.

So the idle client was not in `clients`. The one client who was — the one
whose op panicked — was handed the sentinel, and that write succeeded. The
idle client's `connect` had already returned, but the connection was still in
the listen backlog. The autobind listener listens with a backlog of 128
(`src/kernel/resource.rs:129`), so a Unix `connect` completes before
`accept`. `poll'` prefers a readable client over the listener, the handler
panics, and the process dies without ever accepting the queued connection.
Death of the listening process resets that socket. The read is not a clean
EOF. It is `ECONNRESET`, which `read_into_acc` reports as `io_uring read failed`
(`src/comms/process.rs:931`).

The thread tier does not have this queue. Its rendezvous holds one value, and
`connect` does not return until `accept` has taken it. The same trace logged
`broadcast peers=2` for the thread service. Both thread clients were already
in the set, and both already heard the notice.

The sentinel was not written and then discarded. It was never written to the
idle client. A close that leaves unread inbound data is still a reset, and a
reset would drop a sentinel that had been written, so the cure also reads
those bytes away before it closes. That read does not block.

## The cure

`serve-dispatch-op` now takes the serve loop's listener as well as `clients`.
On a handler crash it still `try_send`s the sentinel to every peer already in
`clients` (`src/kernel/serve.rs:29`). Then `SocketListener::notify_pending_best_effort`
(`src/kernel/listener.rs:306`) `accept`s until `WouldBlock`, skips a peer the
gate does not admit, writes the same sentinel, discards unread inbound bytes,
and drops the socket. Nothing on this path waits. A thread listener has no
backlog, and that arm does nothing.

The notice is still `RecvOutcome.Lost` with the reason-free sentence. The
panic text stays on the owner's handle.

## The row

Pre-cure, the 255.32 probe (`rc=0`):

```
"thread client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread client-b Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread owner Lost P32-SERVICE-PANIC-REASON"
"process client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"process client-b Lost io_uring read failed"
"process owner Lost P32-SERVICE-PANIC-REASON"
```

Post-cure, ten runs, each `rc=0`, byte-identical:

```
"thread client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread client-b Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread owner Lost P32-SERVICE-PANIC-REASON"
"process client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"process client-b Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"process owner Lost P32-SERVICE-PANIC-REASON"
```

No client line contains `P32-SERVICE-PANIC-REASON`. Both owner lines are that
reason.

## Census, floor, ledger

Pre census `.census/2026-09-25T07-26-18Z.txt`: 2275 files, 215 non-zero.
Post `.census/2026-09-25T07-46-42Z.txt`: 2275 files, 215 non-zero. No path
added or removed. No file changed `rc`. `census.sh --diff`: no STOP-8.

Floor `.floor/2026-09-25T07-40-07Z`: `Summary [ 330.719s] 6118 tests run: 6118 passed (9 slow), 22 skipped`. Exit 0.

Clippy: `cargo clippy --all-targets --workspace -- -D warnings`, exit 0.

Delta `.delta/2026-09-25T07-47-34Z`: NEW 2 / RECOVERY 0. The same pair as
255.32 (`probe-c1-clean-surface.wat`, `wat/holon/Ngram.wat`).

Ledger: the floor's frozen-census test passed. `LEDGER_TOTAL` is still 208.
