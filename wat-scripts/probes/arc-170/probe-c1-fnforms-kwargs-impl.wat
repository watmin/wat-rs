;; C1 probe — does fn-forms, given a COMPUTED keyword ":probe::work$impl", ship the
;; ::Kwargs struct dep too? And what does field-names-of/field-types-of on
;; :probe::work::Kwargs return (canonical wat.type/ decomposable forms)?
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
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [impl-kw (:wat::keyword::from-string "probe::work$impl")
     forms   (:wat::kernel::fn-forms impl-kw :user::bracket::work-fn)
     n       (:wat::core::length forms)
     _       (:wat::kernel::println n)
     _       (:wat::kernel::println forms)
     names   (:wat::runtime::field-names-of :probe::work::Kwargs)
     types   (:wat::runtime::field-types-of :probe::work::Kwargs)
     _       (:wat::kernel::println names)
     _       (:wat::kernel::println types)]
    (:wat::kernel::println "c1-fnforms-kwargs-impl: ok")))
