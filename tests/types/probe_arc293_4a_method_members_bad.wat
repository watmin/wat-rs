;; 293.4a NEGATIVE probe — a record with the `color` field but NO `defn :T/area` must NOT satisfy
;; a surface that has an `area` method member.
;;
;; Disconfirms that method-member satisfaction is a real sig-check (not always-accept).
;; RED here = startup FAILS with a TypeMismatch (the checker rejects the call to :t::accept
;; because :t::NoMethod does not satisfy :t::Shape's method member `area`).

(:wat::core::defsurface :t::Shape
  [color <- :wat::core::String
   (area [self] -> :wat::core::f64)])

;; :t::NoMethod has `color` field (satisfies the field member) but NO `defn :t::NoMethod/area`.
(:wat::core::defrecord :t::NoMethod [color <- :wat::core::String])

(:wat::core::defn :t::accept [s <- :t::Shape] -> :wat::core::bool true)

;; Passing :t::NoMethod to :t::accept must FAIL — the `area` method member is not satisfied.
(:t::accept (:t::NoMethod "red"))
