;; tests/types/probe_arc118_2_lazy_map.wat — co-located fixture for the sibling probe (.rs).
;;
;; `:wat::core::map` of a fn that errors only on a LATE element (99), over [1 2 99], pulling only
;; the head. RED while core::map is eager (forces boom(99) at the map call → DivisionByZero);
;; GREEN when core::map is lazy (only boom(1) runs → returns 1). The body runs when the probe
;; eval_in_frozen's the explicit (:my::compute) call.

(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [boom (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::if (:wat::core::= x 99)
              
              (:wat::core::i64::/ x 0)
              x))
     mapped (:wat::core::map boom (:wat::core::Vector :wat::core::i64 1 2 99))]
    (:wat::core::first mapped)))

