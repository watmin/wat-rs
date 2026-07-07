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
(:wat::core::defsurface :wat-tests::HibCounter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat-tests::HibCounter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defrecord :wat-tests::HibCounter::IncrementResponse [value <- :wat::core::i64])]
  :features
  [(increment [self <- :wat-tests::HibCounter  req <- :wat-tests::HibCounter::IncrementRequest] -> :wat-tests::HibCounter::IncrementResponse)])

(:wat::service::defservice :wat-tests::hib-counter
  :satisfies :wat-tests::HibCounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(increment [s req]
     (:wat::core::let [c (:wat::core::i64::+
                           (:wat-tests::hib-counter::Record/count (:wat-tests::hib-counter::State/durable s))
                           (:wat-tests::HibCounter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply
         (:wat-tests::hib-counter::State (:wat-tests::hib-counter::Record c))
         (:wat-tests::HibCounter::IncrementResponse c))))  ]
  ;; :stop projects State → i64 (the count) via State/durable
  :stop (:wat::core::fn [s <- :wat-tests::hib-counter::State] -> :wat::core::i64
          (:wat-tests::hib-counter::Record/count (:wat-tests::hib-counter::State/durable s))))

;; ── thread tier ──────────────────────────────────────────────────────────────
;; 7 → hibernate (::Record snapshot, service dies) → resume fresh → +3 → stop = 10.
(:wat::test::deftest' :wat-tests::service::hibernate-resume-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h     (:wat-tests::hib-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::hib-counter::Record 0))
       c     (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h))
       _     (:wat-tests::HibCounter/increment c (:wat-tests::HibCounter::IncrementRequest 7))
       snap  (:wat-tests::hib-counter/hibernate h)
       h2    (:wat-tests::hib-counter/resume :locus (:wat::spawn::thread) :record snap)
       c2    (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h2))
       _2    (:wat-tests::HibCounter/increment c2 (:wat-tests::HibCounter::IncrementRequest 3))
       final (:wat-tests::hib-counter/stop h2)]
      final)
    10))

;; ── process tier — IDENTICAL except the locus token (the Record snapshot crosses as EDN) ──
(:wat::test::deftest' :wat-tests::service::hibernate-resume-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h     (:wat-tests::hib-counter/start :locus (:wat::spawn::process) :record (:wat-tests::hib-counter::Record 0))
       c     (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h))
       _     (:wat-tests::HibCounter/increment c (:wat-tests::HibCounter::IncrementRequest 7))
       snap  (:wat-tests::hib-counter/hibernate h)
       h2    (:wat-tests::hib-counter/resume :locus (:wat::spawn::process) :record snap)
       c2    (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h2))
       _2    (:wat-tests::HibCounter/increment c2 (:wat-tests::HibCounter::IncrementRequest 3))
       final (:wat-tests::hib-counter/stop h2)]
      final)
    10))
