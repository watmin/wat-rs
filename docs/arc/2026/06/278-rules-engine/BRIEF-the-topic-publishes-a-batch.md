# BRIEF — the topic publishes a batch

Make `Topic::PublishRequest` carry a vector of messages capped at 10, fan the whole batch into one
`Queue/send`, and chunk the driver's 2000 publishes into 200 calls. Read
`DESIGN-the-topic-publishes-a-batch.md` first — especially *the cap interaction* and *stamps are
per message*.

## READ IN ORDER

| room | why |
|---|---|
| `wat-scripts/topic/sns-fanout.wat:41` | `PublishRequest [msg <- String]` — the field that becomes a vector |
| `sns-fanout.wat:42-47` | `PublishResponse` — where `RequestTooManyEntries [entries cap]` is added, beside `RequestTooLarge` |
| `sns-fanout.wat:77-95` | the `publish` impl — folds `range 0 nsubs` into `bodies`, then **one** `Queue/send`. The fold becomes msgs × subscribers |
| `wat-scripts/queue/sqs.wat:103` | the exemplar declaration: `:max-request-bytes 524288 :max-entries [bodies 10]` |
| `circuit.wat:1030-1035` | `publish-stamped-until-accepted!` — stamps `"{m}|{t0}"` **per message**; keep that |
| `circuit.wat:1002-1023` | `publish-until-accepted!*` — the `Full` retry loop. A batch that bounces retries as a batch |
| `circuit.wat:1437-1439` | the 2000-message fold — becomes 200 batches of 10 |
| `circuit.wat:1761-1763` | the SECOND publish loop, in another entry point — **both must change** |

★ **There are two publish loops, not one.** `:1439` and `:1763`. Find both before editing either.

## SKETCH

Surface:

```wat
(:wat::core::defrecord :demo::Topic::PublishRequest
  [msgs <- (:wat::core::Vector :- [:wat::core::String])])
```

```wat
(publish [self <- :demo::Topic  req <- :demo::Topic::PublishRequest]
  -> :demo::Topic::PublishResponse
  :max-request-bytes 524288
  :max-entries [msgs 10])
```

`PublishResponse` gains `:RequestTooManyEntries [entries <- i64  cap <- i64]` — the option
without the variant is a compile error, so the two land together.

Impl — the existing single fold over subscribers becomes a fold over messages **inside** it, so
one request yields `count(msgs) × nsubs` bodies and still exactly **one** `Queue/send`:

```wat
bodies (foldl (fn [acc msg] (foldl (fn [acc2 i] (conj acc2 (format "{i}|{m}" :i i :m msg)))
                                   acc (range 0 nsubs)))
              (Vector :- [String])
              (PublishRequest/msgs req))
```

Driver — chunk, and keep the per-message stamp:

```wat
;; 200 batches of 10; each message carries its OWN t0
(foldl (fn [acc b] (:fanout::publish-batch-until-accepted! topic (:fanout::stamped-chunk b 10)))
       nil (range 0 200))
```

## BLAST RADIUS

`wat-scripts/topic/sns-fanout.wat`, `wat-scripts/fanout/circuit.wat`, and scratch probes.
**No `wat/`, no `src/`** — `:max-entries` is generic machinery already; this is a declaration and
an adopter. **Do not raise `cap`.**

## STOP TRIGGERS

- **STOP-1** — `Full` retries dominate: a 40-body batch against `cap 64` bounces so often that
  `publish + drain` does not improve. **That is a finding, not a failure.** Report the retry count
  and the timings; do **not** raise the cap to make the number move.
- **STOP-2** — the e2e histogram changes shape because stamps became per batch. Stop; each message
  keeps its own `t0`.
- **STOP-3** — batching requires a `PublishResponse` variant beyond `RequestTooManyEntries` (e.g.
  a partial). Stop and report — the DESIGN rules one outcome for N, and a need for more is a
  refutation of that ruling, which is the builder's to re-open.
- **STOP-4** — the second publish loop (`circuit.wat:1763`) cannot take the same change. Report
  the asymmetry.
- **STOP-5** — anything outside the blast radius, or any change to `cap`.

## THE PROBE YOU WILL NEED

A scratch probe showing, through the **surface** spelling `:demo::Topic/publish`: 10 messages
accepted (inbox depth +10×nsubs), 11 rejected `RequestTooManyEntries(11,10)` with depth unchanged,
and `:demo::Topic::PUBLISH-MAX-ENTRIES` readable as `10`.

## GRADE AGAINST

`SCORE-a-batch-declares-how-many.md` — same mechanism, one surface earlier.

Write `SCORE-the-topic-publishes-a-batch.md`, then `pulsare_yield kind=scored`.
