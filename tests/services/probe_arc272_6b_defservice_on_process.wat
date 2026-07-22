;; arc 278 S4c: the counter's protocol is LIFTED into an explicit surface (:my::Counter) the
;; service WEARS via :satisfies + :impls (the retired :ops clause is gone). IDENTICAL to the C.3
;; thread fixture except the locus is (process) — parity = the SAME client face on a forked process.
;; arc 278 S4c — the surface OWNS its protocol messages (:messages), so a :satisfies service
;; ships them across a process fork. (Was: external top-level defrecords — the forked child
;; never received them → StartupError. Now the surface's surface-forms carrier crosses them.)
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest        [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])
   (:wat::core::defrecord :my::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :my::Counter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(get       [self <- :my::Counter  req <- :my::Counter::GetRequest]       -> :my::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :my::Counter  req <- :my::Counter::IncrementRequest] -> :my::Counter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s
       (:my::Counter::GetResponse::Ok (:my::counter::Record/count (:my::counter::State/durable s)))))
   (increment [s req]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s))
                                             (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply (:my::counter::State :durable (:my::counter::Record :count c))
                                      (:my::Counter::IncrementResponse::Ok c))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::process) :record (:my::counter::Record :count 0))
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::Counter/increment c (:my::Counter::IncrementRequest :n 5))
     r  (:my::Counter/get c (:my::Counter::GetRequest))]
    (:wat::core::match r -> :wat::core::i64
      ((:my::Counter::GetResponse::Ok value) value)
      ;; terminal test caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:my::Counter::GetResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))
