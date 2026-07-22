;; Ground truth — what does fn-forms render a PLAIN (non-kwargs) fn's own literal
;; param type annotations as? Mirrors spawn-runner's arity-6 c-nm derivation exactly.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::dial-work
  [peer <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>
   item <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::match (:probe::Echo/echo peer (:probe::Echo::EchoRequest :msg item)) -> :wat::core::String
  ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work-name (:wat::core::keyword/from-string "user::bracket::work-fn")
     forms     (:wat::kernel::fn-forms :probe::dial-work work-name)
     def-node  (:wat::core::Option/expect (:wat::core::last forms) "no define")
     fn-form   (:wat::core::first (:wat::core::drop (:wat::core::ast->children def-node) 2))
     fn-ch     (:wat::core::ast->children fn-form)
     argspec   (:wat::core::first (:wat::core::drop fn-ch 1))
     arg-ch    (:wat::core::ast->children argspec)
     c-ty      (:wat::core::first (:wat::core::drop arg-ch 2))
     c-nm      (:wat::core::ast-name c-ty)]
    (:wat::core::do
      (:wat::kernel::println forms)
      (:wat::kernel::println (:wat::core::string::concat "c-nm: " c-nm)))))
