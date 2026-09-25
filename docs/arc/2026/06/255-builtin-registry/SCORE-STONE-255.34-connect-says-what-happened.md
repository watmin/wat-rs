# SCORE — STONE 255.34: `ConnectOutcome` says what happened

Struck against draw `c2a8c3683` (parent `a14c9d2f3`, the commit this brief names).
The draw added this brief and no code. `Refused` is gone. Each remaining name
is one fact, and a producer builds only the facts its locus can produce.

## The table omitted the field. The arms require it.

The vocabulary table writes `Closed` with no field. Fact D in the finding says
the far end is gone for good and no reason was observed. Every consuming arm
binds `{:cause c}` and reads `Failure/message`, including the quasiquoted
assemble in `wat/core.wat` (`~assemble-c-sym`). A nullary `Closed` makes those
arms illegal.

`Closed`, `Undialable`, and `WrongPeer` keep `[cause <- Failure]`. The variant
name is the fact. The cause is the sentence the producer already built. The
two parts of the brief contradict each other; the arms win.

## The vocabulary

`:wat::kernel::ConnectOutcome :- [S R]`, and `ConnectFail` has the same facts.
`Connected` is unchanged.

| variant | fact | who produces it |
|---|---|---|
| `Connected [peer]` | dialed and admitted | both loci |
| `Closed [cause]` | nothing is listening, and it is gone for good | thread: dropped rendezvous (`address.rs`, the old `:178` arm). process: `connect_addr` returned `ECONNREFUSED` or `ENOENT` |
| `Undialable [cause]` | this address value cannot be dialed here | thread only, `tx` is `None` (inert wire copy). no process locus produces this |
| `WrongPeer [cause]` | the answerer is not who the address names | process only, `OnlyThisPeer`. no thread locus produces this |
| `Failed [cause]` | a transport io failure | process only: `peer_cred`, socket wrap, and any `connect_addr` error that is not the gone-for-good fact. no thread locus produces this |

`Refused` and the word `RETRYABLE` are gone from `address.rs`, `outcome.rs`,
`outcomes.wat`, and the duplicated comment in `types.rs`. `ServiceEvent.Rejected`,
`DeclaredTypes.Refused`, and `TerminationVerdict::Refused` are different enums
and were not touched.

## What `connect_addr` actually yields

An absent abstract name, measured before the change by connecting to
`\0wat.255.34.no-such-listener`, is errno 111 `ECONNREFUSED`
(`Connection refused (os error 111)`). `ENOENT` is not what that path yields.
Both still map to `Closed`, because the brief names both as the gone-for-good
fact. Any other `connect_addr` error is `Failed`.

A blocking `UnixStream::connect_addr` waits out a full listen backlog, so this
arm does not see `EAGAIN`. An autobind name is kernel-minted and nothing
rebinds it, so `ECONNREFUSED` on this locus means the listener is gone for
good. Stop 1 did not fire. Fact O stays reserved: no current locus produces a
retryable refusal.

## Consumers

The codemod is `wat-scripts/fixes/connect-outcome-facts.wat`. Dry-run on a
`/tmp` copy of 130 files, then apply with `./target/release/wat`. The applied
tree matched the dry-run byte for byte. A second run on the fixture is
byte-identical. Replay fixture:
`wat-scripts/fixes/replay/connect-outcome-facts/`.

- a `ConnectOutcome.Refused` arm becomes a `ConnectOutcome.Closed` arm, same body
- a `ConnectOutcome.Rejected` arm becomes two arms, `Undialable` and `WrongPeer`, each with that body
- `Connected` and `Failed` stay
- inside the `ConnectOutcome` defenum only, `:Refused` becomes `:Closed` and `:Rejected` becomes `:Undialable` plus `:WrongPeer`
- `ServiceEvent.Rejected` and other enums keep their own variants

226 `Rejected` arms. 208 are `assertion-failed!` on `Failure/message`. The rest
print that message or assert a fixed sentence. None reads the cause differently
depending on which of the two facts fired. Stop 2 did not fire. The same body
was copied.

One probe printed the old name as a label. After the codemod,
`tests/kernel/probe_arc255_29_thread_address_to_process.wat` still said
`"Refused: "` and `"Rejected: "` inside the bodies. Those four strings were
edited by hand so the row reports `Closed`, `Undialable`, and `WrongPeer`.
That is the label, not a different reading of the cause.

No `.wat.bad` and no `.edn` named the old variants. Six `.edn` goldens pin a
span line. The codemod added one physical line in `wat/core.wat` (net +1; the
`Rejected` arm's body stayed, and `Undialable` took a new line) and one line
in `probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat`. Recaptured with
`UPDATE_EDN=1`. The diff is that line number, plus one, and nothing else:

| golden | was | now |
|---|---|---|
| `probe_arc249_threading__witness_thread_first_empty_step_panics_at_expansion.edn` | `wat/core.wat` 1459 | 1460 |
| `probe_arc258_stone2b_macro_error__contract_02_non_exhaustive_cond_names_else.edn` | 1511 | 1512 |
| `wat_core_cond__cond_refuses_missing_else.edn` | 1511 | 1512 |
| `probe_arc279_format__format_strict_missing_kwarg_is_macro_error.edn` | 2008–2012 | 2009–2013 |
| `probe_arc279_format__format_strict_unused_kwarg_is_macro_error.edn` | 2036–2040 | 2037–2041 |
| `probe_arc209_c0b3bc_post_spawn__accessor_typechecks_at_parse_time.edn` | the fixture's line 18 | line 19 |

The two MCP counter fixtures (`tests/cli/wat_mcp__counter_across_turns.jsonl`
and the thread twin) embed a connect match as JSON, not as a `.wat` file.
They were updated the same way: `Closed`, then `Undialable` and `WrongPeer`,
same `assertion-failed!` body.

## Rows

Pre-stone words are `target/release/wat` as it stood at the draw, before this
stone's `src/` change. `rc` is the next statement.

| row | pre | post |
|---|---|---|
| dropped thread listener | rc=0, `"thread-dropped Refused connect: rendezvous send failed — listener was dropped (no listener)"` | rc=0, the same sentence under `Closed` |
| dead process listener | rc=0, `"process-dead Refused connect abstract UDS: Connection refused (os error 111)"` | rc=0, the same sentence under `Closed` |
| inert wire copy, both sides | rc=0, `["Rejected: a thread address is dialable only through the live value; one that crossed a wire is inert." "Rejected: …"]` | rc=0, both sides `Undialable` with that sentence |
| process wrong-peer | rc=0, `Rejected: comms policy (only-this-peer) refused the connection — server pid 466398 != minter pid 466399, or server euid 1000 != our euid 1000 (the answerer must be the exact process that minted this address)` | the same sentence under `WrongPeer`, pids of the run |

The wrong-peer drive is a live abstract listener in this process and an
address whose minter pid is `getpid() + 1`. `SO_PEERCRED` is set at `connect`,
so the gate runs without an accept. The `OnlyThisPeer` unit tests synthesize
`PeerCred` and do not connect; this one does. The post assertion is
`assert_eq!` of the whole sentence, with the pids and euid of the run filled
in, because a `starts_with` is a loose assert.

`tests/kernel/probe_arc255_34_connect_says_what_happened.wat` is the two
`Closed` rows. The inert row is the updated 255.29 probe. The wrong-peer row
is `a_live_connect_to_the_wrong_minter_pid_is_wrong_peer` in `address.rs`.

## Gates

The first floor, `.floor/2026-09-25T08-20-24Z`, was red:

```
Summary [ 330.511s] 6120 tests run: 6111 passed (9 slow), 9 failed, 22 skipped
```

Nine failures, each named from that log, not re-run:

1. `no_loose_string_assert` — `address.rs` `starts_with` on the wrong-peer sentence. Replaced with `assert_eq!` of the whole sentence.
2. `wat_mcp::a_counter_increments_across_turns` and `a_thread_counter_increments_across_turns` — the jsonl programs still matched `Refused` / `Rejected`. Updated.
3. Six `assert_edn_matches_file!` goldens — the span line moved by one, as the table above. Recaptured. The diagnostic text did not change.

New floor, `.floor/2026-09-25T08-33-51Z`:

```
Summary [ 331.873s] 6120 tests run: 6120 passed (9 slow), 22 skipped
```

Clippy `cargo clippy --all-targets --workspace -- -D warnings` is 0, including
a recheck after the exact assert. Census `.census/2026-09-25T08-18-22Z.txt`
against `.census/2026-09-25T07-51-27Z.txt`: `census-diff: no STOP-8`, files
2275, nonzero 215, zero rc changes. Delta `.delta/2026-09-25T08-19-34Z`:
NEW 2, RECOVERY 0. The keyword heresy ledger test passed; the frozen total
is still 208.

`ConnectOutcome.Refused`, `ConnectFail::Refused`, and `RETRYABLE` remain in
the codemod, its replay fixture, this brief, and the finding. Those name the
old head on purpose. They are gone from `src/`, `wat/`, and the tests.
