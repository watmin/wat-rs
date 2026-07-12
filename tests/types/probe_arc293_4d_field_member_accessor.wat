;; 293.4d own-probe — a FIELD member is an ACCESSOR too. `:Surface/field s` must dispatch
;; by `s`'s runtime type to the satisfier's `:T/field` accessor (a record's auto-generated
;; field accessor, or — for a foreign type — an extend-type method). This is the last seam
;; of "methods are accessors": field-vs-method is invisible at BOTH the surface AND the call.
;;
;; RED at HEAD (post-293.4c): 293.4b generates a dispatcher only for METHOD members; a FIELD
;; member called through the surface (`:t::Colored/color c`) is UnknownCallee.
;;
;; GREEN at 293.4d: every surface member (field or method) dispatches `:Surface/name s` to
;; `:<T>/name` (a field auto-accessor or a method). Isolated to a record here (no extend/Vector).

(:wat::core::defsurface :t::Colored
  :nature :wat::core::Struct
  :features [color <- :wat::core::String])

(:wat::core::defrecord :t::Ball [color <- :wat::core::String  radius <- :wat::core::f64])

;; Call the FIELD member as an accessor THROUGH the surface — routes by runtime type to :t::Ball/color.
(:wat::core::defn :t::hue [c <- :t::Colored] -> :wat::core::String (:t::Colored/color c))

(:wat::core::defn :t::probe [] -> :wat::core::String (:t::hue (:t::Ball :color "red" :radius 2.0)))
