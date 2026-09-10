(:wat::core::defn :user::uf [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))]))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [dotted (:user::uf "(:wat.core/fn [n <- :wat.core/i64] -> :wat.core/bool (:wat.core/> n 3))")
     pf (:wat::core::Result/expect (:wat::eval-ast! dotted) "eval dotted failed")
     r  (:wat::core::apply  pf [5])]
    (:wat::kernel::println "dotted eval+apply(5):")
    (:wat::kernel::println (:wat::core::str r))))
