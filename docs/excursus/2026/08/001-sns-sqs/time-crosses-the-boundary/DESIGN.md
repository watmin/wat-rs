# DESIGN — time crosses the boundary

**Stone B-pre.** Prerequisite for Stone B. Small, surgical, and found by a probe rather than a strike.

## WHY

Stone B replaces the queue's `wait-ns <- :wat::core::i64` with an enum inside
`(defsurface :queue::Queue :nature :wat::kernel::Peer)`:

```
:Immediate []
:UpTo [d <- :wat::time::NonZeroDuration]
```

`ReceiveRequest` is a **wire protocol** record, so that arm requires the type to survive a service
boundary. **It does not — and neither does any other time type.**

`wat-scripts/scratch-pad/probe-nonzeroduration-crosses-the-wire.wat`, 3/3, identical at **both loci**:

```
immediate=[ok:0]
upto=[MALFORMED:expected=:wat::time::NonZeroDuration;got=Integer]
duration-CONTROL=[MALFORMED:expected=:wat::time::Duration;got=Integer]
instant-EXEMPLAR=[MALFORMED:expected=:wat::time::Instant;got=Inst]
```

★ **The three cells say three different things, and the third is the one that names the fix.**

- The **nullary** arm round-trips. Enums cross fine; it is the *payload* that fails.
- `Duration` and `NonZeroDuration` arrive as `Integer` — **the type is erased on encode**
  (`render.rs:4159-4160`: `Value::Duration(ns) => OwnedValue::Integer(*ns)`).
- `Instant` arrives as `Inst` — **the type SURVIVES encode** (`render.rs:4158`,
  `Value::Instant(t) => OwnedValue::Inst(*t)`) **and is still rejected.**

So this is not one defect but the shape of a missing table: **`edn_to_typed_value` has no time-type
target arms at all.** Encoding is a side issue; the coercion is the hole. `Instant` proves it — its
bytes are perfect and it still cannot land.

**This is pre-existing and has nothing to do with Stone A.** Stone A's
`Value::NonZeroDuration(d) => OwnedValue::Integer(d.get() as i64)` faithfully mirrored the `Duration`
line beside it, defect included.

★ And `sqs.wat:11-12` has stated the consequence all along, on the wrong axis:

> *"Instant/Duration on the request record is avoided — journal's wire-proven i64 time-ns is the
> precedent."*

The header argues **testability** — a fixture can drive the visibility window as a value. True, and
it concealed that there was never a choice to make. A stated reason standing in front of an unstated
impossibility is why nobody found this until a probe went looking.

## WHAT IT DELIVERS

Three target arms in **`edn_to_typed_value_inner`** (`src/edn/render.rs:2266`), beside the exemplar
already there at `:2297`:

```rust
":wat::core::i64" => match edn {
    Edn::Integer(n) => Ok(Value::i64(*n)),
    ...
}
```

The algorithm in a sentence: **a typed coercion already knows the target type, so the wire only has
to carry the payload — an `Integer` landing where a `Duration` is declared IS a `Duration`.**

| target | accepts | produces |
|---|---|---|
| `:wat::time::Instant` | `Edn::Inst(t)` | `Value::Instant(t)` |
| `:wat::time::Duration` | `Edn::Integer(n)`, `n >= 0` | `Value::Duration(n)` |
| `:wat::time::NonZeroDuration` | `Edn::Integer(n)`, **`n > 0`** | `Value::NonZeroDuration(…)` |

## ⛔ THE ONE CONTRACT DECISION

**The `NonZeroDuration` arm rejects zero as `EdnCoerceError`, not as a panic.**

This is the design's real prize and it is not merely plumbing. Stone A's wall is **rung 3 for a
literal** and **rung 2 for a computed value** — a runtime panic, surfacing as
`LociDiedError/Panic`, which at process locus **kills the child**. A peer that sends `UpTo 0` over
the wire is the computed case by definition.

With this arm, that peer gets **`RequestMalformed[expected, got]`** — a typed, catchable, named
refusal at the boundary, and the receiving service stays alive. **The zero wall gets a rung-3 home
on the wire that Stone A could not give it in the language.**

Chosen over a tagged EDN form (`#wat.time/Duration 5000000`) because the coercion is *typed*: the
declared type is already in hand, so a tag would carry information the target already provides, and
every existing i64-on-the-wire caller would break. `Instant` keeps `Edn::Inst` because it already
has it and EDN has a native form for instants; durations have none.

## FILES

| file | change |
|---|---|
| `src/edn/render.rs:2266` | three target arms in `edn_to_typed_value_inner` |
| `src/edn/render.rs:~2238` | the doc table (`\| :wat::core::i64 \| Integer \| Value::i64(n) \|`) gains three rows |

**No `.wat` changes. No codemod. No new Value variants.** `render.rs:4158-4160`'s encode is left
exactly as it is — `Integer` is a correct encoding once the coercion can read it.

## OUT OF SCOPE = REJECTED

- **A tagged EDN form for durations.** Rejected above; it breaks every i64-on-the-wire caller.
- **Changing the encode.** Rejected: `Instant` proves encoding was never the blocker.
- **Stone B itself.** This unblocks it; it does not do it.
- **`sqs.wat:11-12`'s comment.** It is now *half* true and should say so — but that is Stone B's
  edit, in the file Stone B owns. **S21.**

## THE PROOF

1. **The probe flips.** All four cells: `immediate=ok:0`, `upto=ok:250000000`,
   `duration-CONTROL=ok:1000000`, `instant-EXEMPLAR=ok:1000000`. It is committed and currently
   fails on three of four — that is the acceptance criterion, already on disk.
2. **★ The zero refusal, at the boundary.** A peer sending `UpTo` with a zero payload must receive
   `RequestMalformed` and the service must **still be alive afterwards**. Show both: the response
   variant, and a subsequent successful call on the same connection. A refusal that kills the peer
   is the panic wearing a different hat.
3. **Negative control:** a `Duration` target handed a `String` still errors, with `expected`/`got`
   naming both. The arms must discriminate, not blanket-accept.
4. **The floor**, Summary line. Expect `5207/5207`.
