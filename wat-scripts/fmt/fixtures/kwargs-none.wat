(:wat::core::defn :fix::kwargs-none
  [path <- :wat::core::String
   msg <- :wat::core::String
   n <- :wat::core::i64
   c <- :wat::core::i64]
  -> :wat::grep::Unreadable
  (:wat::grep::Unreadable :file path :reason msg :line n :col c))
