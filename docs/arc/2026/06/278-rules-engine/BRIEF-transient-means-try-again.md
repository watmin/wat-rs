# BRIEF — transient means try again

Make the queue retry a transient store error instead of dying on it, and make the other failures
say which one they were. `wat-scripts/queue/sqs.wat` only.

Read `DESIGN-transient-means-try-again.md` first — especially *what a retry must not do*.

## READ IN ORDER

| room | why |
|---|---|
| `sqs.wat:217` | `queue.take: scan-index failed` — the `_` arm on `ScanIndexResponse` |
| `sqs.wat:472` | `queue.send: store put failed` — the `_` arm on `PutResponse` |
| `sqs.wat:728` | `queue.ack: store delete failed` — the `_` arm on `DeleteResponse` |
| `sqs.wat:493`, `:523` | *"Do not claim Ok — the put is unknowable. Full is the caller's retry."* The file's existing idiom for an unknowable store outcome |
| `wat/query.wat:519-539` | the four non-`Success` variants each response actually has |
| `circuit.wat:440-470` | the worker's **bounded** retry (`a1`/`a2`/`a3`, then exhaustion) — the shape to copy |
| `wat/query/sqlite-store.wat:57`, `:70` | proof the real store emits `:Transient` |

## THE CHANGE, AT EACH OF THE THREE SITES

```
:Success       → proceed
:Transient     → wait briefly, retry the SAME request, bounded budget
                 exhausted → assert, naming "transient, exhausted after N"
:Constraint    → assert, naming Constraint
:Fatal         → assert, naming Fatal
:RequestTooLarge / :RequestMalformed → assert, naming which
```

⚠ **No `_` arm may remain on a store response.** The whole defect is that one catch-all spoke for
four outcomes; replacing it with a narrower catch-all repeats it.

## BLAST RADIUS

`wat-scripts/queue/sqs.wat` only. **No `wat/`, no `src/`, no surface change, no caller change.**

## STOP TRIGGERS

- **STOP-1** — if a bounded retry cannot be expressed inside a service arm without restructuring
  the impl, STOP and report the shape. `circuit.wat:440-470` does exactly this in a worker; if a
  *service* arm cannot, that asymmetry is the finding.
- **STOP-2** — **do not retry anything but `:Transient`.** `:Constraint` and `:Fatal` are not
  retryable, and retrying them would turn a clear failure into a slow one.
- **STOP-3** — if retrying `put` requires knowing whether the first attempt partially committed,
  STOP and report. **That is step 2's question**; this stone assumes an honest store, where
  `:Transient` means nothing committed.
- **STOP-4** — no surface change. `SendResponse`/`AckResponse` gain no variant here.
- **STOP-5** — nothing outside `sqs.wat`.

## THE PROBE YOU WILL NEED

`probe-entry-three-of-ten.wat`'s wrapper reports `:Transient` **after applying k of n** — a lying
store, deliberately, for step 1. **This stone needs the honest case:** a wrapper mode that fails
**without applying anything**, N times, then succeeds. Extend that probe or add a sibling; either
is fine, both live in `scratch-pad/`.

## PRIOR RESULT TO COPY

`SCORE-entry-three-of-ten.md` — the measurement that found this, including the literal responses
and why the queue dies.
