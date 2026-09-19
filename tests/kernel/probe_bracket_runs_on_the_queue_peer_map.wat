;; Peer-path twin of probe_bracket_runs_on_the_queue_map.wat — same 20 items,
;; ThreadOpts locus. Throughput control.

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
   (:wat::bracket::map (:wat::spawn::thread/runner-count 2)
     (:wat::core::mapv
       (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ i 1))
       (:wat::core::range 0 20))
     (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2))))
