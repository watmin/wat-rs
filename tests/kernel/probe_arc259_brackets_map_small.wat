;; Co-located fixture for probe_arc259_brackets_map.rs — brackets_map_small_in_order.
;; Small sanity: map over [10, 20, 30] adding 1 -> [11, 21, 31].

(:wat::core::defn :user::compute [] -> (wat.type/Vector :- [wat.type/i64])
   (:wat::bracket::map (:wat::spawn::thread)
     (wat.type/Vector :- [wat.type/i64] 10 20 30)
     (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 (:wat::core::+ x 1))))

