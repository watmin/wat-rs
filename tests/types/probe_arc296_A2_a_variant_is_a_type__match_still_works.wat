;; REGRESSION CONTROL — the builder'"'"'s match example. GREEN TODAY, must stay green.
(:wat::core::defenum :usr::Box :- [T] :wat::enum::Pure
  :Full  [inside <- :T]
  :Empty [])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [full-box (:usr::Box.Full {:inside 42})]
    (:wat::core::match full-box
      [:usr::Box.Full  {:inside inside} (:wat::kernel::println inside)]
      [:usr::Box.Empty {}               (:wat::kernel::println "empty")])))
