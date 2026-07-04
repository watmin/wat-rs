;; wat/seq.wat — arc 118.2a NOMINA NOTA, MACHINA TACITA — the clojure-named HOF surface.
;;
;; `:wat::seq::*` (the eager-opt-in namespace from the older 118.2 DESIGN) is RETIRED here —
;; superseded by the ratified decision (`docs/arc/2026/07/300-wat-source-is-edn/REALIZATIONS.md`,
;; the NOMINA NOTA interstitial): surface = clojure names, public in `:wat::core::`; primitives
;; = plumbing (`:wat::stream::` cons/lazy/empty; `foldl`/`length` internal). Its two former
;; aliases (`:wat::seq::reduce` / `:wat::seq::fold`, both -> `:wat::core::foldl`) promote to the
;; single `:wat::core::reduce` added below (proper 2/3-arity clojure reduce). Callers migrated
;; via `wat-scripts/fixes/rename-seq-fold-aliases-to-core-reduce.wat`.
;;
;; This file is the home of the rest of the 118.2a flip's new surface:
;;   - `:wat::core::filter` — the one lazy HOF that ships as wat-over-primitives (Decision B's
;;     original preference). `map`/`take`/`drop` stay Rust intrinsics instead — a forced,
;;     named exception: several stdlib macros (`:wat::core::defn` itself, `:wat::core::defrecord`
;;     / `:wat::holon::defrecord` / `:wat::service::defservice` / `:wat::rete::defrule` /
;;     `:wat::core::format`) call `map`/`take`/`drop` INSIDE their own macro bodies — at
;;     macro-expansion time, before any wat-defined `defclause`'s real clauses would exist. See
;;     `crate::stream::NativeLazyCell`'s doc (src/stream/mod.rs) for the full writeup. `filter`
;;     has no such caller anywhere in the stdlib, so it is safe to self-host.
;;   - the eager materializers: `mapv` / `filterv` / `into` / `doall` / `dorun`. There is NO
;;     `vec` — `:wat::core::vec` is a HARD-retired name in this substrate
;;     (`src/remedy/retirement.rs`: an old verb-equals-type alias for the `Vector`
;;     constructor). Ratified: the clojure-faithful materializer IS `(:wat::core::into []
;;     coll)` (real clojure's own idiom for "force a seq into a vector") — no new name
;;     needed. The retirement message now points there.
;;   - `:wat::core::run!` — the EAGER side-effecting consumer (clojure's `run!`). The flip's
;;     mechanical cascade converted eager side-effecting `(map f coll)` loops into
;;     `(dorun (map f coll))` — the clojure lazy-map-side-effect ANTI-PATTERN (routing effects
;;     through a lazy stage at all, even when something downstream eventually forces it). `run!`
;;     retires that shape: built directly over `foldl` (never over `:wat::stream::` primitives),
;;     it calls `f` exactly once per element of a plain Vector/List/PersistentVector, discards
;;     the return, and yields `nil`.
;;   - `:wat::core::reduce` — proper clojure reduce (2-arity + 3-arity) built over `foldl` for
;;     the eager containers, and a dedicated single-pass walker for `Stream`.
;;   - `:wat::core::count` — a defalias over the KEPT `length` primitive (clojure surface name).
;;
;; STOP surfaced (not built): `reduced` / `reduced?` (clojure's early-exit `reduce` marker).
;; wat's type universe is CLOSED (`:Any` is banned, `src/types.rs` line ~78) — there is no way
;; to type a reducing function's return as "T, or a Reduced<T> early-exit wrapper" without
;; reopening that banned escape hatch, and a control-flow-signal mechanism (mirroring
;; `Result/try`'s `EvalSignal::TryPropagate`) would need NEW Rust plumbing (a new signal variant
;; + checker special-casing) beyond a wat-over-primitives change. Per the STOP doctrine this is
;; surfaced, not guessed/hacked around. `:wat::core::reduce` here has NO early-exit — it always
;; walks the whole input (exactly like the `foldl`/`:wat::seq::reduce` it replaces; no
;; regression, just not a new capability).

;; ─── filter — the one lazy HOF built in wat (Decision B; no bootstrap-critical caller) ──────
;;
;; Same recursive shape as the Rust-native lazy map/take/drop (see `src/collection/transform.rs`):
;; `(:wat::stream::lazy <body>)` defers `<body>`; forcing checks `empty?`, then `pred`, then
;; either yields `(cons head (filter pred (rest coll)))` or recurses PAST a rejected element by
;; calling `filter` again (itself `stream::lazy`-wrapped, so this is O(1) per rejected element,
;; not eager work) — `realize`'s iterative thunk-forcing loop (src/stream/mod.rs) walks through
;; consecutive rejections in one pull, exactly like the Rust-native filter would.
;;
;; Four clauses — one per seqable this arc's `first`/`rest`/`empty?` already support
;; polymorphically (Vector/List/PersistentVector/Stream) — dispatched by arg-2's concrete type
;; (defclause arity+type dispatch; `pred`-first mirrors the retired eager `filter`'s call order).
(:wat::core::defclause :wat::core::filter
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::filter pred (:wat::core::rest coll)))
          (:wat::core::filter pred (:wat::core::rest coll))))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::filter pred (:wat::core::rest coll)))
          (:wat::core::filter pred (:wat::core::rest coll))))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::filter pred (:wat::core::rest coll)))
          (:wat::core::filter pred (:wat::core::rest coll))))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::filter pred (:wat::core::rest coll)))
          (:wat::core::filter pred (:wat::core::rest coll))))))
  ;; Bare (un-parameterized) PersistentVector — arc 278 0d.1 regression guard territory: a
  ;; heterogeneous field (e.g. a record's un-parameterized PersistentVector) must type-check
  ;; through the HOFs too. T is pinned entirely from `pred`'s concrete type at the call site.
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::filter pred (:wat::core::rest coll)))
          (:wat::core::filter pred (:wat::core::rest coll)))))))

;; ─── the eager materializers ─────────────────────────────────────────────────────────────────

;; stream->vec — internal helper: drains a Stream into a Vector, seeded by `acc` (so `into` can
;; append onto an existing Vector, not just build from empty). Tail-recursive (TCO trampoline
;; keeps this O(1) Rust-stack regardless of stream length).
(:wat::core::defn :wat::core::stream->vec<T>
  [acc <- :wat::core::Vector<T> s <- :wat::stream::Stream<T>] -> :wat::core::Vector<T>
  (:wat::core::if (:wat::core::empty? s)
    acc
    (:wat::core::stream->vec (:wat::core::conj acc (:wat::core::first s)) (:wat::core::rest s))))

;; mapv / filterv — the eager forms: force `map`/`filter`'s lazy Stream result to a Vector in
;; one step via `(into [] ...)` (clojure's own materializer idiom — no new name). Two clauses —
;; Vector input (the direct case) AND Stream input (composing after another lazy stage, e.g.
;; `(filterv pred (map f xs))` — `map` stays lazy, `filterv` is the pipeline's eager exit).
;; Extend with more clauses if a call site needs List/PersistentVector input directly (ride the
;; red — 118.2b+).
(:wat::core::defclause :wat::core::mapv
  ([f <- :wat::core::Fn(T)->U coll <- :wat::core::Vector<T>] -> :wat::core::Vector<U>
    (:wat::core::into [] (:wat::core::map f coll)))
  ([f <- :wat::core::Fn(T)->U coll <- :wat::stream::Stream<T>] -> :wat::core::Vector<U>
    (:wat::core::into [] (:wat::core::map f coll))))

(:wat::core::defclause :wat::core::filterv
  ([pred <- :wat::core::Fn(T)->wat::core::bool coll <- :wat::core::Vector<T>] -> :wat::core::Vector<T>
    (:wat::core::into [] (:wat::core::filter pred coll)))
  ([pred <- :wat::core::Fn(T)->wat::core::bool coll <- :wat::stream::Stream<T>] -> :wat::core::Vector<T>
    (:wat::core::into [] (:wat::core::filter pred coll))))

;; stream->pvec — the PersistentVector twin of `stream->vec` (118.2b cascade: rete.wat's
;; PersistentVector<Rule>/PersistentVector<DerivationStep> fields need a Stream materialized
;; into a PersistentVector, not a Vector).
(:wat::core::defn :wat::core::stream->pvec<T>
  [acc <- :wat::core::PersistentVector<T> s <- :wat::stream::Stream<T>] -> :wat::core::PersistentVector<T>
  (:wat::core::if (:wat::core::empty? s)
    acc
    (:wat::core::stream->pvec (:wat::core::PersistentVector/conj acc (:wat::core::first s)) (:wat::core::rest s))))

;; into — clojure's `(into to from)`: append every element of `from` onto `to`. `to` determines
;; the output container kind (Vector or PersistentVector, both in scope); `from` may be a
;; same-kind eager container (delegates to `concat`) or a Stream (delegates to `stream->vec`/
;; `stream->pvec`, seeded by `to` — the general "append a realized pipeline onto an
;; accumulator" shape).
(:wat::core::defclause :wat::core::into
  ([to <- :wat::core::Vector<T> from <- :wat::core::Vector<T>] -> :wat::core::Vector<T>
    (:wat::core::concat to from))
  ([to <- :wat::core::Vector<T> from <- :wat::stream::Stream<T>] -> :wat::core::Vector<T>
    (:wat::core::stream->vec to from))
  ([to <- :wat::core::PersistentVector<T> from <- :wat::stream::Stream<T>] -> :wat::core::PersistentVector<T>
    (:wat::core::stream->pvec to from)))

;; doall / dorun — eager forcers (Stream -> Vector / nil). DIALECT NOTE: clojure's `doall`
;; returns the SAME (now-forced) lazy seq, replayable — wat's Stream is single-pass / NEVER
;; memoized (arc 118 R1, NON BIS IN IDEM FLVMEN: "you cannot walk back a stream"), so there is
;; no "same seq, now forced" to hand back. The honest wat-dialect equivalent: fully realize into
;; a Vector (forces every element / side-effect) and return THAT. `dorun` is the same walk,
;; discarding the values (side-effects only) and returning nil.
(:wat::core::defn :wat::core::doall<T> [coll <- :wat::stream::Stream<T>] -> :wat::core::Vector<T>
  (:wat::core::into [] coll))

(:wat::core::defn :wat::core::dorun<T> [coll <- :wat::stream::Stream<T>] -> :wat::core::nil
  (:wat::core::do (:wat::core::into [] coll) nil))

;; ─── run! — the eager side-effecting consumer (clojure's `run!`) ──────────────────────────────
;;
;; EAGER BY CONSTRUCTION: folds `foldl` directly over the PLAIN input container — never over a
;; `:wat::stream::` Stream. This is the cure for the `(dorun (map f coll))` anti-pattern: a
;; side-effecting loop must never be built by routing `f` through the lazy `map` stage and then
;; force-draining it — `run!` is the one-step eager consumer clojure itself reaches for. Three
;; clauses (Vector/List/PersistentVector — the concrete eager containers `foldl` already spans);
;; deliberately NO Stream clause (a lazy pipeline must be materialized with `mapv`/`into` BEFORE
;; it reaches `run!` — reproducing the anti-pattern by adding a Stream arm here is exactly the
;; mistake this function exists to retire). Calls `f` exactly once per element, in order; `f`'s
;; return (type `U`, deliberately unconstrained — real callers' side-effecting fns return `nil`,
;; an eviction `Option`, whatever) is always discarded, and `run!` itself always yields
;; `:wat::core::nil` (mirrors clojure's `run!`: for effects, not values).
(:wat::core::defclause :wat::core::run!
  ([f <- :wat::core::Fn(T)->U coll <- :wat::core::Vector<T>] -> :wat::core::nil
    (:wat::core::foldl
      (:wat::core::fn [_acc <- :wat::core::nil x <- :T] -> :wat::core::nil (:wat::core::do (f x) nil))
      nil
      coll))
  ([f <- :wat::core::Fn(T)->U coll <- :wat::core::List<T>] -> :wat::core::nil
    (:wat::core::foldl
      (:wat::core::fn [_acc <- :wat::core::nil x <- :T] -> :wat::core::nil (:wat::core::do (f x) nil))
      nil
      coll))
  ([f <- :wat::core::Fn(T)->U coll <- :wat::core::PersistentVector<T>] -> :wat::core::nil
    (:wat::core::foldl
      (:wat::core::fn [_acc <- :wat::core::nil x <- :T] -> :wat::core::nil (:wat::core::do (f x) nil))
      nil
      coll)))

;; ─── reduce — proper clojure reduce (2-arity + 3-arity), no early-exit (see STOP note above) ──
;;
;; reduce-stream — internal helper: the Stream-input walk `foldl` cannot do (foldl is Vector/
;; List/PersistentVector-only, untouched this arc — see src/collection/transform.rs). Tail-
;; recursive, single-pass.
(:wat::core::defn :wat::core::reduce-stream<T,U>
  [f <- :wat::core::Fn(U,T)->U acc <- :U s <- :wat::stream::Stream<T>] -> :U
  (:wat::core::if (:wat::core::empty? s)
    acc
    (:wat::core::reduce-stream f (f acc (:wat::core::first s)) (:wat::core::rest s))))

(:wat::core::defclause :wat::core::reduce
  ;; 3-arity: explicit init — Vector/List/PersistentVector delegate straight to `foldl` (the
  ;; EXACT primitive `:wat::seq::reduce`/`fold` delegated to before this arc — behavior-
  ;; preserving); Stream uses `reduce-stream`.
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::core::Vector<T>] -> :U
    (:wat::core::foldl f init coll))
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::core::List<T>] -> :U
    (:wat::core::foldl f init coll))
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::core::PersistentVector<T>] -> :U
    (:wat::core::foldl f init coll))
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::stream::Stream<T>] -> :U
    (:wat::core::reduce-stream f init coll))
  ;; 2-arity: no init — first element seeds the fold, `f` reduces T,T->T. An empty `coll` raises
  ;; (via `first`'s out-of-range failure) rather than calling a 0-arity `(f)` the way real
  ;; clojure does — wat `fn` values are fixed-arity, so that edge is out of scope; an honest,
  ;; located failure instead of a silent 0-arity dispatch.
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::core::Vector<T>] -> :T
    (:wat::core::foldl f (:wat::core::first coll) (:wat::core::rest coll)))
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::core::List<T>] -> :T
    (:wat::core::foldl f (:wat::core::first coll) (:wat::core::rest coll)))
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::core::PersistentVector<T>] -> :T
    (:wat::core::foldl f (:wat::core::first coll) (:wat::core::rest coll)))
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::stream::Stream<T>] -> :T
    (:wat::core::reduce-stream f (:wat::core::first coll) (:wat::core::rest coll))))

;; count — the clojure surface name over the KEPT `length` primitive (unchanged: an infinite/
;; lazy Stream still correctly rejects `length`/`count` — see `StreamContainer::measurable`).
(:wat::core::defalias :wat::core::count :wat::core::length)
