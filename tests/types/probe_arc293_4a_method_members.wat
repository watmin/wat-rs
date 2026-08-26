;; 293.4a own-probe — a `defsurface` with a METHOD member parses, and a record that
;; backs that method with a `defn :T/name` STRUCTURALLY satisfies the surface.
;;
;; RED at HEAD: `parse_defsurface` runs members through `parse_argspec_triples`
;; (field triples `[name <- :T]` only), so the method member `(area [self] -> :f64)`
;; does not parse → the program fails to type-check.
;;
;; GREEN at 293.4a: method members parse (SurfaceMember::Method, reusing the
;; arc-232 method-sig parser) and a Method member is satisfied by a matching
;; `defn :T/name`. NO dispatcher is exercised here (that is 293.4b) — `accept`
;; only requires the surface in a param position; satisfaction runs via `assignable`.

;; The surface mixes a FIELD member (color) and a METHOD member (area).
(:wat::core::defsurface :t::Shape
  :nature :wat::core::Struct
  :features [color <- :wat::core::String
   (area [self <- :t::Shape] -> :wat::core::f64)])

;; Sq backs `color` with a FIELD and `area` with a METHOD (a defn) — the satisfier's
;; private choice; the surface sees only accessors.
(:wat::core::defrecord :t::Sq [color <- :wat::core::String  side <- :wat::core::f64])
(:wat::core::defn :t::Sq/area [self <- :t::Sq] -> :wat::core::f64
  (:wat::f64::* (:t::Sq/side self) (:t::Sq/side self)))

;; A consumer that REQUIRES the surface in a param position. Passing a Sq makes the
;; checker run `struct_satisfies_surface` (color field + area method) — structural,
;; no declaration at Sq.
(:wat::core::defn :t::accept [s <- :t::Shape] -> :wat::core::bool true)

(:t::accept (:t::Sq :color "red" :side 3.0))
