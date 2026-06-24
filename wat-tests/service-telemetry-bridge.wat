;; wat-tests/service-telemetry-bridge.wat — arc 291 4b-iii: the actor-network bridge.
;;
;; THE composition the arc exists for, and the arc-290 telemetry pattern proven: a service's `:init`
;; creates a CLIENT to ANOTHER service and stashes it as an in-locus resource (`:ephemeral`), then its
;; ops record activity through that client. recorder = the telemetry service (an accumulator); worker =
;; holds a client to recorder, dials it in `:init`, records each `:Work` through it.
;;
;; Proves: (1) an `Address'` rides in the worker's `:durable` record (cap is wire-portable);
;; (2) `connect'` is callable in `:init`, in-locus; (3) the client `Peer'` lives in the `:ephemeral`
;; struct (4b-i resource); (4) the worker's op uses the client to record. recorder's Total == sum.

;; ── recorder: the telemetry/accumulator service ───────────────────────────────
(:wat::service::defservice :wat-tests::recorder
  :durable [total <- :wat::core::i64]
  :ops
  [(:Record [s <- :State n <- :wat::core::i64]
            -> [ok <- :wat::core::bool]
     (:wat::service::Outcome::Reply
       (:wat-tests::recorder::State/new
         (:wat-tests::recorder::Record
           (:wat::core::i64::+
             (:wat-tests::recorder::Record/total (:wat-tests::recorder::State/durable s)) n)))
       (:wat-tests::recorder::RecordResponse true)))
   (:Total [s <- :State]
           -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s
       (:wat-tests::recorder::TotalResponse
         (:wat-tests::recorder::Record/total (:wat-tests::recorder::State/durable s)))))])

;; ── worker: holds a client to recorder; :init dials it; :Work records through it ──
(:wat::service::defservice :wat-tests::worker
  :durable   [recorder-addr <- :wat::kernel::Address'<wat-tests::recorder::Op,wat-tests::recorder::Reply>]
  :ephemeral [client <- :wat::kernel::Peer'<wat-tests::recorder::Op,wat-tests::recorder::Reply>]
  :ops
  [(:Work [s <- :State n <- :wat::core::i64]
          -> [done <- :wat::core::bool]
     (:wat::core::let
       [_ (:wat-tests::recorder/record
            (:wat-tests::worker::State/client s)
            (:wat-tests::recorder/record-request n))]
       (:wat::service::Outcome::Reply s (:wat-tests::worker::WorkResponse true))))]
  :init (:wat::core::fn [r <- :wat-tests::worker::Record] -> :wat-tests::worker::State
          (:wat-tests::worker::State/new r
            (:wat::kernel::connect' (:wat-tests::worker::Record/recorder-addr r)))))

;; ── thread tier: worker dials recorder in init, records 5 + 3, recorder Total == 8 ──
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh (:wat-tests::recorder/start (:wat::spawn::thread) (:wat-tests::recorder::Record 0))
       wh (:wat-tests::worker/start (:wat::spawn::thread)
            (:wat-tests::worker::Record (:wat-tests::recorder::Handle/addr rh)))
       wc (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh))
       _  (:wat-tests::worker/work wc (:wat-tests::worker/work-request 5))
       _2 (:wat-tests::worker/work wc (:wat-tests::worker/work-request 3))
       rc (:wat::kernel::connect' (:wat-tests::recorder::Handle/addr rh))
       r  (:wat-tests::recorder/total rc (:wat-tests::recorder/total-request))]
      (:wat-tests::recorder::TotalResponse/value r))
    8))

;; ── process tier — cross-process bridge (the real telemetry topology) ──────────
;; worker runs in a child process; its :init dials the recorder (another locus) and records across.
;; IGNORED pending arc 291 4b-iv: the worker's CHILD PROCESS cannot resolve recorder's client face
;; (recorder/record) — a service's service-forms ship its OWN forms, not a callee's contract. Needs
;; :calls/client-forms + address-as-:init-arg. Thread + hibernate tiers PASS; this IS the gap-marker.
(:wat::test::ignore "arc 291 4b-iv contract-distribution PENDING — cross-process service-to-service needs the callee's client-forms bundled into the caller's child; see STRIKE-4b-iv-contract-distribution.md")
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh (:wat-tests::recorder/start (:wat::spawn::process) (:wat-tests::recorder::Record 0))
       wh (:wat-tests::worker/start (:wat::spawn::process)
            (:wat-tests::worker::Record (:wat-tests::recorder::Handle/addr rh)))
       wc (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh))
       _  (:wat-tests::worker/work wc (:wat-tests::worker/work-request 5))
       _2 (:wat-tests::worker/work wc (:wat-tests::worker/work-request 3))
       rc (:wat::kernel::connect' (:wat-tests::recorder::Handle/addr rh))
       r  (:wat-tests::recorder/total rc (:wat-tests::recorder/total-request))]
      (:wat-tests::recorder::TotalResponse/value r))
    8))

;; ── hibernate→resume: the worker sheds its client + reconnects (durable bridge) ──
;; work 5 → hibernate worker (returns its :durable record = the recorder-addr) → resume (init RE-dials
;; recorder) → work 3 through the rebuilt client → recorder Total == 8. The connection is reconnected,
;; never serialized; the addr is the durable soul.
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-survives-hibernate
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh   (:wat-tests::recorder/start (:wat::spawn::thread) (:wat-tests::recorder::Record 0))
       wh   (:wat-tests::worker/start (:wat::spawn::thread)
              (:wat-tests::worker::Record (:wat-tests::recorder::Handle/addr rh)))
       wc   (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh))
       _    (:wat-tests::worker/work wc (:wat-tests::worker/work-request 5))
       snap (:wat-tests::worker/hibernate wh)
       wh2  (:wat-tests::worker/resume (:wat::spawn::thread) snap)
       wc2  (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh2))
       _2   (:wat-tests::worker/work wc2 (:wat-tests::worker/work-request 3))
       rc   (:wat::kernel::connect' (:wat-tests::recorder::Handle/addr rh))
       r    (:wat-tests::recorder/total rc (:wat-tests::recorder/total-request))]
      (:wat-tests::recorder::TotalResponse/value r))
    8))
