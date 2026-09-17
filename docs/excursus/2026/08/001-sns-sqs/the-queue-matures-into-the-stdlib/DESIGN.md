# DESIGN — the queue matures into the stdlib

**Drawn 2026-09-17**, builder-directed:

> *"i think we've battle tested queue and topic to the point both can mature into the stdlib.. both
> being used in anger have revealed and demanded corrections for several substrate items..... let's get
> them promoted and then resume this crusade against ungraceful failures"*

**Stone 1 of 3. NOT STRUCK.** Sequence: **queue → topic → move `bracket.wat` after them**, because
`wat-scripts/topic/sns-fanout.wat:41` does `(:wat::load-file! "../queue/sqs.wat")` and names `:queue::`
**154** times, while sqs names `:demo::` **zero**. The dependency is one-way, so queue promotes alone.

## ⭐ THE CENTRAL INSIGHT: PROMOTION IS A SPLIT, NOT A MOVE

`wat-scripts/queue/sqs.wat` is **2288 lines of library and its own test suite in one file**. Its 23
`:user::` definitions are the proof:

```
CLIENT API        dial-queue · dial-queue-peer · send · receive · receive-wait · ack ·
                  send-ok! · park-receive! · recv-envelopes! · await-timer-ms · join-bodies ·
                  read-call-counters · read-queue-counts
TEST DRIVERS      lp-send-wakes · lp-timeout · lp-fifo · lp-fewer-receives · lp-idle ·
                  lifecycle · depth · long-poll · compute · main
```

⛔ **`:user::` IS THE SPLIT CRITERION, and it is principled rather than arbitrary.** The builder's own
definition: *"the 'user' namespace is a rendezvous for known locations user code must be — the kernel
will invoke `:user::main`."* A name in `:user::` exists so something **outside** the program can find
it. **Nothing in the stdlib may be there.** So every `:user::` name is either (a) client API that must
be renamed into `:wat::queue::`, or (b) a driver that stays behind in a `wat-scripts/` program.

⚠ **Five rust tests drive the program half through that rendezvous** —
`probe_queue_long_poll.rs`, `probe_queue_depth.rs`, `probe_ex001_queue.rs`, `probe_async_publish.rs`,
`probe_queue_visibility.rs`. They must keep passing against whatever remains in `wat-scripts/`.

## The measured facts

```
wat-scripts/queue/sqs.wat                    2288 lines
defines                                      :queue:: 14 · :queue::queue:: 4 · :user:: 23
needs wat/query.wat (manifest 49)            147 references  ⇒ it must land at 50+
blast radius of `:queue::`                   30 files · 2035 occurrences corpus-wide
manifest today                               55 entries; bracket 23, service 35, query 49
```

⛔ **2035 occurrences across 30 files is a CODEMOD, not hand edits.** `holon/CLAUDE.md` is explicit and
this is exactly its case: write `wat-scripts/fixes/<migration>.wat`, dry-run on a copy and **diff** it,
then apply to every path, idempotent, committed as the recorded migration. Do **not** reach for
python/sed.

## The target shape

```
wat/queue.wat            NEW, manifest position 50 (immediately after wat/query.wat)
                         namespace :wat::queue::  — the library half, no :user:: names
wat-scripts/queue/sqs.wat  REMAINS, shrunken: the drivers the five rust tests rendezvous with
```

⚠ **`sns-fanout.wat:41`'s `(:wat::load-file! "../queue/sqs.wat")` must stop loading the library half**,
or the definitions arrive twice. Decide and state what that line becomes — the topic stone inherits
whatever you choose.

## Trap-doors

1. ⛔ **The stdlib is FROZEN INTO THE BINARY at build time.** A new entry needs both the
   `include_str!` and the manifest row, and `src/freeze.rs` participates. A stdlib file may not read
   `argv`, may not `readln`, and may not assume a program context.
2. ⛔ **Load order is a hard wall.** `wat/queue.wat` at 50 may name anything at ≤49 and **nothing
   after**. It needs `query.wat` (49) — that is why 50, not earlier. Verify every namespace it names
   resolves at that position; a single forward reference fails the build.
3. ⛔ **`:user::` names in the stdlib half are the bug this stone exists to prevent.** Zero may remain.
4. **The five rust probes are the contract.** If a driver they call moves into the stdlib, they break —
   and if one of them starts passing for the wrong reason, nobody notices. Run them explicitly.
5. **`wat-scripts/` is under the load gate.** Every `.wat` there is parsed and type-checked; a
   half-migrated corpus reddens it loudly, which is the desired behaviour.
6. **Do NOT rebuild brackets on queue here.** That is the destination (stone 3+) and bundling it would
   make one floor prove two things.

## Out of scope

- **topic** — stone 2, and it depends on this landing first.
- **moving `bracket.wat`** — stone 3. Measured today: nothing in the stdlib consumes `bracket.wat`
  (the only outside references are a comment in `process.wat:16` and `spawn.wat` *defining*
  `:wat::bracket::PoolMsg`), so the move is legal — but it buys nothing until brackets is actually
  rebuilt on queue, which is stone 4.
- **Any behaviour change.** This is a relocation and a rename. If the queue's semantics change, the
  stone has failed.
