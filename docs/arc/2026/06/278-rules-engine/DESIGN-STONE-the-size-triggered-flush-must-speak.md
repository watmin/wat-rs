# DESIGN STONE — item (c) stone C: a size-triggered flush must speak

**Found 2026-09-01, grading stone B.** A live silent-drop in shipped, green code — and the floor is
green *over* it, because neither stone A's twelve rows nor stone B's fourteen asked the question.

## The defect

`log`, `incr` and `timed` each flush when the accumulated batch reaches the op's declared cap:

```wat
pair0 (:wat::core::if (…>= bytes cap…)
        (:wat::telemetry::span::flush-logs s)
        (:wat::core::Tuple s (…CloseResponse::Done)))
s1    (:wat::core::first pair0)      ;; the state is used
                                     ;; (second pair0) — the write's outcome — is NEVER READ
```

`(:wat::core::second pair0)` is read at exactly two sites in the file, `:170` (`flush`) and `:179`
(`close`). At the three size triggers it is dropped. So a Journal returning `Constraint`, `Transient`
or `Fatal` to a size-triggered flush is swallowed, and the caller is told `Ok`.

This is the class arc 278's no-hidden-failures law exists to kill, and `REALIZATIONS.md:8459` records
the builder catching this exact shape once already: *"the stdlib service handlers bound the Lost
cause `_cause` and DISCARDED it for a static string (a silent drop I seeded in my own brief; the
builder caught it)."* Same drop, one arc later, in the handler I briefed.

## What is already right — and it bounds this stone

`flush-logs`/`flush-metrics` **reset only on success**. The failure arm is `(_ (Tuple s cresp))`:
the ORIGINAL state, buffer intact, with the failure. **No data is lost today**, and the failure value
is already computed and correctly typed.

So the whole defect is that a value which exists, is correct, and is in hand gets thrown away. This
is not a repair of the flush path; it is three arms learning to read their second tuple element.

## The rule

> A size-triggered flush reports its outcome on the op that triggered it.

`Ok` continues to mean **"accepted"** — which is the honest name for what `log` guarantees, since it
always buffers and only sometimes writes.

## ★ Ruled: NOT two success values

The alternative was `Ok` (written) vs `Buffered` (accepted). Four-questioned and rejected at the
first:

- **Obvious? NO.** A caller cannot act differently on the two. `Ok`-as-written would occur only when
  a size trigger happened to fire *and* succeed, so a caller sees `Buffered` almost always and `Ok`
  occasionally, with no behaviour attached to either. It discriminates something unactionable.
- **Honest? PARTIAL.** It names durability while leaving the failed write on the floor — answering a
  question nobody asked while the real one stays dropped.

The caller does not need to know *which* of buffer-or-write happened. They need to know **when a
write failed.**

## The shape

`IncrResponse`, `TimedResponse` and `LogResponse` today carry `Ok` / `RequestTooLarge` /
`RequestMalformed`. Each gains `Constraint` / `Transient` / `Fatal` — the same three
`:wat::query::` variants `CloseResponse` and `FlushResponse` already carry.

**Derive-is-the-wall:** these are the sink's own failure vocabulary surfaced pass-through, never a
parallel telemetry error taxonomy. It is the discipline `Journal` already follows (`telemetry.wat`:
the write failures *"are the store's `put` failures, surfaced pass-through — NOT a parallel telemetry
error vocabulary"*).

And the caller matches the arms it already matches on `close`.

## The subtlety that must not be missed

**A failed flush still buffers the new item.** The log/sample that triggered the flush is not part of
the failed batch — it is the one that made the batch too big. Reporting the failure must not also
drop it, or the fix trades a silent failure for silent data loss.

So the arm returns the failure response **with the accumulated state**: report what failed, keep what
arrived.

## Blast radius, measured

`Span::{Incr,Timed,Log}Response` are matched in exactly **two** `.wat` files:
`wat/telemetry/span.wat` and `tests/services/probe_arc278_span_surface.wat`. Adding variants makes
those matches non-exhaustive and the checker names each site — the cascade is the progress meter, not
a crisis.

## Out of scope = REJECTED

- Backpressure, bounded buffers, drop policy. Downstream of this, and easier once `log` can say a
  flush failed at all.
- Any change to `flush-logs`/`flush-metrics` — their reset-only-on-success is already correct.
- Retry. A failed batch stays buffered and the next flush retries it by construction; an explicit
  retry policy is its own decision.
