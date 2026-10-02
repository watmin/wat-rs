;; Co-located fixture for probe_arc259_brackets_worker.rs — each_worker_drains_and_returns_nil.
;; each-worker over 50 items returns nil; completion proves pool drained all 50.

;; Arc 170 gap J — each-worker absorbed `uses'`'s provisioning params; a plain caller passes
;; nil grant-handles, a no-op grant-fn/revoke-fn pair, and an EMPTY (Vector :- [D]) (no Setup sent).
(:wat::core::defn :user::compute [] -> wat.type/nil
   (:wat::bracket::each-worker (:wat::spawn::thread)
     (:wat::core::range 0 50)
     (:wat::core::fn [_wid <- wat.type/i64] -> [wat.type/i64 :-> wat.type/i64]
       (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 (:wat::core::* x 2)))
     nil
     (:wat::core::fn [_g <- wat.type/nil _pid <- wat.type/i64] -> wat.type/nil nil)
     (:wat::core::fn [_g <- wat.type/nil _pid <- wat.type/i64] -> wat.type/nil nil)
     (wat.type/Vector :- [wat.type/nil])))

