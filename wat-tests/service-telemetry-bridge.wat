;; wat-tests/service-telemetry-bridge.wat — arc 291 4b-iv: the actor-network bridge.
;;
;; recorder = a telemetry/accumulator service. worker = holds a CLIENT to recorder (:ephemeral), dials it in
;; :init (address is a live :init arg — NOT durable), and forwards each `work` through the stored client to
;; the recorder's `record`.
;;
;; Arc 278 S4c: both services wear an explicit SURFACE (:satisfies + :impls). :ops is retired. The worker's
;; ephemeral peer + :init address are typed on the RECORDER SURFACE's Op/Reply (the uniform wire protocol),
;; and its `work` body dials the recorder via the surface method `:wat-tests::Recorder/record`.

;; ── the Recorder surface (its protocol + per-op request/response records) ───────────────────────
;; arc 278 S4c: each surface OWNS its protocol messages (:messages).
(:wat::core::defsurface :wat-tests::Recorder :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat-tests::Recorder::RecordRequest  [n     <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::Recorder::RecordResponse :wat::enum::Pure
     :Ok              [ok    <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])
   (:wat::core::defrecord :wat-tests::Recorder::TotalRequest   [])
   (:wat::core::defenum :wat-tests::Recorder::TotalResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(record [self <- :wat-tests::Recorder req <- :wat-tests::Recorder::RecordRequest]
           -> :wat-tests::Recorder::RecordResponse)
   (total  [self <- :wat-tests::Recorder req <- :wat-tests::Recorder::TotalRequest]
           -> :wat-tests::Recorder::TotalResponse)])

;; ── the Worker surface ──────────────────────────────────────────────────────────────────────────
(:wat::core::defsurface :wat-tests::Worker :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat-tests::Worker::WorkRequest  [n    <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::Worker::WorkResponse :wat::enum::Pure
     :Ok              [done  <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(work [self <- :wat-tests::Worker req <- :wat-tests::Worker::WorkRequest]
         -> :wat-tests::Worker::WorkResponse)])

;; ── the recorder service — wears :wat-tests::Recorder ───────────────────────────────────────────
(:wat::service::defservice :wat-tests::recorder
  :satisfies :wat-tests::Recorder
  :durable   [total <- :wat::core::i64]
  :ephemeral []
  :impls
  [(record [s req]
     (:wat::service::Outcome::Reply
       (:wat-tests::recorder::State :durable
         (:wat-tests::recorder::Record :total
           (:wat::core::i64::+
             (:wat-tests::recorder::Record/total (:wat-tests::recorder::State/durable s))
             (:wat-tests::Recorder::RecordRequest/n req))))
       (:wat-tests::Recorder::RecordResponse::Ok true)))
   (total [s req]
     (:wat::service::Outcome::Reply s
       (:wat-tests::Recorder::TotalResponse::Ok
         (:wat-tests::recorder::Record/total (:wat-tests::recorder::State/durable s)))))])

;; ── the worker service — wears :wat-tests::Worker, dials a :wat-tests::Recorder peer ─────────────
(:wat::service::defservice :wat-tests::worker
  :satisfies :wat-tests::Worker
  :durable   [job-count <- :wat::core::i64]
  :ephemeral [recorder  <- :wat::kernel::Peer'<wat-tests::Recorder::Op,wat-tests::Recorder::Reply>]
  ;; arc 278 S4d: worker DIALS recorder (holds its client peer above) — declare the s2s DAG edge.
  :peers     [:wat-tests::Recorder]
  :init (:wat::core::fn [record        <- :wat-tests::worker::Record
                         recorder-addr <- :wat::kernel::Address'<wat-tests::Recorder::Op,wat-tests::Recorder::Reply>]
          -> :wat-tests::worker::State
          (:wat-tests::worker::State :durable record :recorder (:wat::kernel::connect' recorder-addr)))
  :impls
  [(work [s req]
     (:wat::core::let
       [rresp (:wat-tests::Recorder/record
                (:wat-tests::worker::State/recorder s)
                (:wat-tests::Recorder::RecordRequest :n (:wat-tests::Worker::WorkRequest/n req)))
        wresp (:wat::core::match rresp -> :wat-tests::Worker::WorkResponse
                ((:wat-tests::Recorder::RecordResponse::Ok _ok)
                  (:wat-tests::Worker::WorkResponse::Ok true))
                ;; s2s consumer: a downstream wire-breach propagates outward as our own op's breach.
                ((:wat-tests::Recorder::RecordResponse::RequestTooLarge bytes cap)
                  (:wat-tests::Worker::WorkResponse::RequestTooLarge bytes cap)))]
       (:wat::service::Outcome::Reply s wresp)))])

;; thread tier: worker dials recorder in init, records 5 + 3, recorder Total == 8.
;; start threads the LIVE recorder address as the worker's 2nd start arg (the :init operating-input).
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh (:wat-tests::recorder/start :locus (:wat::spawn::thread) :record (:wat-tests::recorder::Record :total 0))
       wh (:wat-tests::worker/start :locus (:wat::spawn::thread)
            :record (:wat-tests::worker::Record :job-count 0)
            :recorder-addr (:wat-tests::recorder::Handle/addr rh))
       wc (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh))
       _  (:wat-tests::Worker/work wc (:wat-tests::Worker::WorkRequest :n 5))
       _2 (:wat-tests::Worker/work wc (:wat-tests::Worker::WorkRequest :n 3))
       rc (:wat::kernel::connect' (:wat-tests::recorder::Handle/addr rh))
       r  (:wat-tests::Recorder/total rc (:wat-tests::Recorder::TotalRequest))]
      (:wat::core::match r -> :wat::core::i64
        ((:wat-tests::Recorder::TotalResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::Recorder::TotalResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "recorder-total: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))))
    8))

;; hibernate -> resume: worker sheds its client + reconnects on resume. resume takes the saved record AND
;; the CURRENT recorder address (live topology, re-supplied — the address is never hibernated).
(:wat::test::deftest' :wat-tests::service::telemetry-bridge-survives-hibernate
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [rh   (:wat-tests::recorder/start :locus (:wat::spawn::thread) :record (:wat-tests::recorder::Record :total 0))
       wh   (:wat-tests::worker/start :locus (:wat::spawn::thread)
              :record (:wat-tests::worker::Record :job-count 0)
              :recorder-addr (:wat-tests::recorder::Handle/addr rh))
       wc   (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh))
       _    (:wat-tests::Worker/work wc (:wat-tests::Worker::WorkRequest :n 5))
       snap (:wat-tests::worker/hibernate wh)
       wh2  (:wat-tests::worker/resume :locus (:wat::spawn::thread) :record snap
              :recorder-addr (:wat-tests::recorder::Handle/addr rh))
       wc2  (:wat::kernel::connect' (:wat-tests::worker::Handle/addr wh2))
       _2   (:wat-tests::Worker/work wc2 (:wat-tests::Worker::WorkRequest :n 3))
       rc   (:wat::kernel::connect' (:wat-tests::recorder::Handle/addr rh))
       r    (:wat-tests::Recorder/total rc (:wat-tests::Recorder::TotalRequest))]
      (:wat::core::match r -> :wat::core::i64
        ((:wat-tests::Recorder::TotalResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::Recorder::TotalResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "recorder-total: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))))
    8))
