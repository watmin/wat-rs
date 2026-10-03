;; tests/types/probe_arc293_structural_surface.wat — co-located fixture (positive case)
;;
;; Arc 293.3-core — a STRUCT structurally satisfies a defsurface.
;; RED at HEAD: defsurface is unknown; :geo::Shape does not resolve.

(wat.core/defsurface geo/Shape
  :nature wat.type/Struct
  :features [color :- wat.type/String])

(wat.core/defstruct geo/Circle
  [color :- wat.type/String  radius :- wat.type/f64])

;; accepts ANYTHING with the Shape surface; Circle has `color` ⇒ structurally satisfies it
(wat.core/defn geo/accepts-shape [s :- geo/Shape] :- wat.type/bool
  true)

(wat.core/defn probe/drive [] :- wat.type/bool
  (geo/accepts-shape (geo/Circle :color "red" :radius 2.0)))
