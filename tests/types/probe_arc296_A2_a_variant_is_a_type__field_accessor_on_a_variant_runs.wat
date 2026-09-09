;; THE SIBLING RUN ROW — field access, the OTHER runtime path that matches
;; Value::Aggregate without a Value::Enum arm (the keyword-as-accessor fall-through).
(:wat::core::defenum :usr::Box :- [T] :wat::enum::Pure
  :Full  [inside <- :T]
  :Empty [])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [full-box (:usr::Box::Full {:inside 7})]
    (:wat::kernel::println (:inside full-box))))
