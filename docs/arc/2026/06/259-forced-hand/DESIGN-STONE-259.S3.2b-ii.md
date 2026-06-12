# DESIGN — Stone 259.S3.2b-ii — the coordinator + `brackets/map` (Ruby's `Parallel.map`)

## Why

`brackets/map` is the bounded, dynamically-balanced worker pool — the meatiest stone
of arc 259. A pool of N runners drains `items` through `work-fn`, results returned in
INPUT ORDER. Built over `spawn-program'` + the S3.2a `runner-loop`. This stone is
thread-tier only (the probe's tier); process-tier is an affirmative cut → S3.x (the
coordinator's collect-loop is tier-agnostic; only the runner-spawn line specializes,
so process slots in later without touching the algorithm).

## The contract decision (pinned)

`(:wat::bracket::map host items work-fn) -> Vector<O>`, where `host : ThreadOpts`,
`items : Vector<I>`, `work-fn : Fn(I)->O`. Results in **input order**:
`result[i] = (work-fn items[i])`. N (runner count) = `min(cpu-count, M)`, M = `count items`.
Order is preserved by an **index round-trip**, not assoc-at-index: each work unit ships
as `(idx, item)`; the runner returns `(idx, work-fn item)`; the coordinator collects all
M pairs then `sort-by` idx. Cascade-abort is **structural** (a work-fn raise kills its
runner → that peer's `recv'` inside `select'` raises → propagates out → the let scope
unwinds → all peers drop → RAII drain+join) — no explicit abort code.

## The type threading (grounded against the disk)

- `spawn-program'` thread clause: prog `[Peer'<S,R>] -> nil` yields parent `Thread'<R,S>`
  (wat/spawn.wat:67-69).
- runner-loop `<II,OO>`: `[self <- Peer'<OO,II>, work-fn <- Fn(II)->OO] -> nil` — receives
  II, sends OO.
- We instantiate `II = (i64,I)`, `OO = (i64,O)`. So:
  - runner self : `Peer'<(i64,O),(i64,I)>`  → parent peer : `Thread'<(i64,I),(i64,O)>`.
  - parent sends `(i64,I)` (idx,item); receives `(i64,O)` (idx,result).
  - `select'` over `Vector<Thread'<(i64,I),(i64,O)>>` → `Tuple<i64, (i64,O)>`
    = (peer-position, (item-idx, result)).  (runtime.rs:4536, select probe.)

## The algorithm (1)–(5)

1. **N** = `(if (< cpu-count M) cpu-count M)` — inline; NOT a fake-general `min`
   (min/max generic-over-Orderable is bounded-poly-blocked, parked; see Notes).
2. **wrapped work-fn** `wf : (i64,I) -> (i64,O)` = `(Tuple (first pair) (work-fn (second pair)))`
   — index passthrough so order round-trips; runner-loop stays generic.
3. **spawn + prime** N runners in one `map` over `(range 0 N)`: each runner is born via
   `(spawn-program' host (fn [self <- Peer'<(i64,O),(i64,I)>] -> nil (runner-loop self wf)))`
   and immediately fed its first item `(send' p (Tuple i (nth items i)))`. Returns the
   `Vector<Thread'<(i64,I),(i64,O)>>` of primed peers.
4. **collect-loop** (top-level named-recursion defn — wat has no letfn; TCO fires on the
   tail self-call, runner-loop precedent): state `(peers items pairs-acc cursor collected m)`.
   - `collected == m` → `pairs-acc`.
   - else: `picked = (select' peers)`; `peer-pos = (first picked)`; `pair = (second picked)`
     (= the (idx,result)); accumulate `(conj pairs-acc pair)`; if `cursor < m` feed the just-
     freed runner the next unit `(send' (nth peers peer-pos) (Tuple cursor (nth items cursor)))`
     and `cursor+1`, else `cursor`; recurse with `collected+1`. The feed-the-freed-runner IS
     the dynamic balance; spares idle when `< N` items remain (select' never picks a dataless
     peer).
5. **assemble**: `(sort-by (fn [pr] (first pr)) pairs)` → ascending idx → `(map (fn [pr] (second pr)) sorted)`
   → `Vector<O>` in input order. Scope-exit drops the N peers → RAII drain+join.

## The inviolable rule

The coordinator touches runners ONLY through `select'` / `send'` / `recv'` (the Peer) —
NEVER a shared crossbeam queue. The Peer is transport-blind (thread crossbeam / process
EDN-over-pipe / future remote EDN-over-socket); a shared queue would bake in thread-only
and block `:remote`. This is the remote axis, bought in advance.

## Implementation sketch (the executor fills bodies; the shape is fixed)

Lands in `wat/bracket.wat` after `runner-loop`. TWO top-level defns:

```
(:wat::core::defn :wat::bracket::collect-loop<I,O>
  [peers     <- :wat::core::Vector<:wat::kernel::Thread'<(wat::core::i64,I),(wat::core::i64,O)>>
   items     <- :wat::core::Vector<I>
   pairs-acc <- :wat::core::Vector<(wat::core::i64,O)>
   cursor    <- :wat::core::i64
   collected <- :wat::core::i64
   m         <- :wat::core::i64]
  -> :wat::core::Vector<(wat::core::i64,O)>
  (:wat::core::if (:wat::core::= collected m)
    pairs-acc
    (:wat::core::let
      [picked   (:wat::kernel::select' peers)
       peer-pos (:wat::core::first picked)
       pair     (:wat::core::second picked)
       cursor'  (:wat::core::if (:wat::core::< cursor m)
                  (:wat::core::let [_ (:wat::kernel::send'
                                        (:wat::core::nth peers peer-pos)
                                        (:wat::core::Tuple cursor (:wat::core::nth items cursor)))]
                    (:wat::core::+ cursor 1))
                  cursor)]
      (:wat::bracket::collect-loop peers items
        (:wat::core::conj pairs-acc pair) cursor' (:wat::core::+ collected 1) m))))

(:wat::core::defn :wat::bracket::map<I,O>
  [host    <- :wat::spawn::ThreadOpts
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::Vector<O>
  (:wat::core::let
    [m  (:wat::core::count items)
     cc (:wat::program::cpu-count)
     n  (:wat::core::if (:wat::core::< cc m) cc m)
     wf (:wat::core::fn [pair <- :(wat::core::i64,I)] -> :(wat::core::i64,O)
          (:wat::core::Tuple (:wat::core::first pair) (work-fn (:wat::core::second pair))))
     peers (:wat::core::map
             (:wat::core::fn [i <- :wat::core::i64]
                 -> :wat::kernel::Thread'<(wat::core::i64,I),(wat::core::i64,O)>
               (:wat::core::let
                 [p (:wat::kernel::spawn-program' host
                      (:wat::core::fn [self <- :wat::kernel::Peer'<(wat::core::i64,O),(wat::core::i64,I)>]
                          -> :wat::core::nil
                        (:wat::bracket::runner-loop self wf)))
                  _ (:wat::kernel::send' p (:wat::core::Tuple i (:wat::core::nth items i)))]
                 p))
             (:wat::core::range 0 n))
     pairs (:wat::bracket::collect-loop peers items <EMPTY-PAIRS-VEC> n 0 m)
     sorted (:wat::core::sort-by
              (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :wat::core::i64 (:wat::core::first pr))
              pairs)]
    (:wat::core::map
      (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :O (:wat::core::second pr))
      sorted)))
```

## Watch-points (resolve against the checker; do NOT band-aid by dropping generics)

1. **Tuple type spelling in annotations** — `:(wat::core::i64,I)` (select probe uses
   `-> :(wat::core::i64,wat::core::i64)`, freeze.rs line 61). If a type-var inside the
   tuple type won't parse, that's a real finding — surface it.
2. **`<EMPTY-PAIRS-VEC>`** — the empty `Vector<(i64,O)>` seed for pairs-acc. Spell it the
   way an empty typed vector is spelled elsewhere; if the tuple-element type-arg is awkward,
   fall back to let-destructure or a known-good empty-vector form. Surface if it fights.
3. **`first`/`second` on a Tuple** — stream.wat (wat/stream.wat:88-89) reads tuples this way
   directly (no `Option/expect`), so they return the element typed. If the checker disagrees,
   use let-destructure `(let [[a b] tup] …)` (runtime.rs destructure_tuple) — the guaranteed
   typed tuple read.

## Out of scope (affirmative cuts, not deferrals)

- **process-tier brackets** → S3.x (the runner-spawn line specializes to a forms-server;
  the collect-loop is reused unchanged).
- **`(thread :runners N)` override** → folds with the ThreadOpts `:runners` field, a later stone.
- **`brackets/each`** (map that discards) → S3.3.
- **generic `min`/`max`** → a separate reach-stumble stone: their honest form is
  `min<T> where T:Orderable`, which needs bounded polymorphism (parked, off the critical
  path per arc 256's intrinsic-boundary insight). We do NOT ship an i64-only `min` wearing a
  general name. The coordinator inlines `(if (< cc m) cc m)`.

## Gate

`cargo test --release -p wat --test nursery probe_arc259_brackets_map -- --test-threads=1`
→ `brackets_map_doubles_in_order` (50 items, input order) + `brackets_map_small_in_order`
both GREEN. Plus `probe_arc259_bracket_runner` still green (runner-loop unchanged).
