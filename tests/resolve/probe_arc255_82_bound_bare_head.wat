;; 255.82 — a bare call head that is a let binding, a fn parameter, a match
;; binder, or a local that shadows a retired constructor still checks and runs.
(wat.core/defn user/add [a :- wat.type/i64 b :- wat.type/i64] :- wat.type/i64
  (wat.core/+ a b))

(wat.core/defn user/shadow-some [] :- wat.type/i64
  (wat.core/let [Some (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 x)]
    (Some 4)))

(wat.core/defn user/compute [] :- wat.type/i64
  (wat.core/let [at (wat.core/fn [i :- wat.type/i64] :- wat.type/i64
                      (wat.core/+ i 1))
                 n (at 10)
                 id (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 x)]
    (wat.core/match n
      [k (wat.core/+ (id k) (user/add 2 3))])))
