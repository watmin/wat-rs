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
;;   - `:wat::core::filter` — was the one lazy HOF shipped as wat-over-primitives here (Decision
;;     B's original preference), on the reasoning that (unlike map/take/drop) no stdlib macro
;;     calls it at macro-expansion time, so self-hosting it was bootstrap-safe. That reasoning
;;     was true but incomplete: it never weighed that the ONLY traversal a wat `defclause` could
;;     express (per-container `(rest coll)` stepping) is O(n^2) on every eager container, because
;;     `rest` REBUILDS the whole remaining container each step. Arc-278 DESIGN-STONE
;;     seq-traversal-one-door Strike 2a moved `filter` NATIVE (`eval_filter`,
;;     `src/collection/transform.rs`) — see below, where its old five clauses used to live.
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

;; ═══ 118.B1 — `Seqable<T>`: the type the twins were a workaround for ═══════════════════════════
;;
;; Clojure has exactly one `filter`, one `map`, one `reduce`, because it calls `seq` — the universal
;; coercion every collection implements — and walks the result. wat could not write that, because
;; "any seqable" had no name in the surface language: the concept lived ONLY inside the Rust checker
;; as `extract_lazyable_elem` (`src/collection/infer.rs:665`), a hardcoded match on four heads. So a
;; wat verb accepting several containers had one option — a `defclause` with one arm per concrete
;; container — and since those arms would each duplicate the body, the corpus grew the `<verb>-stream`
;; TWIN. Builder, 2026-07-31: *"The twins are a workaround for the missing type, not a pattern."*
;;
;; This IS that type. It is `ISeq`.
;;
;; The four heads below are exactly `extract_lazyable_elem`'s hardcoded set — deliberately, because
;; B2 deletes that function and this becomes the single definition of what a sequence verb accepts.
;;
;; ⚠ HISTORY, so nobody re-derives it: arc 278 ruled this route a flat NO on Simple, over three
;; blockers (no `:nature` admits a builtin · nothing satisfies a surface · no ad-hoc unions). ALL
;; THREE are dead — refuted or dissolved by stone 118.3-B (`a15f4ea9`), and annotated per-claim in
;; `docs/arc/2026/04/109-kill-std/NOTE-seqable-has-no-name-in-wat.md`. The route was re-posed and
;; ruled in `118-lazy-seqs-vs-threaded-streams/DECISIONS-118.B-four-questioned.md`.
;;
;; ★ `seq` returns a `Stream<T>` and stays LAZY. It is NOT `as-vec`: a materializing coercion would
;; invert this arc's entire purpose. The exploratory probe used `as-vec` only to prove satisfaction.
;;
;; ADDITIVE AS OF B1: nothing below consumes it yet, `extract_lazyable_elem` is untouched, and no
;; twin has died. B2 collapses each verb to ONE clause over `Seqable<T>` walking with
;; `:wat::stream::next`, and deletes the twins and `seqable->stream` in the same motion — a name
;; dies in the stone that removes its last caller.
(:wat::core::defsurface :wat::core::Seqable<T> :nature :wat::core::Struct
  :features [(seq [self <- :wat::core::Seqable<T>] -> :wat::stream::Stream<T>)])

;; The four impls. Each delegates to the native normaliser, which already steps its source BY
;; POSITION (O(n) total) rather than by repeated `rest` (which REBUILDS an eager container per step,
;; O(n^2) — the arc-278 Strike-1 fix). Stream's arm is the identity case and stays lazy.
(:wat::core::extend-type :wat::core::Vector :wat::core::Seqable<T>
  (seq [self] -> :wat::stream::Stream<T> (:wat::core::seqable->stream self)))

(:wat::core::extend-type :wat::core::PersistentVector :wat::core::Seqable<T>
  (seq [self] -> :wat::stream::Stream<T> (:wat::core::seqable->stream self)))

(:wat::core::extend-type :wat::core::List :wat::core::Seqable<T>
  (seq [self] -> :wat::stream::Stream<T> (:wat::core::seqable->stream self)))

(:wat::core::extend-type :wat::stream::Stream :wat::core::Seqable<T>
  (seq [self] -> :wat::stream::Stream<T> (:wat::core::seqable->stream self)))

;; ─── filter — NATIVE now (Arc-278 DESIGN-STONE seq-traversal-one-door, Strike 2a) ─────────────
;;
;; `:wat::core::filter` used to live here as five wat `defclause` arms (Vector<T> / List<T> /
;; PersistentVector<T> / Stream<T> / bare PersistentVector), each stepping its eager source by
;; repeated `(rest coll)` — O(n) per step, O(n^2) per walk, because `rest` REBUILDS the whole
;; remaining container on every eager container. It is a Rust intrinsic now (`eval_filter`,
;; `src/collection/transform.rs`), one body for any seqable, composing through the native
;; `seqable->stream` normaliser (Strike 1) instead of hand-rolling a per-container walk — the
;; same shape `map`/`take`/`drop` already have. See the DESIGN-STONE's "⛔ THE TWIN ROUTE IS
;; DEAD" ruling for why this went native rather than minting a `filter-stream` twin.
;;
;; `filterv` (below) is unchanged: `(into [] (filter pred coll))` still works, unaware its
;; ingredient verb's engine flipped underneath it.

;; ─── the eager materializers ─────────────────────────────────────────────────────────────────

;; stream->vec — internal helper: drains a Stream into a Vector, seeded by `acc` (so `into` can
;; append onto an existing Vector, not just build from empty). Tail-recursive (TCO trampoline
;; keeps this O(1) Rust-stack regardless of stream length).
;; ★ Arc 278 — THIS WAS QUADRATIC, and it is the language's standard materializer.
;; The old body recursed `(conj acc (first s))`, one `conj` per element — and Vector's conj
;; (`vector_conj_inner`, src/collection/eval.rs) does `(**xs).clone()`, a FULL copy of the
;; accumulator, every time. So `(into [] (map f coll))` was O(n^2).
;; MEASURED (wat-scripts/scratch-pad/probe-into-is-quadratic.wat), n=40,000, same output, lengths
;; asserted equal: per-element Vec conj 8,112 ms · the identical drain into an rpds accumulator
;; 113 ms (LINEAR) · one native build 0.8 ms. 8x n gave 114x time; rpds gave 7.8x.
;; So: drain into a PersistentVector (structural sharing, linear), then materialize ONCE via the
;; native `Vector/extend`. Same result, same order, no per-element copy.
(:wat::core::defn :wat::core::stream->vec<T>
  [acc <- :wat::core::Vector<T> s <- :wat::stream::Stream<T>] -> :wat::core::Vector<T>
  (:wat::core::Vector/extend
    acc
    (:wat::core::stream->pvec (:wat::core::PersistentVector) s)))

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
;; same-kind eager container (delegates to `concat`), a Vector (PersistentVector receiver only —
;; delegates to the native `PersistentVector/concat`, DESIGN-STONE-into-pv-from-vector.md), or a
;; Stream (delegates to `stream->vec`/`stream->pvec`, seeded by `to` — the general "append a
;; realized pipeline onto an accumulator" shape).
(:wat::core::defclause :wat::core::into
  ([to <- :wat::core::Vector<T> from <- :wat::core::Vector<T>] -> :wat::core::Vector<T>
    (:wat::core::concat to from))
  ([to <- :wat::core::Vector<T> from <- :wat::stream::Stream<T>] -> :wat::core::Vector<T>
    (:wat::core::stream->vec to from))
  ([to <- :wat::core::PersistentVector<T> from <- :wat::stream::Stream<T>] -> :wat::core::PersistentVector<T>
    (:wat::core::stream->pvec to from))
  ;; DESIGN-STONE-into-pv-from-vector.md — the missing fourth clause: materialize a Vector
  ;; into a PersistentVector in ONE native call, retiring the nine grid axes' hand-rolled
  ;; `foldl`+`conj` bridge (N interpreted closure invocations -> one native concat).
  ([to <- :wat::core::PersistentVector<T> from <- :wat::core::Vector<T>] -> :wat::core::PersistentVector<T>
    (:wat::core::PersistentVector/concat to from))
  ;; Arc 278 — the MIRROR of the clause above, and the one `stream->vec` now needs. Its absence
  ;; was flagged as owed the moment the (PV,Vector) clause landed, and tripped a probe an hour
  ;; later: `query-by-type-string` returns a PersistentVector, so materialising one into a Vector
  ;; had no clause at all. Native one-shot, no per-element conj.
  ([to <- :wat::core::Vector<T> from <- :wat::core::PersistentVector<T>] -> :wat::core::Vector<T>
    (:wat::core::Vector/extend to from)))

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

;; seqable->stream — private plumbing: realize any seqable (Vector/List/PersistentVector/Stream)
;; as an equivalent `Stream<T>`. Used by every stateful form below to collapse the container
;; types down to 1 before threading state.
;;
;; Arc-278 DESIGN-STONE seq-traversal-one-door, Strike 1 — NATIVE now (src/collection/
;; transform.rs's `eval_seqable_to_stream`, dispatched in src/runtime.rs). The wat form this
;; replaced walked its source by repeated `(rest coll)`, and `rest` on any eager container
;; REBUILDS the whole remaining container — O(n^2) over the walk. The native form steps its
;; source BY POSITION instead (List is snapshotted once, O(n) total, then stepped the same
;; way — it has no indexed access), materialising nothing per element. Every clause below is
;; unchanged; they go linear by delegation alone.

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
