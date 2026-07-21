(:wat::core::defrecord :probe::P [a <- :wat::core::i64])
(:wat::core::defn :user::uf [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [f1 (:user::uf "(:wat::core::fn [x <- :wat::telemetry::Level] -> :wat::core::bool (:wat::core::= x :wat::telemetry::Level::Error))")
     f2 (:user::uf "(:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::telemetry::Level (:wat::telemetry::Log/level log))")
     f3 (:user::uf "(:wat::core::fn [] -> :probe::P (:probe::P :a 1))")
     f4 (:user::uf "(:wat::core::fn [p <- :probe::P] -> :wat::core::i64 (:probe::P/a p))")]
    (:wat::kernel::println "F1 enum-variant-ref value (Level::Error):")   (:wat::kernel::println (:wat::core::str (:wat::rete::pure? f1)))
    (:wat::kernel::println "F2 record accessor (Log/level):")             (:wat::kernel::println (:wat::core::str (:wat::rete::pure? f2)))
    (:wat::kernel::println "F3 record constructor (P):")                  (:wat::kernel::println (:wat::core::str (:wat::rete::pure? f3)))
    (:wat::kernel::println "F4 record accessor (P/a):")                   (:wat::kernel::println (:wat::core::str (:wat::rete::pure? f4)))))
