;; Isolate the EXACT crash-reason masking for a defservice: a trivial service whose op
;; crashes with a KNOWN sentinel (assertion-failed!). A connect'-ed client calls it.
;; Question: does the client's recv' carry "BOOM-SENTINEL-SVC-42", or a mute "peer closed"?
;; Run this on THREAD, then swap :locus to process, and compare — that tells us whether the
;; drop is thread-specific (unfuck threads) or the connect'-ed-client path (STOP-2, both loci).

(:wat::core::defsurface :t::Boom :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :t::Boom::PingRequest  [x <- :wat::core::i64])
   (:wat::core::defrecord :t::Boom::PingResponse [x <- :wat::core::i64])]
  :features
  [(ping [self <- :t::Boom req <- :t::Boom::PingRequest] -> :t::Boom::PingResponse)])

(:wat::service::defservice :t::boom'
  :satisfies :t::Boom
  :durable []
  :impls
  [(ping [s req]
     (:wat::service::Outcome::Reply s
       (:wat::core::let
         [_ (:wat::kernel::assertion-failed! "BOOM-SENTINEL-SVC-42" :wat::core::None :wat::core::None)]
         (:t::Boom::PingResponse :x 0))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h   (:t::boom'/start :locus (:wat::spawn::thread) :record (:t::boom'::Record))
     _p0 (:wat::kernel::println "main: started boom svc")
     svc (:wat::kernel::connect' (:t::boom'::Handle/addr h))
     _p1 (:wat::kernel::println "main: connected")
     r   (:t::Boom/ping svc (:t::Boom::PingRequest :x 1))]
    (:wat::kernel::println (:wat::core::string::concat "main: got " (:wat::core::str r)))))
