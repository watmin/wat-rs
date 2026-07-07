;; arc 278 S4c: the counter's protocol is LIFTED into an explicit surface (:my::Counter,
;; :nature :wat::kernel::Peer') that the service WEARS via :satisfies + :impls. The old :ops
;; clause is retired. Drive ENTIRELY through the generated client face: start/Handle stay
;; per-service; the ops go via the surface (:my::Counter/increment, :my::Counter/get).
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
       (:my::Counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))
   (increment [s req]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s))
                                             (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply (:my::counter::State (:my::counter::Record c))
                                      (:my::Counter::IncrementResponse c))))])

;; Drive ENTIRELY through the generated client face: start → connect → surface method calls with
;; explicit request records. `h` stays bound for the whole let, so the service lives until compute
;; returns; scope-exit drops `h` → :Shutdown → join completes.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record 0))
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::Counter/increment c (:my::Counter::IncrementRequest 5))
     r  (:my::Counter/get c (:my::Counter::GetRequest))]
    (:my::Counter::GetResponse/value r)))
