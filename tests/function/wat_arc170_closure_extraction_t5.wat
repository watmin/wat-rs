;; T5: inline lambda captures let-scope struct value (offset=10).
(wat.core/defstruct my/Config
  [offset :- wat.type/i64])
(wat.core/defn my/make-adder [] :- [wat.type/i64 :-> wat.type/i64]
  (wat.core/let
              [cfg (my/Config :offset 10)]
              (wat.core/fn [n :- wat.type/i64] :- wat.type/i64
                (wat.i64/+ n (my.Config/offset cfg)))))
