;; scout-eval-5: isolate WHICH form trips default-deny purity on a record predicate.
(:wat::core::defrecord :user::Log
  [level   <- :wat::core::keyword
   message <- :wat::core::String])

(:wat::core::defn :user::uf
  [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; A) pure intrinsic only, ignores log
     a  (:user::uf "(:wat::core::fn [log <- :user::Log] -> :wat::core::bool (:wat::core::= 1 1))")
     ;; B) uses the generated accessor Log/level
     b  (:user::uf "(:wat::core::fn [log <- :user::Log] -> :wat::core::bool (:wat::core::= (:user::Log/level log) :error))")
     ;; C) just the accessor wrapped in bool-producing =
     c  (:user::uf "(:wat::core::fn [log <- :user::Log] -> :wat::core::keyword (:user::Log/level log))")]
    (:wat::kernel::println (:wat::core::str (:wat::rete::pure? a)))
    (:wat::kernel::println (:wat::core::str (:wat::rete::pure? b)))
    (:wat::kernel::println (:wat::core::str (:wat::rete::pure? c)))))
