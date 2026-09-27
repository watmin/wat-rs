(:wat::core::newtype :u::T :wat::core::i64)
(:wat::core::extend-type :u::T :wat::core::Orderable)
(:wat::core::newtype :u::U :wat::core::i64)
(:wat::core::defn :user::written [] -> :wat::core::bool
  (:wat::core::= (:wat::edn::write (:u::T 7)) "#u/T 7"))
(:wat::core::defn :user::round [] -> :wat::core::bool
  (:wat::core::= (:u::T 7) (:wat::edn::read (:wat::edn::write (:u::T 7)))))
(:wat::core::defn :user::one [] -> :u::T (:u::T 1))
(:wat::core::defn :user::two [] -> :u::T (:u::T 2))
(:wat::core::defn :user::other [] -> :u::U (:u::U 1))
(:wat::core::defn :user::ord [] -> :wat::core::bool
  (:wat::core::< (:u::T 1) (:u::T 2)))
(:wat::core::defn :user::printed [] -> :wat::core::bool
  (:wat::core::= (:wat::edn::write (:u::T 5)) "#u/T 5"))
