;; 255.46 — extend-type as written TODAY: a hello world, checked with `wat --check`.
;; A featureless surface (a class with no methods), one concrete member, one generic member.

(:wat::core::defsurface :hello::Orderable :nature wat.type/Struct
  :features [])

(:wat::core::defrecord :hello::Point [x <- wat.type/i64])
(:wat::core::defrecord :hello::Opaque [f <- wat.type/i64])

;; concrete edge: a Point is Orderable.
(:wat::core::extend-type :hello::Point :hello::Orderable)

;; generic edge: EVERY (Vector :- [T]) is Orderable — the binder names T, and nothing can say "when T is".
(:wat::core::extend-type :- [T] (wat.type/Vector :- [T]) :hello::Orderable)

(:wat::core::defn :hello::takes-orderable [o <- :hello::Orderable] -> wat.type/i64 1)

(:wat::core::defn :hello::uses
  [p <- :hello::Point
   vp <- (wat.type/Vector :- [:hello::Point])
   vo <- (wat.type/Vector :- [:hello::Opaque])]
  -> wat.type/i64
  (:wat::core::+ (:hello::takes-orderable p)
                 (:hello::takes-orderable vp)
                 ;; the hole C closes: Opaque is NOT Orderable, yet a vector of it is admitted.
                 (:hello::takes-orderable vo)))

;; A scalar CAN join a surface by declaration (control, rc=0 at the weigh):
(:wat::core::extend-type wat.type/i64 :hello::Orderable)
(:wat::core::defn :hello::scalar [n <- wat.type/i64] -> wat.type/i64 (:hello::takes-orderable n))
