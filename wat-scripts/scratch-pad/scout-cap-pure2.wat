;; localize: does pure? behave differently on a read-string'd form vs a quoted form?
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src   "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))"
     rs    (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))            ;; container of 1 form
     kid0  (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children rs) 0) "no kid")]
    (:wat::kernel::println (:wat::string::concat "rs   pure="  (:wat::core::str (:wat::rete::pure? rs))))
    (:wat::kernel::println (:wat::string::concat "kid0 pure="  (:wat::core::str (:wat::rete::pure? kid0))))
    (:wat::kernel::println (:wat::string::concat "kid0 det="   (:wat::core::str (:wat::rete::deterministic? kid0))))
    (:wat::kernel::println (:wat::string::concat "kid0 edn="   (:wat::core::write-forms kid0)))))
