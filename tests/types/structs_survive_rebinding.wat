;; structs_survive_rebinding.wat — struct value survives let-rebinding + function call.
(wat.core/defstruct my/Point
  [x :- wat.type/i64
   y :- wat.type/i64])
(wat.core/defn my/y-of [p :- my/Point] :- wat.type/i64 (my.Point/y p))
(wat.core/defn my/compute [] :- wat.type/i64
  (wat.core/let
    [p (my/Point :x 3 :y 7)
     q p]
    (my/y-of q)))
