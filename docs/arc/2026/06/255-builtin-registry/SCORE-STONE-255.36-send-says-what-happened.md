# SCORE — STONE 255.36: `SendOutcome` says what happened

Struck against draw `75f220b9b` (parent `ff53c96c0`, the commit this brief names).
The draw added this brief and no code. `HandleClosed` is this handle.
`Closed` is the far end. `Failed` is an io error. `Lost` is gone from a send.

## The arms bind a cause, so `Closed` keeps one

Non-comment arms outside `wat-scripts/fixes/`, on `git ls-files` of `*.wat` and
`*.wat.bad`:

| arm, before | count | files | binder |
|---|---|---|---|
| `SendOutcome.Closed` | 203 | 87 | every one `{}` |
| `SendOutcome.Lost` | 203 | 87 | every one `{:cause …}` |
| `TrySendOutcome.Closed` | 2 | 2 | `{}` |
| `TrySendOutcome.Lost` | 2 | 2 | `{:cause …}`, ignored |

The brief's 205 arms / 88 files counted copies the corpus does not. The brief's
"about 56 Lost arms read their cause" is stale. Of the 203, most bind
`{:cause _c}` and then `nil`. About ten call `LociDiedError/message`. One file
matches `LociDiedError` variants:
`docs/arc/2026/06/278-rules-engine/probes/red-send-cause-is-not-matchable.wat`.

`Closed` keeps `[cause <- Failure]`. A nullary `Closed` would make those 203
arms illegal. The cause is a description of the departure. Under the supervisor
ruling a sender never receives a death reason, so the carrier is `Failure`,
and `LociDiedError` does not ride a send outcome. `HandleClosed` stays nullary,
same as the old `Closed` arms. The codemod copies each `Lost` body onto both
`Closed` and `Failed`, and rewrites `LociDiedError/message` to `Failure/message`
inside that copy.

## The vocabulary

`:wat::kernel::SendOutcome`, non-parametric, `Pure`. `TrySendOutcome` follows
the same facts and keeps `WouldBlock`.

| variant | fact | who produces it |
|---|---|---|
| `Sent` | delivered | both loci |
| `HandleClosed` | this handle was already closed | both loci, cell `None` |
| `Closed [cause]` | the far end is gone for good | both loci, `SendError::Disconnected`. Cause text: "the far end is gone" |
| `Stopped` | a stop was requested while parked | process `SendError::Shutdown` only |
| `Failed [cause]` | an io failure | process `SendError::Failed` only |

`TrySendError` is only `Full` and `Disconnected` (`src/comms/thread.rs` documents
the same pair; the enum is `comms/mod.rs`). `try-send'` maps cell `None` to
`HandleClosed`, `Full` to `WouldBlock`, and `Disconnected` to `Closed` with
that same departure sentence. `Failed` is in the `TrySendOutcome` defenum so a
match stays exhaustive. Nothing builds it, and the unused Rust constructor was
deleted (`clippy -D warnings` refused the dead function).

A thread send never yields `Stopped`. `thread::Sender::send`
(`src/comms/thread.rs:81`) maps every crossbeam send error to
`SendError::Disconnected`. No row for a fact that cannot fire.

`RecvOutcome.Lost`, `ServiceEvent.Lost`, `CloseOutcome.Closed`, and
`LociDiedError` are untouched.

## Stop 1

Six comments in `wat/service.wat` said "client gone → keep serving" (or "still
stopping") on the old nullary `Closed` arm. Those lines are now
`HandleClosed`, and the comment is "this handle was already closed". The body
at each site keeps serving, or returns nil, on both the old `Closed` arm and
the old `Lost` arm. The rename does not change what the body does. The comment
was the misreading. The brief's lines ~1898/:1920/:1941/:1996 say "owner's
recv' already faces this" and were left as they are.

## Stop 2

`red-send-cause-is-not-matchable.wat` was the one `Lost` arm that matched
`LociDiedError.Stopped`, `Disconnected`, and the other case as different
facts. Those are now `SendOutcome.Stopped`, `Closed`, and `Failed`, and the
prints are "the process is stopping", "the peer is gone", and "an io failure".
The recv control still matches `RecvOutcome` and `LociDiedError`. Census rc
on that file did not flip.

## Rows

Pre-stone words are the draw binary, before this stone's `src/` change.
Stderr on both far-end runs was empty.

| row | pre | post |
|---|---|---|
| far end left, thread | stdout `"thread-far Lost disconnected"` | rc=0, `"thread-far Closed the far end is gone"` |
| far end left, process | stdout `"process-far Lost disconnected"` | rc=0, `"process-far Closed the far end is gone"` |
| send after this handle was closed | the draw source: nullary `send_outcome_closed()` at cell `None` (`message.rs` :200/:236/:282). Not executed. `close'` is kernel-restricted, and a user `:wat::kernel::` defn is a reserved prefix | `HandleClosed`, from `a_send_after_close_is_handle_closed` (a Rust `take` of the `PeerCell`, which is what `close'` does) |

The post far-end lines are `tests/kernel/probe_arc255_36_send_says_what_happened.wat`,
run by the sibling `.rs`. Both tests passed under the floor.

## The codemod

`wat-scripts/fixes/send-outcome-facts.wat`, applied once to the 87 tracked
files plus `wat/kernel/outcomes.wat`. A second run of that first script
renamed the new `Closed {:cause …}` arms to `HandleClosed`. The script now
renames a `Closed` arm only when its binder source is exactly `{}`. The corpus
was not run through it again. A migrate of `wat/service.wat` with the fixed
script is byte-identical. The replay fixture's `after.post` is one migrate of
`before.pre`; a second migrate matches it.

The floor that saw the bad `after.post` is `.floor/2026-09-25T09-19-09Z`:
6124 run, 6122 passed, 2 failed, 22 skipped. Both failures are the replay
gate (`every_recorded_migration_is_fixtured_or_runed` at
`tests/cli/every_recorded_migration_replays.rs:654`, coverage, and
`every_recorded_migration_replays_shard_12` at :541, `result != after.post`).
That floor was not re-run. The fixture was regenerated, and the floor below
is a new run.

Live `SendOutcome.Lost` / `TrySendOutcome.Lost` heads are gone. The name
remains in the codemod (it matches the old head), in
`wat-scripts/fixes/wrap-nested-forms-sendoutcome-stopped.wat` (a comment of
an older migration), and in
`wat-scripts/fixes/face-underscore-bound-send-prime.wat` (the old form, as
the text that migration rewrites). `RecvOutcome.Lost` stays, including the
scratch probe that synchronizes on the worker's crash.

## Gates

Floor `.floor/2026-09-25T09-36-33Z`:

```
Summary [ 345.857s] 6124 tests run: 6124 passed (10 slow), 22 skipped
```

`FLOOR_RC=0`. Clippy `cargo clippy --all-targets --workspace -- -D warnings`
is 0. Census `.census/2026-09-25T09-34-31Z.txt` against the pre-change
`.census/2026-09-25T09-02-02Z.txt`: `census-diff: no STOP-8`, files 2278,
nonzero 215. Delta `.delta/2026-09-25T09-35-33Z`: NEW 2, RECOVERY 0, exit 0.
The two NEW files are the standing conversion pair
(`probe-c1-clean-surface.wat`, `wat/holon/Ngram.wat`). The keyword heresy
ledger test passed; the frozen total is still 208. No `.edn` golden moved.
