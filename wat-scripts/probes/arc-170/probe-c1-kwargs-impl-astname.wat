;; Ground truth #2 — query ast-name DIRECTLY on the kwargs $impl's own item-type
;; node (not the printed/mangled forms text) — is it colon-colon literal (like a
;; plain fn's literal param) or something else?
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]]
  -> :wat::core::String
  (:wat::core::match
    (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [impl-kw   (:wat::core::keyword/from-string "probe::work$impl")
     work-name (:wat::core::keyword/from-string "user::bracket::work-fn")
     forms     (:wat::kernel::fn-forms impl-kw work-name)
     nforms    (:wat::core::length forms)
     def-node  (:wat::core::Option/expect (:wat::core::get forms (:wat::i64::- nforms 2)) "no define")
     dn-ch     (:wat::core::ast->children def-node)
     head0     (:wat::core::first dn-ch)
     head0k    (:wat::core::ast-kind head0)
     head0n    (:wat::core::ast-name head0)
     argspec   (:wat::core::Option/expect (:wat::core::get dn-ch 2) "no argspec")
     arg-ch    (:wat::core::ast->children argspec)
     item-ty   (:wat::core::Option/expect (:wat::core::get arg-ch 2) "no item-ty")
     item-nm   (:wat::core::ast-name item-ty)
     ret-ty    (:wat::core::Option/expect (:wat::core::get dn-ch 4) "no ret-ty")
     ret-nm    (:wat::core::ast-name ret-ty)]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "head0-kind: " head0k))
      (:wat::kernel::println (:wat::string::concat "head0-nm: " head0n))
      (:wat::kernel::println (:wat::string::concat "item-nm: " item-nm))
      (:wat::kernel::println (:wat::string::concat "ret-nm: " ret-nm)))))
