;; ord_instant_gt.wat
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/> (wat.time/at 5) (wat.time/at 2)))
