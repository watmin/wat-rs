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

;; ═══ 118.2-Z strike A — the lazy transformer family ═══════════════════════════════════════════
;;
;; Twelve clojure-core lazy transformers, each a `:wat::core::defclause` mirroring `filter`'s
;; shape above (one clause per seqable — Vector/List/PersistentVector/Stream + bare-
;; PersistentVector — `stream/lazy` + `first`/`rest`/`empty?` + `stream/cons`/`stream/empty`).
;; Forms that carry state across the walk (an index, a seen-set, a running accumulator, the
;; previous element) normalize their input to a genuine `Stream<T>` ONCE (via the private
;; `seqable->stream` helper below) and then delegate to a single Stream-only `<form>-stream`
;; helper `defn` — exactly the way `:wat::core::reduce` above normalizes to `reduce-stream` for
;; its Stream-input clause (the difference: `reduce`'s other 3 clauses already have a
;; state-threading primitive, `foldl`, to delegate to directly; these 12 forms have no such
;; primitive, so `seqable->stream` is the one-time normalization step that lets every clause
;; share the SAME Stream-only walker instead of re-deriving it per container type).

;; seqable->stream — private plumbing: realize any seqable (Vector/List/PersistentVector/Stream/
;; bare-PersistentVector) as an equivalent `Stream<T>`, identity walk (no predicate — mirrors
;; `filter`'s per-type clause shape with `pred` fixed to "always true"). Used by every stateful
;; form below to collapse 5 container types down to 1 before threading state.
(:wat::core::defclause :wat::core::seqable->stream
  ([coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::seqable->stream (:wat::core::rest coll))))))
  ([coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::seqable->stream (:wat::core::rest coll))))))
  ([coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::seqable->stream (:wat::core::rest coll))))))
  ([coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    coll)
  ([coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::seqable->stream (:wat::core::rest coll)))))))

;; ─── remove — filter's negation (keep elements where `pred` is FALSE) ─────────────────────────
(:wat::core::defclause :wat::core::remove
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::remove pred (:wat::core::rest coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::remove pred (:wat::core::rest coll)))))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::remove pred (:wat::core::rest coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::remove pred (:wat::core::rest coll)))))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::remove pred (:wat::core::rest coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::remove pred (:wat::core::rest coll)))))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::remove pred (:wat::core::rest coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::remove pred (:wat::core::rest coll)))))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::remove pred (:wat::core::rest coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::remove pred (:wat::core::rest coll))))))))

;; ─── take-while — cons while `pred` holds; stop (never realize past it) at the first false ────
(:wat::core::defclause :wat::core::take-while
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-while pred (:wat::core::rest coll)))
          (:wat::stream::empty)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-while pred (:wat::core::rest coll)))
          (:wat::stream::empty)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-while pred (:wat::core::rest coll)))
          (:wat::stream::empty)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-while pred (:wat::core::rest coll)))
          (:wat::stream::empty)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-while pred (:wat::core::rest coll)))
          (:wat::stream::empty))))))

;; ─── drop-while — skip while `pred` holds; once it turns false, emit the remainder unchanged ──
;; (the terminal branch delegates to `seqable->stream` — "reuse the filter-clause seqable
;; handling" per the DESIGN: the remaining `coll` is still whatever concrete container this
;; clause's input was, and `seqable->stream` is exactly the shared normalizer for that).
(:wat::core::defclause :wat::core::drop-while
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::drop-while pred (:wat::core::rest coll))
          (:wat::core::seqable->stream coll)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::drop-while pred (:wat::core::rest coll))
          (:wat::core::seqable->stream coll)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::drop-while pred (:wat::core::rest coll))
          (:wat::core::seqable->stream coll)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::drop-while pred (:wat::core::rest coll))
          (:wat::core::seqable->stream coll)))))
  ([pred <- :wat::core::Fn(T)->wat::core::bool
    coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::core::if (pred (:wat::core::first coll))
          (:wat::core::drop-while pred (:wat::core::rest coll))
          (:wat::core::seqable->stream coll))))))

;; ─── take-nth — every nth element (indices 0, n, 2n, ...) ─────────────────────────────────────
;; No stateful helper needed: `:wat::core::drop` (an existing HAVE-listed lazy intrinsic, receiver-
;; first `(drop coll n)` -> Stream<T> over Vector/List/PersistentVector/Stream alike) IS the
;; "skip n and keep going lazily" primitive clojure's own `take-nth` composes over; the recursive
;; call's second arg always comes back as a `Stream<T>`, landing subsequent recursion in the
;; Stream clause below.
(:wat::core::defclause :wat::core::take-nth
  ([n <- :wat::core::i64 coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-nth n (:wat::core::drop coll n))))))
  ([n <- :wat::core::i64 coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-nth n (:wat::core::drop coll n))))))
  ([n <- :wat::core::i64 coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-nth n (:wat::core::drop coll n))))))
  ([n <- :wat::core::i64 coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-nth n (:wat::core::drop coll n))))))
  ([n <- :wat::core::i64 coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll) (:wat::core::take-nth n (:wat::core::drop coll n)))))))

;; ─── interpose — `sep` between every pair of adjacent elements ────────────────────────────────
;; Each clause normalizes to a Stream once (`seqable->stream`), emits the first element bare, then
;; delegates the rest to `interpose-stream` — a single Stream-only helper carrying no extra state
;; (the "prepend sep" split noted in the DESIGN is structural: `interpose-stream` always conses
;; `sep` then the next element, so no boolean flag is needed — only the FIRST element skips it,
;; and that is handled once, here, before delegating).
(:wat::core::defn :wat::core::interpose-stream<T>
  [sep <- :T s <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
  (:wat::stream::lazy
    (:wat::core::if (:wat::core::empty? s)
      (:wat::stream::empty)
      (:wat::stream::cons sep
        (:wat::stream::cons (:wat::core::first s) (:wat::core::interpose-stream sep (:wat::core::rest s)))))))

(:wat::core::defclause :wat::core::interpose
  ([sep <- :T coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll)
          (:wat::core::interpose-stream sep (:wat::core::seqable->stream (:wat::core::rest coll)))))))
  ([sep <- :T coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll)
          (:wat::core::interpose-stream sep (:wat::core::seqable->stream (:wat::core::rest coll)))))))
  ([sep <- :T coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll)
          (:wat::core::interpose-stream sep (:wat::core::seqable->stream (:wat::core::rest coll)))))))
  ([sep <- :T coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll)
          (:wat::core::interpose-stream sep (:wat::core::seqable->stream (:wat::core::rest coll)))))))
  ([sep <- :T coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::stream::lazy
      (:wat::core::if (:wat::core::empty? coll)
        (:wat::stream::empty)
        (:wat::stream::cons (:wat::core::first coll)
          (:wat::core::interpose-stream sep (:wat::core::seqable->stream (:wat::core::rest coll))))))))

;; ─── keep — DIALECT (pinned): `f : Fn(T)->Option<U>`; keep the `Some`s, drop the `None`s ───────
;; (wat's Option-drop IS clojure's nil-drop — the honest dialect form, `VIRTVTE PARES`.)
(:wat::core::defn :wat::core::keep-stream<T,U>
  [f <- :wat::core::Fn(T)->wat::core::Option<U> s <- :wat::stream::Stream<T>] -> :wat::stream::Stream<U>
  (:wat::stream::lazy
    (:wat::core::if (:wat::core::empty? s)
      (:wat::stream::empty)
      (:wat::core::match (f (:wat::core::first s))  
        ((:wat::core::Some v) (:wat::stream::cons v (:wat::core::keep-stream f (:wat::core::rest s))))
        (:wat::core::None (:wat::core::keep-stream f (:wat::core::rest s)))))))

(:wat::core::defclause :wat::core::keep
  ([f <- :wat::core::Fn(T)->wat::core::Option<U>
    coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-stream f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(T)->wat::core::Option<U>
    coll <- :wat::core::List<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-stream f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(T)->wat::core::Option<U>
    coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-stream f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(T)->wat::core::Option<U>
    coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-stream f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(T)->wat::core::Option<U>
    coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<U>
    (:wat::core::keep-stream f (:wat::core::seqable->stream coll))))

;; ─── keep-indexed — as `keep`, `f : Fn(i64,T)->Option<U>`, helper carries `idx` ────────────────
(:wat::core::defn :wat::core::keep-indexed-stream<T,U>
  [idx <- :wat::core::i64
   f   <- :wat::core::Fn(wat::core::i64,T)->wat::core::Option<U>
   s   <- :wat::stream::Stream<T>] -> :wat::stream::Stream<U>
  (:wat::stream::lazy
    (:wat::core::if (:wat::core::empty? s)
      (:wat::stream::empty)
      (:wat::core::match (f idx (:wat::core::first s))  
        ((:wat::core::Some v)
         (:wat::stream::cons v (:wat::core::keep-indexed-stream (:wat::core::+ idx 1) f (:wat::core::rest s))))
        (:wat::core::None
         (:wat::core::keep-indexed-stream (:wat::core::+ idx 1) f (:wat::core::rest s)))))))

(:wat::core::defclause :wat::core::keep-indexed
  ([f <- :wat::core::Fn(wat::core::i64,T)->wat::core::Option<U>
    coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->wat::core::Option<U>
    coll <- :wat::core::List<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->wat::core::Option<U>
    coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->wat::core::Option<U>
    coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<U>
    (:wat::core::keep-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->wat::core::Option<U>
    coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<U>
    (:wat::core::keep-indexed-stream 0 f (:wat::core::seqable->stream coll))))

;; ─── map-indexed — `f : Fn(i64,T)->U`, helper carries `idx` ────────────────────────────────────
(:wat::core::defn :wat::core::map-indexed-stream<T,U>
  [idx <- :wat::core::i64
   f   <- :wat::core::Fn(wat::core::i64,T)->U
   s   <- :wat::stream::Stream<T>] -> :wat::stream::Stream<U>
  (:wat::stream::lazy
    (:wat::core::if (:wat::core::empty? s)
      (:wat::stream::empty)
      (:wat::stream::cons (f idx (:wat::core::first s))
        (:wat::core::map-indexed-stream (:wat::core::+ idx 1) f (:wat::core::rest s))))))

(:wat::core::defclause :wat::core::map-indexed
  ([f <- :wat::core::Fn(wat::core::i64,T)->U
    coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<U>
    (:wat::core::map-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->U
    coll <- :wat::core::List<T>] -> :wat::stream::Stream<U>
    (:wat::core::map-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->U
    coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<U>
    (:wat::core::map-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->U
    coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<U>
    (:wat::core::map-indexed-stream 0 f (:wat::core::seqable->stream coll)))
  ([f <- :wat::core::Fn(wat::core::i64,T)->U
    coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<U>
    (:wat::core::map-indexed-stream 0 f (:wat::core::seqable->stream coll))))

;; ─── dedupe — drop CONSECUTIVE duplicates; helper carries `prev : Option<T>` ───────────────────
(:wat::core::defn :wat::core::dedupe-stream<T>
  [prev <- :wat::core::Option<T> s <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
  (:wat::stream::lazy
    (:wat::core::if (:wat::core::empty? s)
      (:wat::stream::empty)
      (:wat::core::let [x (:wat::core::first s)]
        (:wat::core::match prev  
          (:wat::core::None
           (:wat::stream::cons x (:wat::core::dedupe-stream (:wat::core::Some x) (:wat::core::rest s))))
          ((:wat::core::Some p)
           (:wat::core::if (:wat::core::= p x)
             (:wat::core::dedupe-stream (:wat::core::Some x) (:wat::core::rest s))
             (:wat::stream::cons x (:wat::core::dedupe-stream (:wat::core::Some x) (:wat::core::rest s))))))))))

(:wat::core::defclause :wat::core::dedupe
  ([coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::core::dedupe-stream :wat::core::None (:wat::core::seqable->stream coll)))
  ([coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::core::dedupe-stream :wat::core::None (:wat::core::seqable->stream coll)))
  ([coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::core::dedupe-stream :wat::core::None (:wat::core::seqable->stream coll)))
  ([coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::core::dedupe-stream :wat::core::None (:wat::core::seqable->stream coll)))
  ([coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::core::dedupe-stream :wat::core::None (:wat::core::seqable->stream coll))))

;; ─── distinct — drop ALL duplicates (keep first); helper carries `seen : HashSet<T>` ───────────
;; STOP-2 check: `(:wat::core::HashSet :T)` — an EMPTY HashSet seeded from the enclosing generic
;; defn's OWN rigid type parameter `T` — is exactly the same "declared type params are rigid
;; Path(':T') inside the body" mechanism `check.rs:3119-3122` already uses for every other
;; generic defn/defclause in this file; it type-checks (confirmed by compiling this addition).
(:wat::core::defn :wat::core::distinct-stream<T>
  [seen <- :wat::core::HashSet<T> s <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
  (:wat::stream::lazy
    (:wat::core::if (:wat::core::empty? s)
      (:wat::stream::empty)
      (:wat::core::let [x (:wat::core::first s)]
        (:wat::core::if (:wat::core::contains? seen x)
          (:wat::core::distinct-stream seen (:wat::core::rest s))
          (:wat::stream::cons x (:wat::core::distinct-stream (:wat::core::conj seen x) (:wat::core::rest s))))))))

(:wat::core::defclause :wat::core::distinct
  ([coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::core::distinct-stream (:wat::core::HashSet :T) (:wat::core::seqable->stream coll)))
  ([coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::core::distinct-stream (:wat::core::HashSet :T) (:wat::core::seqable->stream coll)))
  ([coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::core::distinct-stream (:wat::core::HashSet :T) (:wat::core::seqable->stream coll)))
  ([coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::core::distinct-stream (:wat::core::HashSet :T) (:wat::core::seqable->stream coll)))
  ([coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::core::distinct-stream (:wat::core::HashSet :T) (:wat::core::seqable->stream coll))))

;; ─── reductions — emit `init`, then each successive accumulation ───────────────────────────────
;; No separate `-stream` helper: `reductions`'s own 3-arity clauses ARE the stateful walker (the
;; recursive call threads the running accumulation through `init`'s slot directly), exactly
;; mirroring `filter`'s direct self-recursion shape — the DESIGN's pseudocode already IS the
;; state-threading step, so no extra `defn` is needed here (unlike map-indexed/keep/dedupe/
;; distinct, `reductions` has no independent piece of state beyond the arg it already recurses on).
(:wat::core::defclause :wat::core::reductions
  ;; 3-arity: explicit init.
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<U>
    (:wat::stream::lazy
      (:wat::stream::cons init
        (:wat::core::if (:wat::core::empty? coll)
          (:wat::stream::empty)
          (:wat::core::reductions f (f init (:wat::core::first coll)) (:wat::core::rest coll))))))
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::core::List<T>] -> :wat::stream::Stream<U>
    (:wat::stream::lazy
      (:wat::stream::cons init
        (:wat::core::if (:wat::core::empty? coll)
          (:wat::stream::empty)
          (:wat::core::reductions f (f init (:wat::core::first coll)) (:wat::core::rest coll))))))
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<U>
    (:wat::stream::lazy
      (:wat::stream::cons init
        (:wat::core::if (:wat::core::empty? coll)
          (:wat::stream::empty)
          (:wat::core::reductions f (f init (:wat::core::first coll)) (:wat::core::rest coll))))))
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<U>
    (:wat::stream::lazy
      (:wat::stream::cons init
        (:wat::core::if (:wat::core::empty? coll)
          (:wat::stream::empty)
          (:wat::core::reductions f (f init (:wat::core::first coll)) (:wat::core::rest coll))))))
  ([f <- :wat::core::Fn(U,T)->U init <- :U coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<U>
    (:wat::stream::lazy
      (:wat::stream::cons init
        (:wat::core::if (:wat::core::empty? coll)
          (:wat::stream::empty)
          (:wat::core::reductions f (f init (:wat::core::first coll)) (:wat::core::rest coll))))))
  ;; 2-arity: no init — seeds from `(first coll)`, mirroring `reduce`'s 2-arity: an empty `coll`
  ;; raises via `first`'s out-of-range failure rather than a silent 0-arity dispatch.
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::core::Vector<T>] -> :wat::stream::Stream<T>
    (:wat::core::reductions f (:wat::core::first coll) (:wat::core::rest coll)))
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::core::List<T>] -> :wat::stream::Stream<T>
    (:wat::core::reductions f (:wat::core::first coll) (:wat::core::rest coll)))
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::core::PersistentVector<T>] -> :wat::stream::Stream<T>
    (:wat::core::reductions f (:wat::core::first coll) (:wat::core::rest coll)))
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
    (:wat::core::reductions f (:wat::core::first coll) (:wat::core::rest coll)))
  ([f <- :wat::core::Fn(T,T)->T coll <- :wat::core::PersistentVector] -> :wat::stream::Stream<T>
    (:wat::core::reductions f (:wat::core::first coll) (:wat::core::rest coll))))

;; ─── mapcat — STOP-1 (NOT built) ────────────────────────────────────────────────────────────────
;; `(mapcat f coll)` needs its concatenation step to be LAZY over `Stream` (never force the
;; SECOND collection until the first is exhausted — the same property `filter`/`take-while` rely
;; on). Grounded: `:wat::core::concat` is a `defalias` straight to `:wat::core::Vector/concat`
;; (`wat/core.wat:44`), and `Vector/concat`'s registered scheme is `∀T. Vec<T> × Vec<T> -> Vec<T>`
;; (`src/check.rs:19783-19792`) — a Vector-only, fully EAGER binary op with no `Stream` clause at
;; all (confirmed: no other `wat::core::concat`/`Vector/concat` registration anywhere in
;; `src/check.rs`). There is no lazy-over-Stream `concat` to compose `mapcat` over. Per the STOP
;; doctrine this is surfaced, not hand-rolled (a hand-rolled lazy concat would itself need a new
;; Stream-native primitive — out of this pure-wat, no-new-primitives strike). `mapcat` is NOT
;; shipped by this strike; a lazy `:wat::core::concat` (or a dedicated lazy-concat primitive) is a
;; prerequisite, named here for whichever strike picks it up next.
