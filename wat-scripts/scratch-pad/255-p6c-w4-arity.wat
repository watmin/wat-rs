;; Scratch probe — arc 255 Stone P6-c-W4, acceptance rows 3/4.
;;
;; Same mechanism as `255-p6c-w2-arity.wat` (`:wat::eval-ast!` + `:wat::core::quote`
;; to reach the RUNTIME arity guard instead of the static type-checker's own arity
;; gate, which would reject a wrong-arity literal call before it ever reached the
;; handler). One wrong-arity call per verb: `metadata-of`, `field-names-of`,
;; `field-types-of` — all declared arity 1, called here with 2 args.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::runtime::metadata-of :wat::core::if :wat::core::if)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "metadata-of UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "metadata-of kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::runtime::field-names-of :wat::core::if :wat::core::if)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "field-names-of UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "field-names-of kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::runtime::field-types-of :wat::core::if :wat::core::if)))
      ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "field-types-of UNEXPECTED ok: " (:wat::edn::write v))))
      ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "field-types-of kind=" (:wat::core::EvalError/kind e) " message=" (:wat::core::EvalError/message e)))))))
