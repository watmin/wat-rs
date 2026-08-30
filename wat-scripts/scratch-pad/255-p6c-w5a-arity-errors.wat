;; Scratch probe — arc 255 Stone P6-c-W5a, acceptance row 4.
;;
;; Same mechanism as `255-p6c-w2-arity.wat` (`:wat::eval-ast!` + `:wat::core::quote` to reach the
;; RUNTIME arity guard instead of the static type-checker's own arity gate, which would reject a
;; wrong-arity literal call before it ever reached the handler). One wrong-arity call per verb —
;; confirms the same op/expected/got shape survives the move to `#[wat_intrinsic]`, now raised by
;; the shim rather than the deleted hand-rolled `args.len() != N` guard.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::pure? 1 2)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "pure? UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "pure? kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::deterministic?)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "deterministic? UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "deterministic? kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::total? 1 2)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "total? UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "total? kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::primitive?)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "primitive? UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "primitive? kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::vocabulary-admitted? 1 2)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "vocabulary-admitted? UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "vocabulary-admitted? kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::cond-has-deferred-constraint?)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "cond-has-deferred-constraint? UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "cond-has-deferred-constraint? kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::alpha-match 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "alpha-match UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "alpha-match kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::alpha-match-local 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "alpha-match-local UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "alpha-match-local kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::alpha-match-under 1 2)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "alpha-match-under UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "alpha-match-under kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))))
