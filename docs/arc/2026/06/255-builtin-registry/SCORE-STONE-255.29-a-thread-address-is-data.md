# SCORE — STONE 255.29: a thread address is data

Struck against draw `860add63b` (parent `388e30c12`). The wire record is
`ThreadAddressWire`, fields `minter-pid` and `id`. Wire text:

```
#wat.kernel/Address #wat.kernel/ThreadAddressWire {:minter-pid P :id I}
```

Class string `wat::kernel::ThreadAddressWire`, declared in `wat/spawn.wat`,
field names taken from that record by `wat_field_names_from!`. Encode and
trusted decode go through `address_codec` in `src/capability/registry.rs`.
General decode still refuses the capability tag.

`portable_form` stays socket-only. A thread address uses
`thread_portable_form` → `(minter_pid, id)`. `address-wire?` reads
`portable_form`, so a thread address stays false.

## What the code does

The listener id is an `AtomicU64` starting at 1 (`fetch_add`, Relaxed). The
rendezvous is `Option<Arc<Sender<Value>>>`. `None` is a decoded address whose
rendezvous did not resolve here.

The registry is a process-global `Mutex<HashMap<u64, Weak<Sender>>>`. The
`Weak` is why dropping the last `Address` still closes the listener: the map
does not hold the sender. `std` and `crossbeam-channel` have no lock-free map
of `Weak`, and this stone adds no crate. The lock is taken in
`register_rendezvous` (mint) and `resolve_rendezvous` (trusted decode). A live
`connect` whose `tx` is `Some`, and every send, do not take it. Dead weaks are
pruned when `len >= 64` and `len` is a power of two.

A decoded address whose `minter_pid` is not this process connects as
`ConnectFail::Rejected`:

```
thread address minted by process M dialed from process H — a thread address is dialable only inside its minting process
```

A decoded address in the minting process whose weak does not upgrade connects
as `ConnectFail::Refused`:

```
connect: thread address id 0xID is dangling — its listener's rendezvous is gone
```

`Refused` is the retryable label (`ConnectFail::Refused`: the server may come
up). A dangling id does not come back: ids are not reused, and the map holds a
`Weak`. The outcome kind is unchanged. What this door cannot express is a
recycled pid. There is no nonce. A later process that inherits the minter's
pid and still has that id live will upgrade the `Weak` and dial.

`is_pure_type`'s `"wat::kernel::Address"` arm is pure when both payloads are,
for two arguments and for three. A Shared address is pure when its payloads
are. `is_shared_marker` remains the transport marker.

## The brief and the code

The brief names one fixture to invert, the 255.25 Shared field. 255.28 already
substitutes type arguments into a declared head, so a Shared address inside a
generic field or peer payload is pure too. Six 255.28 `.wat.bad` fixtures
started clean. They are inverted to accepted `.wat` files, with the reason in
the comment and in the driver. The generic `Struct` over `i64` and the generic
`Impure` enum over `i64` stay refused. `ThreadSelfPeer` is untouched.

## Rows

Pre-stone words are the draw binary, captured before this stone's release
rebuild. `rc` is the next statement after the wat process.

| row | pre | post |
|---|---|---|
| `tests/types/probe_arc255_25_transport_family_address_shared_field` `--check` | rc=1 | rc=0, empty |
| `tests/kernel/probe_arc255_29_shared_in_pure_record.wat` `--check` | rc=1 | rc=0, empty |
| `tests/kernel/probe_arc255_29_address_wire.wat` | rc=0, `false` then `true` | rc=0, `false` then `true` |
| `tests/kernel/probe_arc255_29_thread_address_to_process.wat` | rc=2, child dies decoding | rc=0, Rejected + the routing sentence, parent `Connected` |
| six 255.28 Shared fixtures `--check` | were `.wat.bad` (see the first red floor) | rc=0, empty |

Pre, 255.25, stderr (the whole log):

```
#wat.type/ImpureFieldInPureAggregate {:message "containment rule (arc 293.W): pure aggregate \":probe::HoldsShared\" may only hold pure fields — field \"addr\" has impure (struct) type \"(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])\". A struct cannot be reconstructed from EDN bytes across a comms boundary; a record or holon holding a struct field could never cross — it must not exist." :location #wat.core/Span {:file "src/check.rs" :line 15245 :col 29 :end #wat.core/Option.None {}} :causes [] :aggregate ":probe::HoldsShared" :field "addr" :field-ty "(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])"}
```

Pre, the 255.29 pure-record probe, same error with aggregate `:probe25529::HoldsShared`. rc=1.

The address-wire probe's first spelling used `str` with four arguments. That
pre-stone run was rc=1, `ArityMismatch`, `:wat::core::str expected 1 arguments,
got 4`. That was the probe. Two `println`s of the bools, still on the draw
binary: rc=0, stdout

```
false
true
```

Post is the same two lines. rc=0.

Pre, the process child, rc=2. Stdout (assertion) and the loci line:

```
#wat.kernel/AssertionFailure {:thread "main" :message "recv EDN decode failed: src/capability/registry.rs:97:26: unsupported substrate tag ##wat.kernel/Address (expected a SocketAddressWire record)" :location #wat.kernel/Location {:file "tests/kernel/probe_arc255_29_thread_address_to_process.wat" :line 49 :col 11} :actual nil :expected nil :frames [#wat.kernel/Frame {:file "tests/kernel/probe_arc255_29_thread_address_to_process.wat" :line 49 :symbol ":p29::recv-back"} #wat.kernel/Frame {:file "src/freeze.rs" :line 1542 :symbol ":user::main"}] :upstream-chain nil}
[#wat.kernel/LociDiedError.Panic {:message "recv EDN decode failed: src/capability/registry.rs:97:26: unsupported substrate tag ##wat.kernel/Address (expected a SocketAddressWire record)" ...}]
```

Post, rc=0, stdout (pids are this run):

```
["Rejected: thread address minted by process 1406168 dialed from process 1406401 — a thread address is dialable only inside its minting process" "Connected"]
```

The six 255.28 Shared fixtures were not re-executed on the draw binary before
the rebuild. Their pre-stone word is that they were `.wat.bad` and the 255.28
driver, green on `388e30c12` (floor 6106/6106), asserted
`ImpureFieldInPureAggregate` or the §7 peer wall. The first floor of this
stone is the post-flip word: each starts clean. After the inversion, `--check`
rc=0 and empty stderr for:

- `bare_shared_field`
- `bare_shared_peer`
- `generic_shared_field`
- `generic_shared_peer`
- `self_referential_shared`
- `unbound_var_instantiated_shared`

Unit tests in `src/kernel/address.rs` (`thread_address_is_data`), locked by
`assert_eq!` on the release binary:

- Round trip of `[7, address]`. The wire equals
  `[7 #wat.kernel/Address #wat.kernel/ThreadAddressWire {:minter-pid P :id I}]`
  with this process's pid and the minted id. `edn_string_to_value` is `Err`.
  Trusted decode connects as `Connected`. The minting listener accepts. The
  decoded address's `portable_form` is `None`.
- Dangling id, after the original address is dropped:
  `Refused: connect: thread address id 0xID is dangling — its listener's rendezvous is gone`.
- Foreign minter (`pid + 1`):
  `Rejected: thread address minted by process {pid+1} dialed from process {pid} — a thread address is dialable only inside its minting process`.

The B3 sweep of 2592 identical payload round-trips is the finding's number.
That probe is not in the tree, and this strike did not repeat it. The new unit
test is one thread address through the same door. The socket path is the
existing floor, which is green on this tree.

## Floors

Three floors. The two reds were not re-run. Each red was fixed at the cause,
then the next floor is a new capture.

1. `.floor/2026-09-25T04-30-37Z`, exit 100. Not re-run.
   `Summary [ 328.975s] 6112 tests run: 6099 passed (9 slow), 13 failed, 22 skipped`.
   Cause: the purity flip. Whole blocks are in that directory's `ARM.txt`.
   The assertion that fired:
   - `every_wat_bad_fixture_actually_fails.rs:392` — `1 .wat.bad file(s) in shard N/16 START UP CLEAN`, shards 00, 01, 03, 04, 06, 07, one file each: the six Shared fixtures named above.
   - `probe_arc255_28_purity_sees_through_a_generic.rs:47` — `must be refused (ImpureFieldInPureAggregate); it froze clean` for the four field fixtures.
   - the same file at line 61 — `must fail check` for the two peer fixtures (the panic then prints a `FrozenWorld`, because `startup_from_file` returned `Ok`).
   - `no_loose_string_assert.rs:135` — 8 sites, `contains`/`starts_with` in `src/kernel/address.rs` (535, 572, 573, 586, 588) and `tests/kernel/probe_arc255_29_thread_address.rs` (50–52).
   Fix: invert the six fixtures and replace the loose checks with `assert_eq!` on the whole message where the pid and id are known, and with a split of the process-child line where the two pids vary.

2. `.floor/2026-09-25T04-44-49Z`, exit 100. Not re-run.
   `Summary [ 329.461s] 6112 tests run: 6111 passed (9 slow), 1 failed, 22 skipped`.
   One failure: `no_inlined_edn::tests_carry_no_inlined_edn` at
   `tests/lint/no_inlined_edn.rs:805`. The offender was
   `tests/kernel/probe_arc255_29_thread_address.rs:58`, a search literal that
   opened with `[`. Fix: the `[` is checked as a char, and the search literal
   opens with the quote.

3. `.floor/2026-09-25T04-54-10Z`, exit 0. This is the floor of the tree that commits.
   `Summary [ 330.553s] 6112 tests run: 6112 passed (9 slow), 22 skipped`.

## Gates

Clippy `cargo clippy --all-targets --workspace -- -D warnings`, after
`touch` of a file in the tree being checked. Two runs on the post-inversion
tree, both exit 0, both `Finished dev profile` with no warning lines: 6.54s
before the inlined-EDN fix, 6.42s after it (the committed tree).

Census. Pre stamp `.census/2026-09-25T04-24-55Z.txt`, files=2259, 215
non-zero. Post `.census/2026-09-25T05-01-27Z.txt`, files=2269, 215 non-zero.
`scripts/replay/census.sh --diff` printed `census-diff: no STOP-8`, exit 0.
Zero files changed rc. Ten new paths, each rc 0: the three 255.29 probes, the
inverted 255.25 Shared field, and the six inverted 255.28 Shared fixtures.
`wat/spawn.wat` is rc 1 on both stamps. Checked apart: HEAD's `spawn.wat` and
the working copy both exit 1 with the same `ReservedPrefix` on
`:wat::kernel::spawn-program` (HEAD line 336, working copy line 349). The
record did not change that rc.

Delta `.delta/2026-09-25T05-02-39Z`, list
`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`, exit 0:

```
  ORIG-CLEAN  160/179
  CONV-CLEAN  158/179
  NEW         2   (orig clean -> converted broken)
  RECOVERY    0   (orig broken -> converted clean)
```

NEW files: `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat`,
`wat/holon/Ngram.wat`.

Ledger. `the_heresy_ledger_matches_its_frozen_census` passed on the green
floor. `LEDGER_TOTAL` is 211. The frozen row
`("src/check.rs", "is_pure_type", 3, "Ex3")` is unchanged. The Address arm no
longer calls `is_shared_marker`; that call was not one of the three `Ex` sites.

## STOP triggers

1. Send path. A live thread `connect` clones the `Arc<Sender>` and
   `typed_send`s the connect-request. Encoding of `ThreadAddressWire` happens
   in `address_codec`, which is the process-wire door. The registry lock is
   not on that path.

2. Timing. The B3 probe `probes-b3/b3-thread-service-roundtrip-timing.wat.txt`,
   six runs on this binary, each rc 0. B3's quoted band is launch 826–873 µs,
   steady round trip 112.8–117.0 µs.

   | run | launch ns | steady ns |
   |---|---|---|
   | 1 | 806836 | 115434 |
   | 2 | 866709 | 114839 |
   | 3 | 867750 | 118179 |
   | 4 | 869468 | 120001 |
   | 5 | 856441 | 119357 |
   | 6 | 829056 | 118659 |

   Launch run 1 (806.8 µs) is under the quoted floor. Steady runs 3–6
   (118.2–120.0 µs) are over the quoted ceiling. The steady probe is an echo
   on an already-connected thread peer. That path does not encode and does not
   take the registry lock. Reported because the numbers sit outside the quoted
   interval.

3. Census. No file changed rc. `no STOP-8`.
