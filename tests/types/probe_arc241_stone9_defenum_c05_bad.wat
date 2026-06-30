;; Negative: empty {} metadata must be rejected.
(:wat::core::defenum :app::Status :wat::enum::Pure
  {}
  :Ok
  :Err)
