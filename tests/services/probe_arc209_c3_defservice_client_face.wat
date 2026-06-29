(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply (:my::counter::State (:my::counter::Record c)) (:my::counter::IncrementResponse c))))])

;; Drive ENTIRELY through the generated client face: start → connect → method calls via request
;; constructors. `h` stays bound for the whole let, so the service lives until compute returns;
;; scope-exit drops `h` → :Shutdown → join completes.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record 0))
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::counter/increment c (:my::counter/increment-request 5))
     r  (:my::counter/get c (:my::counter/get-request))]
    (:my::counter::GetResponse/value r)))
