;; ord_duration_ge.wat
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/>= (wat.time/Day 1) (wat.time/Hour 24)))
