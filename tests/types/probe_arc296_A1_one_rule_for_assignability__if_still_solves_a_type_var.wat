;; ⛔⛔ THE CAPABILITY GUARD — this is what the ruling is ABOUT.
;; The None branch leaves Option's T unsolved; the Some branch pins it to i64.
;; UNIFICATION is what solves it. A stone that subsumes unconditionally keeps every
;; other row green while silently weakening inference across the whole corpus.
(:wat::core::defn :user::pick [b <- :wat::core::bool] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::if b (:wat::core::Option::None {}) (:wat::core::Option::Some {:value 7})))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
