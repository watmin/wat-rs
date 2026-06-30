;; Contract 02: mixed unit + tagged variants.
(:wat::core::defenum :app::Result :wat::enum::Pure
  :Ok
  :Err [code    <- :wat::core::i64
        message <- :wat::core::String])
