;; wat-tests/service-telemetry-bridge.wat — arc 291 4b-iv: the actor-network bridge (cross-process service dep).
;;
;; recorder = a telemetry/accumulator service. worker = holds a CLIENT to recorder (:ephemeral), declares the
;; dependency (:calls), dials it in :init (address is a live :init arg — NOT durable), records each :Work
;; through the stored client. The PROCESS tier proves cross-process contract distribution: worker's forked
;; child loads recorder's client-forms (via :calls) so recorder/record resolves in the child.

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

(:wat::service::defservice :wat-tests::worker
  :durable   [job-count <- :wat::core::i64]
  :ephemeral [recorder  <- :wat::kernel::Peer'<wat-tests::recorder::Op,wat-tests::recorder::Reply>]
  :calls     [:wat-tests::recorder]
  :init (:wat::core::fn [record        <- :wat-tests::worker::Record
                         recorder-addr <- :wat::kernel::Address'<wat-tests::recorder::Op,wat-tests::recorder::Reply>]
          -> :wat-tests::worker::State
          (:wat-tests::worker::State/new record (:wat::kernel::connect' recorder-addr)))
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

;; process tier — THE GATE: worker runs in a child PROCESS; :calls ships recorder's client-forms into that
;; child so recorder/record RESOLVES there (the contract half WORKS — the resolve error is gone).
;; IGNORED pending the TRUST LEG: the proc accept gate refuses the worker child's SIBLING pid (recorder's
;; allow-set = {self, spawner}); needs the locus-dispatched introduction (post-spawn hands the caller's
;; identity -> the callee allow's/cert-grants it). thread + hibernate tiers GREEN.
(:wat::test::ignore "arc 291 trust-leg PENDING — :calls/client-forms resolve works; the proc accept gate refuses the worker child's sibling pid. Needs the locus-dispatched introduction (post-spawn identity -> callee grant).")
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh (:wat-tests::recorder/start :locus (:wat::spawn::process) :record (:wat-tests::recorder::Record 0))
       wh (:wat-tests::worker/start :locus (:wat::spawn::process)
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
