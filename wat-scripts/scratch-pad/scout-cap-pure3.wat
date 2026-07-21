;; THE CRUX: read-string of the DOTTED wire text (as write-forms emits) vs pure?
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [dotted "(:wat.core/fn [n <- :wat.core/i64] -> :wat.core/bool (:wat.core/> n 3))"
     rs     (:wat::core::read-string dotted)
     kid0   (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children rs) 0) "no kid")]
    (:wat::kernel::println (:wat::core::string::concat "dotted kid0 pure=" (:wat::core::str (:wat::rete::pure? kid0))))
    (:wat::kernel::println (:wat::core::string::concat "dotted kid0 edn="  (:wat::core::write-forms kid0)))))
