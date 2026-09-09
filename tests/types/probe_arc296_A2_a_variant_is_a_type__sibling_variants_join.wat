;; ⛔⛔ THE ROW THAT CATCHES THE MISSING JOIN — the builder's canonical conditional option.
;; Two SIBLING variants, one form. Neither is a subtype of the other; the join is Option.
;; 600 of last round's 103 floor failures were this shape and no fixture had it.
(:wat::core::defn :user::pick [b <- :wat::core::bool] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::if b
    (:wat::core::Option::Some {:value 42})
    (:wat::core::Option::None {})))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
