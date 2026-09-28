(:wat::core::defsurface :u::Mark :nature :wat::core::Record :features [])
(:wat::core::defrecord :u::In  [n <- wat.type/i64])
(:wat::core::defrecord :u::Out [n <- wat.type/i64])
(:wat::core::extend-type :u::In :u::Mark)
(:wat::core::extend-type :- [[Ts :< :u::Mark] :..] (wat.type/Tuple :- [Ts :..]) :u::Mark)
(:wat::core::extend-type :- [[T :< :u::Mark]] (wat.type/Vector :- [T]) :u::Mark)
(:wat::core::defn :u::take [m <- :u::Mark] -> wat.type/i64 1)
(:wat::core::defn :user::pair [] -> wat.type/i64
  (:u::take (wat.type/Tuple :- [:u::In :u::In] (:u::In :n 1) (:u::In :n 1))))
(:wat::core::defn :user::triple [] -> wat.type/i64
  (:u::take (wat.type/Tuple :- [:u::In :u::In :u::In] (:u::In :n 1) (:u::In :n 1) (:u::In :n 1))))
(:wat::core::defn :user::nested [] -> wat.type/i64
  (:u::take (:wat::core::Tuple (:u::In :n 1) (wat.type/Tuple :- [:u::In :u::In] (:u::In :n 1) (:u::In :n 1)))))
(:wat::core::defn :user::vec-of-pair [] -> wat.type/i64
  (:u::take (wat.type/Vector :- [(wat.type/Tuple :- [:u::In :u::In])]
              (wat.type/Tuple :- [:u::In :u::In] (:u::In :n 1) (:u::In :n 1)))))
