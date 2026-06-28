(:wat::core::do
  (:wat::core::defn :my::probe-f2::quote-helper [] -> :wat::core::nil nil)
  (:wat::core::quote
    (:my::probe-f2::ghost-quoted deeply-nested-arg)))
