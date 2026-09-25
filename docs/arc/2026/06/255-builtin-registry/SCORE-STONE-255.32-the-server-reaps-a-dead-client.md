# SCORE — STONE 255.32: the server reaps a dead client

Struck against draw `94a4b2b17` (parent `d9bca6907`, the commit this brief names).
The draw added this brief and no code. The serve loop's `Lost` arm reaps and
continues. No outcome variant was renamed.

## The arm

`wat/service.wat` `ServiceEvent.Lost` bound `cause` and called
`assertion-failed!` on `Failure/message`. That raises, so the `remove-at` and
the recurse after it never ran. `poll'` does not build `Lost`, which is why
nothing hit it.

The arm is now the same shape as `Closed`: bind `_cause`, evict `idx` with
`(:wat::seq::remove-at selectables idx)`, recurse into serve. No
`assertion-failed!`, no `eprintln`. The comment states the ruling: a client's
death belongs to its own owner; the server reaps the dead handle and serves on.

## `bracket.wat` stays

Read. `wat/bracket.wat:626` is `ServiceEvent.Lost` in `collect-loop`. It
raises with the runner's cause. The pool is the owner of those runners, which
is the supervisor vantage, where raising is the ruling. The `RecvOutcome.Lost`
arms in the same file (`:49`, `:95`, `:149`, `:219`, `:499`) are a runner
seeing its parent die, and they were left alone too. `git diff` does not
touch `bracket.wat`.

## The expansion row

`tests/services/probe_arc255_32_lost_arm_expansion.wat` macroexpands a
`defservice` at the form level and prints the `Lost` arm. Pre-stone, on the
draw binary (`rc=0`):

```
"ServiceEvent.Lost {:idx idx :cause cause} (:wat.core/do (:wat.kernel/assertion-failed! :message (#wat.ast/Keyword {:path \":wat::kernel::Failure/message\"} cause)) (:p32.echo/serve self l (:wat.seq/remove-at selectables idx) next-id state))] [:wat.spawn/"
```

Post (`rc=0`):

```
"ServiceEvent.Lost {:idx idx :cause _cause} (:p32.echo/serve self l (:wat.seq/remove-at selectables idx) next-id state)] [:wat.spawn/"
```

The tail `[:wat.spawn/` is the next arm's tag, left by the split. The arm
itself is the evict-and-recurse. No `assertion-failed!`, no `eprintln`.

## Who can receive a client's death reason

A connected client is a bare `Peer'`. It has no crash channel. `poll'`'s
comment at `src/kernel/message.rs:1659` says so, and the producers agree.

| path | locus | what a dead client becomes | a reason? |
|---|---|---|---|
| `poll'` client arm `message.rs:1669` | thread | `ServiceEvent.Closed` | no |
| `poll'` client arm `message.rs:1888` (re-poll `:2085`) | process | `ServiceEvent.Closed`. `FrameTooLarge` is `Rejected` at `:1872`, an oversize frame, not a death | no. The wildcard drops `PeerCrashed`, `Failed`, and `Shutdown` on the floor |
| peers-only `select'` `message.rs:1207` | thread | `ServiceEvent.Closed` for any `Err` | no |
| peers-only `select'` `message.rs:1326` | process | `ServiceEvent.Closed` for any `Err` | no |

`select'` over an owner handle does carry a reason, and that is the
supervisor vantage, not a server reading a client:

- thread `Thread'` `message.rs:851` → `ServiceEvent.Lost` from the crash channel
- process `Process'` `message.rs:1012` → `ServiceEvent.Lost` from `classify_peer_error`

Stop 1 did not fire. Nothing on the server vantage was changed.

## What a crashing service tells its clients, and its owner

`tests/services/probe_arc255_32_two_clients_see_the_crash.wat`. A service
panics with `assertion-failed!` message `P32-SERVICE-PANIC-REASON`. Two
clients are connected before the op. One sends it. Both `recv'`, and the
owner `recv'`s `Handle/handle`. Same words pre and post (the `Lost` arm is
not this path). Four runs, identical. `rc=0`. Stdout:

```
"thread client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread client-b Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread owner Lost P32-SERVICE-PANIC-REASON"
"process client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"process client-b Lost io_uring read failed"
"process owner Lost P32-SERVICE-PANIC-REASON"
```

The thread child also writes its dying declaration to the parent's stderr.
That block is the crash, not a client notice:

```
#wat.kernel/AssertionFailure {:thread "wat-thread-peer::<anon>" :message "P32-SERVICE-PANIC-REASON" :location #wat.kernel/Location {:file "tests/services/probe_arc255_32_two_clients_see_the_crash.wat" :line 17 :col 1} ... :frames [#wat.kernel/Frame {:file "tests/services/probe_arc255_32_two_clients_see_the_crash.wat" :line 17 :symbol ":p32::boom::serve"} #wat.kernel/Frame {:file "wat/spawn.wat" :line 376 :symbol ":wat::core::Fn"}] ...}
```

**(b) Clients.** Both thread clients, and the process client that sent the
op, get `RecvOutcome.Lost` whose message is the reason-free `PeerCrashed`
sentence (`src/comms/mod.rs:425`). The panic text does not appear. The idle
process client gets `RecvOutcome.Lost` whose message is `io_uring read failed`
(`src/comms/process.rs:751` and `:931`, then `message.rs:603`). That is not
the panic reason, and it is not the sentinel. Stop 2's two triggers are "the
service's reason" and "no notice". This is a notice with the wrong text. It
is pinned and not changed. The ruling's "every connected client gets the
reasonless notice" is not what the process tier does for the client that did
not send.

**(c) Owner.** Both loci: `RecvOutcome.Lost` and the message
`P32-SERVICE-PANIC-REASON`. The owner gets the reason.

## Census, floor, ledger

The `service.wat` lines pinned by the bijection goldens are 909 and 926,
above this edit. They did not move. No `UPDATE_EDN` recapture.

Pre census `.census/2026-09-25T07-05-26Z.txt`: 2273 files, 215 non-zero.
Post `.census/2026-09-25T07-21-31Z.txt`: 2275 files, 215 non-zero. The two
new probes are `rc` 0. No other file changed `rc`. `census.sh --diff`:
no STOP-8.

Floor `.floor/2026-09-25T07-14-45Z`: `Summary [ 331.012s] 6118 tests run: 6118 passed (9 slow), 22 skipped`. Exit 0. The two new rows are the extra tests.

Clippy: `cargo clippy --all-targets --workspace -- -D warnings`, exit 0.

Delta `.delta/2026-09-25T07-22-25Z`: NEW 2 / RECOVERY 0. The same pair as
255.31 (`probe-c1-clean-surface.wat`, `wat/holon/Ngram.wat`).

Ledger: `the_heresy_ledger_matches_its_frozen_census` passed. `LEDGER_TOTAL`
is still 208.
