;; Contract 05: multiple fields, argspec stays rigid 3-slot triples.
(wat.core/defstruct my/Candle
  [open :- wat.type/f64
   high :- wat.type/f64
   low :- wat.type/f64
   close :- wat.type/f64
   volume :- wat.type/i64])
