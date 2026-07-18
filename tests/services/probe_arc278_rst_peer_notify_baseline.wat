;; DESIGN-STONE-rst-peer-notify.md STEP-1 probe: a PROCESS service whose handler genuinely
;; panics; a SEPARATE connect'-ed client peer `c` reads the reply. At HEAD the client sees a
;; bare clean-EOF (RecvError::Disconnected), never a distinct reset — this is the RED baseline.
(:wat::core::defsurface :my::RstSvc :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::RstSvc::BoomRequest  [])
   (:wat::core::defrecord :my::RstSvc::BoomResponse [ok <- :wat::core::bool])]
  :features
  [(boom [self <- :my::RstSvc  req <- :my::RstSvc::BoomRequest] -> :my::RstSvc::BoomResponse)])

(:wat::service::defservice :my::rstsvc
  :satisfies :my::RstSvc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(boom [s req]
     (:wat::kernel::assertion-failed!
       "RST-BASELINE-SENTINEL-7731 — the handler crashed on purpose"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::rstsvc/start :locus (:wat::spawn::process) :record (:my::rstsvc::Record :count 0))
     c (:wat::kernel::connect' (:my::rstsvc::Handle/addr h))
     _ (:my::rstsvc/boom c (:my::RstSvc::BoomRequest))]
    true))
