(:wat::core::defrecord :u::Pt [n <- wat.type/i64])
(:wat::core::defn :u::take-eq [x <- :wat::core::Equatable] -> wat.type/i64 1)
(:wat::core::defn :u::take-ord [x <- :wat::core::Orderable] -> wat.type/i64 1)
(:wat::core::defn :user::eq [] -> wat.type/i64
  (:u::take-eq (:u::Pt :n 1)))
(:wat::core::defn :user::ord-bigint [] -> wat.type/i64
  (:u::take-ord 1N))
(:wat::core::newtype :u::Meters wat.type/i64)
(:wat::core::extend-type :u::Meters :wat::core::Orderable)
(:wat::core::defn :user::ord-nt [] -> wat.type/i64
  (:u::take-ord (:u::Meters 1)))
