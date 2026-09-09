# BRIEF — transient means try again

Make the queue retry a transient store error instead of dying on it, and make the other failures
say which one they were. `wat-scripts/queue/sqs.wat` only.

Read `DESIGN.md` first — especially *what a retry must not do*.

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

## ⛔⛔ AMENDED MID-STRIKE — READ THIS BEFORE THE RETRY

Two findings after the brief was written. **Both change the work.**

### 1. The batch IS atomic — so retrying the whole batch is correct

`sqlite-store.wat:343-355`: `put` is `begin` → `put-rows` → `commit`, and `delete` is the same
shape. **A failed batch commits nothing**, so "retry only the failed entries" and "retry the whole
batch" are the *same set* — all of them failed, none were enqueued.

★ The DESIGN's *"assumes an honest store"* is therefore not an assumption; it is what the code
does. **State it as the contract: a `Store` batch is all-or-nothing.**

### 2. ⛔ BUT THERE IS NO ROLLBACK. ANYWHERE.

```wat
(:wat::core::match (:wat::sqlite::begin conn)
  ((:wat::core::Ok _)
    (:wat::core::match (:wat::query::put-rows conn names new-rows)
      ((:wat::core::Err e) (:wat::core::Err e))     ;; ← transaction left OPEN
      ((:wat::core::Ok _) (:wat::sqlite::commit conn)))))
```

`grep rollback` over `wat/query/sqlite-store.wat`, `wat/sqlite*.wat` and `src/intrinsic/sqlite*`
returns **nothing**. The verb does not exist to call.

★★ **The retry you are about to add walks straight into this**: a second `begin` on a connection
whose transaction was never closed. **Establish what actually happens before building the retry
on top of it.**

- **STOP-6 (new)** — measure it first: after a failed `put`, does a subsequent `put` on the same
  connection succeed? If it does not, **the rollback is a prerequisite and this stone stops** —
  report it and hand it back. Do not add a retry that retries into a wedged connection.
- If a rollback verb must be added to make the retry safe, that is **its own stone**. Report the
  need; do not grow this one into a sqlite-surface change.

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

`docs/excursus/2026/08/001-sns-sqs/entry-three-of-ten/SCORE.md` — the measurement that found this, including the literal responses
and why the queue dies.

---

# ⛔ AMENDED 2026-09-06 — STOP-6 IS DISCHARGED. THIS STONE IS LIVE AGAIN.

Everything above stands **except** the two blockers. Read this section last; it replaces them.

## STOP-6 — discharged, with the measurement

The rollback did become its own stone, exactly as the amendment above required, and then a second
one when grading it found two more doors:

- `105ecf16d` — `ROLLBACK` at all three layers; the statement-failure path closes.
- `d0a160a9f` — **`closed is the postcondition`**: `close-then-err` replaces `rollback-then-err`,
  the **commit**-failure path closes too, and an already-closed transaction is treated as the
  postcondition rather than as a failure to assert on.

`tests/services/probe_arc278_txn_must_close.wat` is now a floor gate. My runs ×2:

```
put1=Fatal:no such table: main;put2=Fatal:no such table: main
```

★ `put2` reports **its own cause**, not `cannot start a transaction within a transaction`. **A
second `Store/put` after a failed one is no longer a second `begin` on a wedged connection.**

## ⛔ AND THE STRONGER FACT THE RETRY ACTUALLY NEEDS

"Reports the same error" is not "can do work." Measured before re-shipping this brief —
`wat-scripts/scratch-pad/probe-a-retry-can-actually-succeed.wat`:

```
attempt1-stmt=Fatal:no such table: nosuch;closed=Fatal:attempt-1;
retry-begin=Ok;retry-write=1;retry-commit=Ok;rows-after=1
```

★★ After a transaction fails and closes, the **retry commits durable work** — the row lands and is
readable. `closed=Fatal:attempt-1` also confirms the original cause survives the close.

**This is the fact the whole stone rests on, and it is now measured rather than assumed.**

## STOP-3 — its premise is now PROVEN, not assumed

STOP-3 said *"this stone assumes an honest store, where `:Transient` means nothing committed."*
That is no longer an assumption:

- `sqlite-store.wat:373-386` (`put`) and `:392-405` (`delete`) — `begin` → rows → `commit`, with
  **every** failure arm routed through `close-then-err`.
- A failed batch therefore commits nothing **and** leaves the connection usable.

★ So *"retry the failed entries"* and *"retry the whole batch"* remain the same set — **all of
them failed, none were enqueued** — and re-sending the request is not a double-enqueue.

⚠ STOP-3 **still stands as written**: if you find a path where a `put` can partially commit, STOP.
The premise is proven for the sqlite store; it is not a licence to skip checking.

## WHAT DID NOT CHANGE

The three `_` arms (`sqs.wat:217`, `:472`, `:728`) are **untouched** — verified this session.
The blast radius is still `wat-scripts/queue/sqs.wat` only; STOPs 1, 2, 4, 5 stand unchanged.

⚠ The store surface still returns **one** outcome for a whole batch. Per-entry outcomes are step
(2) of the tracker's ordered plan and are **not** this stone. Retry the request as sent.
