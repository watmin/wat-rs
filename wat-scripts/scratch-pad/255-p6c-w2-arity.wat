;; Scratch probe — arc 255 Stone P6-c-W2, acceptance row 3.
;;
;; Same mechanism as `255-p6c-w1-config-arity-all-four.wat` (`:wat::eval-ast!` +
;; `:wat::core::quote` to reach the RUNTIME arity guard instead of the static
;; type-checker's own arity gate, which would reject a wrong-arity literal call
;; before it ever reached the handler). One wrong-arity call per verb.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::stream::empty 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "stream::empty UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "stream::empty kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::stream::cons 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "stream::cons UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "stream::cons kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::stream::next)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "stream::next UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "stream::next kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::program::env 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "program::env UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "program::env kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::stdlib::sources 1)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "stdlib::sources UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "stdlib::sources kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))))
