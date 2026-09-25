;; Stone 255.32 — measurement. A service panics on one op. Two clients are
;; already connected. Print each client's recv' and the owner's handle recv',
;; on a thread locus and on a process locus. Pins whatever the code does.
(:wat::core::defsurface :p32::Boom :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :p32::Boom::BoomRequest [])
   (:wat::core::defenum :p32::Boom::BoomResponse :wat::enum::Pure
     :Ok [ok <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(boom [self <- :p32::Boom req <- :p32::Boom::BoomRequest] -> :p32::Boom::BoomResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :p32::boom
  :satisfies :p32::Boom
  :durable []
  :ephemeral []
  :impls
  [(boom [s ctx req]
     (:wat::kernel::assertion-failed! :message "P32-SERVICE-PANIC-REASON"))])

(:wat::core::defn :p32::connect [addr <- (:wat::kernel::Address :- [:p32::Boom::Op :p32::Boom::Reply])]
  -> (:wat::kernel::Peer :- [:p32::Boom::Op :p32::Boom::Reply])
  (:wat::core::match (:wat::kernel::connect addr)
    [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
    [:wat::kernel::ConnectOutcome.Closed {:cause c}
      (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
    [:wat::kernel::ConnectOutcome.Undialable {:cause c}
      (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.WrongPeer {:cause c}
      (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
    [:wat::kernel::ConnectOutcome.Failed {:cause c}
      (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]))

(:wat::core::defn :p32::show :- [O]
  [label <- :wat::core::String
   o <- (:wat::kernel::RecvOutcome :- [:O])]
  -> :wat::core::nil
  (:wat::core::match o
    [:wat::kernel::RecvOutcome.Message {:msg _m}
      (:wat::kernel::println (:wat::string::concat label " Message"))]
    [:wat::kernel::RecvOutcome.Lost {:cause c}
      (:wat::kernel::println (:wat::string::concat label " Lost " (:wat::kernel::LociDiedError/message c)))]
    [:wat::kernel::RecvOutcome.Stopped {}
      (:wat::kernel::println (:wat::string::concat label " Stopped"))]
    [:wat::kernel::RecvOutcome.Closed {}
      (:wat::kernel::println (:wat::string::concat label " Closed"))]))

(:wat::core::defn :p32::observe [tag <- :wat::core::String h <- :p32::boom::Handle] -> :wat::core::nil
  (:wat::core::let
    [a  (:p32::connect (:p32::boom::Handle/addr h))
     b  (:p32::connect (:p32::boom::Handle/addr h))
     _s (:wat::kernel::send a (:p32::Boom::Op.Boom {:req (:p32::Boom::BoomRequest)}))
     _a (:p32::show (:wat::string::concat tag " client-a") (:wat::kernel::recv a))
     _b (:p32::show (:wat::string::concat tag " client-b") (:wat::kernel::recv b))]
    (:p32::show (:wat::string::concat tag " owner") (:wat::kernel::recv (:p32::boom::Handle/handle h)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_t (:p32::observe "thread"
          (:p32::boom/start :locus (:wat::spawn::thread) :record (:p32::boom::Record)))
     _p (:p32::observe "process"
          (:p32::boom/start :locus (:wat::spawn::process) :record (:p32::boom::Record)))]
    nil))
