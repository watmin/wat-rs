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

**`~peers-only-expr` is spliced RAW at two sites** — the `poll` call and the `serve-dispatch-op`
call (grep `peers-only-expr`; the ctx strike moved the line numbers once already). There is no
`let` hoisting it.

### ★ CORRECTED 2026-08-09 — it is THREE O(N) passes per message, not two

The original count was wat-side only, written from reading `service.wat` and **asserting about the
Rust side without opening it.** `poll` does not consume the projected vector cheaply — it rebuilds
the same information again natively (`runtime.rs`, `eval_poll_prime`, the `arg 2: peers` block):
it walks every element, `downcast_ref_opaque`s each to a `PeerCell`, clones each into a fresh
`Vec<PeerCell>`, then acquires an N-length `Vec<RefGuard>`. **Every call.**

| pass | where | cost |
|---|---|---|
| wat fold → bare peers | the `poll` call site | fresh N-element Value vector |
| downcast + clone + guard | inside `poll` | N downcasts, N `Arc` clones, N guard acquisitions |
| wat fold → bare peers, again | the `serve-dispatch-op` call site | another fresh N-element vector |

Per iteration without a message: the first two.

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
   `serve-dispatch-op` share it. Removes ONE of the three passes, ~5 lines, no Rust.
2. **Teach the intrinsics the tuple.** Let `poll`/`serve-dispatch-op` read element `.1` of a
   `(i64, Peer)` entry, and delete the projection entirely. **⚠ CORRECTED: this is NOT O(0).** It
   removes the two wat folds and leaves `poll`'s own downcast/clone/guard rebuild completely
   untouched — the largest of the three passes. The original claim ("delete the projection
   entirely. O(0)") was wrong.
3. **★ Own the peer set on the Rust side** (the builder's proposal, 2026-08-09) — a persistent,
   serve-loop-owned table that `poll` reads directly instead of being handed a fresh vector each
   call. This is the only option that removes all three passes, because it attacks the actual
   cause: **`poll`'s signature is "hand me the whole set again every time."** The set changes only
   on connect and eviction — rare events — so per-message rebuilding recomputes something already
   known unchanged, which the lockstep serve loop has perfect knowledge of.
   Legibility, the one thing the current shape does better, is recoverable by a **query intrinsic**
   — the substrate already does exactly this with `:wat::program::env` (`runtime.rs`, returns a
   pure `Env` record; the serve loop already calls it). Ask-and-be-told beats read-the-form-and-
   infer, and it cannot go stale: single-threaded, nothing runs between the query and the act.
4. **Measure first.** None is worth doing blind. A service-with-N-clients bench does not exist;
   building one is the honest precondition, and it would serve the connection-scoped world too.

## Standing caution

Do NOT "optimise" this by reintroducing a parallel bare-peer vector kept in sync alongside
`selectables`. That is exactly STOP-2, and the projection's whole virtue is that it is a DERIVED view
of one canonical structure — it cannot desync because there is nothing to forget to update. Any fix
must preserve that property.

**Fix 3 does NOT violate STOP-2 — provided the table is the SINGLE OWNER**, not a mirror. A Rust-side
table that *replaces* `selectables` as the one canonical structure has nothing to desync from; a
Rust-side table kept *alongside* the wat vector is the exact defect STOP-2 names. The distinction is
ownership, and it is the whole design.

## ⚠ A silent trap for anyone "simplifying" the projection away

`broadcast_peer_crashed_best_effort` (`src/kernel/peer.rs`) walks `clients` and **`continue`s on any
element that is not a `Peer'` opaque**:

```rust
let crate::value::Value::RustOpaque(inner) = elem else { continue; };
```

Hand it the `Tuple<i64,Peer>` vector raw and it skips **every element** — the service crashes and
**not one client is notified**, with no error anywhere. It is a `serve-dispatch-op` call site, so it
is on the panic path, where nothing is watching. That is a mask of exactly the class R55 spent an arc
tearing out, and it ships green. Any change to what these two intrinsics receive must confirm this
function still sees real peers.
