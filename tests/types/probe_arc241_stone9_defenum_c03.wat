;; Contract 03: interleaved unit and tagged variants.
(wat.core/defenum app/Event wat.enum/Pure
  :Tick
  :Move [x :- wat.type/i64
         y :- wat.type/i64]
  :Reset
  :Resize [width :- wat.type/i64])
