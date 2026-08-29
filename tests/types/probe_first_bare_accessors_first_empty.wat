;; first on an EMPTY Vector — must raise (runtime) or fail check (HEAD). Expect error.
(:wat::core::defn :p::f [] -> :wat::core::i64 (:wat::core::first (:wat::core::Vector :- [:wat::core::i64])))
