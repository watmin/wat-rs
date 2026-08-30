# ⛔ NOTE (arc 301) — `journal` SILENTLY LOSES METRICS on `sqlite-store`, because its sort key is time-only

**Found 2026-08-30 by measurement, while grading stone 2c.** **Pre-existing — untouched by
this arc.** `put-one-row` is unchanged across every commit here (stone 2's 54 lines are pure
additions; 2c did not open the file). This is live today.

## The measurement

`tests/services/probe_arc278_span_macros.wat` runs `with-span`, which on close emits **three**
Metrics — an aggregated `:requests` counter, `:fetch/count`, and `:fetch/duration` — through
`journal` into a store, then counts the rows. The fixture uses `mem-store`. Swap **only** the
backend for `sqlite-store(":memory:")` and change nothing else:

```
mem-store    → 3      ← what the test asserts
sqlite-store → 1      ← two metrics gone
```

## The mechanism

`wat/telemetry/journal.wat`, `metric->row`:

```wat
sk (:wat::telemetry::time-sk (:wat::telemetry::Metric/time-ns m))
pk (:wat::edn::write (:wat::telemetry::PartitionKey :namespace … :kind Kind::Metric))
```

**The base key is `(namespace + kind, time-ns)`. Nothing in it distinguishes one metric from
another.** Three metrics emitted by a single `close` share a nanosecond, so they share a
`(pk, sk)` — and `put` is a replace-by-primary-key (DynamoDB `PutItem`; see
`NOTE-mem-store-put-appends-where-sqlite-replaces.md`). Last writer wins. Two rows never exist.

The `uuid` **is** carried, but only as a GSI projection (`uuid-index-keys` → `index-keys`), and
a GSI does not make the base key unique.

## ⚠ This is NOT "a test asserting a state DynamoDB cannot represent"

Stone 2c's STOP-1 was drawn expecting exactly that, and the executor reported the failure in
those terms. **On re-grading, that framing is too generous to the fix, and it matters:**

- The test's assertion — *three metrics survive a close* — is **correct and desirable**.
  Nobody wants a telemetry sink that drops two thirds of a span's metrics.
- What is wrong is **`journal`'s key**, not the test.
- `mem-store`'s append was **masking a production data-loss bug**. The test passed on a fake
  that could represent a state the real backend cannot.

So stone 2c did not *cause* a regression; it **removed the blindfold**. The bug it exposed is
older than the arc and lives in `wat/telemetry/journal.wat`.

★ **This is the value of fixing an oracle, stated concretely.** The oracle was wrong in a
direction that made a real defect invisible — and the defect it hid is data loss in the
telemetry path, on the only backend anyone would ship.

## What is owed

**`journal`'s base sort key must be unique per metric.** The `uuid` already exists on every
`Metric` and `Log` and is already computed into the GSI; folding it (or the metric name) into
`sk` after the timestamp keeps the constant-width prefix that makes `sk` sort chronologically
— which is the property `time-sk` was built for and which a range `scan` depends on — while
making collisions impossible.

⚠ **That is a change to the stored key layout**, so it is not a one-liner: existing rows and
any `scan` range assumptions move with it. It belongs to the telemetry arc, drawn on its own.

Until then, **`journal` on `sqlite-store` loses same-nanosecond metrics.** That should be
stated where a consumer will see it, not only here.

## The wider hole this is one instance of

`tests/services/probe_arc278_span_macros.wat` is one of many fixtures that exercise `journal`
against `mem-store` alone. Every one of them is now suspect in the same way, and the honest
instrument is not to read them — it is to **run them against both backends**, the way
`probe_arc278_journal_backend_differential` already does for one path.

`SCORE-stone-2b`'s lesson, again and one level up: *a differential proves agreement only over
the sequences it drives.* A fixture proves behaviour only on the backend it runs.

## Kin

- `NOTE-mem-store-put-appends-where-sqlite-replaces.md` — why `put` replaces, and the ⛔
  CORRECTED section that establishes DynamoDB as the referent.
- `BRIEF-stone-2c-mem-put-is-a-replace.md` STOP-1 — drawn to catch exactly this, and it did.
- `wat/telemetry/journal.wat` `metric->row` / `log->row` — the site.
