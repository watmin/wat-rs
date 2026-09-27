;; Stone 255.51 — a bounded type parameter is a member of its bound.
(:wat::core::defsurface :u::Mark :nature :wat::core::Record :features [])
(:wat::core::defrecord :u::In  [n <- :wat::core::i64])
(:wat::core::defrecord :u::Out [n <- :wat::core::i64])
(:wat::core::extend-type :u::In :u::Mark)
(:wat::core::defn :u::takes-mark [m <- :u::Mark] -> :wat::core::i64 1)
(:wat::core::defn :u::pass :- [[T :< :u::Mark]] [x <- :T] -> :wat::core::i64
  (:u::takes-mark x))
(:wat::core::defn :user::admitted [] -> :wat::core::i64
  (:u::pass (:u::In :n 1)))
(:wat::core::defn :user::anon [] -> :wat::core::i64
  (:wat::core::let
    [f (:wat::core::fn :- [[T :< :u::Mark]] [x <- :T] -> :wat::core::i64
         (:u::takes-mark x))]
    (f (:u::In :n 1))))
