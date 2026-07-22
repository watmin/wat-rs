;; Arc 278 — startup(:init)-crash reason parity RED gate.
;;
;; A defservice whose :init crashes with a KNOWN sentinel. The OWNER (:user::compute, which
;; holds the /start Handle) must learn WHY — the request must RAISE carrying the sentinel,
;; NOT deadlock (thread) and NOT collapse to a bare ECONNREFUSED with the reason discarded
;; (process). At HEAD: thread hangs; process → "connect abstract UDS: Connection refused"
;; (no sentinel). GREEN when an :init crash surfaces its reason to the owner, both loci.
;;
;; The .rs drives this on BOTH loci (thread + process) under a bounded harness.

(:wat::core::defsurface :t::Boom :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :t::Boom::PingRequest  [x <- :wat::core::i64])
   (:wat::core::defenum :t::Boom::PingResponse :wat::enum::Pure
     :Ok              [x <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(ping [self <- :t::Boom req <- :t::Boom::PingRequest] -> :t::Boom::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :t::boominit'
  :satisfies :t::Boom
  :durable []
  :init (:wat::core::fn
          [record <- :t::boominit'::Record]
          -> :t::boominit'::State
          (:wat::core::let
            [_ (:wat::kernel::assertion-failed! "BOOM-INIT-SENTINEL-99" :wat::core::None :wat::core::None)]
            (:t::boominit'::State :durable record)))
  :impls
  [(ping [s req]
     (:wat::service::Outcome::Reply s (:t::Boom::PingResponse::Ok 0)))])

;; The owner starts the crashing service and dials it. This MUST raise carrying the sentinel
;; (the :init crash reason reached the owner), not hang and not lose the reason.
;; THREAD locus — at HEAD this HUNG (bound-but-never-accepted address → connect' rendezvous
;; deadlock). GREEN: /start's crash-aware Started-wait raises the reason before connect' runs.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h   (:t::boominit'/start :locus (:wat::spawn::thread) :record (:t::boominit'::Record))
     svc (:wat::kernel::connect' (:t::boominit'::Handle/addr h))
     r   (:t::Boom/ping svc (:t::Boom::PingRequest :x 1))]
    (:wat::core::match r -> :wat::core::i64
      ((:t::Boom::PingResponse::Ok x) x)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:t::Boom::PingResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))

;; PROCESS locus — at HEAD /start SUCCEEDED (Started sent before :init ran) and the owner's
;; connect' collapsed to a bare ECONNREFUSED with the reason discarded. GREEN: the reordered
;; launch handshake (send ship → recv Started) makes an :init crash surface over the crash-aware
;; `recv' svc`, so /start raises the ProcessPanics envelope carrying the sentinel.
(:wat::core::defn :user::compute-process [] -> :wat::core::i64
  (:wat::core::let
    [h   (:t::boominit'/start :locus (:wat::spawn::process) :record (:t::boominit'::Record))
     svc (:wat::kernel::connect' (:t::boominit'::Handle/addr h))
     r   (:t::Boom/ping svc (:t::Boom::PingRequest :x 1))]
    (:wat::core::match r -> :wat::core::i64
      ((:t::Boom::PingResponse::Ok x) x)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:t::Boom::PingResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute-process: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))
