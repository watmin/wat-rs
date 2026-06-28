;; Co-located fixture for probe_arc259_brackets_worker.rs — each_worker_drains_and_returns_nil.
;; each-worker over 50 items returns nil; completion proves pool drained all 50.

(:wat::core::defn :user::compute [] -> :wat::core::nil
   (:wat::bracket::each-worker (:wat::spawn::thread)
     (:wat::core::range 0 50)
     (:wat::core::fn [_wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
       (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))))

