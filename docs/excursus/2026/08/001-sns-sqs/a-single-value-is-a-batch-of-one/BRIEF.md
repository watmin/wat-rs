# BRIEF — a single value is a batch of one

Make `ack`, `check` and `mark` batch-only, and collapse the per-row folds into one call each.

Read `DESIGN.md` first — especially *per-entry results only
where outcomes differ*, and the widening correctness window.

## ⛔ NO SINGULAR FORM SURVIVES

`:id` and `:seq` are **deleted**, not supplemented. A caller with one item passes a vector of
one. An optional batch API grows a singular caller the first tired afternoon.

## READ IN ORDER

| room | why |
|---|---|
| `sqs.wat:44-47` | `SendRequest [queue bodies <- Vector …]` — **the shape to copy.** It is already right |
| `sqs.wat:92-96` | `AckRequest [queue id]` → `[queue ids <- (Vector :- [String])]`; `AckResponse :Ok []` stays |
| `sqs.wat` — the `ack` impl | one `Store/delete` per id today; the batch deletes each. **Cap 10** |
| `circuit.wat:47-60` | `Seen::CheckRequest` / `MarkRequest` → `seqs <- (Vector :- [String])` |
| `circuit.wat` — `check` impl | returns **a vector of `Recorded`/`Absent` aligned to input order** |
| `circuit.wat` — `mark` impl | writes each; `:Ok []` — it cannot partially fail |
| `circuit.wat:460-530` | **the worker fold.** Today: per envelope check→emit→mark→ack. Becomes: **check all → emit the absent → mark those → ack all** |
| `sns-fanout.wat:453-470` | the topic worker's ack `foldl` → one batched ack per bucket |

## THE NEW SHAPE OF THE WORKER LOOP

```
envs        <- receive (limit 10)
results     <- Seen/check  [seqs of envs]        ;; ONE call
emit             those whose result is Absent
            <- Seen/mark   [seqs of the emitted] ;; ONE call
            <- Queue/ack   [ids of ALL envs]     ;; ONE call
```

⚠ **Order is load-bearing and unchanged**: the receipt is still written *after* the emit. A
`mark` moved before the emit is claim-before under a new name, and the ledger stops being a
receipt.

## BLAST RADIUS

`wat-scripts/` only: `sqs.wat`, `circuit.wat`, `sns-fanout.wat`, and the two probes that
construct `AckRequest`. **No `wat/`, no `src/`, no codemod.**

## STOP TRIGGERS

- **STOP-1** — if `check` cannot return results **aligned to input order**, STOP and report. The
  caller must know which answer belongs to which seq; a set of answers is useless.
- **STOP-2** — if collapsing the fold forces `mark` before `emit`, STOP. That inverts the receipt
  discipline and re-opens the dead-owner loss the arc closed.
- **STOP-3** — if `distinct` moves at all under any chaos cell, STOP and report. `dup` may rise
  (the window widens); **`distinct` may not**.
- **STOP-4** — do not add a `Failed[]` list to `mark` or `ack`. Our impls cannot populate it, and
  a field that is always empty is a lying surface. The DESIGN records the expiry condition.
- **STOP-5** — no `Store`-verb batching, no cap changes, nothing outside `wat-scripts/`.

## PRIOR RESULT TO COPY

`docs/excursus/2026/08/001-sns-sqs/stop-fetching-rows-to-get-a-number/SCORE.md` — the last perf strike, and its discipline of
measuring against a named baseline on a quiet box.
