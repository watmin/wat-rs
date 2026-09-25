;; Stone 255.39 — a featureless parametric surface whose parameter an
;; implementing binder edge writes. Pre-stone this is UnconsumedTypeParam
;; (the check ran before the edge existed). The edge is the consumption.
(:wat::core::defstruct :probe::Holder :- [S R]
  [sent <- :S
   recv <- :R])
(:wat::core::defsurface :probe::Owner :- [S R] :nature :wat::core::Struct
  :features [])
(:wat::core::extend-type :- [S R]
  (:probe::Holder :- [S R])
  (:probe::Owner :- [S R]))
