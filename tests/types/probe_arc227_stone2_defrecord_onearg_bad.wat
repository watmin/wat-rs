;; Negative fixture: single-arg defrecord (no field list) must error with ArityMismatch.
(:wat::core::defrecord :test::Orphan)
(:wat::core::defn :user::compute [] -> :wat::core::bool :wat::core::true)
