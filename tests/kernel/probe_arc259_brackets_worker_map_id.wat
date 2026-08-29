;; Co-located fixture for probe_arc259_brackets_worker.rs — map_worker_delivers_worker_id_as_runner_index.
;; worker-init returns work-fn that ignores item and returns worker-id;
;; proves every runner ran and produced its index.

;; Arc 170 gap J — map-worker absorbed `uses'`'s provisioning params; a plain caller passes
;; nil grant-handles, a no-op grant-fn/revoke-fn pair, and an EMPTY (Vector :- [D]) (no Setup sent).
(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
   (:wat::bracket::map-worker (:wat::spawn::thread)
     (:wat::core::range 0 50)
     (:wat::core::fn [wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
       (:wat::core::fn [_item <- :wat::core::i64] -> :wat::core::i64 wid))
     nil
     (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
     (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
     (:wat::core::Vector :- [:wat::core::nil])))

