;; scout-ab: does ast-name expose the ::-form VERBATIM (basis for a ::-faithful printer)?
;; And re-confirm the ::-text read->eval->apply chain green in one file.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src     "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))"
     wrapped (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     kids    (:wat::core::ast->children wrapped)
     form    (:wat::core::first kids)
     ;; head keyword child of the fn form:
     fkids   (:wat::core::ast->children form)
     head    (:wat::core::first fkids)
     hname   (:wat::core::ast-name head)
     hkind   (:wat::core::ast-kind head)
     ;; end-to-end eval:
     pure    (:wat::rete::pure? form)
     pf      (:wat::core::Result/expect (:wat::eval-ast! form) "eval failed")
     keeps5  (:wat::core::apply  pf [5])]
    (:wat::kernel::println (:wat::string::concat "HEAD-NAME=" hname))
    (:wat::kernel::println (:wat::string::concat "HEAD-KIND=" hkind))
    (:wat::kernel::println (:wat::string::concat "pure=" (:wat::core::str pure) " keeps5=" (:wat::core::str keeps5)))))
