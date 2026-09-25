# SCORE — STONE 255.35: `AcceptOutcome` says what happened

Struck against draw `392a2f21c` (parent `3c11f46da`, the commit this brief names).
The draw added this brief and no code. `Closed` is the drop. `Stopped` is the
stop. The thread `Failed` arm is gone.

## The arms bind nothing on `Closed`

Four files match `AcceptOutcome`. Each `Closed` arm is nullary `{}`. None
binds a field. `Closed` stays nullary, and `Stopped` is nullary too. The
255.34 lesson applies: the fact table's nullary `Closed` matches the arms
this time.

| file | body |
|---|---|
| `tests/comms/probe_arc272_6a_capability_handoff.wat` | `assertion-failed!` "listener closed before the parent dialed" |
| `tests/comms/probe_arc209_c0b1_thread_connection.wat` | `assertion-failed!` "listener closed before a client connected" |
| `tests/services/probe_arc209_c0b3bc_post_spawn.wat` | `assertion-failed!` "listener closed before the hook channel was accepted" |
| `tests/services/probe_arc209_c0b3bc_post_spawn_thread.wat` | the same sentence |

A `Stopped` arm was added by hand beside each `Closed` arm, with that same
body. Four sites, so no codemod. Each body dies either way. None reads a
cause, and none branches on which fact fired. Stop 1 did not fire. The
sentence still says "closed" because the body was copied, not rewritten.

## The vocabulary

`:wat::kernel::AcceptOutcome :- [R S]`, and `AcceptFail` has the same facts.

| variant | fact | who produces it |
|---|---|---|
| `Accepted [peer]` | a peer was admitted | both loci |
| `Closed` | every sender dropped; the listener is gone for good | thread `RecvOutcome::Disconnected` only |
| `Stopped` | a stop was requested; nothing was dropped | thread `RecvOutcome::Shutdown`, process `SelectOutcome::Shutdown` |
| `Failed [cause]` | an io failure | process only: `select`, `peer_cred`, socket wrap, `accept` |

Process `Closed` is gone. That arm was only ever the select shutdown
(`listener.rs`, the old `:466`). A process listener does not observe "the
address was dropped".

## The thread `Failed` arm

The brief points at `RecvError::Failed` / `PeerCrashed`. The code at the
thread accept is `RecvOutcome::DecodeError`, which is the fold `typed_recv`
makes of those two (`channel/transfer.rs`). `comms::thread::Receiver::recv`
(`src/comms/thread.rs:191`) returns only a value, `Disconnected`, or
`Shutdown`. This listener's receiver is that thread receiver
(`ReceiverInner::Comms`). `DecodeError` cannot arrive. The arm that built
`AcceptFail::Failed` from it is deleted. If it ever arrives it is a substrate
bug and raises, which is what a must-never-happen does.

## Rows

Pre-stone words are the draw binary, before this stone's `src/` change. `rc`
is the next statement.

| row | pre | post |
|---|---|---|
| dropped thread listener | rc=0, `"thread-dropped Closed"` | rc=0, the same line |
| thread stop, sender still held | rc=0, `Closed` | `Stopped` |
| process stop, broadcast readable | rc=0, `Closed` | `Stopped` |

Wat has no cascade verb. The stop rows are driven from
`tests/kernel/probe_arc255_35_accept_says_what_happened.rs` the way
`probe_arc278_shutdown_priority_is_the_ruling` fires the cascade: write the
wake pipe, `poll` until the broadcast fd is readable, then `accept`. The
thread row also calls `trigger_shutdown`, which drops `SHUTDOWN_TX`, the
sever `thread::Receiver::recv` selects on. The sender is held, so the accept
is not a drop. nextest forks each test, so the one-way cascade stays in that
process. The drop row is the sibling `.wat`.

No `.edn` golden moved. Nothing pinned a span in the files that grew by one
arm line.

## Gates

Floor `.floor/2026-09-25T08-52-29Z`:

```
Summary [ 333.312s] 6122 tests run: 6122 passed (9 slow), 22 skipped
```

Clippy `cargo clippy --all-targets --workspace -- -D warnings` is 0. Census
`.census/2026-09-25T08-50-55Z.txt` against the pre-change
`.census/2026-09-25T08-45-56Z.txt`: `census-diff: no STOP-8`, files 2277,
nonzero 215, zero rc changes. Delta `.delta/2026-09-25T08-51-41Z`: NEW 2,
RECOVERY 0. The keyword heresy ledger test passed; the frozen total is still
208.
