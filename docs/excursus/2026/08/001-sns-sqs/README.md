# excursus 001 — SNS and SQS

**What this is:** free experimentation, not commissioned work. `docs/excursus/` is the sibling
of `docs/arc/`; see `docs/excursus/README.md` for why it exists and for the `(301)` residue in
the commit log.

**Artifacts are per stone**, four kinds, all named `<KIND>-stone-<slug>.md`:
`BRIEF` (what to do) · `EXPECTATIONS` (written before, so the result cannot move the goalposts)
· `HANDOFF-…` (the executor's entry point) · `SCORE` (written after the orchestrator's OWN
re-run, never from the report).

## The stones, in order

| stone | what | state |
|---|---|---|
| 1 | **SNS in userland** — one topic, N subscribers, both loci | ✅ `wat-scripts/topic/sns-fanout.wat` → `"3 3"` |
| 2 | `:wat::query::Store` gains **`delete`** — SQS's ack needs it | ✅ struck |
| 2b | the **delete differential**, mem vs sqlite, GSI included | ✅ backends agree |
| 2c | **mem's `put` becomes a replace** — `PutItem` is the referent | ✅ and it exposed a live bug |
| INST | **`#inst` at constant nanosecond width** — one token | ✅ struck |
| WRITE-OPTS | serialization options become a **value the caller passes** | ✅ struck |
| WO-OPT | that opts argument becomes **optional** | ✅ struck |
| JOURNAL-CENSUS | run every journal fixture against **both** backends | ✅ 13/15 agree — and why that is not reassuring |
| SORTKEY | **a telemetry event carries its own identity** | ✅ first fully green floor |
| 3 | **SQS in userland** — `wat-scripts/queue/`, **wat-queue** | ✅ `"bound=x;r1=a,b;r2=c;r3=;redel=b"`, floor 5122 |
| 4 | the **fan-out circuit** — proving topic and queue compose | ⛔ **STOP-5**: wired, but process workers could not consume the queue |
| 5 | the **surface guard's reach** — it could not see a parametric field type | ✅ struck; floor deliberately red (the queue), ARM kept |
| 6 | **`Envelope` moves into `:messages`** | ✅ struck as STOP-4; queue green, `:fanout::Outcome` is the next hole |
| 7 | **the fan-out proof, re-attempted** | ✅ `"n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=9;empty=1"`, floor 5127 |

**Stones 4–7 are one arc, not four.** Stone 4 halted on a STOP trigger rather than improvising;
what it surfaced (a peer surface silently missing a domain type) took stones 5 and 6 to root out,
and only then could 7 land the proof stone 4 was drawn for. The halt is why this worked.

## The shape of the detour, because it is not obvious from the list

Stone 1 built SNS. **Drawing stone 3 is what uncovered everything between them**: `receive`
needs a *re-put* (which found `mem-store` appending where `sqlite-store` replaced), and `ack`
needs a *delete* (which the `Store` did not have). Fixing the first made `mem` stop hiding a
`journal` key collision that was silently dropping two metrics in three from every span close —
on every conforming backend. Stones INST through SORTKEY are that debt, paid.

★ **Stone 2c broke nothing. It removed a blindfold.** And the eventual fix was cheap *because*
the substrate was made honest first — `SortKey.time` is a real `Instant` rather than a
hand-padded string only because INST fixed the renderer.

## Findings that outlived their stone

- `NOTE-mem-store-put-appends-where-sqlite-replaces.md` — an oracle that admits a state the
  subject cannot represent is not an oracle. Carries a ⛔ CORRECTED section: it first called
  the divergence a tie between two defensible readings. It was not; DynamoDB rules.
- `NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md` — also ⛔ CORRECTED: the
  table at its top is **half a measurement**, and the bug is not sqlite-specific.
- `NOTE-a-userland-peer-surface-must-carry-its-domain-types-in-messages.md` — the stone-4
  wall, generalized: a type absent from `:messages` does not cross a fork, and the diagnostic
  that should have said so could not see a parametric field type.
- `NOTE-a-record-accessor-in-value-position-loses-its-receiver-type.md` — filed to its home arc
  as `docs/arc/2026/04/109-kill-std/NOTE-a-callable-keyword-in-value-position-has-four-kinds-and-three-answers.md`.

## Where the code lives

```
wat-scripts/topic/    wat-topic   — the SNS half
wat-scripts/queue/    wat-queue   — the SQS half (stone 3)
wat-scripts/fanout/   circuit.wat — the composition proof (stone 7); not a promotion candidate
wat/topic.wat, wat/queue.wat      — the builder's to grant, once they demonstrate excellence
```

**Re-run the three counts after any move** — never take them from a report. The grep precedent
(`349a2ea52`) is the promotion standard: *"mostly a MOVE of proven code, and the counts are the
proof it moved intact."*

---

# The distributed-model campaign — September 2026

**Filed here on 2026-09-08, reclaimed from `docs/arc/2026/06/278-rules-engine/`.** A session's
worth of artifacts were written into arc 278 instead of into this excursus. 417 files moved; the
authoritative census was the branch point, not the wall clock:

```
git diff --diff-filter=A --name-only main...HEAD -- docs/arc/2026/06/278-rules-engine/
```

**Structure changed with the move.** The stones above are flat `<KIND>-stone-<slug>.md` at this
directory's root. Everything below is grouped instead — one directory per effort, holding that
effort's `DESIGN.md` / `BRIEF.md` / `EXPECTATIONS.md` / `SCORE.md` (and `HANDOFF.md`,
`BRIEF-v2.md`, `SCORE-v2.md` where those exist):

```
excursus/2026/08/001-sns-sqs/<task-slug>/DESIGN.md
                                        /BRIEF.md
                                        /EXPECTATIONS.md
                                        /SCORE.md
```

★ **A `DESIGN-STONE-<name>.md` did not always share its BRIEF's name.** 24 of them paired with a
differently-named BRIEF — `DESIGN-STONE-the-async-publish.md` belonged to `BRIEF-async-publish.md`,
`DESIGN-STONE-the-batched-writer.md` to `BRIEF-item-b-batched-writer.md`. The pairing was not
guessed: **each BRIEF names its own DESIGN in its "Read in order" list**, and that citation is what
put every DESIGN in the right directory.

**Cross-cutting artifacts stay flat at this root**, beside the `NOTE-*.md` already here, because a
finding is not a task: `FINDING-batching-needs-a-linear-store.md`,
`FINDING-durability-is-store-op-bound.md`, `FINDING-the-buffer-was-the-bottleneck.md`,
`FINDING-the-drain-variance.md`, `FINDING-unacked-is-receives-minus-acks.md`, and
`TRACKER-the-distributed-model.md` — the living campaign index; read it first.

⛔ **`TRACKER-the-distributed-model.md` still says it "tracks work inside arc 278".** That line was
true when written and is not now. It is left as written rather than silently edited: it is a
statement about *whose* work this is, and correcting it is the builder's ruling, not a chore.

## The io_uring / signal / fd line went to arc 170 instead

Twenty-nine files did **not** come here. They are the transport substrate, not the distributed
model, and they joined their siblings in `docs/arc/2026/05/170-program-entry-points/` keeping their
flat filenames: the `a-cancelled-write-…` / `a-short-write-…` / `a-duped-fd-…` /
`can-an-io-uring-write-be-raced` / `a-write-never-blocks-outside-the-multiplexer` /
`sigaction-not-signal` stones, plus
`docs/arc/2026/05/170-program-entry-points/FINDING-the-writes-kept-the-1970s.md` — the floor RED that
found a months-old blocking-write bug — and
`docs/arc/2026/05/170-program-entry-points/NOTE-the-write-path-joins-io-uring.md`, its
undrawn third stone.

## The efforts

| effort | artifacts | what |
|---|---|---|
| `a-batch-declares-how-many/` | BRIEF DESIGN EXPECTATIONS SCORE | Add `:max-entries [field N]` as a surface-feature option, enforce it client-side before the send |
| `a-call-outcome-cannot-lie/` | BRIEF DESIGN EXPECTATIONS SCORE | Replace `call-by-deadline`'s `(Option O, i64)` return with a parametric three-arm enum. |
| `a-caller-chunks-to-the-declared-limit/` | BRIEF DESIGN EXPECTATIONS SCORE | Generate `<op>-all` for any op declaring both `:max-entries` and an `Accepted [count]` response, |
| `a-claim-remembers-its-owner/` | BRIEF DESIGN EXPECTATIONS SCORE | Make the claim ledger record **who** holds a seq, and let a worker's own retry be answered |
| `a-client-has-a-deadline/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone T1 — every client call waited forever for a reply that might never come; give each one a deadline. |
| `a-count-never-reads-more-than-it-needs/` | BRIEF DESIGN EXPECTATIONS SCORE | Add `limit` to `CountIndexRequest`, make both store impls saturate at it, and pass the bound the |
| `a-give-back-is-counted/` | BRIEF DESIGN EXPECTATIONS SCORE | Add one durable counter to `:fanout::worker`, surface it on the existing `disrupts` channel, and |
| `a-ledger-is-a-receipt/` | BRIEF DESIGN EXPECTATIONS SCORE | Move `:fanout::seen`'s write from **claim time** to **after the outcome is emitted**. The ledger |
| `a-message-is-fanned-once/` | BRIEF DESIGN EXPECTATIONS SCORE | When the inbox admits a prefix that splits a message's fanout, the topic tops up that message's |
| `a-peer-is-dead-only-when-redial-fails/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone 3c-pre — a severed peer is only dead once a redial fails. The gate in front of chaos. |
| `a-read-declares-its-page/` | BRIEF DESIGN EXPECTATIONS SCORE | Add `:max-page [field N]` bounding a response collection, generate `<op>-all` for cursor reads, |
| `a-reconnect-is-not-an-abandonment/` | BRIEF DESIGN EXPECTATIONS SCORE | Make `Lost` and `Closed` redial **and retry** instead of redialing and discarding the work, in all |
| `a-single-value-is-a-batch-of-one/` | BRIEF DESIGN EXPECTATIONS SCORE | Make `ack`, `check` and `mark` batch-only, and collapse the per-row folds into one call each. |
| `a-transaction-that-fails-must-close/` | BRIEF DESIGN EXPECTATIONS SCORE | Add `ROLLBACK` at all three layers and call it on the store's error paths, so one failed |
| `a-worker-gives-the-message-back/` | BRIEF DESIGN EXPECTATIONS SCORE | Restore a drop knob on `Seen/check`, then make retry-exhaustion release the envelope instead of |
| `async-publish/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Make `publish` return once the message is accepted, and deliver to subscribers on the topic's own |
| `chaos-is-a-rate/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone 3c — the disruptor: opt-in, seeded, self-arming, rate 0 by default. |
| `clamp-the-request-not-the-handler/` | BRIEF DESIGN EXPECTATIONS SCORE | Move the clamp's conditional inside the `let` binding so the handler body is spliced once. |
| `closed-is-the-postcondition/` | BRIEF DESIGN EXPECTATIONS SCORE | Add the `is_autocommit` predicate at both layers, reshape `rollback-then-err` into a |
| `connections-are-reacquirable/` | BRIEF DESIGN EXPECTATIONS SCORE | Services throw away the `Address` they dialed, so a broken pipe cannot be recovered — and all 20 |
| `deferred-reply/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Let a `defservice` arm reply to a client **other than the one that invoked it**, named by the |
| `depth-is-read-not-counted/` | BRIEF DESIGN EXPECTATIONS SCORE | Delete the queue's two hand-maintained depth counters and answer every depth question by |
| `entry-three-of-ten/` | BRIEF DESIGN EXPECTATIONS SCORE | Build a store that fails entry *k* of *n*, point a queue at it, and report what the stack does. |
| `every-client-call-has-a-deadline/` | BRIEF DESIGN EXPECTATIONS SCORE | Extract the deadline pattern into one parametric stdlib helper, then put all four of the |
| `every-tier-reports-its-backlog/` | BRIEF DESIGN EXPECTATIONS SCORE | Measurement only — the backlog counter every later decision needed and none of them had. |
| `exhaustion-cannot-be-discarded/` | BRIEF DESIGN EXPECTATIONS SCORE | Give the worker's three retry ladders one **outcome type** whose `Exhausted` variant must be |
| `exhaustion-is-a-named-variant/` | BRIEF DESIGN EXPECTATIONS SCORE | Give the worker's three retry ladders **two closed enums** whose `Exhausted` variant every match |
| `fill-deep-then-drain/` | BRIEF DESIGN EXPECTATIONS SCORE | Make the circuit able to fill its subscriber queues to a known depth with nothing consuming, then |
| `find-the-934-milliseconds/` | BRIEF DESIGN EXPECTATIONS SCORE | Find what the send arm's restructure costs, name the mechanism, and recover the time **without |
| `four-in-sequence/` | HANDOFF | Four independent stones deliberately bundled and measured between each, so no result is unattributable. |
| `is-the-queue-saturated/` | BRIEF DESIGN EXPECTATIONS SCORE | Add `handler-ns` to the queue: cumulative time spent inside op handlers. Sample it at the drain |
| `item-b-batched-writer/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Fragment an oversized batch into submissions that fit the op's declared cap, write them in order, and |
| `item-c-stone-a-buffered-span/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Make `span'` buffer its logs and emit metrics as **deltas**, so a flush can happen more than once |
| `item-c-stone-b-two-clocks/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Give the buffered span **two independent cadences** — logs flushed fast, counters and durations on a |
| `item-c-stone-c-flush-must-speak/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Three arms compute a write failure and throw it away. Make them report it. `Ok` keeps meaning |
| `item-c-stone-d-bounded-buffer/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Give `logs` and `durations` a bound, drop the oldest on overflow, count every drop, and tell the |
| `no-client-call-can-hang/` | BRIEF DESIGN EXPECTATIONS SCORE | ⛔ **THIS SUPERSEDES THE EARLIER BRIEF OF THE SAME NAME.** That one had the deadline raise inside |
| `partial-only-when-it-must/` | BRIEF DESIGN EXPECTATIONS SCORE | Take a prefix only when the request cannot fit in `cap` at all; otherwise all-or-nothing. Read |
| `perf-1-incremental-byte-measure/` | BRIEF DESIGN EXPECTATIONS HANDOFF | Stop re-encoding the whole buffer on every `log`/`incr`/`timed`. Carry a running byte total; each |
| `perf-2-store-read-path/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Make `mem-store`'s reads cost O(result), not O(table). **Measured**: 46 ms per scan returning one row |
| `perf-3-indexed-vector-update/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Give `PersistentVector` an indexed `set` and a `drop-last`, then make the store's `put`/`delete` use |
| `queue-long-poll/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Let `receive` wait for a message instead of returning empty, so a worker stops spending a network |
| `seen-ids-stops-cloning/` | BRIEF DESIGN EXPECTATIONS SCORE | The queue's `seen-ids` set uses `:wat::core::HashSet`, whose `conj` **clones the entire set on every |
| `stop-fetching-rows-to-get-a-number/` | BRIEF DESIGN EXPECTATIONS SCORE | Halve the send path's store round trips, and make each one return a count instead of rows. |
| `store-ns/` | BRIEF DESIGN EXPECTATIONS SCORE | Add `store-ns` as a **named field** on `TakeAcc` and on the queue's `:ephemeral` state, accumulate a |
| `the-accumulator-is-declared-where-the-child-can-see-it/` | BRIEF DESIGN EXPECTATIONS SCORE | Replace sqs.wat's three nested-Tuple waiter-fold accumulators with `(Tuple TakeAcc box)`, where |
| `the-ack-retries-like-the-publisher/` | BRIEF DESIGN EXPECTATIONS SCORE | Replace the ack's three-flat-tries ladder with the publisher's proven retry shape — exponential |
| `the-benchmark-has-more-than-one-publisher/` | BRIEF DESIGN EXPECTATIONS SCORE | Give `run-with` a publisher count, run P publishers concurrently each with its own seed, and |
| `the-call-site-reads-as-english/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone C — two renames at the call site. Closes no defect, and says so up front. |
| `the-circuit-goes-persistent/` | BRIEF DESIGN EXPECTATIONS SCORE | Two accumulators in the circuit are `:wat::core::Vector`, whose `conj` is O(n). Move both to |
| `the-circuit-runs-on-sqlite/` | SCORE | The circuit fixture moves to sqlite; mem-store stays the differential oracle. |
| `the-client-backs-off-intelligently/` | BRIEF DESIGN EXPECTATIONS SCORE | Replace the publisher's fixed `await-timer-ms 1` with exponential backoff, full jitter and a cap. |
| `the-consumer-is-idempotent/` | BRIEF DESIGN EXPECTATIONS SCORE | The circuit asserts exactly-once on an at-least-once system, and its duplicate detector cannot see |
| `the-dedupe-map-stops-cloning-itself/` | BRIEF DESIGN EXPECTATIONS SCORE | Change `Seen`'s `claimed` from `HashMap` to `PersistentMap` and move its call sites from |
| `the-drain-reports-its-own-store-time/` | BRIEF DESIGN EXPECTATIONS SCORE | Sample `sum-store-calls` and `sum-store-ns` at the drain's phase boundaries and report the delta |
| `the-fanout-is-concurrent/` | BRIEF DESIGN EXPECTATIONS SCORE | The topic's `-deliver` awaits each subscriber before sending to the next, so one message costs the |
| `the-gsi-delete-gets-its-reverse-mapping/` | BRIEF DESIGN EXPECTATIONS SCORE | `sqlite-store.wat` models DynamoDB: a GSI is a separate table, and a delete removes the row's projection |
| `the-inbox-holds-messages-not-pairs/` | DESIGN | DESIGN ONLY, arc-shaped — move the ×nsubs expansion off the inbox so tier 1 holds messages, not pairs. |
| `the-instrument-fits-the-question/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone D — the measurement helpers stop lying about what they consume and about what they saw. |
| `the-instrument-reports-what-happened/` | BRIEF DESIGN EXPECTATIONS SCORE | Split `stop` into `collect` + `stop`, and add a counter for total publish round trips. |
| `the-ledger-counts-what-it-absorbs/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone S30 — the instrument that makes `dup=0` mean something. |
| `the-message-carries-its-trace/` | BRIEF DESIGN EXPECTATIONS SCORE | Make each message carry timestamps through the circuit, and report per-stage latency as a histogram |
| `the-outcome-composes/` | BRIEF DESIGN EXPECTATIONS SCORE | `Outcome` encodes four independent things — state, a reply to my caller, sends to other named |
| `the-outcome-crosses-the-resource-stays/` | BRIEF DESIGN EXPECTATIONS SCORE | Give the worker's three retry ladders a **nullary-payload closed enum** whose `Exhausted` variant |
| `the-poller-sweeps-once/` | BRIEF DESIGN EXPECTATIONS SCORE | `fanout::poll-until-drained*` asks the queue services for the same `Queue/stats` data three times |
| `the-queue-can-drop-too/` | BRIEF DESIGN EXPECTATIONS SCORE | Give `:queue::queue` per-verb drop knobs for `receive` and `ack`, so the last two undeadlined- |
| `the-queue-counts-its-store-round-trips/` | BRIEF DESIGN EXPECTATIONS SCORE | Add one counter to `Queue::StatsResponse`: how many round trips this queue has made to its store. |
| `the-queue-is-bounded/` | BRIEF DESIGN EXPECTATIONS SCORE | `Queue/send` always accepts, so bounding the topic only moved the reservoir downstream. Give the |
| `the-queue-reports-time-inside-its-store/` | BRIEF DESIGN EXPECTATIONS SCORE | Add a second counter to `Queue::StatsResponse`: nanoseconds spent inside `Store/*` calls. Sum it |
| `the-sane-circuit/` | BRIEF DESIGN EXPECTATIONS HANDOFF SCORE | Make `circuit.wat` a program that could exist: consumers that consume until stopped, a producer that |
| `the-server-drops-a-reply/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone R2 — a rate-gated reply drop inside the reactor seam. NOT STRUCK. |
| `the-server-manages-its-own-capacity/` | BRIEF DESIGN EXPECTATIONS SCORE | Make `Queue::send` admit a prefix of what it is handed, report `Accepted [count]`, and stop |
| `the-sets-get-a-persistent-variant/` | BRIEF DESIGN EXPECTATIONS SCORE | `:wat::map::` is `PersistentMap` and shares structure; `:wat::hashmap::` clones. The set family only has |
| `the-slope-belongs-to-a-phase/` | BRIEF DESIGN EXPECTATIONS SCORE | Run the circuit at **n=1000, 2000 and 4000**, at least **three times each**, with `vis-ms=1000` pinned and |
| `the-store-reports-time-per-operation/` | BRIEF DESIGN EXPECTATIONS SCORE | The queue times its store calls in aggregate — one `store-calls`, one `store-ns` — across **five distinct |
| `the-summary-counts-with-a-set/` | BRIEF DESIGN EXPECTATIONS SCORE | `:fanout::summarize` folds over every outcome — 8000 at n=2000 — building two `HashMap`s with |
| `the-tick-drains-a-batch/` | BRIEF DESIGN EXPECTATIONS SCORE | The topic's `-deliver` handles exactly one message per tick — 2000 ticks for 2000 messages, |
| `the-topic-is-durable/` | BRIEF DESIGN EXPECTATIONS SCORE | `publish` returns `Ok` for a message that exists only in `:ephemeral` state — if the topic dies it |
| `the-topic-names-which-send-failed/` | BRIEF DESIGN EXPECTATIONS SCORE | The topic returns `Accepted 0` from three different failure arms — `Lost`, `Closed`, `TimedOut` on its |
| `the-topic-publishes-a-batch/` | BRIEF DESIGN EXPECTATIONS SCORE | Make `Topic::PublishRequest` carry a vector of messages capped at 10, fan the whole batch into one |
| `the-topic-worker-batches/` | SCORE | One tick of ≤10 inbox rows is grouped by subscriber: one `Queue/send` per subscriber, then ack the batch. |
| `the-trace-separates-durable-work-from-waiting/` | BRIEF DESIGN EXPECTATIONS SCORE | Add `t0b` and `t3b`, delete the phantom `t2`, and report five real stages. |
| `the-trace-stamp-stops-round-tripping/` | SCORE | `Queue/send` stops inspecting the body; the topic-worker stamps t1/t2/t3 itself. |
| `the-unknowable-state/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone 3d — the reply-drop: work done, caller unaware. NOT STRUCK; the fault has no live-service form. |
| `the-vocabulary-stops-mumbling/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone D2 — the residue of the `intueri` cast: names that lie or mumble. |
| `the-wait-names-its-verb/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone B — the queue stops spelling a mode as a magnitude. |
| `the-waiter-folds-carry-a-named-aggregate/` | BRIEF DESIGN EXPECTATIONS SCORE | Replace sqs.wat's three nested-Tuple waiter-fold accumulators with one `defstruct` carrying named |
| `the-wakeup-is-level-triggered/` | BRIEF DESIGN EXPECTATIONS SCORE | Both the queue and the topic arm a self-tick **on the empty→non-empty edge** and re-arm it from |
| `the-window-gets-a-test/` | BRIEF DESIGN EXPECTATIONS SCORE | Add the third redelivery case: a redelivery arriving **mid-processing**, which under record-after |
| `the-wire-carries-a-batch/` | BRIEF DESIGN EXPECTATIONS SCORE | `Sub::DeliverRequest` and `Queue::SendRequest` each carry one message. Make them carry many, so one |
| `the-workers-stop-polling/` | BRIEF DESIGN EXPECTATIONS SCORE | Make the circuit's workers **park** on their queue instead of polling it. Today each worker calls |
| `time-crosses-the-boundary/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone B-pre — prerequisite for Stone B, found by a probe rather than a strike. |
| `transient-means-try-again/` | BRIEF DESIGN EXPECTATIONS SCORE | Make the queue retry a transient store error instead of dying on it, and make the other failures |
| `vis-is-swept-not-chosen/` | BRIEF-v2 BRIEF DESIGN EXPECTATIONS-v2 EXPECTATIONS SCORE-v2 | Make `vis` a `run-with` parameter with today's value as the default, then sweep it to find where |
| `what-does-publish-actually-cost/` | BRIEF DESIGN EXPECTATIONS SCORE | Measure five primitives on one quiet box, then check whether unit × count explains the 37.3 s |
| `zero-is-not-a-wait/` | BRIEF DESIGN EXPECTATIONS SCORE | Stone A of four — zero-as-a-wait has no form, so it is not rejected at the call site but removed. |

## What was left in arc 278 on purpose

Thirty-three files this branch added to 278 stayed there. Each is substrate whose *subject* is the
language or the service machinery rather than the distributed model, and each continues a line that
already existed in 278 before this branch:

| left in 278 | why |
|---|---|
| `select-returns-what-it-sees` (4) | `src/runtime.rs`; fixes a red against `rst_peer_notify_baseline`, and `DESIGN-STONE-rst-peer-notify.md` is already in 278 on `main` |
| `the-death-notice-is-not-a-malformed-frame` (4) | same file, same pre-existing line — `RecvOutcome` / `LociDiedError` |
| `the-reactor-grows-a-seam` + `-v2` + `-v3` (12) | `wat/service.wat` seam extraction; claims no campaign membership, and died twice on `peers_bijection` goldens |
| `impls-completeness-guard`, `DESIGN-STONE-impls-completeness`, `NOTE-impls-completeness-is-unenforced` (6) | the `:impls` / `:satisfies` completeness guard — a `wat/service.wat` language guard, found *during* this campaign but not of it |
| `random-is-threaded` (4) | adds `rand` / `random` / `shuffle`; wat had no randomness interface at all. A language primitive, drawn as chaos's precondition |
| `SKETCH-connection-lifecycle-ops-the-stashed-implementation.diff` (1) | its `BRIEF-connection-lifecycle-ops.md` and `DESIGN-STONE-connection-lifecycle-ops.md` are both in 278 on `main` |
| `probes/internal-arm-replies.wat`, `probes/red-partial-satisfier.wat` (2) | `probes/` is shared 278 infrastructure with four pre-existing members, and `tests/services/probe_impls_completeness.rs:19` hard-codes the second path |

`REALIZATIONS.md` (R1–R70) was not touched. It is one numbered, cross-referenced sequence, and
splitting it is a ruling rather than a chore.

## ⛔ Owed: 24 stale references in non-`.md` files

Every `.md` cross-reference was rewritten with the move (299 occurrences in 144 files). **The
non-`.md` ones were not**, and they are listed here rather than left to be rediscovered. Two reasons,
both deliberate:

- `.wat` files are **not** edited with python or sed here, and the self-hosted codemod (`wat/fix.wat`)
  walks the *form* tree — it does not rewrite comments. Every hit below is inside a comment, so the
  codemod cannot reach it and a manual pass is the only correct route.
- The reorganization itself was scoped to `docs/**/*.md`; touching `src/` and `tests/` would put
  source edits in a filing commit.

| file:line | stale name | now at |
|---|---|---|
| `.config/nextest.toml:102` | `FINDING-unacked-is-receives-minus-acks` | `001-sns-sqs/` root |
| `src/check.rs:14203, :14348, :21062` | `BRIEF-zero-is-not-a-wait` | `001-sns-sqs/zero-is-not-a-wait/BRIEF.md` |
| `src/collection/eval.rs:999`, `src/value/pvec.rs:126` | `DESIGN-STONE-the-indexed-vector-update` | `001-sns-sqs/perf-3-indexed-vector-update/DESIGN.md` |
| `tests/comms/probe_arc278_cancelled_partial_write.rs:2` | `DESIGN-a-cancelled-write-reports-what-it-delivered` | `arc/2026/05/170-program-entry-points/` |
| `tests/comms/probe_arc278_io_uring_write_race.rs:2` | `DESIGN-can-an-io-uring-write-be-raced` | `arc/2026/05/170-program-entry-points/` |
| `tests/kernel/probe_zero_is_not_a_wait.rs:1` | `BRIEF-zero-is-not-a-wait` | `001-sns-sqs/zero-is-not-a-wait/BRIEF.md` |
| `tests/services/probe_arc278_txn_must_close.wat:3` | `DESIGN-a-transaction-that-fails-must-close` | `001-sns-sqs/a-transaction-that-fails-must-close/DESIGN.md` |
| `wat-scripts/fixes/add-timedout-arm.wat:10` | `DESIGN-no-client-call-can-hang` | `001-sns-sqs/no-client-call-can-hang/DESIGN.md` |
| `wat-scripts/queue/sqs.wat:1670`, `wat-scripts/scratch-pad/probe-does-an-unused-arm-cost.wat:3` | `SCORE-find-the-934-milliseconds` | `001-sns-sqs/find-the-934-milliseconds/SCORE.md` |
| `wat-scripts/scratch-pad/probe-a-ledger-is-a-receipt-not-a-lock.wat:14` | `DESIGN-the-unknowable-state` | `001-sns-sqs/the-unknowable-state/DESIGN.md` |
| `wat-scripts/scratch-pad/probe-fanout-is-max.wat:1` | `BRIEF-the-topic-is-durable` | `001-sns-sqs/the-topic-is-durable/BRIEF.md` |
| `wat-scripts/scratch-pad/probe-parked-waiters-stop.wat:3` | `SCORE-the-sane-circuit` | `001-sns-sqs/the-sane-circuit/SCORE.md` |
| `wat-scripts/scratch-pad/probe-time-coerce-negatives.wat:1`, `probe-zero-at-the-boundary.wat:1` | `EXPECTATIONS-time-crosses-the-boundary` | `001-sns-sqs/time-crosses-the-boundary/EXPECTATIONS.md` |
| `wat-scripts/scratch-pad/probe-zero-is-not-a-wait-computed.wat:1`, `probe-zero-is-not-a-wait-rows.wat:1` | `EXPECTATIONS-zero-is-not-a-wait` | `001-sns-sqs/zero-is-not-a-wait/EXPECTATIONS.md` |
| `wat-tests/service-deferred-reply.wat:6` | `EXPECTATIONS-deferred-reply` | `001-sns-sqs/deferred-reply/EXPECTATIONS.md` |
| `wat/query/sqlite-store.wat:222` | `SCORE-the-store-reports-time-per-operation` | `001-sns-sqs/the-store-reports-time-per-operation/SCORE.md` |
| `wat/query/sqlite-store.wat:232` | `SCORE-the-gsi-delete-gets-its-reverse-mapping` | `001-sns-sqs/the-gsi-delete-gets-its-reverse-mapping/SCORE.md` |
| `wat/service.wat:3786` | `DESIGN-a-client-has-a-deadline` | `001-sns-sqs/a-client-has-a-deadline/DESIGN.md` |

★ **The instrument that found these is worth keeping.** `grep -f <(…)` piped through `xargs`
**silently undercounts**: the process-substitution FIFO is drained by the first `grep` invocation, so
every later batch greps an empty pattern file. Three sweeps here reported 9, 20 and 0 hits on the
same tree before a real pattern file — and then a Python pass — reported 24.
