# NOTE — the serve loop rebuilds its peer vector TWICE per message, and no gate measures it

> **Found by reading the diff, 2026-08-09**, while weighing the call-context strike. **Not a defect,
> not a STOP** — correctness is fine and every gate is green. It is a per-event COST introduced into
> the multiplexer's hot path, scaling with connected-client count, that nothing in the suite measures.
> Recorded so it is a decision rather than a discovery.

## What landed

The call-context strike changed `selectables` from `Vector<Peer<R,O>>` to
`Vector<(i64, Peer<R,O>)>` — the caller id travels WITH its peer (STOP-2 of
`BRIEF-the-call-context.md`; the alternative, a parallel position-keyed vector, desynchronises when a
fired alarm timer is removed from one and not the other).

But `:wat::kernel::poll` and `:wat::kernel::serve-dispatch-op` are Rust intrinsics that downcast every
`Vector` element to a real `Peer` opaque — **a tuple wrapper is invisible to them.** So the macro
splices a projection that folds the entries back down to bare peers:

```clojure
peers-only-expr `(:wat::core::foldl ~peers-fold-fn (:wat::core::Vector ~selectable-peer-ty) selectables)
```

## The cost

**`~peers-only-expr` is spliced RAW at two sites** — `wat/service.wat:1412` (the `poll` call) and
`:1495` (the `serve-dispatch-op` call). There is no `let` hoisting it.

So at runtime, per message: **two full O(N) vector rebuilds**, N = connected clients. Per iteration
without a message: one.

⚠ **The macro comment reads *"Computed ONCE here (outer macro scope) so `serve-body` below can splice
it at both call sites."*** That "once" is **macro-EXPANSION-time** — the expression is built once and
spliced twice, so it EVALUATES twice. A reader can easily take it as a runtime claim. Worth one line
of correction next time `service.wat` is touched.

## Why no gate caught it

The acceptance tests use **1–3 clients**. At N=3 two folds are free. The cost is invisible until a
service holds tens or hundreds of connections — which is precisely the multi-tenant case the
connection-scoped world exists for. **There is no many-client service benchmark in the suite at all**,
so this is unmeasured rather than measured-and-accepted.

That is the honest framing: `[[feedback_a_green_test_can_prove_nothing]]` — name what would have to
break for it to go red. Nothing here would.

## The fixes, cheapest first

1. **Hoist it.** Bind the projection once per iteration at the top of `serve-body`; `poll` and
   `serve-dispatch-op` share it. Halves the cost, ~5 lines, no Rust. Still O(N) per iteration.
2. **Teach the intrinsics the tuple.** Let `poll`/`serve-dispatch-op` read element `.1` of a
   `(i64, Peer)` entry, and delete the projection entirely. O(0). A `src/` change to two intrinsics —
   the correct fix, and it removes the whole class rather than halving it.
3. **Measure first.** Neither is worth doing blind. A service-with-N-clients bench does not exist;
   building one is the honest precondition, and it would serve the connection-scoped world too.

## Standing caution

Do NOT "optimise" this by reintroducing a parallel bare-peer vector kept in sync alongside
`selectables`. That is exactly STOP-2, and the projection's whole virtue is that it is a DERIVED view
of one canonical structure — it cannot desync because there is nothing to forget to update. Any fix
must preserve that property.
