;; Co-located fixture for probe_arc259_brackets_map.rs — brackets_map_doubles_in_order.
;; 50-item pool: doubles each via map (M=50 > N=cpu-count); result in input order.

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
   (:wat::bracket::map (:wat::spawn::thread)
     ;; Arc 118.2a — `map` flipped LAZY; `bracket::map`/`map-worker` need `items` eagerly.
     (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ i 1))
                      (:wat::core::range 0 50))
     (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2))))

