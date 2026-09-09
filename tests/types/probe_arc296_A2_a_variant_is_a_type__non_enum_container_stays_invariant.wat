;; ⛔⛔ THE ROW THAT SCOPES E. A defrecord is NOT an enum — it can hold a resource and
;; its T is not guaranteed to be read-only. Widening a variant INSIDE a non-enum
;; container must STAY REFUSED. General covariance would pass every other row in this
;; file and fail only here.
(:wat::core::defrecord :usr::Holder :- [T] [v <- :T])
(:wat::core::defn :user::takes-holder
  [h <- (:usr::Holder :- [(:wat::core::Option :- [:wat::core::i64])])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::takes-holder (:usr::Holder (:wat::core::Option::Some {:value 1}))))
