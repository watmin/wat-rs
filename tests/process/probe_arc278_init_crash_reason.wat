;; Arc 278 — startup(:init)-crash reason parity RED gate.
;;
;; A defservice whose :init crashes with a KNOWN sentinel. The OWNER (:user::compute, which
;; holds the /start Handle) must learn WHY — the request must RAISE carrying the sentinel,
;; NOT deadlock (thread) and NOT collapse to a bare ECONNREFUSED with the reason discarded
;; (process). At HEAD: thread hangs; process → "connect abstract UDS: Connection refused"
;; (no sentinel). GREEN when an :init crash surfaces its reason to the owner, both loci.
;;
;; The .rs drives this on BOTH loci (thread + process) under a bounded harness.

(:wat::core::defsurface :t::Boom :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :t::Boom::PingRequest  [x <- :wat::core::i64])
   (:wat::core::defenum :t::Boom::PingResponse :wat::enum::Pure
     :Ok              [x <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :t::Boom req <- :t::Boom::PingRequest] -> :t::Boom::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :t::boominit
  :satisfies :t::Boom
  :durable []
  :init (:wat::core::fn
          [record <- :t::boominit::Record]
          -> :t::boominit::State
          (:wat::core::let
            [_ (:wat::kernel::assertion-failed! "BOOM-INIT-SENTINEL-99" :wat::core::None :wat::core::None)]
            (:t::boominit::State :durable record)))
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:t::Boom::Reply::Ping (:t::Boom::PingResponse::Ok 0))) (:wat::core::Vector :- [(:wat::service::Directed :- [:t::Boom::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:t::boominit::Op])])))])

;; The owner starts the crashing service and dials it. This MUST raise carrying the sentinel
;; (the :init crash reason reached the owner), not hang and not lose the reason.
;; THREAD locus — at HEAD this HUNG (bound-but-never-accepted address → connect' rendezvous
;; deadlock). GREEN: /start's crash-aware Started-wait raises the reason before connect' runs.
;; Arc 278 recv'-wall: the :init crash surfaces to the OWNER. The launch handshake's crash-aware
;; recv' inside `/start` (wat/spawn.wat) gets the crash as a matchable RecvOutcome::Lost and — because
;; /start's contract returns a Handle (there is no value channel for a start-failure) — re-raises it,
;; so the OWNER's `/start` call raises the reason (the .rs catches that raise). If a tier instead
;; surfaced the crash at the ping's recv' (a matchable ::Lost VALUE), this body RETURNS the reason as
;; a String; the .rs handles both (raise-at-/start OR value-at-ping) and asserts the sentinel.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [h   (:t::boominit/start :locus (:wat::spawn::thread) :record (:t::boominit::Record))
     svc (:wat::core::match (:wat::kernel::connect (:t::boominit::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r   (:t::Boom/ping svc (:t::Boom::PingRequest :x 1))]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:t::Boom::PingResponse::Ok x) "UNEXPECTED-OK")
          ((:t::Boom::PingResponse::RequestTooLarge bytes cap) "UNEXPECTED-TOO-LARGE")
          ((:t::Boom::PingResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::LociDiedError/message __cause))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; PROCESS locus — at HEAD /start SUCCEEDED (Started sent before :init ran) and the owner's
;; connect' collapsed to a bare ECONNREFUSED with the reason discarded. GREEN: the reordered
;; launch handshake (send ship → recv Started) makes an :init crash surface over the crash-aware
;; `recv' svc`, so /start raises the ProcessPanics envelope carrying the sentinel.
(:wat::core::defn :user::compute-process [] -> :wat::core::String
  (:wat::core::let
    [h   (:t::boominit/start :locus (:wat::spawn::process) :record (:t::boominit::Record))
     svc (:wat::core::match (:wat::kernel::connect (:t::boominit::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r   (:t::Boom/ping svc (:t::Boom::PingRequest :x 1))]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:t::Boom::PingResponse::Ok x) "UNEXPECTED-OK")
          ((:t::Boom::PingResponse::RequestTooLarge bytes cap) "UNEXPECTED-TOO-LARGE")
          ((:t::Boom::PingResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::LociDiedError/message __cause))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
