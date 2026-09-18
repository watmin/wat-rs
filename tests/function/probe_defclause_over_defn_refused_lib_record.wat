;; THE LIBRARY, second shape: a record whose own body uses the GENERATED accessor.
;; `:mylib::Point/x` is never spelled as a declaration anywhere — codegen mints it —
;; which is exactly why a consumer `defclause` on that name was silent.
(:wat::core::defrecord :mylib::Point [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :mylib::sum [p <- :mylib::Point] -> :wat::core::i64
  (:wat::core::+ (:mylib::Point/x p) (:mylib::Point/y p)))
