;; Contract 08: defenum registers the type and variant constructors.
(:wat::core::defenum :app::Status :wat::enum::Pure
  :Ok
  :Pending
  :Error)
(:wat::core::defn :test::pick [] -> :app::Status :app::Status::Ok)
