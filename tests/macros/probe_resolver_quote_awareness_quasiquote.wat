(:wat::core::do
  (:wat::core::defn :my::probe-f2::live-fn [] -> :wat::core::nil nil)
  (:wat::core::quasiquote
    (:my::probe-f2::ghost-template
      (:wat::core::unquote (:my::probe-f2::live-fn)))))
