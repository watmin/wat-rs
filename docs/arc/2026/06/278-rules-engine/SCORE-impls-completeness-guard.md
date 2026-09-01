# SCORE — the `:impls` completeness guard

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. One weigh.

```
Summary [ 334.494s] 5162 tests run: 5162 passed (3 slow), 15 skipped
FLOOR=0
```

**All eleven rows pass, no deltas.** Third stone running with none.

| # | what | result |
|---|---|---|
| 1 | ★ partial satisfier rejected, ALL missing ops named | ✅ `ImplsIncomplete` names `:probe::partial` and **both** `pang` and `pong` |
| 2 | ★ complete satisfier compiles | ✅ `:probe::complete` never named |
| 3 | ★ extra INTERNAL arm compiles | ✅ `:probe::ticking` never named — the shape a symmetric rule breaks |
| 4 | the census, RUN | ✅ **1705 files, one rejection: the red probe** |
| 5 | self-scheduling survives | ✅ `span.wat` clean (five features, seven arms) |
| 6 | parametric surfaces survive | ✅ `cache.wat`, `service-cache-lru.wat` clean |
| 7 | no runtime change | ✅ `src/runtime.rs` and `wat/service.wat` both empty |
| 8 | no rune on the criterion | ✅ zero rune forms |
| 9 | the probe is in `probes/` | ✅ |
| 10 | the error teaches | ✅ *"defservice :probe::partial :satisfies :probe::Trio but :impls is missing op(s): pang, pong"* |
| 11 | floor | ✅ 5162/5162, my own run |

## The census the brief refused to invent

The BRIEF deliberately shipped **no** census, because the one I attempted was noise — it reported
`:wat::telemetry::journal` missing five ops that belong to `Span`, and listed `:wat::core::let` as a
missing op after matching `let` bindings inside impl bodies.

The guard produced the real one. I ran it wider than the report: **1705 `.wat` files** across
`tests/`, `wat-scripts/`, `wat/`, `wat-tests/`, `examples/` and `docs/` — an order of magnitude more
than the 175 reported — and the only rejection is the deliberate red probe.

So the corpus has **zero live partial satisfiers**, and the stone ships as drawn. That is worth more
than a number in a brief would have been: it is a measurement of the whole tree rather than an
estimate of part of it, and it was produced by the thing it validates.

## The implementation is better than the sketch

The BRIEF said to resolve `:satisfies` to its surface and compare op-name sets. The strike instead
rides the **`surface::Op <: service::Op` derive edge that `defservice` already emits** — the same
structural relation the dispatch uses. It is not a second source of truth about what a surface
declares; it is the one that already exists, asked a new question.

## What made this stone quiet

Three stones in a row now with no delta — item (b), stone D, and this one. The common property is
that each brief's load-bearing decisions were made against measurements taken first, and where a
measurement could not be trusted, **the brief said so instead of guessing**:

- item (b): `Stream`/`WriteResult`/chunker all absent → build over a Vector.
- stone D: which accumulators actually grow → bound two, leave `counters`.
- this one: the census was noise → ship no census, make it the guard's own output.

Every earlier stone in this campaign corrected a specification error of mine, and every one of those
came from a row written out of memory or generalisation instead of a read. The fix was not to write
more carefully. It was to **measure first, and to say plainly when the measurement failed.**

## Closes

`NOTE-impls-completeness-is-unenforced.md` — the guard it recorded as unbuilt is built. A promise
made at a `defservice` is now checked at that `defservice`, rather than discovered at a call.
