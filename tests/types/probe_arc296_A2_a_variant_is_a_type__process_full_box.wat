;; SUBJECT — the builder'"'"'s own program: a fn that takes ONLY Full boxes, destructuring directly.
(:wat::core::defenum :usr::Box :- [T] :wat::enum::Pure
  :Full  [inside <- :T]
  :Empty [])
(:wat::core::defn :user::process-full-box :- [T] [full-box <- (:usr::Box::Full :- [:T])] -> :T
  (:wat::core::let [{:keys [inside]} full-box] inside))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
