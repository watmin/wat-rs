;; Scratch probe — arc 255 Stone P6-c-W6, acceptance row 4.
;;
;; Same mechanism as `255-p6c-w5c-arity-errors.wat` (`:wat::eval-ast!` + `:wat::core::quote` to
;; reach the RUNTIME arity guard instead of the static type-checker's own arity gate, which would
;; reject a wrong-arity literal call before it ever reached the handler). One wrong-arity call per
;; verb — confirms the same op/expected/got shape survives the move to `#[wat_intrinsic]`, now
;; raised by the shim rather than the deleted hand-rolled `args.len() != N` guard. The pre-image
;; shape (identical op/expected/got text) was confirmed against a real HEAD clone before homing.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::core::length (:wat::core::Vector 1 2) 9)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "length UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "length kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::core::empty?)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "empty? UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "empty? kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::core::nth (:wat::core::Vector 1 2))))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "nth UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "nth kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::core::last (:wat::core::Vector 1 2) 9)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "last UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "last kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::core::rest)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "rest UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "rest kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::core::reverse (:wat::core::Vector 1 2) 9)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "reverse UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "reverse kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::core::range 0)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "range UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "range kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))))
