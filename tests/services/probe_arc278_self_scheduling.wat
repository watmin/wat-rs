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
(:wat::core::defsurface :probe::Ticker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Ticker::StartRequest [])
   (:wat::core::defenum :probe::Ticker::StartResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::Ticker::PollRequest [])
   (:wat::core::defenum :probe::Ticker::PollResponse :wat::enum::Pure
     :Count           [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start [self <- :probe::Ticker  req <- :probe::Ticker::StartRequest] -> :probe::Ticker::StartResponse
     :max-request-bytes 524288)
   (poll  [self <- :probe::Ticker  req <- :probe::Ticker::PollRequest]  -> :probe::Ticker::PollResponse
     :max-request-bytes 524288)])

;; ── the SELF-SCHEDULING service ──────────────────────────────────────────────────────────────────
(:wat::service::defservice :probe::ticker
  :satisfies :probe::Ticker
  :durable   [count <- :wat::core::i64  target <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::ticker::Record] -> :probe::ticker::State
          (:probe::ticker::State :durable record))
  :impls
  [;; client op: arm the FIRST tick (reply Ok + arm) — the tick re-arms itself thereafter.
   (start [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Ticker::Reply::Start (:probe::Ticker::StartResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Ticker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)]))

   ;; client op: reply the current count (proves the reactor serves clients between ticks).
   (poll [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Ticker::Reply::Poll (:probe::Ticker::PollResponse::Count
         (:probe::ticker::Record/count (:probe::ticker::State/durable s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Ticker::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::ticker::Op])])))

   ;; INTERNAL reactor op (leading dash): fire → +1; re-arm until target, else stop ticking.
   (-tick [s ctx]
     (:wat::core::let
       [rec  (:probe::ticker::State/durable s)
        n    (:wat::i64::+ (:probe::ticker::Record/count rec) 1)
        rec' (:probe::ticker::Record :count n :target (:probe::ticker::Record/target rec))
        s'   (:probe::ticker::State :durable rec')]
       (:wat::core::if (:wat::i64::< n (:probe::ticker::Record/target rec))
         (:wat::service::SelfOutcome::Continue s'
           (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Ticker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)])
         (:wat::service::SelfOutcome::Continue s' (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Ticker::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::ticker::Op])])))))])

;; ── nap — mora-honest wait (select' on a one-shot after; the driver runs on a thread) ─────────────
(:wat::core::defn :probe::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::select
      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
        (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done)))
    
    ((:wat::spawn::ServiceEvent::Message _i _m) nil)
    ((:wat::spawn::ServiceEvent::Closed _i) nil)
    ((:wat::spawn::ServiceEvent::Lost _i _c) nil)
    ((:wat::spawn::ServiceEvent::Malformed _i _c) nil)
    ((:wat::spawn::ServiceEvent::Rejected _i _c) nil)
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection _p) nil)
    ((:wat::spawn::ServiceEvent::Admin _m) nil)))

;; poll-until — a TCO poll-loop that terminates on the OBSERVED count reaching `target`, not on
;; elapsed time; polls DURING ticking (exercising "the reactor serves between ticks"), bounded by
;; a generous `attempts` failsafe with a small non-correctness-bearing `nap 5` backoff between polls.
(:wat::core::defn :probe::poll-until
  [c <- (:wat::kernel::Peer :- [:probe::Ticker::Op :probe::Ticker::Reply])  target <- :wat::core::i64  attempts <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= attempts 0)
    -2                                              ;; bound exhausted without reaching target
    (:wat::core::match (:probe::Ticker/poll c (:probe::Ticker::PollRequest))
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:probe::Ticker::PollResponse::Count n)
            (:wat::core::if (:wat::i64::>= n target)
              n                                     ;; observed the target — done, no timing guess
              (:wat::core::let [_ (:probe::nap 5)]  ;; bounded backoff, NOT a correctness-bearing sleep
                (:probe::poll-until c target (:wat::i64::- attempts 1)))))
          ((:probe::Ticker::PollResponse::RequestTooLarge _b _cp) -1)
          ((:probe::Ticker::PollResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; the shared driver: start a ticker at `target`, kick it, FACE the start outcome (a start-time
;; death now speaks), then poll-until the observed count reaches `target` — wire-synced, not a sleep-guess.
;; (a) the self-tick fired + re-armed to `target`; (b) poll still replied → the reactor kept serving.
(:wat::core::defn :probe::drive-ticker
  [h <- :probe::ticker::Handle] -> :wat::core::i64
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect (:probe::ticker::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _s (:probe::Ticker/start c (:probe::Ticker::StartRequest))
     ;; ⚠ THE DRIVE IS A BINDING, NOT THE BODY — and that is load-bearing, not style.
     ;; `h` is this fn's handle on the ticker, and a service dies when its owner's handle is
     ;; released. In the let's BODY the drive would be in TAIL position, which releases this
     ;; scope BEFORE the call runs (`eval_let_tail` propagates the tail-call signal out and the
     ;; scope goes with it) — so `h` would drop, the ticker would be severed, and `poll-until`
     ;; would face a dead service. That is what this test reported for 38 days, and it was read
     ;; as the timer being broken: `recv': peer closed`, blamed on a `remove-at` idx-shift that
     ;; was never real. Held here, the tick fires and re-arms at BOTH loci.
     ;; The severed-vs-clean-close distinction has its own gate,
     ;; tests/services/probe_severed_reaches_the_client.rs; the underlying tail-position release
     ;; is a live language defect, tracked separately and NOT what this test measures.
     n  (:wat::core::match _s
      ((:wat::kernel::RecvOutcome::Message __start)
        (:wat::core::match __start
          ((:probe::Ticker::StartResponse::Ok) (:probe::poll-until c 3 40))
          ((:probe::Ticker::StartResponse::RequestTooLarge _b _cp) -3)
          ((:probe::Ticker::StartResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))]
    n))

;; entrypoint (thread locus): expect the count == target (3).
(:wat::core::defn :user::self-tick-rearms-thread [] -> :wat::core::i64
  (:probe::drive-ticker
    (:probe::ticker/start :locus (:wat::spawn::thread)
      :record (:probe::ticker::Record :count 0 :target 3))))

;; entrypoint (process locus — env-grab arms the -tick at the process tier): expect count == target (3).
(:wat::core::defn :user::self-tick-rearms-process [] -> :wat::core::i64
  (:probe::drive-ticker
    (:probe::ticker/start :locus (:wat::spawn::process)
      :record (:probe::ticker::Record :count 0 :target 3))))
