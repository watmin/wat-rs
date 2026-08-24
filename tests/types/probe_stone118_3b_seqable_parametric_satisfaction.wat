;; tests/types/probe_stone118_3b_seqable_parametric_satisfaction.wat — co-located fixture.
;;
;; Stone 118.3-B — `src/check.rs`'s `(Parametric actual, Parametric expected)` arm (~14858)
;; string-compared a registered `extend-type` edge (stored VERBATIM with the SURFACE's own
;; declared param name, e.g. `:sq::Seqable<T>`) against the CALL SITE's rendered expected type
;; (a fresh unification var, e.g. `:sq::Seqable<?454>`) — "<?454>" != "<T>", always, so a
;; concrete container could never satisfy a PARAMETRIC surface bound. See
;; docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/{BRIEF,EXPECTATIONS,MEASURED}-118.3-B*.md.
;;
;; Two independent surfaces below, deliberately DIFFERENT base names so neither the registry nor
;; this fixture can confuse the two arms under test:
;;   `BareSeqable`  — NOT parametric. Goes through arm 3 (`(Parametric actual, Path expected)`,
;;                    `parametric_head_fqdn` lookup). Must stay byte-identical (row 2 / STOP-3).
;;   `Seqable :- [T]`   — parametric. Goes through the FIXED arm 5. All four containers extend it,
;;                    matching `extract_lazyable_elem`'s hardcoded four-head set (row 1).

;; ─── bare (non-parametric) surface — arm 3, unchanged ──────────────────────────────
;; NOTE: method name deliberately DISTINCT from the parametric surface's `as-vec` below —
;; `extend-type` methods register as `<ConcreteType>/<method>` GLOBALLY (not scoped per
;; surface), so both surfaces implementing `Vector` with the SAME method name collide as a
;; `DuplicateDefine`. Not a stone-118.3-B concern; a pre-existing global-scheme constraint.
(:wat::core::defsurface :t118b::BareSeqable
  :nature :wat::core::Struct
  :features [(as-vec-bare [self <- :t118b::BareSeqable] -> (:wat::core::Vector :- [:wat::core::i64]))])

(:wat::core::extend-type :wat::core::Vector :t118b::BareSeqable
  (as-vec-bare [self] -> (:wat::core::Vector :- [:wat::core::i64]) self))

(:wat::core::extend-type :wat::core::PersistentVector :t118b::BareSeqable
  (as-vec-bare [self] -> (:wat::core::Vector :- [:wat::core::i64])
    (:wat::core::into (:wat::core::Vector :wat::core::i64) self)))

(:wat::core::defn :t118b::bare-count-of [s <- :t118b::BareSeqable] -> :wat::core::i64
  (:wat::core::length (:t118b::BareSeqable/as-vec-bare s)))

;; ─── parametric surface — arm 5, THE FIX ────────────────────────────────────────────
(:wat::core::defsurface :t118b::Seqable :- [T] :nature :wat::core::Struct
  :features [(as-vec [self <- (:t118b::Seqable :- [T])] -> (:wat::core::Vector :- [T]))])

(:wat::core::extend-type :wat::core::Vector (:t118b::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T]) self))

(:wat::core::extend-type :wat::core::PersistentVector (:t118b::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T]) (:wat::core::into (:wat::core::Vector :T) self)))

(:wat::core::extend-type :wat::core::List (:t118b::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T])
    (:wat::core::foldl (:wat::core::fn [acc <- (:wat::core::Vector :- [T]) x <- :T] -> (:wat::core::Vector :- [T])
                         (:wat::core::conj acc x))
                       (:wat::core::Vector :T) self)))

(:wat::core::extend-type :wat::stream::Stream (:t118b::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T]) (:wat::core::into (:wat::core::Vector :T) self)))

(:wat::core::defn :t118b::count-of :- [T] [s <- (:t118b::Seqable :- [T])] -> :wat::core::i64
  (:wat::core::length (:t118b::Seqable/as-vec s)))

;; ─── entry points, driven via call_beside_value ─────────────────────────────────────

;; row 2 — bare-surface path untouched.
(:wat::core::defn :t::bare-vector [] -> :wat::core::i64
  (:t118b::bare-count-of (:wat::core::Vector :wat::core::i64 10 20 30)))

(:wat::core::defn :t::bare-persistent-vector [] -> :wat::core::i64
  (:t118b::bare-count-of (:wat::core::PersistentVector 1 2 3 4)))

;; row 1 — all four containers dispatch through the parametric surface.
(:wat::core::defn :t::param-vector [] -> :wat::core::i64
  (:t118b::count-of (:wat::core::Vector :wat::core::i64 1 2 3)))

(:wat::core::defn :t::param-persistent-vector [] -> :wat::core::i64
  (:t118b::count-of (:wat::core::PersistentVector 1 2 3 4)))

(:wat::core::defn :t::param-list [] -> :wat::core::i64
  (:t118b::count-of (:wat::core::List/of 1 2 3 4 5)))

(:wat::core::defn :t::param-stream [] -> :wat::core::i64
  (:t118b::count-of (:wat::stream::cons 1
                       (:wat::stream::lazy
                         (:wat::stream::cons 2
                           (:wat::stream::lazy (:wat::stream::empty)))))))
