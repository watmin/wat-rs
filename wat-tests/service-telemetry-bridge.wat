;; wat-tests/service-telemetry-bridge.wat — arc 291 4b-iv: the actor-network bridge.
;;
;; recorder = a telemetry/accumulator service. worker = holds a CLIENT to recorder (:ephemeral), dials it in
;; :init (address is a live :init arg — NOT durable), and forwards each :Work through the stored client to
;; recorder/record.

(:wat::service::defservice :wat-tests::recorder
  :durable [total <- :wat::core::i64]
  :ops
  [(:Record [s <- :State n <- :wat::core::i64]
            -> [ok <- :wat::core::bool]
     (:wat::service::Outcome::Reply
       (:wat-tests::recorder::State
         (:wat-tests::recorder::Record
           (:wat::core::i64::+
             (:wat-tests::recorder::Record/total (:wat-tests::recorder::State/durable s)) n)))
       (:wat-tests::recorder::RecordResponse true)))
   (:Total [s <- :State]
           -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s
       (:wat-tests::recorder::TotalResponse
         (:wat-tests::recorder::Record/total (:wat-tests::recorder::State/durable s)))))])

(:wat::service::defservice :wat-tests::worker
  :durable   [job-count <- :wat::core::i64]
  :ephemeral [recorder  <- :wat::kernel::Peer'<wat-tests::recorder::Op,wat-tests::recorder::Reply>]
  :init (:wat::core::fn [record        <- :wat-tests::worker::Record
                         recorder-addr <- :wat::kernel::Address'<wat-tests::recorder::Op,wat-tests::recorder::Reply>]
          -> :wat-tests::worker::State
          (:wat-tests::worker::State record (:wat::kernel::connect' recorder-addr)))
  :ops
  [(:Work [s <- :State n <- :wat::core::i64]
          -> [done <- :wat::core::bool]
     (:wat::core::let
       [_ (:wat-tests::recorder/record
            (:wat-tests::worker::State/recorder s)
            (:wat-tests::recorder/record-request n))]
       (:wat::service::Outcome::Reply s (:wat-tests::worker::WorkResponse true))))])

;; thread tier: worker dials recorder in init, records 5 + 3, recorder Total == 8.
;; start threads the LIVE recorder address as the worker's 2nd start arg (the :init operating-input).
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh (:wat-tests::recorder/start :locus (:wat::spawn::thread) :record (:wat-tests::recorder::Record 0))
       wh (:wat-tests::worker/start :locus (:wat::spawn::thread)
            :record (:wat-tests::worker::Record 0)
            :recorder-addr (:wat-tests::recorder::Handle/addr rh))
       wc (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh))
       _  (:wat-tests::worker/work wc (:wat-tests::worker/work-request 5))
       _2 (:wat-tests::worker/work wc (:wat-tests::worker/work-request 3))
       rc (:wat::kernel::connect' (:wat-tests::recorder::Handle/addr rh))
       r  (:wat-tests::recorder/total rc (:wat-tests::recorder/total-request))]
      (:wat-tests::recorder::TotalResponse/value r))
    8))

;; hibernate -> resume: worker sheds its client + reconnects on resume. resume takes the saved record AND
;; the CURRENT recorder address (live topology, re-supplied — the address is never hibernated).
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-survives-hibernate
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh   (:wat-tests::recorder/start :locus (:wat::spawn::thread) :record (:wat-tests::recorder::Record 0))
       wh   (:wat-tests::worker/start :locus (:wat::spawn::thread)
              :record (:wat-tests::worker::Record 0)
              :recorder-addr (:wat-tests::recorder::Handle/addr rh))
       wc   (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh))
       _    (:wat-tests::worker/work wc (:wat-tests::worker/work-request 5))
       snap (:wat-tests::worker/hibernate wh)
       wh2  (:wat-tests::worker/resume :locus (:wat::spawn::thread) :record snap
              :recorder-addr (:wat-tests::recorder::Handle/addr rh))
       wc2  (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh2))
       _2   (:wat-tests::worker/work wc2 (:wat-tests::worker/work-request 3))
       rc   (:wat::kernel::connect' (:wat-tests::recorder::Handle/addr rh))
       r    (:wat-tests::recorder/total rc (:wat-tests::recorder/total-request))]
      (:wat-tests::recorder::TotalResponse/value r))
    8))
