# DESIGN — a batch declares how many

**`:max-entries [field N]`** — a per-field entry cap on a surface feature, enforced client-side
before the send, exactly as `:max-request-bytes` already is. `src/types/surface.rs` +
`wat/service.wat` + `wat-scripts/queue/sqs.wat`.

## WHY — we cap bytes and we do not cap count

Census of the tree, this session:

```
:max-request-bytes    declared on every serviceable feature      wat/query.wat:601 (10 MiB)
                                                                 sqs.wat:101-108   (512 KiB)
:max-entries          ⛔ DOES NOT EXIST ANYWHERE
```

Every batch surface takes a vector — `Store::PutRequest [rows]`, `DeleteRequest [keys]`,
`Queue::SendRequest [bodies]`, `AckRequest [ids]`, `Seen::MarkRequest [seqs]` — and **not one of
them bounds how many.** The workers' `:limit 10` is SQS's number reproduced by habit, an argument
at a call site, not a wall.

★ The builder's rule: *"dense messages between components — maximize data transfer within
constraints. If we only have 3 messages we send an array of three; if 20, then two batches of
ten."* **That rule needs the ten to be a declared constraint**, or every caller re-invents it and
the surface cannot say no.

## THE MECHANISM ALREADY EXISTS — for the other dimension

`wat/service.wat:2293` — the generated client method, gated on `peer-wire?` so the thread tier
pays nothing:

```wat
(:wat::core::if (:wat::kernel::peer-wire? c)
  (:wat::core::let [~n-sym (:wat::string::length (:wat::edn::write req))]
    (:wat::core::if (:wat::i64::> ~n-sym ~cap-const-kw)
      (:wat::kernel::RecvOutcome::Message (~rtl-ctor-kw ~n-sym ~cap-const-kw))
      ~send-recv-form))
  ~send-recv-form)
```

★★ **The request is rejected without ever being sent**, and the caller gets the same variant a
server would have returned. That is the shape to copy, and it is the right one: an over-size batch
should never reach the wire.

## ⛔ THE ONE CONTRACT DECISION — the cap NAMES ITS FIELD

`:max-request-bytes` works on the whole request; **entries are a property of one field**, and the
macro cannot guess which. So the option carries the field:

```wat
(send [self <- :queue::Queue  req <- :queue::Queue::SendRequest]
  -> :queue::Queue::SendResponse
  :max-request-bytes 524288
  :max-entries [bodies 10])
```

⚠ The rejected alternative: `:max-entries 10` plus a rule that the request has exactly one vector
field, discovered by the macro. It is shorter and it fails **Obvious** — a reader of the feature
cannot see what is being counted. *Our verbosity is our shield.* Name the field.

## THE VIOLATION HAS ITS OWN VARIANT — and no form without it

A count violation is not `RequestTooLarge{bytes,cap}`; returning that would misname the cause,
which is the failure class this arc has spent itself closing. The response declares:

```wat
:RequestTooManyEntries [entries <- :wat::core::i64  cap <- :wat::core::i64]
```

★★★ **Declaring `:max-entries` on a feature whose response lacks that variant is a compile
error.** That is the wall: the cap cannot be declared without a way to report it, so the
half-built state has no form. Same derivation as `rtl-ctor-kw` (`wat/service.wat:1733`).

## SCOPE — the mechanism, and ONE adopter

Land `:max-entries` and adopt it on **`Queue::send` only**. `Store::put/delete`, `Queue::ack` and
`Seen::mark` come later as a mechanical sweep, and the topic batch adopts it natively when built.

★ Proactive stepping stone: the topic-batch stone then operates on settled machinery instead of
introducing the cap and using it in one breath.

## OUT OF SCOPE — REJECTED

- **The atomicity contract.** Ruled this session (both stores are all-or-nothing, measured), but
  it is a different invariant with a different mechanism — a floor test over both `Store`
  implementations, not a surface option. **Its own stone.**
- **`:limit` on receive.** A page size, not a request cap, and it already has a home.
- **The queue `cap 64` vs batch size interaction.** Real — 10 msgs × 4 subscribers = 40 bodies
  against a depth cap of 64 — and it belongs to the topic-batch stone, with a measurement, not to
  this one.
