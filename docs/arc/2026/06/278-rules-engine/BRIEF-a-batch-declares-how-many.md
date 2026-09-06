# BRIEF — a batch declares how many

Add `:max-entries [field N]` as a surface-feature option, enforce it client-side before the send
exactly as `:max-request-bytes` is enforced, and adopt it on `Queue::send` only. Read
`DESIGN-a-batch-declares-how-many.md` first — it carries the contract decision and the compile-error
wall.

## READ IN ORDER

| room | why |
|---|---|
| `src/types/surface.rs:441` | the `":max-request-bytes" =>` arm in the option `match` — your exemplar for parsing, duplicate-detection and the positive-literal check |
| `src/types/surface.rs:492` | the unrecognized-option error, which **lists the recognized options by name** — it must learn the new one, or the diagnostic lies |
| `src/types/surface.rs:504` | `max_request_bytes_explicit` — how explicitness is captured *before* defaulting. `:max-entries` has **no default**: absent means uncapped |
| `wat/service.wat:1720`, `:1733` | `cap-const-kw` and `rtl-ctor-kw` — how a cap constant and a response constructor are derived at macro-expand time. Yours are the same shape |
| `wat/service.wat:1927-1931` | emission site **one** |
| `wat/service.wat:2293-2297` | emission site **two** — the `peer-wire?`-gated guard quoted in the DESIGN |
| `wat-scripts/queue/sqs.wat:44` | `Queue::SendRequest [queue bodies now-ns]` — `bodies` is the field you cap |
| `wat-scripts/queue/sqs.wat:48-53` | `Queue::SendResponse` — where `RequestTooManyEntries` is added, beside `RequestTooLarge` |
| `wat-scripts/queue/sqs.wat:101` | the `send` feature — where the option is declared |

★ **There are TWO emission sites, not one.** Find both before writing either; a guard added to
one and not the other is a cap that fires depending on how you dialled.

## SKETCH

Surface declaration:

```wat
(send [self <- :queue::Queue  req <- :queue::Queue::SendRequest]
  -> :queue::Queue::SendResponse
  :max-request-bytes 524288
  :max-entries [bodies 10])
```

Response gains, beside `RequestTooLarge`:

```wat
:RequestTooManyEntries [entries <- :wat::core::i64  cap <- :wat::core::i64]
```

Generated guard — **before** the byte guard, and NOT behind `peer-wire?`. A count violation is a
contract violation at every tier; only the byte measure needs a wire because only it costs an
encode:

```wat
(:wat::core::let [~k-sym (:wat::core::count (~field-accessor-kw req))]
  (:wat::core::if (:wat::i64::> ~k-sym ~entries-cap-kw)
    (:wat::kernel::RecvOutcome::Message (~rte-ctor-kw ~k-sym ~entries-cap-kw))
    <the existing peer-wire? byte guard, unchanged>))
```

`~field-accessor-kw` is derived from the request type and the declared field name the same way
`rtl-ctor-kw` is derived from the response type at `wat/service.wat:1733`.

## THE WALL

Declaring `:max-entries` on a feature whose response enum has no `RequestTooManyEntries` variant
must be a **compile error naming the missing variant** — not a silent skip, not a fallback to
`RequestTooLarge`. Write the failing case as a test.

## BLAST RADIUS

`src/types/surface.rs`, `wat/service.wat`, `wat-scripts/queue/sqs.wat`, and probes/tests.
**`Store::put/delete`, `Queue::ack` and `Seen::mark` are NOT adopted here** — one adopter, then a
sweep in its own stone.

## STOP TRIGGERS

- **STOP-1** — the option value cannot be a vector (`[bodies 10]`) because the parser takes only
  scalar literals. Report the exact parser constraint; do **not** fall back to a bare
  `:max-entries 10` with field discovery — the DESIGN rejects that on Obvious.
- **STOP-2** — the field accessor cannot be derived at macro-expand time from the request type and
  a field name. Report what `rtl-ctor-kw`'s derivation has that this lacks.
- **STOP-3** — the two emission sites cannot take the same guard. That asymmetry is the finding;
  surface it rather than guarding one.
- **STOP-4** — enforcing the cap requires the count at a tier that cannot see the field (e.g. a
  decoded-but-untyped request). Report where.
- **STOP-5** — anything outside the blast radius. In particular: **do not adopt the option on a
  second surface.**

## THE PROBE YOU WILL NEED

A scratch probe that sends 11 bodies to a `Queue::send` capped at 10 and shows
`RequestTooManyEntries{11,10}` **with no message enqueued** — depth unchanged — at **both**
`:locus (:wat::spawn::thread)` and `:locus (:wat::spawn::process)`, because the byte guard is
wire-gated and this one must not be.

## GRADE AGAINST

`SCORE-a-single-value-is-a-batch-of-one.md` — the arc's prior surface-shape stone.

Write `SCORE-a-batch-declares-how-many.md`, then `pulsare_yield kind=scored`.
