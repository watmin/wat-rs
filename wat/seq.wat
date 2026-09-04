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
;;   - `:wat::core::reduce` — a `defalias` for `:wat::core::foldl` (arc 255 Stone 1c-f,
;;     2026-09-03: `reduce` was `foldl`'s body wearing a second name; now it IS `foldl`, 3-arity
;;     only). No dedicated `Stream` walker — `foldl` itself walks any `Seqable`, `Stream`
;;     included.
;;   - `:wat::core::count` — a defalias over the KEPT `length` primitive (clojure surface name).
;;
;; STOP surfaced (not built): `reduced` / `reduced?` (clojure's early-exit `reduce` marker).
;; wat's type universe is CLOSED (`:Any` is banned, `src/types.rs` line ~78) — there is no way
;; to type a reducing function's return as "T, or a (Reduced :- [T]) early-exit wrapper" without
;; reopening that banned escape hatch, and a control-flow-signal mechanism (mirroring
;; `Result/try`'s `EvalSignal::TryPropagate`) would need NEW Rust plumbing (a new signal variant
;; + checker special-casing) beyond a wat-over-primitives change. Per the STOP doctrine this is
;; surfaced, not guessed/hacked around. `:wat::core::reduce` here has NO early-exit — it always
;; walks the whole input (exactly like the `foldl`/`:wat::seq::reduce` it replaces; no
;; regression, just not a new capability).

;; ═══ 118.B1 — `(Seqable :- [T])`: the type the twins were a workaround for ═══════════════════
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
;; ★ `seq` returns a `(Stream :- [T])` and stays LAZY. It is NOT `as-vec`: a materializing coercion would
;; invert this arc's entire purpose. The exploratory probe used `as-vec` only to prove satisfaction.
;;
;; ADDITIVE AS OF B1: nothing below consumes it yet, `extract_lazyable_elem` is untouched, and no
;; twin has died. B2 collapses each verb to ONE clause over `(Seqable :- [T])` walking with
;; `:wat::stream::next`, and deletes the twins and `seqable->stream` in the same motion — a name
;; dies in the stone that removes its last caller.
(:wat::core::defsurface :wat::core::Seqable :- [T] :nature :wat::core::Struct
  :features [(seq [self <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T]))])

;; The four impls. Each delegates to the native normaliser, which already steps its source BY
;; POSITION (O(n) total) rather than by repeated `rest` (which REBUILDS an eager container per step,
;; O(n^2) — the arc-278 Strike-1 fix). Stream's arm is the identity case and stays lazy.
(:wat::core::extend-type :wat::core::Vector (:wat::core::Seqable :- [T])
  (seq [self] -> (:wat::stream::Stream :- [T]) (:wat::core::seqable->stream self)))

(:wat::core::extend-type :wat::core::PersistentVector (:wat::core::Seqable :- [T])
  (seq [self] -> (:wat::stream::Stream :- [T]) (:wat::core::seqable->stream self)))

(:wat::core::extend-type :wat::core::List (:wat::core::Seqable :- [T])
  (seq [self] -> (:wat::stream::Stream :- [T]) (:wat::core::seqable->stream self)))

(:wat::core::extend-type :wat::stream::Stream (:wat::core::Seqable :- [T])
  (seq [self] -> (:wat::stream::Stream :- [T]) (:wat::core::seqable->stream self)))

;; ─── filter — NATIVE now (Arc-278 DESIGN-STONE seq-traversal-one-door, Strike 2a) ─────────────
;;
;; `:wat::core::filter` used to live here as five wat `defclause` arms ((Vector :- [T]) / (List :- [T]) /
;; (PersistentVector :- [T]) / (Stream :- [T]) / bare PersistentVector), each stepping its eager source by
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

;; stream->vec-spec — the wat reference engine (the SPEC / differential oracle) for
;; `:wat::core::stream->vec` (stone 118.B5: promoted to a native Rust intrinsic — see
;; `src/collection/transform.rs::eval_stream_to_vec`). Drains a Stream into a Vector, seeded by
;; `acc` (so `into` can append onto an existing Vector, not just build from empty).
;; Tail-recursive (TCO trampoline keeps this O(1) Rust-stack regardless of stream length).
;; ★ Arc 278 — THIS WAS QUADRATIC, and it is the language's standard materializer.
;; The old body recursed `(conj acc (first s))`, one `conj` per element — and Vector's conj
;; (`vector_conj_inner`, src/collection/eval.rs) does `(**xs).clone()`, a FULL copy of the
;; accumulator, every time. So `(into [] (map f coll))` was O(n^2).
;; MEASURED (wat-scripts/scratch-pad/probe-into-is-quadratic.wat), n=40,000, same output, lengths
;; asserted equal: per-element Vec conj 8,112 ms · the identical drain into an rpds accumulator
;; 113 ms (LINEAR) · one native build 0.8 ms. 8x n gave 114x time; rpds gave 7.8x.
;; So: drain into a PersistentVector (structural sharing, linear), then materialize ONCE via the
;; native `Vector/extend`. Same result, same order, no per-element copy.
;; ⚠ `stream->vec-spec` MUST NEVER delegate to `stream->vec` (its native subject) — a spec that
;; calls its subject proves nothing. So it drains through its OWN sibling oracle,
;; `stream->pvec-spec`, below — NOT the (now-native) `stream->pvec`.
;; `[[feedback_a_green_test_can_prove_nothing]]` / `[[feedback_an_oracle_must_be_written_in_the_other_language]]`
(:wat::core::defn :wat::core::stream->vec-spec :- [T]
  [acc <- (:wat::core::Vector :- [T]) s <- (:wat::stream::Stream :- [T])] -> (:wat::core::Vector :- [T])
  (:wat::vec::extend
    acc
    (:wat::core::stream->pvec-spec (:wat::core::PersistentVector) s)))

;; mapv / filterv — the eager forms: force `map`/`filter`'s lazy Stream result to a Vector in
;; one step via `(into [] ...)` (clojure's own materializer idiom — no new name). Two clauses —
;; Vector input (the direct case) AND Stream input (composing after another lazy stage, e.g.
;; `(filterv pred (map f xs))` — `map` stays lazy, `filterv` is the pipeline's eager exit).
;; Extend with more clauses if a call site needs List/PersistentVector input directly (ride the
;; red — 118.2b+).
;; mapv is native (`eval_mapv`). Eager walk of Vector / PersistentVector / List;
;; Stream input maps then drains. Wat clauses retired so a PersistentVector of
;; query answers type-checks (`DESIGN-STONE-mapv-eager`).

(:wat::core::defclause :wat::core::filterv
  ([pred <- [T :-> :wat::core::bool] coll <- (:wat::core::Vector :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::into [] (:wat::core::filter pred coll)))
  ([pred <- [T :-> :wat::core::bool] coll <- (:wat::stream::Stream :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::into [] (:wat::core::filter pred coll))))

;; stream->pvec-spec — the wat reference engine (the SPEC / differential oracle) for
;; `:wat::core::stream->pvec` (stone 118.B5: promoted to a native Rust intrinsic — see
;; `src/collection/transform.rs::eval_stream_to_pvec`). The PersistentVector twin of
;; `stream->vec-spec` (118.2b cascade: rete.wat's (PersistentVector :- [Rule])/
;; (PersistentVector :- [DerivationStep]) fields need a Stream materialized into a PersistentVector,
;; not a Vector).
;; 118.B2 — migrated from the three-call (`empty?`/`first`/`rest`) walk onto the single-force
;; `:wat::stream::next` pull primitive. ★ THE DRAIN — the recursive call MUST stay in the
;; `match`'s `Item` arm tail position (proven: `probe-118B-match-tco-drain.wat`, with a
;; non-tail-position sibling control that SIGSEGVs at the same depth). Nesting it inside a
;; `cons`/argument here would silently make the language's one materializer O(n)-stack.
;; ⚠ THE ORACLE STAYS WAT — `[[feedback_an_oracle_must_be_written_in_the_other_language]]`.
;; This is `stream->vec-spec`'s own drain (see its ⚠ note above): if this body ever called the
;; native `stream->pvec` instead of recursing on itself, the differential against that same
;; native would become a tautology. `wat/rete.wat:1508`'s `insert-all-spec` is the recorded
;; shape — a composed oracle calls its OWN sibling `-spec`, never the subject it is honesty-
;; checking.
(:wat::core::defn :wat::core::stream->pvec-spec :- [T]
  [acc <- (:wat::core::PersistentVector :- [T]) s <- (:wat::stream::Stream :- [T])] -> (:wat::core::PersistentVector :- [T])
  (:wat::core::match (:wat::stream::next s)
    ((:wat::stream::NextOutcome::Item value rest)
      (:wat::core::stream->pvec-spec (:wat::vector::conj acc value) rest))
    (:wat::stream::NextOutcome::Exhausted acc)))

;; into — clojure's `(into to from)`: append every element of `from` onto `to`. `to` determines
;; the output container kind (Vector or PersistentVector, both in scope); `from` may be a
;; same-kind eager container (delegates to `concat`), a Vector (PersistentVector receiver only —
;; delegates to the native `PersistentVector/concat`, DESIGN-STONE-into-pv-from-vector.md), or a
;; Stream (delegates to `stream->vec`/`stream->pvec`, seeded by `to` — the general "append a
;; realized pipeline onto an accumulator" shape).
(:wat::core::defclause :wat::core::into
  ([to <- (:wat::core::Vector :- [T]) from <- (:wat::core::Vector :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::concat to from))
  ([to <- (:wat::core::Vector :- [T]) from <- (:wat::stream::Stream :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::stream->vec to from))
  ([to <- (:wat::core::PersistentVector :- [T]) from <- (:wat::stream::Stream :- [T])] -> (:wat::core::PersistentVector :- [T])
    (:wat::core::stream->pvec to from))
  ;; DESIGN-STONE-into-pv-from-vector.md — the missing fourth clause: materialize a Vector
  ;; into a PersistentVector in ONE native call, retiring the nine grid axes' hand-rolled
  ;; `foldl`+`conj` bridge (N interpreted closure invocations -> one native concat).
  ([to <- (:wat::core::PersistentVector :- [T]) from <- (:wat::core::Vector :- [T])] -> (:wat::core::PersistentVector :- [T])
    (:wat::vector::concat to from))
  ;; Arc 278 — the MIRROR of the clause above, and the one `stream->vec` now needs. Its absence
  ;; was flagged as owed the moment the (PV,Vector) clause landed, and tripped a probe an hour
  ;; later: `query-by-type-string` returns a PersistentVector, so materialising one into a Vector
  ;; had no clause at all. Native one-shot, no per-element conj.
  ([to <- (:wat::core::Vector :- [T]) from <- (:wat::core::PersistentVector :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::vec::extend to from)))

;; doall / dorun — eager forcers (Stream -> Vector / nil). DIALECT NOTE: clojure's `doall`
;; returns the SAME (now-forced) lazy seq, replayable — wat's Stream is single-pass / NEVER
;; memoized (arc 118 R1, NON BIS IN IDEM FLVMEN: "you cannot walk back a stream"), so there is
;; no "same seq, now forced" to hand back. The honest wat-dialect equivalent: fully realize into
;; a Vector (forces every element / side-effect) and return THAT.
;;
;; `dorun` is NOT the same walk with the result discarded — that would still pay O(n) memory to
;; build the Vector it then throws away (118.B8: measured linear, ~50 B/element). `dorun`'s whole
;; contract is "walk for effects, keep nothing", so its body is the drain's own shape
;; (`stream->pvec-spec` above) instead of `doall`'s: a tail-recursive walk over
;; `:wat::stream::next` that retains nothing. ★ THE RECURSIVE CALL MUST STAY IN THE `match`'s
;; `Item` ARM TAIL POSITION (proven: `probe-118B-match-tco-drain.wat`, non-tail sibling control
;; SIGSEGVs at the same depth) — nesting it inside an argument would silently make this O(n)-stack.
;; Forcing still happens (that is what `next` does, and what makes the side effects run); only the
;; retention goes, O(n) live -> O(1) live (measured flat, `probe-118B8-dorun-retention.wat`).
(:wat::core::defn :wat::core::doall :- [T] [coll <- (:wat::stream::Stream :- [T])] -> (:wat::core::Vector :- [T])
  (:wat::core::into [] coll))

(:wat::core::defn :wat::core::dorun :- [T] [coll <- (:wat::stream::Stream :- [T])] -> :wat::core::nil
  (:wat::core::match (:wat::stream::next coll)
    ((:wat::stream::NextOutcome::Item _value rest) (:wat::core::dorun rest))
    (:wat::stream::NextOutcome::Exhausted nil)))

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
  ([f <- [T :-> U] coll <- (:wat::core::Vector :- [T])] -> :wat::core::nil
    (:wat::core::foldl
      (:wat::core::fn [_acc <- :wat::core::nil x <- :T] -> :wat::core::nil (:wat::core::do (f x) nil))
      nil
      coll))
  ([f <- [T :-> U] coll <- (:wat::core::List :- [T])] -> :wat::core::nil
    (:wat::core::foldl
      (:wat::core::fn [_acc <- :wat::core::nil x <- :T] -> :wat::core::nil (:wat::core::do (f x) nil))
      nil
      coll))
  ([f <- [T :-> U] coll <- (:wat::core::PersistentVector :- [T])] -> :wat::core::nil
    (:wat::core::foldl
      (:wat::core::fn [_acc <- :wat::core::nil x <- :T] -> :wat::core::nil (:wat::core::do (f x) nil))
      nil
      coll)))

;; ─── foldl-spec — THE WAT ORACLE for the native `:wat::core::foldl` ───────────────────────────
;;
;; Stone 118.B6. `:wat::core::foldl` is a Rust intrinsic (`eval_vec_foldl`,
;; src/collection/transform.rs). THIS is its SPECIFICATION: the same fold written in wat, as
;; obviously as it can be written — pull one element, apply `f`, recur on the rest. Correct and
;; slow, on purpose.
;;
;; ★ THE RELATIONSHIP, and it is the point of the whole stone. Builder, 2026-08-18: *"we should be
;; striving to build correct-but-slow wat-oracles that are references for wat-native to satisfy
;; fast-and-correct.... we build wat-oracles that guide the rust code... the wat-native using rust
;; provided intrinsics must be faster than wat-oracle."* This is the same shape as
;; `:wat::rete::insert-all-spec` (wat/rete.wat:1508), whose sibling comment states it exactly:
;; *"the native kernel is the fast impl, the spec keeps it honest."*
;;
;; ⚠ SO ITS SLOWNESS IS THE DESIGN, NOT A DEFECT. Measured ~4.6x the native on 200k i64
;; (`wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat`, re-derived 2026-09-03; it
;; read ~5.1x when B6 was decided, and both arms have since got faster). A first cut of B6 read that
;; ratio as an argument AGAINST routing folds through wat and built two RUST implementations
;; instead, calling one an "oracle" — two variants of one thing in one language, neither able to
;; specify the other. The differential is only meaningful because this side is INDEPENDENT.
;;
;; ⚠ AND IT MUST STAY A HAND-WRITTEN FOLD. Do NOT "simplify" it by delegating to
;; `:wat::core::foldl` (or to `reduce`, which delegates there): a spec that calls its subject
;; proves nothing. `[[feedback_a_green_test_can_prove_nothing]]`
;;
;; Its only caller is `wat-tests/core/core-foldl-spec.wat`. Zero production callers is the CORRECT
;; state for a spec — an inventory entry WITH a disposition, not an offender (task #48).
(:wat::core::defn :wat::core::foldl-spec :- [T U]
  [f    <- [U T :-> U]
   init <- :U
   coll <- (:wat::core::Seqable :- [T])] -> :U
  (:wat::core::foldl-spec-walk f init (:wat::core::Seqable/seq coll)))

(:wat::core::defn :wat::core::foldl-spec-walk :- [T U]
  [f <- [U T :-> U] acc <- :U s <- (:wat::stream::Stream :- [T])] -> :U
  (:wat::core::match (:wat::stream::next s)
    ((:wat::stream::NextOutcome::Item value rest)
      (:wat::core::foldl-spec-walk f (f acc value) rest))
    (:wat::stream::NextOutcome::Exhausted acc)))

;; ─── reduce — an alias for foldl (see STOP note above) ──────────────────────────────────────
;;
;; 118.B2 — `reduce-stream` (the Stream-input walk `foldl` cannot do; foldl is Vector/List/
;; PersistentVector-only) was DELETED as a named twin: its walk migrated inline into `reduce`'s
;; own Stream arms, over `:wat::stream::next` — one force per element, tail-recursive.
;; 118.B6 + 118.B7 — `reduce` WAS two clauses, one per arity, both over `(Seqable :- [T])`.
;; ⚠ Both paragraphs are HISTORY, superseded below by Stone 1c-f — `reduce` is no longer a
;; `defclause` with Stream arms of its own; it is an alias, and `foldl` does the Stream walk.
;;
;; It was EIGHT: three eager arms per arity delegating to the native `foldl`, plus a Stream arm per
;; arity that had to walk in wat because `foldl` REFUSED a Stream (`mappable()`'s "later strike.
;; ○ gap"). Three stones removed the three reasons it could not collapse:
;;   118.B2c  a `defclause` arm typed with a SURFACE now dispatches at runtime
;;   118.B2d  a generic satisfier's surface param binds from the receiver
;;   118.B6   the native `foldl` walks any seqable — so there is nothing left to hand-walk
;;
;; ⚠ AND IT COST NOTHING, which was the whole point of doing B6 first. The 3-arity body was
;; `(foldl f init coll)` — the value handed over is still a concrete Vector/List/PersistentVector,
;; so `foldl` takes its DIRECT iterator exactly as before. There is NO `(Seqable/seq coll)`
;; normalisation here, deliberately: that would force every eager reduce onto the lazy path for a
;; Stream it never needed. Measured before/after in
;; `wat-scripts/scratch-pad/bench-118B7-reduce-collapse.wat`.
;;
;; ⛔ `reduce-walk` IS GONE. It existed for exactly one reason — `foldl` could not walk a Stream —
;; and it carried a long comment calling itself "A WORKAROUND FOR A SUBSTRATE GAP, NOT A PATTERN TO
;; COPY" and naming the clause-TCO stone that would free it. Both the gap and the TCO stone are
;; closed, so the workaround dies with them rather than lingering as a name nobody calls.
;; A name dies in the stone that removes its last caller.
;;
;; ✅ Arc 255 Stone 1c-f, 2026-09-03 — `reduce`'s 3-arity arm above was ALREADY `foldl`'s body
;; verbatim: `(foldl f init coll)`, nothing else. That is not a delegation, it is a second name for
;; the same verb — the heresy `[[RULING-the-registry-is-the-sole-authority]]` exists to kill. It is
;; now a genuine `defalias`, not a `defclause` wrapping a call.
;;
;; The 2-arity seed-from-first arm is GONE — it was the only part of `reduce` that was not `foldl`
;; (it seeded from the first element and raised `assertion-failed!` on empty), so it cannot survive
;; becoming an alias. Its one caller — `probe-118B2-rider-verification.wat` — is augmented to call
;; the 3-arity form. `foldl`'s own retained `TypeScheme` (`src/check.rs`, near its registration) is
;; widened `Vector` -> `Seqable` in the same stone, since `defalias` derives its signature from that
;; scheme (direct `foldl` calls go through `infer_foldl` instead and never see it) — without the
;; widening, aliasing loses every non-Vector caller (Stream, PersistentVector).
(:wat::core::defalias :wat::core::reduce :wat::core::foldl)

;; count — the clojure surface name over the KEPT `length` primitive (unchanged: an infinite/
;; lazy Stream still correctly rejects `length`/`count` — see `StreamContainer::measurable`).
(:wat::core::defalias :wat::core::count :wat::core::length)

;; ═══ 118.2-Z strike A — the lazy transformer family ═══════════════════════════════════════════
;;
;; Twelve clojure-core lazy transformers, each a `:wat::core::defclause` mirroring `filter`'s
;; shape above (one clause per seqable — Vector/List/PersistentVector/Stream + bare-
;; PersistentVector — `stream/lazy` + `first`/`rest`/`empty?` + `stream/cons`/`stream/empty`).
;; Forms that carry state across the walk (an index, a seen-set, a running accumulator, the
;; previous element) normalize their input to a genuine `(Stream :- [T])` ONCE (via the private
;; `seqable->stream` helper below) and then delegate to a single Stream-only `<form>-stream`
;; helper `defn` — exactly the way `:wat::core::reduce` above normalizes to `reduce-stream` for
;; its Stream-input clause (the difference: `reduce`'s other 3 clauses already have a
;; state-threading primitive, `foldl`, to delegate to directly; these 12 forms have no such
;; primitive, so `seqable->stream` is the one-time normalization step that lets every clause
;; share the SAME Stream-only walker instead of re-deriving it per container type).
;;
;; 118.B2 — the description above is now historical for SIX of the twelve: `interpose`, `keep`,
;; `keep-indexed`, `map-indexed`, `dedupe`, `distinct` no longer have per-container `defclause`
;; arms or a `<form>-stream` twin — each is ONE `defn` over `(:wat::core::Seqable :- [T])`, walking with
;; `:wat::stream::next` (see each verb's own comment for its specific migration). `remove`,
;; `take-while`, `drop-while`, `take-nth`, `reductions` are UNTOUCHED this stone and still match
;; the description above exactly.

;; seqable->stream — private plumbing: realize any seqable (Vector/List/PersistentVector/Stream)
;; as an equivalent `(Stream :- [T])`. Used by every stateful form below to collapse the container
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
;; 118.B2b — ONE `defn` over `(Seqable :- [T])`, walking with `:wat::stream::next`. The five per-container
;; clauses (bodies byte-identical) are gone; `rest` comes back as a `(Stream :- [T])`, which IS a
;; `(Seqable :- [T])`, so the recursion lands right back here. Stateless, so no `-walk` helper is needed —
;; same shape as `keep` above.
(:wat::core::defn :wat::core::remove :- [T]
  [pred <- [T :-> :wat::core::bool]
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::if (pred value)
          (:wat::core::remove pred rest)
          (:wat::stream::cons value (:wat::core::remove pred rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; ─── take-while — cons while `pred` holds; stop (never realize past it) at the first false ────
;; 118.B2b — ONE `defn` over `(Seqable :- [T])`. ★ THE LAZINESS PROPERTY IS TESTED: the `Exhausted`/false
;; branches return `(stream/empty)` WITHOUT touching `rest`, so the cell after the first false is
;; never realized — `tests/types/probe_arc118_2z_takewhile_lazy.rs` proves it by making that cell
;; divide by zero.
(:wat::core::defn :wat::core::take-while :- [T]
  [pred <- [T :-> :wat::core::bool]
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::if (pred value)
          (:wat::stream::cons value (:wat::core::take-while pred rest))
          (:wat::stream::empty)))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; ─── drop-while — skip while `pred` holds; once it turns false, emit the remainder unchanged ──
;; 118.B2b — ONE `defn` over `(Seqable :- [T])`. The old terminal branch re-normalized the WHOLE `coll`
;; through `seqable->stream` (it still held the un-consumed container). With `next` the head is
;; already in hand, so the remainder is just `(stream/cons value rest)` — one cell, no
;; re-normalization and no second walk of anything.
(:wat::core::defn :wat::core::drop-while :- [T]
  [pred <- [T :-> :wat::core::bool]
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::if (pred value)
          (:wat::core::drop-while pred rest)
          (:wat::stream::cons value rest)))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; ─── take-nth — every nth element (indices 0, n, 2n, ...) ─────────────────────────────────────
;; 118.B2b — ONE `defn` over `(Seqable :- [T])` plus a private `(Stream :- [T])` walker.
;;
;; ⚠ THE DEGENERATE `n` IS LOAD-BEARING AND IT IS MEASURED, NOT ASSUMED.
;; At HEAD, `(take [] 5 (take-nth 0 [1 2 3]))` yields `1,1,1,1,1` — an infinite repeat of the head,
;; which is what clojure's own `take-nth` does. The mechanism: the old recursion dropped `n` from the
;; FULL `coll` (head included) and `:wat::core::drop` CLAMPS a negative `n` to 0
;; (`src/collection/transform.rs:201`), so n=0 re-consumed the same collection forever.
;;
;; The obvious `next` rewrite — emit `value`, recurse on `(drop rest (- n 1))` — SILENTLY changes
;; that to `1,2,3`. Nothing in the corpus calls `take-nth` with n<=0, so a green floor would not
;; catch it. So: REBUILD the consumed head before dropping. `(stream/cons value rest)` is a plain
;; `Cons` — `realize` on it is the identity — so `drop` walks it for free, n=0 drops nothing and
;; hands the same cell back (the repeat, preserved), n>=1 skips `value` plus n-1 from `rest`, and
;; every downstream cell is still forced EXACTLY ONCE. Baseline pinned in
;; `wat-scripts/scratch-pad/probe-118B-six-walkers-baseline.wat`.
(:wat::core::defn :wat::core::take-nth-walk :- [T]
  [n <- :wat::core::i64 s <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next s)
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::stream::cons value
          (:wat::core::take-nth-walk n
            (:wat::core::drop (:wat::stream::cons value rest) n))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

(:wat::core::defn :wat::core::take-nth :- [T]
  [n <- :wat::core::i64 coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::core::take-nth-walk n (:wat::core::Seqable/seq coll)))

;; ─── interpose — `sep` between every pair of adjacent elements ────────────────────────────────
;; 118.B2 — ONE clause over `(Seqable :- [T])`. `interpose-stream` (the twin that carried the "always
;; sep-prefix" recursion) is deleted; there is no local-recursion primitive in wat to smuggle
;; that split into a second, unnamed helper (no letrec — src/runtime.rs:4273), and threading a
;; boolean "have I emitted yet" flag into `interpose`'s own params would be an arity change
;; (STOP-2). Instead: ONE-ELEMENT LOOKAHEAD. Pull `value`/`rest`, then peek `rest` with a second
;; `next` — if it still has an item, emit `value, sep` and recurse on the (unconsumed) `rest`; if
;; `rest` is exhausted, `value` was the LAST element and gets no trailing sep. Peeking costs a
;; second force per element (2 `next` calls/element instead of 1) — still O(n), not a complexity
;; class change, just a constant-factor cost of not having a second named helper to carry the
;; "not the first element" state across the recursion.
(:wat::core::defn :wat::core::interpose-walk :- [T]
  [sep <- :T value <- :T s <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next s)
      ((:wat::stream::NextOutcome::Item next-value next-rest)
        (:wat::stream::cons value
          (:wat::stream::cons sep (:wat::core::interpose-walk sep next-value next-rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::cons value (:wat::stream::empty))))))

(:wat::core::defn :wat::core::interpose :- [T]
  [sep <- :T coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::interpose-walk sep value rest))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; ─── keep — DIALECT (pinned): `f : [T :-> (Option :- [U])]`; keep the `Some`s, drop the `None`s ──
;; (wat's Option-drop IS clojure's nil-drop — the honest dialect form, `VIRTVTE PARES`.)
;; 118.B2 — ONE clause over `(Seqable :- [T])`, walking with `:wat::stream::next`. `keep-stream` twin
;; deleted; this is the DESIGN's own worked example (`probe-118B2-one-clause-lazy-producer.wat`).
(:wat::core::defn :wat::core::keep :- [T U]
  [f    <- [T :-> (:wat::core::Option :- [U])]
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [U])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::match (f value)
          ((:wat::core::Some v) (:wat::stream::cons v (:wat::core::keep f rest)))
          (:wat::core::None (:wat::core::keep f rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; ─── keep-indexed — as `keep`, `f : [i64 T :-> (Option :- [U])]` ────────────────────────────────
;; 118.B2 — ONE clause over `(Seqable :- [T])`. `keep-indexed-stream` (the twin that threaded an `idx`
;; accumulator as an extra param) is deleted; adding an `idx` param to `keep-indexed` itself would
;; be an arity change (STOP-2). Instead the index rides on `f`: each recursive step wraps the
;; CURRENT `f` in a fresh closure that adds 1 before delegating — `f` at recursion depth `k` calls
;; `0 -> f(k, value)` by unwinding `k` closures. This keeps the public arity `[f coll]` exact and
;; needs no new primitive, at the cost of O(n) chained calls per element (O(n^2) total instead of
;; the twin's O(n) `idx` counter) — the honest price of not having a second named helper to carry
;; the counter across the recursion the way `keep-indexed-stream` did.
(:wat::core::defn :wat::core::keep-indexed-walk :- [T U]
  [idx <- :wat::core::i64
   f   <- [:wat::core::i64 T :-> (:wat::core::Option :- [U])]
   s   <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [U])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next s)
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::match (f idx value)
          ((:wat::core::Some v)
            (:wat::stream::cons v (:wat::core::keep-indexed-walk (:wat::core::+ idx 1) f rest)))
          (:wat::core::None (:wat::core::keep-indexed-walk (:wat::core::+ idx 1) f rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

(:wat::core::defn :wat::core::keep-indexed :- [T U]
  [f    <- [:wat::core::i64 T :-> (:wat::core::Option :- [U])]
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [U])
  (:wat::core::keep-indexed-walk 0 f (:wat::core::Seqable/seq coll)))

;; ─── map-indexed — `f : [i64 T :-> U]` ──────────────────────────────────────────────────────────
;; 118.B2 — ONE clause over `(Seqable :- [T])`, same closure-composition trick as `keep-indexed`
;; (see its comment): `map-indexed-stream`'s `idx` param is gone; the index rides on `f` via a
;; fresh wrapping closure per recursive step. Public arity `[f coll]` unchanged; O(n) chained
;; calls per element traded for not adding a param.
(:wat::core::defn :wat::core::map-indexed-walk :- [T U]
  [idx <- :wat::core::i64
   f   <- [:wat::core::i64 T :-> U]
   s   <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [U])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next s)
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::stream::cons (f idx value)
          (:wat::core::map-indexed-walk (:wat::core::+ idx 1) f rest)))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

(:wat::core::defn :wat::core::map-indexed :- [T U]
  [f    <- [:wat::core::i64 T :-> U]
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [U])
  (:wat::core::map-indexed-walk 0 f (:wat::core::Seqable/seq coll)))

;; ─── dedupe — drop CONSECUTIVE duplicates ──────────────────────────────────────────────────────
;; 118.B2 — ONE clause over `(Seqable :- [T])`. `dedupe-stream`'s `prev : (Option :- [T])` param is gone;
;; `dedupe` has NO caller-supplied `f` to smuggle state through the way `keep-indexed`/
;; `map-indexed` do, and adding a `prev` param would be an arity change (STOP-2). Instead: emit
;; `value`, then recurse on `(drop-while (= value) rest)` — `drop-while` (unchanged, this same
;; file) already skips the run of elements equal to `value`, which is exactly "the consecutive
;; duplicates of what I just emitted." Each input element is inspected by at most ONE active
;; `drop-while` call (it stops at the first non-match), so this stays O(n) amortized — no
;; complexity trade-off here, unlike `keep-indexed`/`map-indexed`/`distinct` below.
(:wat::core::defn :wat::core::dedupe-walk :- [T]
  [prev <- (:wat::core::Option :- [T]) s <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next s)
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::match prev
          (:wat::core::None
            (:wat::stream::cons value (:wat::core::dedupe-walk (:wat::core::Some value) rest)))
          ((:wat::core::Some p)
            (:wat::core::if (:wat::core::= p value)
              (:wat::core::dedupe-walk (:wat::core::Some value) rest)
              (:wat::stream::cons value (:wat::core::dedupe-walk (:wat::core::Some value) rest))))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

(:wat::core::defn :wat::core::dedupe :- [T]
  [coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::core::dedupe-walk :wat::core::None (:wat::core::Seqable/seq coll)))

;; ─── distinct — drop ALL duplicates (keep first) ───────────────────────────────────────────────
;; 118.B2 — ONE clause over `(Seqable :- [T])`. `distinct-stream`'s `seen : (HashSet :- [T])` accumulator is
;; gone; `distinct` has no caller-supplied `f` to carry it on, and adding a `seen` param would be
;; an arity change (STOP-2). Same shape as the `dedupe` rewrite above, using `remove` (unchanged,
;; this same file) instead of `drop-while`: emit `value`, recurse on `(remove (= value) rest)` —
;; ALL later occurrences of `value` are filtered out, not just the immediately-following run. ⚠
;; UNLIKE `dedupe`, this is a genuine complexity trade: the twin's `HashSet` gave O(n) total
;; (O(1) amortized membership check per element); this composition re-scans the remaining stream
;; once per distinct value found, so a stream of N all-distinct elements costs O(n^2), not O(n).
;; Traded deliberately for staying at ONE clause with no new param — flagged, not hidden, per the
;; same complexity-honesty this file's own `stream->pvec`/`seqable->stream` history demands.
(:wat::core::defn :wat::core::distinct-walk :- [T]
  [seen <- (:wat::core::HashSet :- [T]) s <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next s)
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::if (:wat::core::contains? seen value)
          (:wat::core::distinct-walk seen rest)
          (:wat::stream::cons value
            (:wat::core::distinct-walk (:wat::core::conj seen value) rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

(:wat::core::defn :wat::core::distinct :- [T]
  [coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::core::distinct-walk (:wat::core::HashSet :- [:T]) (:wat::core::Seqable/seq coll)))

;; ─── reductions — emit `init`, then each successive accumulation ───────────────────────────────
;; 118.B2b — the three-call walk is GONE: every arm now delegates to ONE private `(Stream :- [T])` walker
;; that pulls with `:wat::stream::next` (one force per cell, where the old bodies forced three).
;;
;; ✅ 118.B2c + 118.B2d — TEN ARMS COLLAPSE TO TWO, one per ARITY, both over `(Seqable :- [T])`.
;;
;; B2b left this verb with ten per-container arms and a comment explaining that it COULD NOT
;; collapse, because a `defclause` ARM typed with a surface never dispatched:
;;
;;     no clause of :wat::core::reductions matched (3 args);
;;     clause 0 skipped (arg 2: expected :wat::core::Seqable<T>, got :wat::core::Vector)
;;
;; Two doors were shut. `118.B2c` strike 2 taught the runtime clause selector to ask the CHECKER's
;; own satisfaction question (`satisfies_bare_surface`) instead of enumerating concrete container
;; heads; `118.B2d` made a GENERIC satisfier's surface param bind from the receiver, so
;; `(Seqable/seq v)` on a `(Vector :- [i64])` yields `(Stream :- [i64])` rather than `(Stream :- [T])`. This collapse
;; is those two stones' payoff, and the first production code to walk through both.
;;
;; ⚠ `:wat::core::reduce` ABOVE IS DELIBERATELY *NOT* COLLAPSED, and the reason is measured, not
;; taste. Its eager arms delegate to `:wat::core::foldl`, a NATIVE intrinsic (src/runtime.rs:6354);
;; routing them through the interpreted walker instead costs **~4.6x** (200k i64 sum, both block
;; orderings, non-vacuity held: foldl ~81ms vs walk ~377ms —
;; `wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat`, re-derived 2026-09-03. It read
;; 5.1x when B6 was decided on 2026-08-18; both arms have since got faster and the ratio fell. The
;; walker is now `foldl-spec-walk` — the same body `reduce-walk` had, under its live name). `reductions` has no such
;; native path — every one of its ten arms already delegated to `reductions-walk` — so this
;; collapse is free and `reduce`'s would not be. The two verbs LOOK like the same shape and are not.
;;
;; ⚠ THE 2-ARITY EMPTY CASE — the old comment here was FALSE, and this is the measured record.
;; It claimed: *"an empty `coll` raises via `first`'s out-of-range failure rather than a silent
;; 0-arity dispatch."* Against HEAD:
;;
;;     empty Vector  ->  RAISES  ":wat::core::first: sequence has 0 element(s); no element at index 0"
;;     empty Stream  ->  yields a one-element stream containing `nil`   <- the claim was a LIE here
;;
;; The Stream arm reached that `nil` through the tracked B5 hole (`first` on an exhausted Stream
;; returns a bare `nil`). ★ NOT A NEW RULING: `reduce`'s own 2-arity Stream arm made the identical
;; call in B2, for the identical reason. Every arm now seeds from ONE `next` and raises by name on
;; empty — which is what this comment always claimed, now true for all five containers.
(:wat::core::defn :wat::core::reductions-walk :- [T U]
  [f <- [U T :-> U] init <- :U s <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [U])
  (:wat::stream::lazy
    (:wat::stream::cons init
      (:wat::core::match (:wat::stream::next s)
        ((:wat::stream::NextOutcome::Item value rest)
          (:wat::core::reductions-walk f (f init value) rest))
        (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty))))))

;; The 2-arity seed: pull the first element with ONE force, or raise by name. Shared by all five
;; 2-arity arms below so the message and the empty-contract exist in exactly one place.
(:wat::core::defn :wat::core::reductions-seed :- [T]
  [f <- [T T :-> T] s <- (:wat::stream::Stream :- [T])] -> (:wat::stream::Stream :- [T])
  (:wat::core::match (:wat::stream::next s)
    ((:wat::stream::NextOutcome::Item value rest)
      (:wat::core::reductions-walk f value rest))
    (:wat::stream::NextOutcome::Exhausted
      (:wat::kernel::assertion-failed!
        "reductions: the 2-arity form needs at least one element to seed the accumulation; got an empty collection"
        :wat::core::None :wat::core::None))))

(:wat::core::defclause :wat::core::reductions
  ;; 3-arity: explicit init.
  ([f <- [U T :-> U] init <- :U coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [U])
    (:wat::core::reductions-walk f init (:wat::core::Seqable/seq coll)))
  ;; 2-arity: no init — the first element seeds the accumulation. Empty raises, by name (above).
  ([f <- [T T :-> T] coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [T])
    (:wat::core::reductions-seed f (:wat::core::Seqable/seq coll))))

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
