;; arc 278 S4c: the counter's protocol is LIFTED into an explicit surface (:my::Counter) the
;; service WEARS via :satisfies + :impls (the retired :ops clause is gone). Same client-face
;; round-trip as C.3, but start now takes a LOCUS — `(thread)` selects the shared-memory launch.
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

;; Drive through the client face; start takes a LOCUS — `(thread)` selects the shared-memory
;; launch via the Locus protocol. Same round-trip as C.3 (increment 5 → get → 5).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record :count 0))
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::Counter/increment c (:my::Counter::IncrementRequest :n 5))
     r  (:my::Counter/get c (:my::Counter::GetRequest))]
    (:my::Counter::GetResponse/value r)))
