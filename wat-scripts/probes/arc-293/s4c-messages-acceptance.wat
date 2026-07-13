;; arc 278 S4c ACCEPTANCE — a `:messages` peer surface OWNS its protocol and SHIPS it across a
;; process fork. The counter's Request/Response records live INSIDE the surface's :messages block
;; (not as external top-level forms). The service :satisfies the surface; started on (process) the
;; forked child boots stdlib + service-forms, and service-forms now concats the surface's
;; surface-forms carrier → the child resolves ::Op/::Reply + the message records at its fresh
;; startup. Round-trip: start (process) 0 → connect → increment 5 → get → prints 5.

(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest        [])
   (:wat::core::defrecord :my::Counter::GetResponse       [value <- :wat::core::i64])
   (:wat::core::defrecord :my::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defrecord :my::Counter::IncrementResponse [value <- :wat::core::i64])]
  :features
  [(get       [self <- :my::Counter  req <- :my::Counter::GetRequest]       -> :my::Counter::GetResponse)
   (increment [self <- :my::Counter  req <- :my::Counter::IncrementRequest] -> :my::Counter::IncrementResponse)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s
       (:my::Counter::GetResponse :value (:my::counter::Record/count (:my::counter::State/durable s)))))
   (increment [s req]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s))
                                             (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply (:my::counter::State :durable (:my::counter::Record :count c))
                                      (:my::Counter::IncrementResponse :value c))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::process) :record (:my::counter::Record :count 0))
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::Counter/increment c (:my::Counter::IncrementRequest :n 5))
     r  (:my::Counter/get c (:my::Counter::GetRequest))]
    (:my::Counter::GetResponse/value r)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::i64::to-string (:user::compute))))
