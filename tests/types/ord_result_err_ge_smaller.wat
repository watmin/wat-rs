;; ord_result_err_ge_smaller.wat — Err("z") >= Err("a")
(wat.core/defn user/err [e :- wat.type/String]
  :- (wat.core/Result :- [wat.type/i64 wat.type/String])
  (wat.core/Result.Err {:error e}))
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/let
    [a (user/err "z")
     b (user/err "a")]
    (wat.core/>= a b)))
