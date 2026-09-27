(:wat::core::defsurface :u::Mark :nature :wat::core::Record :features [])
(:wat::core::defsurface :u::Any :nature :wat::core::Record :features [])
(:wat::core::defrecord :u::In  [n <- :wat::core::i64])
(:wat::core::defrecord :u::Out [n <- :wat::core::i64])
(:wat::core::extend-type :u::In :u::Mark)
(:wat::core::extend-type :- [[T :< :u::Mark]] (:wat::core::Vector :- [T]) :u::Mark)
(:wat::core::extend-type :- [T] (:wat::core::Vector :- [T]) :u::Any)
(:wat::core::defn :u::take [m <- :u::Mark] -> :wat::core::i64 1)
(:wat::core::defn :u::take-any [m <- :u::Any] -> :wat::core::i64 1)
(:wat::core::defn :u::f :- [[T :< :u::Mark]] [x <- :T] -> :wat::core::i64 1)
(:wat::core::defn :user::vec-in [] -> :wat::core::i64
  (:u::take (:wat::core::Vector :- [:u::In] (:u::In :n 1))))
(:wat::core::defn :user::vec-nested [] -> :wat::core::i64
  (:u::take (:wat::core::Vector :- [(:wat::core::Vector :- [:u::In])]
              (:wat::core::Vector :- [:u::In] (:u::In :n 1)))))
(:wat::core::defn :user::f-in [] -> :wat::core::i64
  (:u::f (:wat::core::Vector :- [:u::In] (:u::In :n 1))))
(:wat::core::defn :user::any-out [] -> :wat::core::i64
  (:u::take-any (:wat::core::Vector :- [:u::Out] (:u::Out :n 1))))
