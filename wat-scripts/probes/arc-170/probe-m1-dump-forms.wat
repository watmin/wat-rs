;; Dump the forms fn-forms emits for the dial work-fn — hunt the duplicate define.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [wf (:wat::core::fn [c <- :wat::kernel::Peer<probe::Echo::Op,probe::Echo::Reply>  s <- :wat::core::String]
             -> :wat::core::String
           (:wat::core::match
             (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg s)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv
  ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
     forms (:wat::kernel::fn-forms wf (:wat::core::keyword/from-string "user::bracket::work-fn"))]
    (:wat::core::foldl
      (:wat::core::fn [_a <- :wat::core::nil  f <- :wat::WatAST] -> :wat::core::nil
        (:wat::core::let
          [ch (:wat::core::ast->children f)
           hd (:wat::core::ast-name (:wat::core::first ch))
           nm (:wat::core::ast-name (:wat::core::nth ch 1))]
          (:wat::kernel::println (:wat::core::string::concat hd (:wat::core::string::concat "  " nm)))))
      nil
      forms)))
