;; wat-tests/service-hibernate-resume.wat — arc 291 strike-4a RED probe: the soul, digitized.
;;
;; THE PROPHECY, proven at the surface (R1 PROBANDUM EST → PROBATUM EST): a service's live State survives
;; TERMINATION and reanimates elsewhere, none the wiser. `hibernate` renders the State to a Snapshot (the
;; WHOLE State, EDN on the process wire) and terminates the service; `resume` spawns a FRESH service whose
;; initial State IS the Snapshot, bypassing `init` (a snapshot is pure data — no resources to rebuild).
;; resume : snapshot :: start : init-args.
;;
;; The proof: increment to 7 → hibernate (service DIES, returns the count-7 snapshot) → resume a fresh
;; service from the snapshot → increment 3 on the reborn service → stop returns 10. The reborn service
;; CONTINUED from the hibernated state (7 + 3 = 10); on the process tier the snapshot crossed as EDN — the
;; only bridge across the process death. The service cannot tell it was reborn.
;;
;; :stop projects State → i64 (the count) so the final assertion is a clean `== 10` (also exercises 3b).
;;
;; RED at HEAD: `hibernate`/`resume` methods + `Admin::Hibernate`/`:Resume` + `LineageUp::Hibernated` don't
;; exist → unknown-function / not-a-tagged-variant. GREEN when the soul survives death and migrates.

(:wat::service::defservice :wat-tests::hib-counter
  :state [count <- :wat::core::i64]
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ (:wat-tests::hib-counter::State/count s) n)]
       (:wat::service::Outcome::Reply (:wat-tests::hib-counter::State s')
         (:wat-tests::hib-counter::IncrementResponse s'))))]
  :init (:wat::core::fn [seed <- :wat::core::i64] -> :wat-tests::hib-counter::State
          (:wat-tests::hib-counter::State seed))
  :stop (:wat::core::fn [s <- :wat-tests::hib-counter::State] -> :wat::core::i64
          (:wat-tests::hib-counter::State/count s)))

;; ── thread tier ──────────────────────────────────────────────────────────────
;; 7 → hibernate (snapshot, service dies) → resume fresh → +3 → stop = 10.
(:wat::test::deftest' :wat-tests::service::hibernate-resume-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h     (:wat-tests::hib-counter/start (:wat::spawn::thread) 0)
       c     (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h))
       _     (:wat-tests::hib-counter/increment c (:wat-tests::hib-counter/increment-request 7))
       snap  (:wat-tests::hib-counter/hibernate h)
       h2    (:wat-tests::hib-counter/resume (:wat::spawn::thread) snap)
       c2    (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h2))
       _2    (:wat-tests::hib-counter/increment c2 (:wat-tests::hib-counter/increment-request 3))
       final (:wat-tests::hib-counter/stop h2)]
      final)
    10))

;; ── process tier — IDENTICAL except the locus token (the snapshot crosses as EDN) ──
(:wat::test::deftest' :wat-tests::service::hibernate-resume-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h     (:wat-tests::hib-counter/start (:wat::spawn::process) 0)
       c     (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h))
       _     (:wat-tests::hib-counter/increment c (:wat-tests::hib-counter/increment-request 7))
       snap  (:wat-tests::hib-counter/hibernate h)
       h2    (:wat-tests::hib-counter/resume (:wat::spawn::process) snap)
       c2    (:wat::kernel::connect' (:wat-tests::hib-counter::Handle/addr h2))
       _2    (:wat-tests::hib-counter/increment c2 (:wat-tests::hib-counter/increment-request 3))
       final (:wat-tests::hib-counter/stop h2)]
      final)
    10))
