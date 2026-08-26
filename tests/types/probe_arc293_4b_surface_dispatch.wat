;; 293.4b own-probe — the GENERATED DISPATCHER. A call `(:t::Shape/area s)` routes by
;; `s`'s RUNTIME type to that type's `:T/area` defn. Two satisfiers (Circle, Square),
;; each backing `area` with its own method; one polymorphic consumer dispatches both.
;;
;; RED at HEAD (post-293.4a): method members parse + satisfy, but the call head
;; `:t::Shape/area` has no dispatcher → it resolves as UnknownFunction at check time,
;; so the program fails to type-check.
;;
;; GREEN at 293.4b: a `:Surface/method` head where Surface is a registered surface with
;; that method member dispatches on the receiver's concrete type to `:<T>/<method>`
;; (LIFT the arc-232 protocol dispatch shape, runtime.rs:5101 — but route to the plain
;; `defn :T/method`, NOT an `extend:<P>:<T>` impl).

(:wat::core::defsurface :t::Shape
  :nature :wat::core::Struct
  :features [(area [self <- :t::Shape] -> :wat::core::f64)])

(:wat::core::defrecord :t::Circle [radius <- :wat::core::f64])
(:wat::core::defn :t::Circle/area [self <- :t::Circle] -> :wat::core::f64
  (:wat::f64::* 3.14159 (:wat::f64::* (:t::Circle/radius self) (:t::Circle/radius self))))

(:wat::core::defrecord :t::Square [side <- :wat::core::f64])
(:wat::core::defn :t::Square/area [self <- :t::Square] -> :wat::core::f64
  (:wat::f64::* (:t::Square/side self) (:t::Square/side self)))

;; THE DISPATCHER under test — one consumer, accepts ANY Shape, routes :Shape/area by type.
(:wat::core::defn :t::describe [s <- :t::Shape] -> :wat::core::f64 (:t::Shape/area s))

(:wat::core::defn :t::circle-area [] -> :wat::core::f64 (:t::describe (:t::Circle :radius 2.0)))
(:wat::core::defn :t::square-area [] -> :wat::core::f64 (:t::describe (:t::Square :side 3.0)))
