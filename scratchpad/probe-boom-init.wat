;; Isolate: does a crash in :INIT (before the serve loop) get honestly reported (PeerCrashed,
;; like a serve-loop crash) or MASKED as clean-EOF + deadlock (the sift symptom)?
;; Same trivial service as probe-boom-svc, but the crash is in :init, not the op.

(:wat::core::defsurface :t::Boom :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :t::Boom::PingRequest  [x <- :wat::core::i64])
   (:wat::core::defrecord :t::Boom::PingResponse [x <- :wat::core::i64])]
  :features
  [(ping [self <- :t::Boom req <- :t::Boom::PingRequest] -> :t::Boom::PingResponse)])

(:wat::service::defservice :t::boominit'
  :satisfies :t::Boom
  :durable []
  :init (:wat::core::fn
          [record <- :t::boominit'::Record]
          -> :t::boominit'::State
          (:wat::core::let
            [_i0 (:wat::kernel::println "init: entered")
             _   (:wat::kernel::assertion-failed! "BOOM-INIT-SENTINEL-99" :wat::core::None :wat::core::None)]
            (:t::boominit'::State :durable record)))
  :impls
  [(ping [s req]
     (:wat::service::Outcome::Reply s (:t::Boom::PingResponse :x 0)))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h   (:t::boominit'/start :locus (:wat::spawn::process) :record (:t::boominit'::Record))
     _p0 (:wat::kernel::println "main: started boominit svc")
     svc (:wat::kernel::connect' (:t::boominit'::Handle/addr h))
     _p1 (:wat::kernel::println "main: connected")
     r   (:t::Boom/ping svc (:t::Boom::PingRequest :x 1))]
    (:wat::kernel::println (:wat::core::string::concat "main: got " (:wat::core::str r)))))
