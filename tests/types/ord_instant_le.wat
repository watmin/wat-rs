;; ord_instant_le.wat
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/<= (wat.time/at 3) (wat.time/at 3)))
