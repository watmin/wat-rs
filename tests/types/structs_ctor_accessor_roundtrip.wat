;; structs_ctor_accessor_roundtrip.wat — user struct ctor + accessor round-trip.
(wat.core/defstruct my.market/Bar
  [open  :- wat.type/f64
   close :- wat.type/f64])
(wat.core/defn my/compute [] :- wat.type/f64
  (wat.core/let
    [b (my.market/Bar :open 1.0 :close 2.0)
     o (my.market.Bar/open b)
     c (my.market.Bar/close b)]
    (wat.f64/- c o)))
