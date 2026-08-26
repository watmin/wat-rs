;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; The surface (:nature :wat::kernel::Peer') synthesizes the Op/Reply enums from its features;
;; per-op Request/Response are user-declared records named `<Surface>::<Op>Request/Response`.
;; This probe still validates the GENERATED op enum (wrapped-record shape): the CAPITALIZED
;; variant `:my::Counter::Op::Increment` wraps the user-declared `:my::Counter::IncrementRequest`.
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest       [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :my::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :my::Counter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get       [self <- :my::Counter  req <- :my::Counter::GetRequest]       -> :my::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :my::Counter  req <- :my::Counter::IncrementRequest] -> :my::Counter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s (:my::Counter::GetResponse::Ok (:my::counter::Record/count (:my::counter::State/durable s)))))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply (:my::counter::State :durable (:my::counter::Record :count c)) (:my::Counter::IncrementResponse::Ok c))))])

;; Exercise the surface-synthesized op enum (wrapped-record shape):
;;   1. Build an IncrementRequest via the user-declared record constructor.
;;   2. Wrap it in the CAPITALIZED Op::Increment variant.
;;   3. Match: Get arm returns 0 (proves Op::Get exists wrapping GetRequest);
;;      Increment arm extracts n via IncrementRequest/n accessor → 5.
(:wat::core::defn :user::probe-op [] -> :wat::core::i64
  (:wat::core::let [req (:my::Counter::IncrementRequest :n 5)
                    op  (:my::Counter::Op::Increment req)]
    (:wat::core::match op 
      ((:my::Counter::Op::Get _r) 0)
      ((:my::Counter::Op::Increment req) (:my::Counter::IncrementRequest/n req)))))
