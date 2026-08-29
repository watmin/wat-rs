;; Scratch probe — arc 255 Stone P6-c-W5b, acceptance row 4.
;;
;; Same mechanism as `255-p6c-w5a-arity-errors.wat` (`:wat::eval-ast!` + `:wat::core::quote` to
;; reach the RUNTIME arity guard instead of the static type-checker's own arity gate, which
;; would reject a wrong-arity literal call before it ever reached the handler). One wrong-arity
;; call per verb — confirms the same op/expected/got shape survives the move to
;; `#[wat_intrinsic]`, now raised by the shim rather than the deleted hand-rolled
;; `args.len() != N` guard.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::arm-session 1 2)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "arm-session UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "arm-session kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::release-session)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "release-session UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "release-session kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::export 1 2)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "export UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "export kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::import)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "import UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "import kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::eval-insert 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "eval-insert UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "eval-insert kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::eval-test 1 2 3)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "eval-test UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "eval-test kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))))
