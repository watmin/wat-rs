;; Arc 278 item (c) — RED gate: SELF-SCHEDULING DEFSERVICES (the substrate stone).
;; See docs/arc/2026/06/278-rules-engine/DESIGN-self-scheduling-defservices.md.
;;
;; A service arms a `-tick` op — LEADING DASH = reactor-internal: NOT on the `:satisfies` surface (no
;; client can name it), a member of the service's own select loop. It is armed via an `Alarm` carried
;; in the handler's `Outcome`, fires on the service's own timer (env-grab tier → both loci), re-arms
;; itself, and advances a durable counter; meanwhile a client `poll` still replies (the reactor serves
;; between ticks). The mechanism is proven hand-rolled in
;; wat-scripts/scratch-pad/probe-self-scheduling-loop.wat — this gate proves the GENERATED serve loop.
;;
;; RED at HEAD: no `Alarm` / `Outcome::ReplyAndArm` / `Outcome::NoReplyAndArm`; a `-tick` op cannot be
;; armed; the serve loop threads `clients`, not `selectables`; the leading dash is dropped by
;; kebab->pascal. GREEN when the stone lands (the count reaches `target`, poll still replies).

;; ── the WIRE surface — the ONLY client-callable ops (poll; start = kick the first tick) ───────────
(:wat::core::defsurface :probe::Ticker :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Ticker::StartRequest [])
   (:wat::core::defenum :probe::Ticker::StartResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])
   (:wat::core::defrecord :probe::Ticker::PollRequest [])
   (:wat::core::defenum :probe::Ticker::PollResponse :wat::enum::Pure
     :Count           [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(start [self <- :probe::Ticker  req <- :probe::Ticker::StartRequest] -> :probe::Ticker::StartResponse
     :max-request-bytes 524288)
   (poll  [self <- :probe::Ticker  req <- :probe::Ticker::PollRequest]  -> :probe::Ticker::PollResponse
     :max-request-bytes 524288)])

;; ── the SELF-SCHEDULING service ──────────────────────────────────────────────────────────────────
(:wat::service::defservice :probe::ticker'
  :satisfies :probe::Ticker
  :durable   [count <- :wat::core::i64  target <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::ticker'::Record] -> :probe::ticker'::State
          (:probe::ticker'::State :durable record))
  :impls
  [;; client op: arm the FIRST tick (reply Ok + arm) — the tick re-arms itself thereafter.
   (start [s req]
     (:wat::service::Outcome::ReplyAndArm s (:probe::Ticker::StartResponse::Ok)
       [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)]))

   ;; client op: reply the current count (proves the reactor serves clients between ticks).
   (poll [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Ticker::PollResponse::Count
         (:probe::ticker'::Record/count (:probe::ticker'::State/durable s)))))

   ;; INTERNAL reactor op (leading dash): fire → +1; re-arm until target, else stop ticking.
   (-tick [s]
     (:wat::core::let
       [rec  (:probe::ticker'::State/durable s)
        n    (:wat::core::i64::+ (:probe::ticker'::Record/count rec) 1)
        rec' (:probe::ticker'::Record :count n :target (:probe::ticker'::Record/target rec))
        s'   (:probe::ticker'::State :durable rec')]
       (:wat::core::if (:wat::core::i64::< n (:probe::ticker'::Record/target rec))
         (:wat::service::Outcome::NoReplyAndArm s'
           [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)])
         (:wat::service::Outcome::NoReply s'))))])

;; ── nap — mora-honest wait (select' on a one-shot after; the driver runs on a thread) ─────────────
(:wat::core::defn :probe::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::select'
      (:wat::core::Vector :wat::kernel::Peer'<wat::core::nil,wat::core::keyword>
        (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done)))
    
    ((:wat::spawn::ServiceEvent::Message _i _m) nil)
    ((:wat::spawn::ServiceEvent::Closed _i) nil)
    ((:wat::spawn::ServiceEvent::Lost _i _c) nil)
    ((:wat::spawn::ServiceEvent::Malformed _i _c) nil)
    ((:wat::spawn::ServiceEvent::Rejected _i _c) nil)
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection _p) nil)
    ((:wat::spawn::ServiceEvent::Admin _m) nil)))

;; the shared driver: start a ticker at `target`, kick it, nap past `target` ticks, poll → the count.
;; (a) the self-tick fired + re-armed to `target`; (b) poll still replied → the reactor kept serving.
(:wat::core::defn :probe::drive-ticker
  [h <- :probe::ticker'::Handle] -> :wat::core::i64
  (:wat::core::let
    [c  (:wat::kernel::connect' (:probe::ticker'::Handle/addr h))
     _s (:probe::Ticker/start c (:probe::Ticker::StartRequest))
     _w (:probe::nap 100)                                          ;; 100ms >> target ticks @ 5ms — robust
     r  (:probe::Ticker/poll c (:probe::Ticker::PollRequest))]
    (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:probe::Ticker::PollResponse::Count n) n)
      ((:probe::Ticker::PollResponse::RequestTooLarge _b _cp) -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; entrypoint (thread locus): expect the count == target (3).
(:wat::core::defn :user::self-tick-rearms-thread [] -> :wat::core::i64
  (:probe::drive-ticker
    (:probe::ticker'/start :locus (:wat::spawn::thread)
      :record (:probe::ticker'::Record :count 0 :target 3))))

;; entrypoint (process locus — env-grab arms the -tick at the process tier): expect count == target (3).
(:wat::core::defn :user::self-tick-rearms-process [] -> :wat::core::i64
  (:probe::drive-ticker
    (:probe::ticker'/start :locus (:wat::spawn::process)
      :record (:probe::ticker'::Record :count 0 :target 3))))
