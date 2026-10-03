;; Co-located fixture for probe_arc259_brackets_worker.rs — map_worker_doubles_in_order_ignoring_worker_id.
;; map-worker with worker-init that ignores worker-id; doubles 1..50 in input order.

;; Arc 170 gap J — map-worker absorbed `uses'`'s provisioning params; a plain caller passes
;; nil grant-handles, a no-op grant-fn/revoke-fn pair, and an EMPTY (Vector :- [D]) (no Setup sent).
(wat.core/defn user/compute [] :- (wat.type/Vector :- [wat.type/i64])
   (wat.bracket/map-worker (wat.spawn/thread)
     ;; Arc 118.2a — `map` flipped LAZY; `map-worker` needs `items` eagerly ((Vector :- [I]) param).
     (wat.core/mapv (wat.core/fn [i :- wat.type/i64] :- wat.type/i64 (wat.core/+ i 1))
                      (wat.core/range 0 50))
     (wat.core/fn [_wid :- wat.type/i64] :- [wat.type/i64 :-> wat.type/i64]
       (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.core/* x 2)))
     nil
     (wat.core/fn [_g :- wat.type/nil _pid :- wat.type/i64] :- wat.type/nil nil)
     (wat.core/fn [_g :- wat.type/nil _pid :- wat.type/i64] :- wat.type/nil nil)
     (wat.type/Vector :- [wat.type/nil])))

