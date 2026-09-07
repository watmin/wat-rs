# BRIEF — a caller chunks to the declared limit

Generate `<op>-all` for any op declaring both `:max-entries` and an `Accepted [count]` response,
restore `Queue::send`'s limit, and move the topic onto it. Read
`DESIGN-a-caller-chunks-to-the-declared-limit.md` first — it carries the 3.3× measurement and the
rule that limits are never raised to fit a caller.

## READ IN ORDER

| room | why |
|---|---|
| `wat/service.wat:2267` | `method-name` — how the op-method's name is built. `<op>-all` is the sibling |
| `wat/service.wat:2434` | where the op-method `defn` is conj'd into the emitted forms. Your helper is emitted beside it |
| `wat/service.wat:1810` | `cap-const-kw` / `rtl-ctor-kw` — deriving a constant name and a ctor keyword at expand time. Same shapes you need |
| `src/types.rs:3799` | where `<S>::<OP>-MAX-ENTRIES` is emitted as a def — the limit you chunk to |
| `wat-scripts/queue/sqs.wat:101` | the `send` feature — where `:max-entries [bodies 64]` is restored |
| `wat-scripts/queue/sqs.wat:49` | `SendResponse::Accepted [count]` — the arm that makes chunking sound |
| `wat/query.wat:522` | `Store::PutResponse::Success []` — **no count**, so `put-all` must NOT be emitted |
| `wat-scripts/topic/sns-fanout.wat` `publish` | the single `Queue/send` that becomes `Queue/send-all` |

## SKETCH

Emission condition — both must hold, else emit nothing (absence, not error):

```
op declares :max-entries [field N]        AND     <Op>Response declares :Accepted [count <- i64]
```

The helper, in the shape of the existing generated methods:

```wat
;; send chunks of at most N, IN ORDER, summing the prefix; stop at the first short chunk.
(:wat::core::defn :<Proto>/<op>-all [c req] -> <RecvOutcome of Response>
  ;; loop over (partition field N):
  ;;   Accepted k, k == len(chunk)  -> total += k, continue
  ;;   Accepted k, k <  len(chunk)  -> return Accepted (total + k)      ;; prefix ends here
  ;;   anything else                -> return it unchanged             ;; TooLarge / Malformed
  ;;   recv failure                 -> return it unchanged
  )
```

★ **Stop at the first short chunk.** Sending chunk 3 after chunk 2 came back short would break
contiguity, and `Accepted n` would no longer be a prefix.

`Queue::send` regains `:max-entries [bodies 64]` — the derivation is `N <= cap`, and this queue's
`cap` is 64. Do **not** change any `:cap`.

`sns-fanout.wat`'s `publish` swaps its one `Queue/send` for `Queue/send-all`. **Nothing else in
the topic changes** — in particular it must not read `SEND-MAX-ENTRIES`.

## BLAST RADIUS

`wat/service.wat`, `wat-scripts/queue/sqs.wat`, `wat-scripts/topic/sns-fanout.wat`, scratch probes.
**No `src/`** — `<OP>-MAX-ENTRIES` is already emitted. **No `circuit.wat`.** No `:cap` change.

## STOP TRIGGERS

- **STOP-1** — the response's `Accepted [count]` arm cannot be detected at expand time. Report it;
  the emission condition is the whole design and a helper emitted without it is unsound.
- **STOP-2** — a chunked call cannot return the same type as the unchunked one. Report the shape;
  they must be interchangeable at a call site.
- **STOP-3** — `publish-all`/`send-all` appears on an op with no `Accepted` arm (e.g. any `Store`
  op). That is the emission condition leaking; stop.
- **STOP-4** — at m=4 the topic **regresses**. 40 bodies against a 64 limit is one chunk, so it
  should be byte-identical work to today. A regression means the chunker fires when it should not.
- **STOP-5** — anything outside the blast radius, or any `:cap` change.

## THE MEASUREMENT

`circuit.wat` at two configurations, ×3 each. The m=8 cell is the point of the stone; the m=4 cell
is the guard that it costs nothing when it does not apply.

```
                      publish   retries  receives   (measured today, @25ms fixed)
n=2000 m=4 j=3          18472      540      4687     ← must not regress
n=1000 m=8 j=3          61431     1652     10387     ← the 3.3x this stone targets
```

⚠ Both cells use the shipped adaptive backoff unless you pin the delay; if you pin it, pin **both**
sites (`circuit.wat` parent *and* the Publisher child inline) — patching only the parent silently
measures adaptive, which cost me a false result this session.

## GRADE AGAINST

`SCORE-a-batch-declares-how-many.md` — the stone that built `:max-entries` and its defs.

Write `SCORE-a-caller-chunks-to-the-declared-limit.md`, then `pulsare_yield kind=scored`.
