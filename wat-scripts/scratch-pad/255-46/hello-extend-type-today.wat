;; 255.46 — extend-type as written TODAY: a hello world, checked with `wat --check`.
;; A featureless surface (a class with no methods), one concrete member, one generic member.

(:wat::core::defsurface :hello::Orderable :nature :wat::core::Struct
  :features [])

(:wat::core::defrecord :hello::Point [x <- :wat::core::i64])
(:wat::core::defrecord :hello::Opaque [f <- :wat::core::i64])

;; concrete edge: a Point is Orderable.
(:wat::core::extend-type :hello::Point :hello::Orderable)

;; generic edge: EVERY (Vector :- [T]) is Orderable — the binder names T, and nothing can say "when T is".
(:wat::core::extend-type :- [T] (:wat::core::Vector :- [T]) :hello::Orderable)

(:wat::core::defn :hello::takes-orderable [o <- :hello::Orderable] -> :wat::core::i64 1)

(:wat::core::defn :hello::uses
  [p <- :hello::Point
   vp <- (:wat::core::Vector :- [:hello::Point])
   vo <- (:wat::core::Vector :- [:hello::Opaque])]
  -> :wat::core::i64
  (:wat::core::+ (:hello::takes-orderable p)
                 (:hello::takes-orderable vp)
                 ;; the hole C closes: Opaque is NOT Orderable, yet a vector of it is admitted.
                 (:hello::takes-orderable vo)))

;; ⚠ A SECOND HOLE, found while writing this file (2026-09-26): a featureless, non-parametric
;; surface admits ANY record with NO edge at all. `bad` below type-checks, although Opaque never
;; declared `extend-type :hello::Opaque :hello::Orderable`. Controls (not kept; this directory is
;; type-check gated): a String is refused; a surface with ONE feature refuses Opaque; the parametric
;; featureless `(Spawned :- [S R])` refuses Opaque. So membership in a marker surface is not
;; declaration-driven today, and the concrete edge above is decorative.
(:wat::core::defn :hello::bad [o <- :hello::Opaque] -> :wat::core::i64 (:hello::takes-orderable o))

;; A scalar CAN join a surface by declaration (control, rc=0 at the weigh):
(:wat::core::extend-type :wat::core::i64 :hello::Orderable)
(:wat::core::defn :hello::scalar [n <- :wat::core::i64] -> :wat::core::i64 (:hello::takes-orderable n))
