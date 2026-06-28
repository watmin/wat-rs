;; Contract 02: defstruct with :restricted-to metadata.
(:wat::core::defstruct :my::Token
  {:restricted-to [:my::]}
  [value <- :wat::core::i64])
