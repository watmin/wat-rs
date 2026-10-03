;; tests/types/probe_arc293_record_surface_holon.wat — co-located fixture (holon record positive)
;;
;; Arc 293.3-records — a HOLON record satisfies a core surface (R2 headline).

(wat.core/defsurface geo/Shape :nature wat.type/Struct :features [color :- wat.type/String])
(wat.holon/defrecord geo/HCircle [color :- wat.type/String  radius :- wat.type/f64])
(wat.core/defn geo/describe [s :- geo/Shape] :- wat.type/String
  "ok")
(wat.core/defn probe/drive [] :- wat.type/String
  (geo/describe (geo/HCircle :color "red" :radius 2.0)))
