;; Co-located fixture for probe_arc259_brackets_worker.rs — each_worker_drains_and_returns_nil.
;; each-worker over 50 items returns nil; completion proves pool drained all 50.

;; Arc 170 gap J — each-worker absorbed `uses'`'s provisioning params; a plain caller passes
;; nil grant-handles, a no-op grant-fn/revoke-fn pair, and an EMPTY (Vector :- [D]) (no Setup sent).
(:wat::core::defn :user::compute [] -> :wat::core::nil
   (:wat::bracket::each-worker (:wat::spawn::thread)
     (:wat::core::range 0 50)
     (:wat::core::fn [_wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
       (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))
     nil
     (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
     (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
     (:wat::core::Vector :- [:wat::core::nil])))

