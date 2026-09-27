(:wat::core::defsurface :u::Mark :nature :wat::core::Record :features [])
(:wat::core::defrecord :u::In  [n <- :wat::core::i64])
(:wat::core::defrecord :u::Out [n <- :wat::core::i64])
(:wat::core::extend-type :u::In :u::Mark)
(:wat::core::extend-type :- [[Ts :< :u::Mark] :..] (:wat::core::Tuple :- [Ts :..]) :u::Mark)
(:wat::core::extend-type :- [[T :< :u::Mark]] (:wat::core::Vector :- [T]) :u::Mark)
(:wat::core::defn :u::take [m <- :u::Mark] -> :wat::core::i64 1)
(:wat::core::defn :user::pair [] -> :wat::core::i64
  (:u::take (:wat::core::Tuple (:u::In :n 1) (:u::In :n 1))))
(:wat::core::defn :user::triple [] -> :wat::core::i64
  (:u::take (:wat::core::Tuple (:u::In :n 1) (:u::In :n 1) (:u::In :n 1))))
(:wat::core::defn :user::nested [] -> :wat::core::i64
  (:u::take (:wat::core::Tuple (:u::In :n 1) (:wat::core::Tuple (:u::In :n 1) (:u::In :n 1)))))
(:wat::core::defn :user::vec-of-pair [] -> :wat::core::i64
  (:u::take (:wat::core::Vector :- [(:wat::core::Tuple :- [:u::In :u::In])]
              (:wat::core::Tuple (:u::In :n 1) (:u::In :n 1)))))
