;; wat-scripts/demos/sns/sns-fanout.wat — SNS in userland: one topic, N subscribers.
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
     (:wat::service::Outcome::Reply s
       (:demo::Sub::DeliverResponse::Ok (:demo::Sub::DeliverRequest/msg req))))])

;; ── TOPIC ───────────────────────────────────────────────────────────────────────
(:wat::core::defsurface :demo::Topic :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :demo::Topic::PublishRequest [msg <- :wat::core::String])
   (:wat::core::defenum :demo::Topic::PublishResponse :wat::enum::Pure
     :Ok [delivered <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(publish [self <- :demo::Topic  req <- :demo::Topic::PublishRequest]
     -> :demo::Topic::PublishResponse :max-request-bytes 524288)])

(:wat::service::defservice :demo::topic
  :satisfies :demo::Topic
  :durable   []
  ;; `bijection-anchor` exists ONLY to satisfy the :peers bijection — see THE ONE WART above.
  :ephemeral [bijection-anchor <- (:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])
              subs <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])])]
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
                addrs)))
  :impls
  [(publish [s ctx req]
     (:wat::core::let
       [msg (:demo::Topic::PublishRequest/msg req)
        n   (:wat::core::foldl
              (:wat::core::fn
                [acc <- :wat::core::i64
                 p   <- (:wat::kernel::Peer :- [:demo::Sub::Op :demo::Sub::Reply])]
                -> :wat::core::i64
                (:wat::core::match (:demo::Sub/deliver p (:demo::Sub::DeliverRequest :msg msg))
                  ((:wat::kernel::RecvOutcome::Message __r)
                    (:wat::core::match __r
                      ((:demo::Sub::DeliverResponse::Ok _reply) (:wat::i64::+ acc 1))
                      (_ acc)))
                  (_ acc)))
              0
              (:demo::topic::State/subs s))]
       (:wat::service::Outcome::Reply s (:demo::Topic::PublishResponse::Ok n))))])

;; ── the two runs ────────────────────────────────────────────────────────────────
(:wat::core::defn :demo::dial-topic
  [a <- (:wat::kernel::Address :- [:demo::Topic::Op :demo::Topic::Reply])]
  -> (:wat::kernel::Peer :- [:demo::Topic::Op :demo::Topic::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::read-count
  [rr <- (:wat::kernel::RecvOutcome :- [:demo::Topic::PublishResponse])] -> :wat::core::i64
  (:wat::core::match rr
    ((:wat::kernel::RecvOutcome::Message __r)
      (:wat::core::match __r
        ((:demo::Topic::PublishResponse::Ok delivered) delivered)
        (_ -1)))
    (_ -1)))

;; THREAD: no grant needed — a thread-tier subscriber shares the parent's admission.
(:wat::core::defn :demo::run-thread [] -> :wat::core::i64
  (:wat::core::let
    [h1 (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     h2 (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     h3 (:demo::sub/start :locus (:wat::spawn::thread) :record (:demo::sub::Record))
     addrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
             (:demo::sub::Handle/addr h1) (:demo::sub::Handle/addr h2) (:demo::sub::Handle/addr h3))
     th (:demo::topic/start :locus (:wat::spawn::thread) :record (:demo::topic::Record) :addrs addrs)
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))]
    (:demo::read-count (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "hello")))))

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
          :record (:demo::topic::Record) :addrs addrs)
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))]
    (:demo::read-count (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "hello")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [t (:demo::run-thread)
     p (:demo::run-process)]
    (:wat::kernel::println (:wat::string::concat (:wat::core::str t)
                             (:wat::string::concat " " (:wat::core::str p))))))
