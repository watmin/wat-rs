(:wat::core::defn :user::uf [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [real (:user::uf "(:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error))")
     eq   (:user::uf "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::= n n))")
     acc  (:user::uf "(:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::telemetry::Level (:wat::telemetry::Log/level log))")]
    (:wat::kernel::println "real-telemetry-pred pure:")
    (:wat::kernel::println (:wat::core::str (:wat::rete::pure? real)))
    (:wat::kernel::println "real-telemetry-pred det:")
    (:wat::kernel::println (:wat::core::str (:wat::rete::deterministic? real)))
    (:wat::kernel::println "eq-only pure:")
    (:wat::kernel::println (:wat::core::str (:wat::rete::pure? eq)))
    (:wat::kernel::println "core-accessor-only pure:")
    (:wat::kernel::println (:wat::core::str (:wat::rete::pure? acc)))))
