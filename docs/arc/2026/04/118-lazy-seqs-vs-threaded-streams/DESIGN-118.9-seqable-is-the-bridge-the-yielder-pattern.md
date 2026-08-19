# ⛔⛔ WRONG — THIS PROPOSES THE DESIGN ARC 118 ALREADY KILLED. Do not build it.

> **Builder, 2026-08-17:** *"we needed this CSP thing that doesn't put the producer in a dedicated
> thread… we need to not force ourselves back onto the 'threaded producer over a crossbeam' — that's
> not right, we started there and had to kill it."*
>
> **He is right, and the epitaph is on disk.** `src/stdlib.rs:226`:
>
> > *Arc 118 — `wat/stream.wat` **ANNIHILATED** (2026-06-27). The **thread-per-pure-stage**
> > `:wat::stream::*` HOFs were **built wrong, successfully**; the namespace is reclaimed for the
> > **lazy single-pass Stream family**.*
>
> I found `send`/`recv` and `ZERO-MUTEX`'s tier 3 and never looked for the epitaph. **A
> channel-backed producer is a spawned program — a thread — which is precisely what was killed seven
> weeks ago.** Third variant today of the same failure: found prior art, did not read what replaced
> it and why.
>
> ## ★ THE THUNK IS ALREADY THE COROUTINE — no thread required
>
> `src/stream/mod.rs:122`:
>
> ```rust
> pub struct NativeLazyCell {
>     pub thunk:  NativeThunk,                    // ← the suspended producer. THIS is the yielder.
>     pub forced: Arc<OnceLock<Arc<Stream>>>,     // ← and THIS is the defect.
> }
> ```
>
> The thunk *is* Ruby's Fiber-without-a-thread: it pauses by returning, resumes when forced, and
> carries producer state in its closure. The builder's paginated `Enumerator` is a thunk chain whose
> closure holds `next_token` — **no channel, no spawn, no crossbeam.**
>
> **The defect is `forced: OnceLock<Arc<Stream>>` — the cell memoizes its tail forever.** Every cell
> pins its successor, so while anything holds the head the whole realized chain is retained. That is
> the measured **585 B/element** (`MEASURED-118.8`), and it is why *"lazy single-pass"* — the
> family's own name for itself — does not describe what runs.
>
> **The work is to make the existing lazy Stream actually single-pass, not to replace it.**
> `Seqable` remains the interface question; the retention is a separate, mechanical defect in one
> struct field. Everything below about `Iterator`-as-a-trait and one-interface-N-implementations
> still holds — what is struck is the claim that the second implementation must be a channel.

---

# DESIGN — 118.9 · `Seqable` is the BRIDGE. The yielder pattern is a channel, and we already have channels.

**Builder, 2026-08-17**, naming the pattern he wants:

```ruby
paginated = Enumerator.new do |yielder|
  next_token = nil
  loop do
    resp = client.get_items(next_token)
    resp.items.each { |i| yielder << i }
    next_token = resp.next_token
    break if next_token.nil?
  end
end
# do work on paginated
```

> *"the `yielder << i` is a blocking operation on the producer… we pause that block if we don't
> consume. if we stop consuming the loop breaks itself out. producers and consumers are lockstep…
> this is `ZERO-MUTEX.md`."*

**He is describing Tier 3, and Tier 3 is already built.**

## ★ The real split brain — two tiers, one concept, no bridge

Measured on `61f1ee64`:

```
COLLECTION TIER                          CONCURRENCY TIER
:wat::stream::{lazy, cons, empty}        :wat::kernel::{send, recv}
a memoized cons-cell chain               bounded channels, spawned programs
O(n) retention — 585 B/element           O(1) BY CONSTRUCTION — the producer blocks
map/filter/keep/... are built on it      SendOutcome/RecvOutcome/TrySendOutcome, faced
                        ⛔  NO BRIDGE  ⛔
```

`grep` for any channel↔stream connection in `wat/`: **nothing.** Two mechanisms for *"a sequence of
values produced over time"*, neither aware of the other, and every collection verb sits on the one
that retains.

## ★★ Ruby's trick, named: `Enumerator` is the CONCURRENCY shape wearing the COLLECTION interface

`Enumerator.new { |y| … }` is a **Fiber** — a coroutine — that satisfies `Enumerable`. `y << i`
suspends the producer until the consumer pulls. That is not a lazy list; **it is a bounded(1)
rendezvous**, which is exactly what `ZERO-MUTEX.md:260` already calls *mini-TCP*:

> *"producer writes on one pipe, blocks on the companion pipe until the consumer signals 'done.'
> Two pipes per producer, bounded(1) on each, mutually blocking through the substrate's existing
> rendezvous discipline."*

**The mechanism the builder wants is on disk. What is missing is that it cannot be spelled as a
sequence.**

## And Rust says the same thing structurally

`Iterator` is a **trait**, not a data structure:

| impl | backing |
|---|---|
| `std::vec::IntoIter` | in-memory, cheap, no threads |
| `std::sync::mpsc::IntoIter` | **channel-backed** — `next()` blocks on `recv()` |

One interface, N implementations, and **the channel-backed one is an `Iterator` like any other.**
Every combinator (`map`, `filter`, `take`) works on both without knowing which it has.

**That is `Seqable`.** Not "one verb for four containers" — *the interface that lets a channel be a
sequence.*

## The design

```wat
(:wat::core::defsurface :wat::core::Seqable<T> :nature :wat::core::Struct
  :features [(seq [self <- :wat::core::Seqable<T>] -> :wat::stream::Stream<T>)])
```

with **three kinds of implementation**, not one:

| implementor | cost | for |
|---|---|---|
| `Vector` · `List` · `PersistentVector` | O(1) extra — walk in place | in-memory data |
| **a channel Rx** | **O(1) — lockstep, producer blocks** | streaming input: pagination, sockets, files |
| a user's own container | theirs | extensibility |

And the builder's pattern becomes wat:

```wat
;; the producer — a spawned program writing into a bounded(1) channel.
;; `send` BLOCKS when full: that is `yielder << i`.
;; the consumer stopping is the channel closing: that is `break if next_token.nil?`.
(:wat::core::defn :my::paginate [client <- :my::Client] -> :wat::core::Seqable<my::Item>
  …spawn a producer over a bounded(1) channel; return its Rx as a Seqable…)

;; …and then ORDINARY verbs work on it, with no idea it is a channel
(:wat::core::filter :my::interesting? (:my::paginate client))
(:wat::core::keep   :my::parse        (:my::paginate client))
```

★ **Constant memory is not a property we add — it is what the channel already is.** The producer is
suspended until the consumer pulls. One item in flight. No cell chain to retain, so the 585 B/element
does not exist on this path.

## What this reframes

1. **`Seqable` is more urgent than "the missing type" argued.** It is the only thing that makes the
   substrate's *own* streaming tier usable from the collection surface. Today a paginated producer
   and `filter` cannot meet.
2. **The memoizing cons-`Stream` may not need to exist.** It is a third implementation whose measured
   retention is the defect. **Do not assume it dies** — the in-memory path may still want a cheap
   non-threaded lazy cell, and a thread per `map` over a 3-element vector would be absurd. **That is
   a measurement, not a ruling**: compare a channel-backed stage against the cons cell at small N.
3. **The `-stream` twins are downstream of this**, not the point.

## ⚠ NOT MEASURED — the honest gaps

- **Whether a channel Rx can satisfy a surface today.** `118.3-B` proved *builtins* can
  (`Vector`, `PersistentVector`, `List`, `Stream`). A `Peer`/channel handle is a different `:nature`
  (`:wat::kernel::Peer'`), and `nature_floor_ok` may refuse it. **This is the first probe** — it
  decides whether the bridge is buildable at all.
- **Thread cost per stage.** A bounded(1) channel per `map` is a thread. Unmeasured, and it decides
  whether channel-backing is the default or the opt-in for I/O-scale sources.
- **Whether `send`/`recv` block the way the pattern needs**, including the consumer-stops-→-producer-
  unwinds half. `ZERO-MUTEX.md` describes it; I have not run it.
- **The 585 B/element decomposition** (`MEASURED-118.8`) is still not isolated, and it must not be
  assumed to be "the memoized value."

## The order this implies

1. **Probe: can a channel handle satisfy a `defsurface`?** One file. It gates everything else.
2. **Probe: the yielder end-to-end** — spawn a bounded(1) producer, consume it, and measure maxRSS at
   the four sizes from `MEASURED-118.8`. **The acceptance test is that the per-element delta is ~0.**
3. *Then* mint `Seqable`, with the channel implementation as a first-class member rather than an
   afterthought — because designing it around only the four in-memory containers is what would bake
   the split brain back in.
