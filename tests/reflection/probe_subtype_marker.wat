;; Tracked pin: subtype? of a derive-marker is true (2a1b).
(:wat::core::defrecord :probe::Rec [alpha <- wat.type/i64])
(:wat::core::derive :probe::Rec :probe::Marker)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println
    (:wat::core::str (:wat::core::subtype? :probe::Rec :probe::Marker))))
