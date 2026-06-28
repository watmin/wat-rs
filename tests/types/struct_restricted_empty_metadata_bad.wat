;; struct_restricted_empty_metadata_bad.wat — empty metadata map is illegal. Must FAIL.
(:wat::core::defstruct :my::Bad
  {}
  [field <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
