;; ord_result_ok_gt_err.wat — Ok > Err
(wat.core/defn user/err [e :- wat.type/String]
  :- (wat.core/Result :- [wat.type/i64 wat.type/String])
  (wat.core/Result.Err {:error e}))
(wat.core/defn user/ok [n :- wat.type/i64]
  :- (wat.core/Result :- [wat.type/i64 wat.type/String])
  (wat.core/Result.Ok {:value n}))
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/let
    [a (user/ok 100)
     b (user/err "anything")]
    (wat.core/> a b)))
