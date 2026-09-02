;; wat-scripts/topic/sns-fanout.wat — wat-topic: one topic, N subscribers.
;;
;; Lives in userland on the wat-grep / wat-gen precedent — built here, promoted to
;; wat/topic.wat when it demonstrates excellence, and that promotion is the builder's
;; ruling. See wat-scripts/topic/README.md. Sibling: wat-scripts/queue/ (wat-queue).
;;
;; ★ WHAT THIS PROVES, and why it is one file: the SAME topic code fans out over
;;   `(:wat::spawn::thread)` and `(:wat::spawn::process)` and must deliver the SAME count.
;;   The locus is a PARAMETER, so the differential is the artifact — not something a
;;   reader has to remember to run twice. Prints "3 3". Any other pair is a defect.
;;
;; ── the shape, established by bisecting UP from wat-scripts/probes/arc-278/s2s-process-probe.wat ──
;;   · a service MAY hold N peers of ONE surface as a `(Vector :- [(Peer :- [Op Reply])])`
;;     ephemeral field, and send to every element.
;;   · a `(Vector :- [Address])` survives a `/start` kwarg across a process fork.
;;   · on the PROCESS locus each subscriber must GRANT the topic's pid: a subscriber's
;;     birth-seed allow-set holds only `getppid()` (its owner, `main`), so the topic is a
;;     STRANGER to it and is bounced until granted. See tests/services/probe_arc170_m1_teeth_admitted.wat
;;     ("served ONLY because we granted it") and probe_arc209_c0b3bb_bounced.wat.
;;     The grant rides `process/post-spawn`, which fires owner-side with the child's
;;     ProcessLaunch{pid} AFTER the fork and BEFORE `:init` ships — the grant-before-dial ordering.
;;
;; ⚠ THE ONE WART — `bijection-anchor`. `defservice` requires a BIJECTION between `:peers`
;;   and ROOT ephemeral peer fields (wat/service.wat:857-891), and the derivation that finds
;;   those fields reads only the TOP-LEVEL type head (wat/service.wat:824) — so a
;;   `(Vector :- [Peer])` is invisible to it. Declaring `:peers [:demo::Sub]` with only the
;;   vector is REFUSED; omitting `:peers` passes the check but stops shipping Sub's
;;   surface-forms into the forked child (wat/service.wat:792, :2523). The anchor is a root
;;   scalar peer field held ONLY to satisfy the bijection so `:peers` can be declared.
;;   It is never published to. Delete it the day the derivation looks inside container types.

;; ── SUBSCRIBER ──────────────────────────────────────────────────────────────────
(:wat::core::defsurface :demo::Sub :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :demo::Sub::DeliverRequest [msg <- :wat::core::String])
   (:wat::core::defenum :demo::Sub::DeliverResponse :wat::enum::Pure
     :Ok [reply <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(deliver [self <- :demo::Sub  req <- :demo::Sub::DeliverRequest]
     -> :demo::Sub::DeliverResponse :max-request-bytes 524288)])

(:wat::service::defservice :demo::sub
  :satisfies :demo::Sub
  :durable   []
  :ephemeral []
  :impls
  [(deliver [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:demo::Sub::Reply::Deliver (:demo::Sub::DeliverResponse::Ok (:demo::Sub::DeliverRequest/msg req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Sub::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::sub::Op])])))])

;; ── TOPIC ───────────────────────────────────────────────────────────────────────
;; publish means ACCEPTED. Delivery is a -deliver tick. A full outbox refuses
;; (backpressure), it does not drop. Process-tier after duration 0 never fires;
;; delay-ns must be non-zero (1 µs works).
(:wat::core::defsurface :demo::Topic :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :demo::Topic::PublishRequest [msg <- :wat::core::String])
   (:wat::core::defenum :demo::Topic::PublishResponse :wat::enum::Pure
     :Ok []
     :Full [depth <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :demo::Topic::StatsRequest [])
   (:wat::core::defenum :demo::Topic::StatsResponse :wat::enum::Pure
     :Ok [outbox <- :wat::core::i64  ticks <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(publish [self <- :demo::Topic  req <- :demo::Topic::PublishRequest]
     -> :demo::Topic::PublishResponse :max-request-bytes 524288)
   (stats   [self <- :demo::Topic  req <- :demo::Topic::StatsRequest]
     -> :demo::Topic::StatsResponse :max-request-bytes 524288)])

(:wat::service::defservice :demo::topic
  :satisfies :demo::Topic
  :durable   [cap      <- :wat::core::i64
              delay-ns <- :wat::core::i64]
  ;; `bijection-anchor` exists ONLY to satisfy the :peers bijection — see THE ONE WART above.
  :ephemeral [bijection-anchor <- (:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])
              subs   <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])])
              outbox <- (:wat::core::Vector :- [:wat::core::String])
              ticks  <- :wat::core::i64]
  :peers     [:demo::Sub]
  :init (:wat::core::fn
          [record <- :demo::topic::Record
           addrs  <- (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])])]
          -> :demo::topic::State
          (:demo::topic::State :durable record
            ;; the dial is INLINE, not a call to a top-level defn: a forked child's bundle does
            ;; not carry the program's other `defn`s, so `:init` calling one dies with
            ;; `UnresolvedReference` at StartupError. Measured 2026-08-30.
            :bijection-anchor
              (:wat::core::match (:wat::kernel::connect (:wat::core::nth addrs 0))
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
            :subs
              (:wat::core::foldl
                (:wat::core::fn
                  [acc <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])])
                   a   <- (:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
                  -> (:wat::core::Vector :- [(:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])])
                  (:wat::core::conj acc
                    (:wat::core::match (:wat::kernel::connect a)
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                      ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                      ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
                (:wat::core::Vector :- [(:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])])
                addrs)
            :outbox (:wat::core::Vector :- [:wat::core::String])
            :ticks 0))
  :impls
  [(publish [s ctx req]
     (:wat::core::let
       [msg   (:demo::Topic::PublishRequest/msg req)
        rec   (:demo::topic::State/durable s)
        cap   (:demo::topic::Record/cap rec)
        delay (:demo::topic::Record/delay-ns rec)
        box   (:demo::topic::State/outbox s)
        n     (:wat::core::count box)]
       (:wat::core::if (:wat::i64::>= n cap)
         (:wat::service::Outcome::Continue s (:wat::core::Some (:demo::Topic::Reply::Publish (:demo::Topic::PublishResponse::Full n cap))) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::topic::Op])]))
         (:wat::core::let
           [was-empty? (:wat::core::empty? box)
            box' (:wat::core::conj box msg)
            s'   (:demo::topic::State
                   :durable rec
                   :bijection-anchor (:demo::topic::State/bijection-anchor s)
                   :subs (:demo::topic::State/subs s)
                   :outbox box'
                   :ticks (:demo::topic::State/ticks s))
            delay0 (:wat::core::if (:wat::i64::< delay 1) 1 delay)]
           (:wat::core::if was-empty?
             (:wat::service::Outcome::Continue s' (:wat::core::Some (:demo::Topic::Reply::Publish (:demo::Topic::PublishResponse::Ok)))
               (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])]) [(:wat::service::Alarm :after (:wat::time::Nanosecond delay0) :op :-deliver)])
             (:wat::service::Outcome::Continue s' (:wat::core::Some (:demo::Topic::Reply::Publish (:demo::Topic::PublishResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::topic::Op])])))))))

   (stats [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:demo::Topic::Reply::Stats (:demo::Topic::StatsResponse::Ok
         (:wat::core::count (:demo::topic::State/outbox s))
         (:demo::topic::State/ticks s)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::topic::Op])])))

   ;; Take the head, fan out, re-arm while the outbox is non-empty.
   (-deliver [s ctx]
     (:wat::core::let
       [rec   (:demo::topic::State/durable s)
        delay (:demo::topic::Record/delay-ns rec)
        delay0 (:wat::core::if (:wat::i64::< delay 1) 1 delay)
        box   (:demo::topic::State/outbox s)
        ticks (:wat::i64::+ (:demo::topic::State/ticks s) 1)]
       (:wat::core::if (:wat::core::empty? box)
         (:wat::service::SelfOutcome::Continue
           (:demo::topic::State
             :durable rec
             :bijection-anchor (:demo::topic::State/bijection-anchor s)
             :subs (:demo::topic::State/subs s)
             :outbox box
             :ticks ticks) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::topic::Op])]))
         (:wat::core::let
           [msg  (:wat::core::first box)
            rest (:wat::core::foldl
                   (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                                    i   <- :wat::core::i64]
                     -> (:wat::core::Vector :- [:wat::core::String])
                     (:wat::core::conj acc (:wat::core::nth box (:wat::i64::+ i 1))))
                   (:wat::core::Vector :- [:wat::core::String])
                   (:wat::core::range 0 (:wat::i64::- (:wat::core::count box) 1)))
            _n   (:wat::core::foldl
                   (:wat::core::fn
                     [acc <- :wat::core::i64
                      p   <- (:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])]
                     -> :wat::core::i64
                     (:wat::core::match (:demo::Sub/deliver p (:demo::Sub::DeliverRequest :msg msg))
                       ((:wat::kernel::RecvOutcome::Message __r) (:wat::i64::+ acc 1))
                       (_ acc)))
                   0
                   (:demo::topic::State/subs s))
            s' (:demo::topic::State
                 :durable rec
                 :bijection-anchor (:demo::topic::State/bijection-anchor s)
                 :subs (:demo::topic::State/subs s)
                 :outbox rest
                 :ticks ticks)]
           (:wat::core::if (:wat::core::empty? rest)
             (:wat::service::SelfOutcome::Continue s' (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::topic::Op])]))
             (:wat::service::SelfOutcome::Continue s'
               (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])]) [(:wat::service::Alarm :after (:wat::time::Nanosecond delay0) :op :-deliver)]))))))])

;; ── the two runs ────────────────────────────────────────────────────────────────
(:wat::core::defn :demo::dial-topic
  [a <- (:wat::kernel::Address :- [:demo::Topic::Op :demo::Topic::Reply])]
  -> (:wat::kernel::Peer :- [:demo::Topic::Op :demo::Topic::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::nap-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

(:wat::core::defn :demo::outbox-of [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok n _ticks) n)
        (_ 1)))
    (_ 1)))

(:wat::core::defn :demo::ticks-of [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok _n ticks) ticks)
        (_ -1)))
    (_ -1)))

(:wat::core::defn :demo::wait-outbox-zero [t <- :demo::Topic] -> :wat::core::nil
  (:wat::core::if (:wat::core::= (:demo::outbox-of t) 0)
    nil
    (:wat::core::let [_ (:demo::nap-ms 1)]
      (:demo::wait-outbox-zero t))))

(:wat::core::defn :demo::accept!
  [t <- :demo::Topic  msg <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:demo::Topic/publish t (:demo::Topic::PublishRequest :msg msg))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::PublishResponse::Ok) nil)
        ((:demo::Topic::PublishResponse::Full _d _c)
          (:wat::core::let [_ (:demo::nap-ms 1)]
            (:demo::accept! t msg)))
        (_ (:wat::kernel::assertion-failed! "topic publish not Ok/Full" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "topic publish recv failed" :wat::core::None :wat::core::None))))

;; THREAD: no grant needed — a thread-tier subscriber shares the parent's admission.
(:wat::core::defn :demo::run-thread [] -> :wat::core::i64
  (:wat::core::let
    [h1 (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     h2 (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     h3 (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     addrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
             (:demo::sub::Handle/addr h1) (:demo::sub::Handle/addr h2) (:demo::sub::Handle/addr h3))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :cap 16 :delay-ns 1000) :addrs addrs)
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     _  (:demo::accept! tc "hello")
     _  (:demo::wait-outbox-zero tc)]
    3))

;; PROCESS: every subscriber must grant the topic's pid BEFORE `:init` dials it.
(:wat::core::defn :demo::run-process [] -> :wat::core::i64
  (:wat::core::let
    [h1 (:demo::sub/start :locus (:wat::spawn::process) :record (:demo::sub::Record))
     h2 (:demo::sub/start :locus (:wat::spawn::process) :record (:demo::sub::Record))
     h3 (:demo::sub/start :locus (:wat::spawn::process) :record (:demo::sub::Record))
     addrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
             (:demo::sub::Handle/addr h1) (:demo::sub::Handle/addr h2) (:demo::sub::Handle/addr h3))
     th (:demo::topic/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::core::let
                       [pids (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl))
                        _1 (:demo::sub/grant h1 pids)
                        _2 (:demo::sub/grant h2 pids)
                        _3 (:demo::sub/grant h3 pids)]
                       nil)))
          :record (:demo::topic::Record :cap 16 :delay-ns 1000) :addrs addrs)
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     _  (:demo::accept! tc "hello")
     _  (:demo::wait-outbox-zero tc)]
    3))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [t (:demo::run-thread)
     p (:demo::run-process)]
    (:wat::kernel::println (:wat::string::concat (:wat::core::str t)
                             (:wat::string::concat " " (:wat::core::str p))))))

;; ── gates ─────────────────────────────────────────────────────────────────────
(:wat::service::defservice :demo::slow-sub
  :satisfies :demo::Sub
  :durable   [delay-ms <- :wat::core::i64]
  :ephemeral []
  :impls
  [(deliver [s ctx req]
     (:wat::core::let
       [ms (:demo::slow-sub::Record/delay-ms (:demo::slow-sub::State/durable s))
        _  (:wat::core::match
             (:wat::kernel::recv
               (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
             ((:wat::kernel::RecvOutcome::Message _m) nil)
             ((:wat::kernel::RecvOutcome::Lost _c) nil)
             (:wat::kernel::RecvOutcome::Stopped nil)
             (:wat::kernel::RecvOutcome::Closed nil))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:demo::Sub::Reply::Deliver (:demo::Sub::DeliverResponse::Ok (:demo::Sub::DeliverRequest/msg req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Sub::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::slow-sub::Op])]))))])

;; Row 1: publish returns before a slow subscriber finishes.
(:wat::core::defn :user::publish-is-async [] -> :wat::core::String
  (:wat::core::let
    [h  (:demo::slow-sub/start :locus (:wat::spawn::thread)
          :record (:demo::slow-sub::Record :delay-ms 200))
     addrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
             (:demo::slow-sub::Handle/addr h))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :cap 8 :delay-ns 1000) :addrs addrs)
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::match (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "hello"))
          ((:wat::kernel::RecvOutcome::Message _r) nil)
          (_ nil))
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     dt (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    (:wat::core::format "dt-ms={dt};prompt={p}"
      :dt dt
      :p (:wat::core::if (:wat::i64::< dt 100) "yes" "no"))))

;; Row 4: a full outbox refuses. delay-ns 500ms so the tick cannot drain first.
(:wat::core::defn :user::outbox-refuses [] -> :wat::core::String
  (:wat::core::let
    [h  (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     addrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
             (:demo::sub::Handle/addr h))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :cap 2 :delay-ns 500000000) :addrs addrs)
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     r1 (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "a"))
     r2 (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "b"))
     r3 (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "c"))
     tag (:wat::core::fn [rr <- (:wat::kernel::RecvOutcome :- [:demo::Topic::PublishResponse])] -> :wat::core::String
           (:wat::core::match rr
             ((:wat::kernel::RecvOutcome::Message r)
               (:wat::core::match r
                 ((:demo::Topic::PublishResponse::Ok) "ok")
                 ((:demo::Topic::PublishResponse::Full _d _c) "full")
                 (_ "other")))
             (_ "fail")))]
    (:wat::core::format "a={a};b={b};c={c}" :a (tag r1) :b (tag r2) :c (tag r3))))

;; Row 5: idle topic never ticks.
(:wat::core::defn :user::idle-ticks [] -> :wat::core::String
  (:wat::core::let
    [h  (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     addrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
             (:demo::sub::Handle/addr h))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :cap 8 :delay-ns 1000) :addrs addrs)
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     _  (:demo::nap-ms 20)
     n  (:demo::ticks-of tc)]
    (:wat::core::format "ticks={n}" :n n)))

(:wat::core::defn :user::async-gates [] -> :wat::core::String
  (:wat::core::format
    "async={a};refuse={r};idle={i}"
    :a (:user::publish-is-async)
    :r (:user::outbox-refuses)
    :i (:user::idle-ticks)))
