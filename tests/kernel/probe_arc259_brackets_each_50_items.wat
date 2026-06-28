;; Co-located fixture for probe_arc259_brackets_each.rs — brackets_each_drains_50_and_returns_nil.
;; 50-item pool: each returns nil + completion proves drainage (no hang = pool drained all 50).

(:wat::core::defn :user::compute [] -> :wat::core::nil
   (:wat::bracket::each (:wat::spawn::thread)
     (:wat::core::range 0 50)
     (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2))))

