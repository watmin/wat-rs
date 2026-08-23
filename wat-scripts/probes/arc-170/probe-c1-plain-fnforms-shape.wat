;; Ground truth — what does fn-forms render a PLAIN (non-kwargs) fn's own literal
;; param type annotations as? Mirrors spawn-runner's arity-6 c-nm derivation exactly.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::dial-work
  [peer <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])
   item <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::match (:probe::Echo/echo peer (:probe::Echo::EchoRequest :msg item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work-name (:wat::core::keyword/from-string "user::bracket::work-fn")
     forms     (:wat::kernel::fn-forms :probe::dial-work work-name)
     def-node  (:wat::core::Option/expect (:wat::core::last forms) "no define")
     fn-form   (:wat::core::nth (:wat::core::ast->children def-node) 2)
     fn-ch     (:wat::core::ast->children fn-form)
     argspec   (:wat::core::nth fn-ch 1)
     arg-ch    (:wat::core::ast->children argspec)
     c-ty      (:wat::core::nth arg-ch 2)
     c-nm      (:wat::core::ast-name c-ty)]
    (:wat::core::do
      (:wat::kernel::println forms)
      (:wat::kernel::println (:wat::core::string::concat "c-nm: " c-nm)))))
