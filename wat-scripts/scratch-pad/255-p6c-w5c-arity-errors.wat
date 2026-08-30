;; Scratch probe — arc 255 Stone P6-c-W5c, acceptance row 4.
;;
;; Same mechanism as `255-p6c-w5b-arity-errors.wat` (`:wat::eval-ast!` + `:wat::core::quote` to
;; reach the RUNTIME arity guard instead of the static type-checker's own arity gate, which would
;; reject a wrong-arity literal call before it ever reached the handler). One wrong-arity call per
;; verb — confirms the same op/expected/got shape survives the move to `#[wat_intrinsic]`, now
;; raised by the shim rather than the deleted hand-rolled `args.len() != N` guard. The pre-image
;; shape (identical op/expected/got text) was confirmed against a real HEAD clone before homing.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::lower 1 2)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "lower UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "lower kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::collect-rules)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "collect-rules UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "collect-rules kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::step-payload 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "step-payload UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "step-payload kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::rete::axis-violation)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "axis-violation UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "axis-violation kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))))
