# NOTE — `Seqable` has no name in wat, and that absence costs 12 functions

**Filed 2026-07-31, out of arc 278's seq-traversal work
(`278-rules-engine/DESIGN-STONE-seq-traversal-one-door.md`). Not fixed. Tracked here because it is
a TYPE-SYSTEM gap, not a collections bug.**

## The finding

Clojure has exactly one `filter`. It calls `seq` on its argument — the universal coercion every
collection implements — and walks an `ISeq`. One body, no per-container variants. `filterv` is just
`(into [] (filter pred coll))`.

wat cannot write that, because **wat has no way to spell "any seqable" as a parameter type.**

The concept exists — but only inside the Rust checker, as `extract_lazyable_elem`
(`src/collection/infer.rs:637`), a hardcoded match on four heads: `Vector`, `List`,
`PersistentVector`, `Stream`. It is checker-internal. No wat signature can name it.

So a wat-level sequence verb that wants to accept several containers has exactly one option: a
`defclause` with **one arm per concrete container**. And because those arms would otherwise each
duplicate the whole body, the corpus grew a workaround — the `<verb>-stream` twin: a Stream-only
walker every eager arm delegates to after normalising.

**Those twins exist only because this type is missing.** There are seven today —
`keep-stream`, `keep-indexed-stream`, `dedupe-stream`, `distinct-stream`, `map-indexed-stream`,
`interpose-stream`, `reduce-stream` — and arc 278's Strike-2 census found five more would have to
be minted (`filter`, `remove`, `take-while`, `drop-while`, `reductions`) to finish the pattern.
Twelve functions, none of which a Clojure-shaped substrate would have.

## Why it is not a small fix

Three separate type-system extensions, braided:

1. **No surface nature admits a builtin container.** Every `defsurface` carries a `:nature`, and the
   ones that exist are `:wat::core::Record` and `:wat::kernel::Peer'`. A `Vector` is neither.
2. **No builtin satisfies any surface today** — grep-verified: zero `extend-type` on
   `Vector` / `PersistentVector` / `List`. Surface satisfaction is an aggregate mechanism
   (attributes + methods, R28); builtins are outside it.
3. **wat has no ad-hoc unions, deliberately** (R7: *"an ADT substrate with no ad-hoc unions"* — the
   reason `:wat::core::Value` is a one-line universal top rather than a union). A bound over four
   concrete builtins is structurally a union unless the surface mechanism genuinely subsumes it.

Four-questioned in the 278 design: this option **fails Simple** on those three, which is why arc
278 took the native route instead. That was a scope ruling, not a verdict that this is wrong.

## What the 278 work does for it

Arc 278's Strike 2 makes the affected lazy stages native, dispatching through `StreamContainer` and
`extract_lazyable_elem` the way `map`/`take`/`drop`/`seqable->stream` already do.

That is **a precondition for this note's fix, not a competitor to it.** Today "what is seqable" is
spelled twice over — once in the checker, and implicitly again in ~29 `defclause` arms across six
verbs. You cannot promote a concept to a type while it is redundantly re-spelled in 29 places and
hoping they agree. After 278, the set lives in **one** function, and this becomes: *give that one
thing a name in the surface language.*

278 also pre-deletes the twelve twins, which any fix here would have had to delete anyway.

## The honest risk, recorded so it is not a surprise

**278 removes the pain that would motivate this.** Once the verbs are native, fast and correct,
nothing hurts, and this degrades from "fixes a real defect" to "improves legibility" — the class of
work that never gets scheduled. Arc 278's own 24q found the pattern: *"nearly everything deleted was
a stepping stone that outlived its mechanism."* A native seq library is a candidate for exactly
that fate.

The mitigation 278 carries: the checker-side seqable set is **named** for what it is at its single
definition site, with these blockers written beside it, so this note is a marked delta at a known
location rather than a good intention.

## What "done" would look like

`filter` (and every sibling) is **one** wat `defn` taking a seqable-bounded parameter, with no
per-container arms and no `-stream` twin — the shape Clojure has. The twelve twins are gone. A new
lazy stage cannot hand-roll per container, because it never names a container.

That last sentence is the real prize: 278's native route reaches the **check** rung (a convention
plus a lint), while this reaches **no-form**. It is the difference between "sequence verbs should be
native" and "a sequence verb cannot see a container."

## Kin

- `278-rules-engine/DESIGN-STONE-seq-traversal-one-door.md` — the measurement, the native ruling,
  and the Strike-2 census that counts the twins.
- R14 (`Phoenix`) — the seq-container registry unified container **classification**; **traversal**
  was left in the quarry, and this note is the last piece of that same quarry: the *type* that would
  let the surface language express what the registry already knows.
- R7 — no ad-hoc unions; why the universal top is a coordinate you point at rather than a union.
- R28 — surfaces as the structural contract of what-may-pass; the mechanism this would extend.
