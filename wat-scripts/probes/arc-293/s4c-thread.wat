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
     (:wat::service::Outcome::Reply {:state s
       :reply (:my::Counter::GetResponse::Ok {:value (:my::counter::Record/count (:my::counter::State/durable s))})}))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+ (:my::counter::Record/count (:my::counter::State/durable s))
                                             (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply {:state (:my::counter::State :durable (:my::counter::Record :count c))
                                      :reply (:my::Counter::IncrementResponse::Ok {:value c})})))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record :count 0))
     c  (:wat::core::match (:wat::kernel::connect (:my::counter::Handle/addr h)) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     _  (:wat::core::match (:my::Counter/increment c (:my::Counter::IncrementRequest :n 5))
          [:wat::kernel::RecvOutcome::Message {:msg _resp} nil]
          [:wat::kernel::RecvOutcome::Lost {:cause _c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message _c))]
          [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
     r  (:my::Counter/get c (:my::Counter::GetRequest))]
    (:wat::core::match r [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv
  [:my::Counter::GetResponse::Ok {:value value} value]
  [:my::Counter::GetResponse::RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:my::Counter::GetResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::i64::to-string (:user::compute))))
