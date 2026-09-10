;; ⛔⛔ THE RUN ROW. Every other A-2 fixture asserts a --check EXIT CODE. This one
;; RUNS, and it is the only row that can see a checker/runtime disagreement.
;; At the WIP it CHECKS CLEAN and dies:
;;   ":wat::core::let: expected an aggregate type, got wat::core::Enum `(:usr::Box::Full 42)`"
;; The checker's {:keys} predicate was widened by shape; the runtime's was not.
(:wat::core::defenum :usr::Box :- [T] :wat::enum::Pure
  :Full  [inside <- :T]
  :Empty [])
(:wat::core::defn :user::process-full-box :- [T]
  [full-box <- (:usr::Box.Full :- [:T])] -> :T
  (:wat::core::let [{:keys [inside]} full-box] inside))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [full-box (:usr::Box.Full {:inside 42})]
    (:wat::kernel::println (:user::process-full-box full-box))))
