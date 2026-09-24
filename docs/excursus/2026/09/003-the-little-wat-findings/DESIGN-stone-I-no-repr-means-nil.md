# DESIGN — STONE I: a value with no EDN representation renders as tagged nil — no exceptions

> ⛔ **AMENDED 2026-09-24 — `Value::Vector` is OUT of this stone.** Builder: *"why is Value::Vector
> not just [...] — [1 2 3] is a vec of i64 ?"* Measured: `Value::Vector` is `Arc<holon::Vector>`, the
> VSA hypervector, backed by `&[i8]` — pure numeric DATA, not a resource. It HAS an honest EDN repr
> (its components), so rendering it `nil` would lie in the other direction. It goes to its own stone:
> `#wat.holon/Vector [i8 …]` plus the reader arm it has never had. (wat's `[1 2 3]` is `Value::Vec` and
> already renders as `[1 2 3]`.) Everything below about Vector is superseded; HandlePool and Stream stand.

**Drawn 2026-09-24.** From stone H's measurement (c): four values the writer claims can cross, and
none survives the trip.

```
Value::Vector     → Lost: unknown tag #wat.holon/Vector
HandlePool        → Lost: unknown tag #wat.kernel/HandlePool
Stream (forced)   → Lost: unknown tag #wat.stream/Stream
Stream (empty)    → ARRIVES AS A List in a Stream-typed slot — fails later at :wat::stream::next
```

## The builder's rulings, 2026-09-24

> *"HandlePool ..... Stream... i don't think these have any EDN repr at all.... stream must be
> materialized into a vec.... handle-pool.... this has no repr ... its nil if someone attempts to
> render it as data..."*
>
> *"holon was the orignal EdnRepresentable tool .. #wat.holon/Vector .... is very likely one of the
> last few hold outs where we abuse holon... we have been (for months) trying to kill off all holon
> abuse for non VSA/HDC things...."*

## ⛔ This REVERSES three deliberate "preserve the data" calls — named, not silently overwritten

| value | writer today (`src/edn/render.rs`) | the call it reverses |
|---|---|---|
| `HandlePool` | `#wat.kernel/HandlePool "<name>"` (`:4788`) | arc 294.i: *"the ONE exception to 'everything decorates nil' … flattening to nil would silently drop data"* |
| `Stream` | forced → `#wat.stream/Stream <head>`; **empty → `()`** (`:4882`) | arc 118 / 294.i: *"the forced head is real data the model does not discard"* |
| `Value::Vector` | `#wat.holon/Vector {:dim N}` (`:4770`) | 294.i: *":dim, the one legitimate non-secret fact about an opaque handle"* |

The reason the reversal is right: the writer's output is a CLAIM that the value can be read back.
A body that no reader can decode is the writer lying, and the empty-Stream `()` is the worst case —
it decodes successfully as the WRONG TYPE. Arc 294's own doctrine settles it (`BRIEF-294.i:15`):
*"A resource has no EDN representation. The tag says what it was; the `nil` body says you learn
nothing more."* These three were the exceptions to that rule; the rule wins.

## The cure

All three render through `opaque_nil_or_refuse` like every other opaque:
`#wat.kernel/HandlePool nil`, `#wat.stream/Stream nil` (every Stream state, Empty included),
`#wat.holon/Vector nil`. Consequence for free: stone G/H's strict wire encoder now REFUSES all three
at the sender, because they are `opaque_nil`. A program that wants a stream's data over a wire
materializes it into a vector first — the builder's stated route.

⚠ `Value::Vector`: the builder said *"very likely"*. The DESIGN treats it as opaque like the other
two; the BRIEF asks the executor to report what (if anything) consumes `{:dim}` before removing it.

## Out of scope

- Killing the remaining holon-for-EDN uses in general — arc 294's campaign; this stone takes only
  the one on the wire's path.
- Printing: a user who `println`s a stream to debug now sees `#wat.stream/Stream nil`. That is the
  ruling's direct consequence, not a regression.
