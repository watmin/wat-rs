;; ord_bytes_gt.wat — [9] > [1]
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/>
    (wat.type/Vector :- [wat.type/u8] (wat.type/u8 9))
    (wat.type/Vector :- [wat.type/u8] (wat.type/u8 1))))
