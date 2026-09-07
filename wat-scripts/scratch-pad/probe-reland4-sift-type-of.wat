;; RELAND 4: can eval-with-defs! of sift-rules-defsvc (no caller defns) answer type-of
;; for the synthesized SiftRulesResponse?

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src  (:wat::io::read-file "tests/services/probe_arc278_sift_rules.wat")
     tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome::Forms {:forms f} f]
             [:wat::core::ReadOutcome::Malformed {:cause c}
               (:wat::kernel::assertion-failed! (:wat::core::Error/message c) :wat::core::None :wat::core::None)])
     ch   (:wat::core::ast->children tree)
     decls (:wat::core::into []
             (:wat::core::filter
               (:wat::core::fn [n <- :wat::WatAST] -> :wat::core::bool
                 (:wat::core::let [h (:wat::fix::head-name n)]
                   (:wat::core::or
                     (:wat::core::= h ":wat::core::defrecord")
                     (:wat::core::= h ":wat::query::sift-rules-defsvc"))))
               ch))
     form (:wat::core::quote
            (:wat::runtime::type-of :usr::my-sift::SiftRulesResponse))]
    (:wat::kernel::pprintln (:wat::core::count decls))
    (:wat::kernel::pprintln (:wat::eval-with-defs! form decls))))
