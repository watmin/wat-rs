# SCORE — entry three of ten

**STRUCK. Measurement only; no production change.** Executor: grok, 2026-09-06.
Tree safe, uncommitted. One new file: `wat-scripts/scratch-pad/probe-entry-three-of-ten.wat`.

```
Summary [ 377.220s] 5215 tests run: 5215 passed (5 slow), 22 skipped
```

`.floor/2026-09-06T01-44-06Z/`

The wrapper applies k of n and returns the only variant the surface allows. The
queue does not retry. It dies. The predicted mechanism is wrong, and the numbers
say where.

## THE LITERAL RESPONSES

Witnessed by a direct `Store/put` / `Store/delete` of 10 on `:fs::failing-store`
(applies 9, drops index 2):

| verb | store returns | landed / remaining |
|---|---|---|
| `put` 10 | **`:Transient`** | **9 / 10** applied |
| `delete` 10 | **`:Transient`** | **1 / 10** remaining |

`:Transient` is expressible. It is not truthful. It means *retry, momentarily
unavailable*. Nine of the ten already happened.

The other variants are also lies, for a different reason:

| variant | if returned after k of n | the lie |
|---|---|---|
| `:Success` | all n landed | n−k did not |
| `:Fatal` | do not retry | leaves the partial state as the last word |
| `:Constraint` | schema / uniqueness | this is not that |

★ **STOP-1 fires as a finding, not an abort.** The partial case has no honest
variant. That is the DESIGN's thesis, measured: the wrapper had to pick a lie
in order to return at all. Rows 1–2 are what that lie does to the layer above.

## WHAT THE QUEUE INFERS — nothing; it dies

`sqs.wat` send matches `PutResponse::Success` and `_` is

```
assertion-failed! "queue.send: store put failed"
```

Ack matches `DeleteResponse::Success` and `_` is

```
assertion-failed! "queue.ack: store delete failed"
```

`SendResponse` / `AckResponse` have no `:Transient`. There is no arm that retries
the batch. The DESIGN's mechanism — "the queue cannot know which landed, so it
retries the whole batch" — is not in the code. The queue **cannot name the fault
to its caller** and **does not retry**.

The client of `Queue/send` / `Queue/ack` sees **`RecvOutcome::Lost`**. Quoted from
the probe's stderr, both cells, every run:

```
queue.send: store put failed     (queue thread dies)
queue.ack: store delete failed   (queue thread dies)
```

## ★★ ROW 1 — partial put. Predicted: duplicate. Measured: LOSS.

```
PUT=store=Transient;landed=9/10;send=Lost;drain-n=9;total=9;distinct=9
```

×3 identical.

Nine bodies are in the store and drain. The dropped body (`m2`) was never written.
`total=9; distinct=9` — **one message is lost.** The caller sees `Lost`, so it does
not even know the nine landed.

The prediction required a retry of the whole batch. The queue does not retry. Without
that retry there is no duplicate path; there is a hole.

A caller that *did* retry `Queue/send` on a fresh queue would then duplicate the
nine and (if the injector still fires) still lose entry 3. That path was not the
cell — the cell is one send, then drain, as briefed.

## ★★ ROW 2 — partial delete. Predicted: safe. Measured: no loss, one redelivery.

```
DEL=store=Transient;remaining=1/10;got=10;ack=Lost;redelivered=1;total=1;distinct=1
```

×3 identical.

Ten received, then one `AckRequest` of ten ids. Nine keys deleted, one left. Queue
dies. Past visibility, **the undeleted row redelivers** (`redelivered=1`). Nothing
of the ten is gone from the world: nine are consumed, one comes back.

DESIGN said *nothing lost and nothing duplicated*. The first half holds. The second
does not: the undeleted entry is a redelivery, which under record-after is a
duplicate, not a loss.

The "retry re-deletes missing rows as no-ops" path never runs. Same reason as put:
the queue assertion-fails instead of retrying.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ partial put duplicates | **mechanism wrong.** `total=9; distinct=9`. **LOSS of 1**, not a duplicate. Store `:Transient`, send `Lost` |
| 2 | ★★ partial delete is safe | **no loss.** `redelivered=1; total=1; distinct=1`. Nine consumed, one returns. Store `:Transient`, ack `Lost` |
| 3 | ★★ what the surface can SAY | **`:Transient`**, and it cannot name which. Queue infers nothing; `_` is assertion-fail; `Lost` to the caller |
| 4 | ⛔ blast radius | ✅ one file, `wat-scripts/scratch-pad/` only |
| 5 | floor untouched | ✅ `Summary [ 377.220s] 5215 tests run: 5215 passed (5 slow), 22 skipped` |

Rows 1 and 2 were a prediction, not a requirement. Either outcome is a pass. The
failed strike would have been not measuring `total`/`distinct`.

## STOP-1 / STOP-2 / STOP-3 / STOP-4

- **STOP-1 fired as the type finding.** No honest variant. Probe still ran, returning
  `:Transient` (the only "something went wrong" the surface has), because rows 1–2
  are the behaviour the types cannot answer.
- **STOP-2** did not fire. Queue takes `store-addr`; the wrapper drops in. Both cells
  dialled it.
- **STOP-3** did not fire. No per-entry outcomes added. Real stores untouched.
- **STOP-4** did not fire. `git status` is the one untracked probe.

## WHAT THIS EARNS FOR STEP 2

The collapsed surfaces are **not** honest. Step 2 does not shrink to nothing.

**`put` needs per-entry outcomes.** A partial put without them is a lost message,
and the caller is told only that the queue died.

**`delete` did not lose**, but the queue still dies, and the undeleted entry
redelivers. Per-entry outcomes on delete are how the caller would ack the one that
remains without guessing. The DESIGN's "delete may not" is the weaker half and is
not free: the assertion-fail is itself a collapsed surface.

The queue's `_` arms are the layer that would have to *speak* those per-entry
results. Adding them only on Store leaves the queue still dying.

## NOT TOUCHED

`src/`. `wat/`. `sqs.wat`. `circuit.wat`. The real stores. No knobs, no `Failed[]`.

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-06

**STRUCK.** Re-run by me, ×3 byte-identical, and the code read confirms the mechanism.

```
PUT=store=Transient;landed=9/10;send=Lost;drain-n=9;total=9;distinct=9
DEL=store=Transient;remaining=1/10;got=10;ack=Lost;redelivered=1;total=1;distinct=1
floor  Summary [ 371.648s] 5215 passed, 22 skipped, 0 FAIL — .floor/2026-09-06T01-52-00Z/
blast  2 untracked files, scratch-pad + the SCORE. Nothing else.
```

## ⛔ BOTH MY PREDICTIONS WERE WRONG, FROM ONE CAUSE

I predicted partial `put` would **duplicate** and partial `delete` would be **safe with nothing
duplicated**. Measured: partial `put` **LOSES a message**, and partial `delete` **redelivers one**.

★ **Both predictions assumed a retry loop that does not exist.** `sqs.wat:472` and `:728`:

```wat
(_ (:wat::kernel::assertion-failed! "queue.send: store put failed" …))
(_ (:wat::kernel::assertion-failed! "queue.ack: store delete failed" …))
```

**The queue does not retry. It dies.** I reasoned about what it *would* do without reading
whether it does anything at all — and my DESIGN even claimed the mechanism as the reason to trust
the prediction.

★★ **The rule this earns, beside the one already recorded:** *gate what the stone controls* has a
sibling — **predict only through code you have read.** A mechanism named from imagination is
worth no more than a number, and this arc has now paid for that four times.

## ★★ THE FINDING IS WORSE THAN THE PREDICTION

**A partial put is a silent lost message.** Nine rows land, the store says `:Transient`, the queue
asserts and dies, and the caller sees only `Lost` — it is not told that nine landed. The message
that did not land is gone, and `total=9; distinct=9` is the whole record of it.

★ **The type finding, measured rather than read:** the wrapper had to **pick a lie in order to
return at all.** `:Success` lies (n−k did not land), `:Transient` lies (nine already happened),
`:Fatal` lies (leaves partial state as the last word), `:Constraint` is the wrong category.
**The partial case has no honest variant** — STOP-1 fired as the finding it was written to be.

## WHAT STEP 2 IS NOW, AND IT IS BIGGER THAN I SCOPED

My DESIGN guessed *"`put` needs per-entry outcomes; `delete` may not."* The measurement says:

- **`put` needs them** — without them a partial put is a lost message.
- **`delete` needs them too, for a different reason.** It did not lose, but the queue still dies
  and the undeleted entry redelivers. Per-entry results are how a caller acks the one that
  remains instead of guessing.
- ★★ **And the Store surface alone is not enough.** The queue's `_` arms are where those results
  must be *spoken*. Adding per-entry outcomes to `Store` and stopping there leaves the queue
  asserting on a fault it now has the words for.

**Step 2 spans `Store` and `Queue` together.** That is earned by measurement, not scoped by
guess — which is exactly what this stone was for.
