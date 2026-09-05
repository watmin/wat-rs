;; arc 278 S4c ACCEPTANCE — a `:messages` peer surface OWNS its protocol and SHIPS it across a
;; process fork. The counter's Request/Response records live INSIDE the surface's :messages block
;; (not as external top-level forms). The service :satisfies the surface; started on (process) the
;; forked child boots stdlib + service-forms, and service-forms now concats the surface's
;; surface-forms carrier → the child resolves ::Op/::Reply + the message records at its fresh
;; startup. Round-trip: start (process) 0 → connect → increment 5 → get → prints 5.

(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest        [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure :Ok       [value <- :wat::core::i64] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                        :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :my::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :my::Counter::IncrementResponse :wat::enum::Pure :Ok [value <- :wat::core::i64] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                        :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get       [self <- :my::Counter  req <- :my::Counter::GetRequest]       -> :my::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :my::Counter  req <- :my::Counter::IncrementRequest] -> :my::Counter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:my::Counter::Reply::Get (:my::Counter::GetResponse::Ok (:my::counter::Record/count (:my::counter::State/durable s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:my::Counter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:my::counter::Op])])))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+ (:my::counter::Record/count (:my::counter::State/durable s))
                                             (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Continue (:my::counter::State :durable (:my::counter::Record :count c))
                                      (:wat::core::Some (:my::Counter::Reply::Increment (:my::Counter::IncrementResponse::Ok c))) (:wat::core::Vector :- [(:wat::service::Directed :- [:my::Counter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:my::counter::Op])]))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::process) :record (:my::counter::Record :count 0))
     c  (:wat::core::match (:wat::kernel::connect (:my::counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _  (:wat::core::match (:my::Counter/increment c (:my::Counter::IncrementRequest :n 5))
          ((:wat::kernel::RecvOutcome::Message _resp) nil)
          ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     r  (:my::Counter/get c (:my::Counter::GetRequest))]
    (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv
  ((:my::Counter::GetResponse::Ok value) value)
  ((:my::Counter::GetResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:my::Counter::GetResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::i64::to-string (:user::compute))))
