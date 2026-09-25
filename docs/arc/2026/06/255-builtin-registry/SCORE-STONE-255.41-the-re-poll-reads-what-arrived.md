# SCORE — STONE 255.41: the process poll re-poll reads what arrived

Struck against draw `3e2db3b6c`. The process `poll` re-poll on the owner's
lineage reads the frame. An admin message is `ServiceEvent.Admin`. The
owner's drop is `ServiceEvent.Shutdown`.

## The re-poll

The spurious-`POLLIN` retry sits in `eval_poll_prime`, after `accept`
returns `WouldBlock`. Index 0 used to build `Shutdown` and ignore `res2`.
The first select already did the other thing: `Ok` decodes to `Admin`,
`Err` is `Shutdown`.

Both arms now call `process_lineage_event` (`src/kernel/message.rs`
1364–1409). The helper is the main arm's body moved out: UTF-8, then
`decode_trusted_wire`, with the same two error sentences. `Err` is
`Shutdown` with no fields. The main arm's behaviour is that body.

The re-poll path is `select_raw` returning `Listener`, then `accept`
returning `EAGAIN`, then a second `select_raw`. Nothing in the harness
produces that kernel pair, and a sleep is not a way to. The helper is
pinned directly: bytes `42` are `ServiceEvent.Admin` whose field is `42`;
`RecvError::Disconnected` is `ServiceEvent.Shutdown`.
`tests/kernel/probe_arc255_41_both_poll_arms_call_the_lineage_helper.rs`
asserts `process_lineage_event(` occurs four times in `message.rs`: the
definition, the first select, the re-poll, and the unit test.

## For the sns-sqs merge

`src/kernel/message.rs` only:

| what | lines |
|---|---|
| `process_lineage_event` | 1360–1409 |
| `eval_poll_prime` first select, index 0 | 1816–1829 |
| `eval_poll_prime` re-poll, index 0 | 2036–2046 |
| `process_lineage_event_tests` | 2147–2179 |

The client arms of both selects are unchanged. Nothing was renamed.

## Gates

Clippy `--all-targets --workspace -- -D warnings` exited 0.

Census `.census/2026-09-25T22-07-15Z.txt` against
`.census/2026-09-25T22-02-53Z.txt`: `census-diff: no STOP-8`. 0 rc flips.
2281 files, 215 nonzero.

Delta `.delta/2026-09-25T22-08-03Z`: NEW 2 / RECOVERY 0. The two NEW files
are `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Ledger 198. `the_heresy_ledger_matches_its_frozen_census` passed.

`.floor/2026-09-25T22-09-19Z`: `6135 tests run: 6135 passed (10 slow), 22 skipped`.
