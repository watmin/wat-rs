(:wat::core::do
  (:wat::core::defn :my::probe-f2::forms-helper [] -> :wat::core::nil nil)
  (:wat::core::forms
    (:my::probe-f2::ghost-inner some-arg)
    (:my::probe-f2::ghost-other 1 2 3)))
