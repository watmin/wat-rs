;; C1 probe — inspect ast-kind/ast-name/ast->children on field-types-of's canonical
;; wat.type/ forms, so we know how to rebuild ":wat::kernel::Peer'<S,R>"-style
;; colon-angle-bracket keyword strings from them (matching the established idiom
;; the existing arity-6 AST-walk uses).
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
    [types    (:wat::runtime::field-types-of :probe::work::Kwargs)
     ty0      (:wat::core::first types)
     kind0    (:wat::core::ast-kind ty0)
     ch0      (:wat::core::ast->children ty0)
     nch0     (:wat::core::length ch0)
     head     (:wat::core::first ch0)
     hkind    (:wat::core::ast-kind head)
     hname    (:wat::core::ast-name head)
     arg1     (:wat::core::Option/expect (:wat::core::get ch0 1) "no arg1")
     a1kind   (:wat::core::ast-kind arg1)
     a1name   (:wat::core::ast-name arg1)
     arg2     (:wat::core::Option/expect (:wat::core::get ch0 2) "no arg2")
     a2name   (:wat::core::ast-name arg2)
     names    (:wat::runtime::field-names-of :probe::work::Kwargs)
     nm0      (:wat::core::first names)]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "ty0-kind: " kind0))
      (:wat::kernel::println (:wat::core::string::concat "nch0: " (:wat::core::string::interpolate "{n}" :n nch0)))
      (:wat::kernel::println (:wat::core::string::concat "head-kind: " hkind))
      (:wat::kernel::println (:wat::core::string::concat "head-name: " hname))
      (:wat::kernel::println (:wat::core::string::concat "arg1-kind: " a1kind))
      (:wat::kernel::println (:wat::core::string::concat "arg1-name: " a1name))
      (:wat::kernel::println (:wat::core::string::concat "arg2-name: " a2name))
      (:wat::kernel::println (:wat::core::keyword/to-string nm0))
      (:wat::kernel::println "c1-ast-shape: ok"))))
