;; DESIGN-STONE-rst-peer-notify.md STEP-1 probe: a PROCESS service whose handler genuinely
;; panics; a SEPARATE connect'-ed client peer `c` reads the reply. At HEAD the client sees a
;; bare clean-EOF (RecvError::Disconnected), never a distinct reset — this is the RED baseline.
(:wat::core::defsurface :my::RstSvc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::RstSvc::BoomRequest  [])
   (:wat::core::defenum :my::RstSvc::BoomResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(boom [self <- :my::RstSvc  req <- :my::RstSvc::BoomRequest] -> :my::RstSvc::BoomResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::rstsvc
  :satisfies :my::RstSvc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(boom [s ctx req]
     (:wat::kernel::assertion-failed!
       "RST-BASELINE-SENTINEL-7731 — the handler crashed on purpose"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))])

;; arc 278 recv'-wall: the generated client method `/boom` returns a matchable (RecvOutcome :- [BoomResponse])
;; VALUE, never a raise. A genuine far-side handler panic makes the client's recv' surface a DISTINCT
;; ::Lost (a reason-free 500 — the crash reason is administrative, owner-channel-only), NOT a bare
;; clean-EOF ::Closed (the old mute disconnect) and NOT a fake ::Message. We MATCH and RETURN a marker:
;; "LOST:<reason-free msg>" on the crash, "MESSAGE"/"CLOSED" otherwise — the .rs asserts the client saw
;; the peer crashed (::Lost), distinct from a bare disconnect (::Closed), carrying no crash sentinel.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [h (:my::rstsvc/start :locus (:wat::spawn::process) :record (:my::rstsvc::Record :count 0))
     c (:wat::core::match (:wat::kernel::connect (:my::rstsvc::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))]
    (:wat::core::match (:my::rstsvc/boom c (:my::RstSvc::BoomRequest))
      ((:wat::kernel::RecvOutcome::Message _m) "MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost _cause) "LOST")
      (:wat::kernel::RecvOutcome::Stopped "STOPPED")
      (:wat::kernel::RecvOutcome::Closed "CLOSED"))))
