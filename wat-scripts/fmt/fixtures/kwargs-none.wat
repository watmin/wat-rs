(:wat::core::defn :fix::kwargs-none
  [path <- wat.type/String
   msg <- wat.type/String
   n <- wat.type/i64
   c <- wat.type/i64]
  -> :wat::grep::Unreadable
  (:wat::grep::Unreadable :file path :reason msg :line n :col c))
