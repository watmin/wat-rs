;; GROUNDING 2 — can a kwargs work-fn hold Peer' (impure) kwargs? i.e. is the <name>::Kwargs
;; bundle a struct that accepts resources (the dialed services), not a pure Record that'd reject them?
;; EXPECT (fix reachable): COMPILES — the work-fn declares kv/echo as Peer' kwargs, binds them in
;;   the body; the ::Kwargs bundle holds the impure peers.
;; EXPECT (wall): a nature/purity error rejecting Peer' in the kwargs bundle.
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Kv::GetReq  [k <- :wat::core::String])
             (:wat::core::defenum :probe::Kv::GetResp :wat::enum::Pure :Ok [v <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(get [self <- :probe::Kv  req <- :probe::Kv::GetReq] -> :probe::Kv::GetResp :max-request-bytes 524288)])
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Echo::Req  [msg <- :wat::core::String])
             (:wat::core::defenum :probe::Echo::Resp :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                        :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::Req] -> :probe::Echo::Resp :max-request-bytes 524288)])

;; the bracket work-fn: item POSITIONAL, the services as Peer' KWARGS — bound directly in the body
(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [kv   <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>
      echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]]
  -> :wat::core::String
  (:wat::core::match
    (:probe::Echo/echo echo
      (:probe::Echo::Req :msg
        (:wat::core::match (:probe::Kv/get kv (:probe::Kv::GetReq :k item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Kv::GetResp::Ok v) v)
  ((:probe::Kv::GetResp::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Kv::GetResp::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Echo::Resp::Ok reply) reply)
  ((:probe::Echo::Resp::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::Resp::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "kwargs work-fn with Peer' kwargs: defined + type-checked ok"))
