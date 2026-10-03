;; Co-located fixture for probe_arc259_brackets_each.rs — brackets_each_small_returns_nil.
;; Small 3-item case: each over [10, 20, 30] returns nil.

(wat.core/defn user/compute [] :- wat.type/nil
   (wat.bracket/each (wat.spawn/thread)
     (wat.type/Vector :- [wat.type/i64] 10 20 30)
     (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.core/+ x 1))))

