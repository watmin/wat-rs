;; Contract 08: defenum registers the type and variant constructors.
(:wat::core::defenum :app::Status
  :Ok
  :Pending
  :Error)
(:wat::core::defn :test::pick [] -> :app::Status :app::Status::Ok)
