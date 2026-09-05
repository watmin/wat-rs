;; wat-tests/service-hibernate-resume.wat — arc 291 strike-4a RED probe: the soul, digitized.
;;
;; THE PROPHECY, proven at the surface (R1 PROBANDUM EST → PROBATUM EST): a service's live State survives
;; TERMINATION and reanimates elsewhere, none the wiser. `hibernate` renders the State to a ::Record (the
;; EDN soul) and terminates the service; `resume` spawns a FRESH service whose initial State IS rebuilt
;; from the Record via `:init`, bypassing pre-built state. resume : snapshot :: start : init-args.
;;
;; The proof: increment to 7 → hibernate (service DIES, returns the count-7 Record) → resume a fresh
;; service from the Record → increment 3 on the reborn service → stop returns 10. The reborn service
;; CONTINUED from the hibernated state (7 + 3 = 10); on the process tier the Record crossed as EDN — the
;; only bridge across the process death. The service cannot tell it was reborn.
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; hibernate returns ::Record (the soul). resume takes ::Record.
;; :stop projects State → i64 by reading through State/durable.
;; :init defaults (ephemeral empty). start takes ::Record(0).
;; Op body reads through State/durable. State building uses State/new (Record c).

;; ── the surface (the counter protocol, lifted) ───────────────────────────────
;; arc 278 S4c: the surface OWNS its protocol messages (:messages) so a :satisfies
;; service ships them across a process fork.
(:wat::core::defsurface :wat-tests::HibCounter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::HibCounter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::HibCounter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(increment [self <- :wat-tests::HibCounter  req <- :wat-tests::HibCounter::IncrementRequest] -> :wat-tests::HibCounter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat-tests::hib-counter
  :satisfies :wat-tests::HibCounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+
                           (:wat-tests::hib-counter::Record/count (:wat-tests::hib-counter::State/durable s))
                           (:wat-tests::HibCounter::IncrementRequest/n req))]
       (:wat::service::Outcome::Continue
         (:wat-tests::hib-counter::State :durable (:wat-tests::hib-counter::Record :count c))
         (:wat::core::Some (:wat-tests::HibCounter::Reply::Increment (:wat-tests::HibCounter::IncrementResponse::Ok c))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::HibCounter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat-tests::hib-counter::Op])]))))  ]
  ;; :stop projects State → i64 (the count) via State/durable
  :stop (:wat::core::fn [s <- :wat-tests::hib-counter::State] -> :wat::core::i64
          (:wat-tests::hib-counter::Record/count (:wat-tests::hib-counter::State/durable s))))

;; ── thread tier ──────────────────────────────────────────────────────────────
;; 7 → hibernate (::Record snapshot, service dies) → resume fresh → +3 → stop = 10.
(:wat::test::deftest :wat-tests::service::hibernate-resume-on-thread
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h     (:wat-tests::hib-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::hib-counter::Record :count 0))
       c     (:wat::core::match (:wat::kernel::connect (:wat-tests::hib-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _     (:wat::core::match (:wat-tests::HibCounter/increment c (:wat-tests::HibCounter::IncrementRequest :n 7))
               ((:wat::kernel::RecvOutcome::Message _resp) nil)
               ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
               (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
               (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
       snap  (:wat-tests::hib-counter/hibernate h)
       h2    (:wat-tests::hib-counter/resume :locus (:wat::spawn::thread) :record snap)
       c2    (:wat::core::match (:wat::kernel::connect (:wat-tests::hib-counter::Handle/addr h2)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _2    (:wat-tests::HibCounter/increment c2 (:wat-tests::HibCounter::IncrementRequest :n 3))
       final (:wat-tests::hib-counter/stop h2)]
      final)
    10))

;; ── process tier — IDENTICAL except the locus token (the Record snapshot crosses as EDN) ──
(:wat::test::deftest :wat-tests::service::hibernate-resume-on-process
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h     (:wat-tests::hib-counter/start :locus (:wat::spawn::process) :record (:wat-tests::hib-counter::Record :count 0))
       c     (:wat::core::match (:wat::kernel::connect (:wat-tests::hib-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _     (:wat::core::match (:wat-tests::HibCounter/increment c (:wat-tests::HibCounter::IncrementRequest :n 7))
               ((:wat::kernel::RecvOutcome::Message _resp) nil)
               ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
               (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
               (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
       snap  (:wat-tests::hib-counter/hibernate h)
       h2    (:wat-tests::hib-counter/resume :locus (:wat::spawn::process) :record snap)
       c2    (:wat::core::match (:wat::kernel::connect (:wat-tests::hib-counter::Handle/addr h2)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _2    (:wat-tests::HibCounter/increment c2 (:wat-tests::HibCounter::IncrementRequest :n 3))
       final (:wat-tests::hib-counter/stop h2)]
      final)
    10))
