# BRIEF — item (b): the batched writer

Fragment an oversized batch into submissions that fit the op's declared cap, write them in order, and
report **exactly how many items landed**. Then make the span's flush use it, so an over-cap buffer
can drain instead of sticking forever.

Read `DESIGN-STONE-the-batched-writer.md` beside this first. It carries the scope ruling (do **not**
build `Stream`), the cut-at-`>` rule, and the one decision that is a data bug in both directions if
missed.

## Read in order, and why you are being sent there

1. **`wat/telemetry/span.wat:361-384`** — `flush-logs`. Today it calls `Journal/write-logs` with the
   WHOLE buffer and resets to empty only on `Done`. **This is the caller you are changing**, and its
   reset-only-on-success discipline is the thing you are making finer-grained, not replacing.
2. **`wat/telemetry/span.wat:75`** — the span's `>=` size trigger, and **`wat/service.wat:1779`** —
   the server's `>` rejection. That asymmetry is what makes an over-cap buffer unflushable. Your
   chunker cuts at `>`: a chunk sized exactly to the cap is legal.
3. **`wat/telemetry.wat`** — `Journal::WRITE-{LOGS,METRICS}-MAX-REQUEST-BYTES`, and the
   `write-logs`/`write-metrics` ops. The cap comes from these constants, never a literal.
4. **`wat/telemetry/span.wat`, the `log` arm** — how the span measures:
   `(:wat::string::length (:wat::edn::write (…WriteLogsRequest would)))`. Use the same measure; the
   design's CRUX-2 settled that exact beats estimated because the encode is needed anyway.

## The work

**1. Two batched writers** in `wat/telemetry.wat`: `write-logs-batched` and
`write-metrics-batched`. Each takes the sink peer and a Vector of items, folds accumulating encoded
byte-length, cuts a chunk when the next item **would cross** the cap, writes each chunk in order, and
stops at the first failure.

**2. Return the written count with the outcome.** The caller must be able to compute the un-written
suffix exactly.

**3. One item over the cap** → `RequestTooLarge{bytes, cap}`, reported for that item. Never skipped,
never retried forever.

**4. Rewire the span.** `flush-logs`/`flush-metrics` call the batched writer and reset to the
**un-written suffix** — `drop(items, written)` — rather than to empty. On full success that suffix is
empty, so today's behaviour is preserved exactly.

## Blast radius

`wat/telemetry.wat` (two new fns), `wat/telemetry/span.wat` (two flush fns). **No new type. No new
surface op. No `Journal` change. No runtime change.**

## STOP triggers

**STOP-1 — the count must be exact.** Report fewer than landed and those items are re-sent
(duplicate logs); report more and they are dropped (lost logs). If you cannot make the count exact,
STOP — an approximate count is a data bug wearing a success.

**STOP-2 — do not build `Stream`.** It does not exist, `WriteResult` does not exist, and nothing in
the tree streams. If the work seems to want them, STOP and report why rather than building an
abstraction with one user. See the DESIGN's scope ruling.

**STOP-3 — cut at `>`, not `>=`.** Copying the span's `>=` into the chunker re-creates the
unflushable buffer at a smaller scale: a chunk sized exactly to the cap is legal and must be sent.

**STOP-4 — a single over-cap item must not loop.** If your fold can produce an empty chunk and go
round again, STOP: that hangs the flush, which is worse than the failure it is trying to report.

## The gates to write

- **an over-cap buffer drains:** a buffer larger than the cap, against a working sink, is fully
  written across multiple submissions. **This is RED today** — it is the finding this stone exists
  for.
- **★ partial progress is exact:** a sink that accepts the first chunk and refuses the second — the
  span's buffer afterwards holds exactly the un-written suffix. Not one item more (duplicate), not
  one fewer (loss). Prove it by draining against a working sink and counting.
- **one item over the cap:** reported as `RequestTooLarge`, and the flush returns rather than hangs.
- **the un-chunked path is unchanged:** a buffer under the cap is one write, and every stone A/B/C
  gate still passes.

## Prior comparable result

`SCORE-item-c-stone-c-flush-must-speak.md` beside this — and read its Row 2 section, which is the
finding this stone closes and an example of a row of mine that was impossible as written.
