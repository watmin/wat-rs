# DESIGN — the topic publishes a batch

**`:demo::Topic::PublishRequest [msgs <- Vector[String]]`**, capped at 10, with the driver
chunking. `wat-scripts/topic/sns-fanout.wat` + `wat-scripts/fanout/circuit.wat`. This is the
arc's headline number.

## WHY — the entry point is the only surface that never got batched

Census, this session:

```
Store::PutRequest [rows]      batch, called with real batches   ✅
Store::DeleteRequest [keys]   batch, called with real batches   ✅
Queue::SendRequest [bodies]   batch — topic sends 4, worker sends a whole bucket  ✅
Queue::AckRequest [ids]       batch — whole bucket's ids        ✅
Seen::MarkRequest [seqs]      batch — the whole checked set     ✅
Topic::PublishRequest [msg]   ⛔ SINGULAR
```

`circuit.wat:1437-1439` folds over `range 0 2000` calling `publish-stamped-until-accepted!`
**once per message**. Each call is a topic round trip → one `Queue/send` → one `Store/put`.

```
publish  23.6 s of a 23.9 s publish+drain          2000 calls
```

★ At 10 per request that is **200 calls**. The builder's rule: *dense messages between
components — 3 messages is an array of three, 20 is two batches of ten.*

## ⛔ THE ONE CONTRACT DECISION — one outcome for N, and it is HONEST

`PublishResponse` stays a single outcome for the whole batch. That is not a collapsed surface, and
this session established why by reading rather than assuming:

- a batched publish bottoms out in **one** `Queue/send` → **one** `Store/put`
- `sqlite-store.wat:373-405` is `begin` → rows → `commit` with every failure arm through
  `close-then-err`; `mem.wat:633` is a pure fold into a new state installed by the actor
- and **a partial failure is not reachable through the honest surface at all**: `put-one-row` is
  DELETE → clear-index → INSERT → insert-projections, every step on a table `:init` created, all
  `TEXT` params, and a **missing projection is skipped** (`None` → recurse), not an error

★★ So per-entry outcomes would report a state nothing can produce. The chaos stone's k-of-n
wrapper **had to lie to return at all** — it was an invalid Store, not a fault injection.

⚠ Expiry, as ruled: the day a Store cannot hold atomicity, per-entry outcomes is correct and this
re-opens.

## THE CAP INTERACTION — a measurement, deliberately NOT a tuning knob

10 messages × 4 subscribers = **40 bodies** in one `Queue/send`, against the inbox's `cap 64`
(`sns-fanout.wat:807`). A send is all-or-nothing, so a 40-body batch needs 40 free slots or the
whole thing bounces `Full` and the driver retries **all ten**.

⛔ **This stone does NOT raise the cap.** Raising it to make the number look better is precisely
the Goodhart move rule 1 was written for — the perf phase already watched 15 s move from
`publish` into `drain` with throughput unchanged. Batch at 10, keep `cap 64`, **measure the `Full`
retries**, and bring the trade to the builder with a number.

## STAMPS ARE PER MESSAGE, NOT PER BATCH

`publish-stamped-until-accepted!` (`circuit.wat:1030`) stamps `"{msg}|{t0}"`, and the e2e
histogram is built from those stamps. **Each message keeps its own `t0`.** A per-batch stamp would
make all ten share an origin and silently change what `e2e` means — a measurement corrupted to
serve the thing being measured.

## OUT OF SCOPE — REJECTED, not deferred

- **Raising `cap`.** A ruling with a measurement behind it, not a knob turned inside this stone.
- **Per-entry outcomes.** Ruled out (above), with its expiry recorded.
- **The `no_unpaired_begin` lint.** The real wall for the transaction class, named in
  `docs/excursus/2026/08/001-sns-sqs/closed-is-the-postcondition/DESIGN.md` and still unbuilt. It is a source-property gate and does
  **not** block this stone — atomicity already holds.
- **`setup` / `stop`.** 16 s and 40 % of the run; next after this.
