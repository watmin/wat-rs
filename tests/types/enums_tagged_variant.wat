;; enums_tagged_variant.wat — tagged variant construction + match with binders.
(wat.core/defenum my/Event wat.enum/Pure
  :Candle  [open :- wat.type/f64 close :- wat.type/f64]
  :Deposit [amount :- wat.type/f64]
  :Nothing)
(wat.core/defn my/a-candle [] :- my/Event (my/Event.Candle {:open 100.0 :close 105.0}))
(wat.core/defn my/summary [e :- my/Event] :- wat.type/String
  (wat.core/match e 
    [my/Event.Candle {:open o :close c} (wat.f64/to-string c)]
    [my/Event.Deposit {:amount amt} (wat.f64/to-string amt)]
    [my/Event.Nothing {}       "nothing"]))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println (my/summary (my/a-candle))))
