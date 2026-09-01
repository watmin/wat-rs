;; wat/telemetry/span.wat — arc 278 STONE Span.2: :wat::telemetry'::span' — the PRODUCER service.
;;
;; A short-lived `:satisfies :wat::telemetry'::Span` actor, one per unit of work. It HOLDS a
;; `:wat::telemetry'::Journal` peer (S4d `:peers` — the sink, given at start) and threads PURE
;; accumulating state (counters + duration samples + logs) through its serve loop. `flush` emits
;; deltas since the last flush and RESETS; `close` is that same path for the remainder. Each
;; counter -> 1 Metric; each duration name -> `<name>/count` + `<name>/duration` + one
;; `<name>/sample` per sample.
;;
;; Provisioning is INLINE at the call site (the scope law: start+connect+use in one lexical scope),
;; done by the `with-span` macro (stone Span.3); the durable Record carries namespace/uuid/tags/
;; start-time-ns (minted at the call site) + empty counters/durations.
;;
;; Loads after wat/telemetry/journal.wat (needs Journal + Metric/Log/Scope + time).

;; Cadence is configuration, not a constant — overridable per span at span/start.
;; Defaults: logs 1s (fast), metrics 30s (slow beat). Tests override to milliseconds.
(:wat::core::def :wat::telemetry::span::DEFAULT-LOGS-FLUSH-AFTER-MS 1000)
(:wat::core::def :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS 30000)

;; ── the service ─────────────────────────────────────────────────────────────────
(:wat::service::defservice :wat::telemetry::span
  :satisfies :wat::telemetry::Span
  :durable   [namespace     <- :wat::core::String
              uuid          <- :wat::core::Uuid
              tags          <- :wat::telemetry::Tags
              start-time-ns <- :wat::core::i64
              counters      <- (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
              durations     <- (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
              logs          <- (:wat::core::Vector :- [:wat::telemetry::Log])
              logs-flush-after-ms    <- :wat::core::i64
              metrics-flush-after-ms <- :wat::core::i64]
  :ephemeral [sink <- (:wat::kernel::Peer :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply])]
  :peers     [:wat::telemetry::Journal]
  :init (:wat::core::fn
          [record    <- :wat::telemetry::span::Record
           sink-addr <- (:wat::kernel::Address :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply])]
          -> :wat::telemetry::span::State
          ;; arc 278 the connect'-outcome wall — face all four arms; ::Connected → the sink
          ;; Peer'; failure arms → assertion-failed! (fatal, preserving the pre-wall
          ;; raise-unwind: a span service whose sink dial fails at :init cannot start).
          (:wat::telemetry::span::State :durable record
            :sink (:wat::core::match (:wat::kernel::connect sink-addr)
                    ((:wat::kernel::ConnectOutcome::Connected p) p)
                    ((:wat::kernel::ConnectOutcome::Refused c)
                      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                    ((:wat::kernel::ConnectOutcome::Rejected c)
                      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                    ((:wat::kernel::ConnectOutcome::Failed c)
                      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [;; incr — PURE: counters[name] + 1. Arm -flush-metrics on empty→non-empty.
   (incr [s ctx req]
     (:wat::core::let
       [name (:wat::telemetry::Span::IncrRequest/name req)
        rec  (:wat::telemetry::span::State/durable s)
        cs   (:wat::telemetry::span::Record/counters rec)
        was-empty? (:wat::telemetry::span::metrics-empty? rec)
        next (:wat::core::match (:wat::hashmap::get cs name)
               (:wat::core::None 1)
               ((:wat::core::Some v) (:wat::core::+ v 1)))
        rec' (:wat::telemetry::span::record-accum rec
               (:wat::hashmap::assoc cs name next)
               (:wat::telemetry::span::Record/durations rec)
               (:wat::telemetry::span::Record/logs rec))
        s'   (:wat::telemetry::span::State :durable rec' :sink (:wat::telemetry::span::State/sink s))]
       (:wat::core::if was-empty?
         (:wat::service::Outcome::ReplyAndArm s' (:wat::telemetry::Span::IncrResponse::Ok)
           [(:wat::service::Alarm
              :after (:wat::time::Millisecond
                       (:wat::telemetry::span::Record/metrics-flush-after-ms rec'))
              :op :-flush-metrics)])
         (:wat::service::Outcome::Reply s' (:wat::telemetry::Span::IncrResponse::Ok)))))

   ;; timed — PURE: durations[name] ++ nanos. Size trigger calls flush-metrics ONLY.
   ;; Arm -flush-metrics on empty→non-empty of (counters AND durations).
   (timed [s ctx req]
     (:wat::core::let
       [name  (:wat::telemetry::Span::TimedRequest/name req)
        nanos (:wat::telemetry::Span::TimedRequest/nanos req)
        rec   (:wat::telemetry::span::State/durable s)
        ds    (:wat::telemetry::span::Record/durations rec)
        samples (:wat::core::match (:wat::hashmap::get ds name)
                  (:wat::core::None (:wat::core::Vector :- [:wat::core::i64]))
                  ((:wat::core::Some v) v))
        rec-would (:wat::telemetry::span::record-accum rec
                    (:wat::telemetry::span::Record/counters rec)
                    (:wat::hashmap::assoc ds name (:wat::core::conj samples nanos))
                    (:wat::telemetry::span::Record/logs rec))
        now   (:wat::time::epoch-nanos (:wat::time::now))
        bytes (:wat::string::length
                (:wat::edn::write
                  (:wat::telemetry::Journal::WriteMetricsRequest
                    (:wat::telemetry::span::build-metrics rec-would now))))
        cap   :wat::telemetry::Journal::WRITE-METRICS-MAX-REQUEST-BYTES
        prior? (:wat::core::not (:wat::telemetry::span::metrics-empty? rec))
        pair0 (:wat::core::if
                (:wat::core::and (:wat::i64::>= bytes cap) prior?)
                (:wat::telemetry::span::flush-metrics s)
                (:wat::core::Tuple s (:wat::telemetry::Span::CloseResponse::Done)))
        s1    (:wat::core::first pair0)
        rec1  (:wat::telemetry::span::State/durable s1)
        was-empty? (:wat::telemetry::span::metrics-empty? rec1)
        ds1   (:wat::telemetry::span::Record/durations rec1)
        samples1 (:wat::core::match (:wat::hashmap::get ds1 name)
                   (:wat::core::None (:wat::core::Vector :- [:wat::core::i64]))
                   ((:wat::core::Some v) v))
        rec2  (:wat::telemetry::span::record-accum rec1
                (:wat::telemetry::span::Record/counters rec1)
                (:wat::hashmap::assoc ds1 name (:wat::core::conj samples1 nanos))
                (:wat::telemetry::span::Record/logs rec1))
        s'    (:wat::telemetry::span::State :durable rec2 :sink (:wat::telemetry::span::State/sink s1))]
       (:wat::core::if was-empty?
         (:wat::service::Outcome::ReplyAndArm s' (:wat::telemetry::Span::TimedResponse::Ok)
           [(:wat::service::Alarm
              :after (:wat::time::Millisecond
                       (:wat::telemetry::span::Record/metrics-flush-after-ms rec2))
              :op :-flush-metrics)])
         (:wat::service::Outcome::Reply s' (:wat::telemetry::Span::TimedResponse::Ok)))))

   ;; log — conj onto :durable logs, return Ok (buffered). Size trigger calls flush-logs ONLY.
   ;; Arm -flush-logs on empty→non-empty of the current logs buffer.
   (log [s ctx req]
     (:wat::core::let
       [rec (:wat::telemetry::span::State/durable s)
        now (:wat::time::epoch-nanos (:wat::time::now))
        eid (:wat::uuid::v4)
        l   (:wat::telemetry::Log
              :namespace (:wat::telemetry::span::Record/namespace rec)
              :uuid (:wat::telemetry::span::Record/uuid rec)
              :tags (:wat::telemetry::span::Record/tags rec)
              :time-ns now
              :event-id eid
              :emitted-from (:wat::telemetry::Span::LogRequest/emitted-from req)
              :level (:wat::telemetry::Span::LogRequest/level req)
              :message (:wat::telemetry::Span::LogRequest/message req))
        logs0 (:wat::telemetry::span::Record/logs rec)
        would (:wat::core::conj logs0 l)
        bytes (:wat::string::length
                (:wat::edn::write
                  (:wat::telemetry::Journal::WriteLogsRequest would)))
        cap   :wat::telemetry::Journal::WRITE-LOGS-MAX-REQUEST-BYTES
        pair0 (:wat::core::if
                (:wat::core::and
                  (:wat::i64::>= bytes cap)
                  (:wat::i64::> (:wat::core::count logs0) 0))
                (:wat::telemetry::span::flush-logs s)
                (:wat::core::Tuple s (:wat::telemetry::Span::CloseResponse::Done)))
        s1    (:wat::core::first pair0)
        rec1  (:wat::telemetry::span::State/durable s1)
        was-empty? (:wat::core::= (:wat::core::count (:wat::telemetry::span::Record/logs rec1)) 0)
        rec2  (:wat::telemetry::span::record-accum rec1
                (:wat::telemetry::span::Record/counters rec1)
                (:wat::telemetry::span::Record/durations rec1)
                (:wat::core::conj (:wat::telemetry::span::Record/logs rec1) l))
        s'    (:wat::telemetry::span::State :durable rec2 :sink (:wat::telemetry::span::State/sink s1))]
       (:wat::core::if was-empty?
         (:wat::service::Outcome::ReplyAndArm s' (:wat::telemetry::Span::LogResponse::Ok)
           [(:wat::service::Alarm
              :after (:wat::time::Millisecond
                       (:wat::telemetry::span::Record/logs-flush-after-ms rec2))
              :op :-flush-logs)])
         (:wat::service::Outcome::Reply s' (:wat::telemetry::Span::LogResponse::Ok)))))

   ;; flush — emit deltas since the last flush and RESET. THE emission path.
   (flush [s ctx req]
     (:wat::core::let
       [pair  (:wat::telemetry::span::flush-accumulators s)
        s'    (:wat::core::first pair)
        cresp (:wat::core::second pair)
        fresp (:wat::telemetry::span::close-response->flush-response cresp)]
       (:wat::service::Outcome::Reply s' fresp)))

   ;; close — flush the remainder of BOTH groups. Each group still has one builder.
   (close [s ctx req]
     (:wat::core::let
       [pair  (:wat::telemetry::span::flush-accumulators s)
        s'    (:wat::core::first pair)
        cresp (:wat::core::second pair)]
       (:wat::service::Outcome::Reply s' cresp)))

   ;; INTERNAL: timer for logs. Flush this group; do NOT re-arm (accumulator is empty;
   ;; the next log re-arms on empty→non-empty). An idle span never reaches here.
   (-flush-logs [s ctx]
     (:wat::service::Outcome::NoReply
       (:wat::core::first (:wat::telemetry::span::flush-logs s))))

   ;; INTERNAL: timer for counters+durations. Same: flush, no re-arm.
   (-flush-metrics [s ctx]
     (:wat::service::Outcome::NoReply
       (:wat::core::first (:wat::telemetry::span::flush-metrics s))))])

;; ── emit-and-reset (item (c) stone A) ────────────────────────────────────────────
;; THE emission path. `flush` and `close` both call this. A second path is the double-count.

(:wat::core::defn :wat::telemetry::span::build-metrics
  [rec <- :wat::telemetry::span::Record  now <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::telemetry::Metric])
  (:wat::core::let
    [ns    (:wat::telemetry::span::Record/namespace rec)
     uuid  (:wat::telemetry::span::Record/uuid rec)
     tags  (:wat::telemetry::span::Record/tags rec)
     start (:wat::telemetry::span::Record/start-time-ns rec)
     cs    (:wat::telemetry::span::Record/counters rec)
     ds    (:wat::telemetry::span::Record/durations rec)
     counter-metrics
     (:wat::core::foldl
       (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::telemetry::Metric]) name <- :wat::core::keyword]
         -> (:wat::core::Vector :- [:wat::telemetry::Metric])
         (:wat::core::conj acc
           (:wat::telemetry::Metric :namespace ns :uuid uuid :tags tags :time-ns now
             :event-id (:wat::uuid::v4)
             :start-time-ns start :name name
             :value (:wat::telemetry::Numeric::I64
                      (:wat::core::Option/expect (:wat::hashmap::get cs name) "counter present"))
             :unit :wat::telemetry::Unit::Count)))
       (:wat::core::Vector :- [:wat::telemetry::Metric])
       (:wat::hashmap::keys cs))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::telemetry::Metric]) name <- :wat::core::keyword]
        -> (:wat::core::Vector :- [:wat::telemetry::Metric])
        (:wat::core::let
          [samples (:wat::core::Option/expect (:wat::hashmap::get ds name) "duration present")
           cnt (:wat::core::count samples)
           total (:wat::core::foldl
                   (:wat::core::fn [a <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ a x))
                   0 samples)
           base (:wat::keyword::to-string name)
           count-name (:wat::keyword::from-string (:wat::core::format "{base}/count" :base base))
           dur-name   (:wat::keyword::from-string (:wat::core::format "{base}/duration" :base base))
           sample-name (:wat::keyword::from-string (:wat::core::format "{base}/sample" :base base))
           with-agg
           (:wat::core::conj
             (:wat::core::conj acc
               (:wat::telemetry::Metric :namespace ns :uuid uuid :tags tags :time-ns now
                 :event-id (:wat::uuid::v4)
                 :start-time-ns start :name count-name
                 :value (:wat::telemetry::Numeric::I64 cnt) :unit :wat::telemetry::Unit::Count))
             (:wat::telemetry::Metric :namespace ns :uuid uuid :tags tags :time-ns now
               :event-id (:wat::uuid::v4)
               :start-time-ns start :name dur-name
               :value (:wat::telemetry::Numeric::I64 total) :unit :wat::telemetry::Unit::Nanos))]
          ;; fidelity: one <name>/sample per sample, Unit::Nanos. Additive; count+duration unchanged.
          (:wat::core::foldl
            (:wat::core::fn [a <- (:wat::core::Vector :- [:wat::telemetry::Metric]) x <- :wat::core::i64]
              -> (:wat::core::Vector :- [:wat::telemetry::Metric])
              (:wat::core::conj a
                (:wat::telemetry::Metric :namespace ns :uuid uuid :tags tags :time-ns now
                  :event-id (:wat::uuid::v4)
                  :start-time-ns start :name sample-name
                  :value (:wat::telemetry::Numeric::I64 x) :unit :wat::telemetry::Unit::Nanos)))
            with-agg
            samples)))
      counter-metrics
      (:wat::hashmap::keys ds))))

(:wat::core::defn :wat::telemetry::span::record-accum
  [rec <- :wat::telemetry::span::Record
   cs  <- (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
   ds  <- (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
   logs <- (:wat::core::Vector :- [:wat::telemetry::Log])]
  -> :wat::telemetry::span::Record
  (:wat::telemetry::span::Record
    :namespace (:wat::telemetry::span::Record/namespace rec)
    :uuid (:wat::telemetry::span::Record/uuid rec)
    :tags (:wat::telemetry::span::Record/tags rec)
    :start-time-ns (:wat::telemetry::span::Record/start-time-ns rec)
    :counters cs
    :durations ds
    :logs logs
    :logs-flush-after-ms (:wat::telemetry::span::Record/logs-flush-after-ms rec)
    :metrics-flush-after-ms (:wat::telemetry::span::Record/metrics-flush-after-ms rec)))

(:wat::core::defn :wat::telemetry::span::reset-accumulators
  [rec <- :wat::telemetry::span::Record] -> :wat::telemetry::span::Record
  (:wat::telemetry::span::record-accum rec
    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
    (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
    (:wat::core::Vector :- [:wat::telemetry::Log])))

(:wat::core::defn :wat::telemetry::span::metrics-empty?
  [rec <- :wat::telemetry::span::Record] -> :wat::core::bool
  (:wat::core::and
    (:wat::core::= (:wat::core::count (:wat::hashmap::keys (:wat::telemetry::span::Record/counters rec))) 0)
    (:wat::core::= (:wat::core::count (:wat::hashmap::keys (:wat::telemetry::span::Record/durations rec))) 0)))

(:wat::core::defn :wat::telemetry::span::map-write-metrics-recv
  [resp <- (:wat::kernel::RecvOutcome :- [:wat::telemetry::Journal::WriteMetricsResponse])]
  -> :wat::telemetry::Span::CloseResponse
  (:wat::core::match resp
    ((:wat::kernel::RecvOutcome::Message sresp)
      (:wat::core::match sresp
        ((:wat::telemetry::Journal::WriteMetricsResponse::Success)
          (:wat::telemetry::Span::CloseResponse::Done))
        ((:wat::telemetry::Journal::WriteMetricsResponse::Constraint err)
          (:wat::telemetry::Span::CloseResponse::Constraint err))
        ((:wat::telemetry::Journal::WriteMetricsResponse::Transient err)
          (:wat::telemetry::Span::CloseResponse::Transient err))
        ((:wat::telemetry::Journal::WriteMetricsResponse::Fatal err)
          (:wat::telemetry::Span::CloseResponse::Fatal err))
        ((:wat::telemetry::Journal::WriteMetricsResponse::RequestTooLarge bytes cap)
          (:wat::telemetry::Span::CloseResponse::RequestTooLarge bytes cap))
        ((:wat::telemetry::Journal::WriteMetricsResponse::RequestMalformed mpath mexpected mgot)
          (:wat::telemetry::Span::CloseResponse::RequestMalformed mpath mexpected mgot))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::telemetry::Span::CloseResponse::Fatal
        (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::telemetry::Span::CloseResponse::Fatal
        (:wat::query::Fatal :reason (:wat::query::Fault :message "span.wat: stop requested mid-call — the journal sink peer was ALIVE"))))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::telemetry::Span::CloseResponse::Fatal
        (:wat::query::Fatal :reason (:wat::query::Fault :message "span.wat: journal sink peer closed"))))))

(:wat::core::defn :wat::telemetry::span::map-write-logs-recv
  [resp <- (:wat::kernel::RecvOutcome :- [:wat::telemetry::Journal::WriteLogsResponse])]
  -> :wat::telemetry::Span::CloseResponse
  (:wat::core::match resp
    ((:wat::kernel::RecvOutcome::Message sresp)
      (:wat::core::match sresp
        ((:wat::telemetry::Journal::WriteLogsResponse::Success)
          (:wat::telemetry::Span::CloseResponse::Done))
        ((:wat::telemetry::Journal::WriteLogsResponse::Constraint err)
          (:wat::telemetry::Span::CloseResponse::Constraint err))
        ((:wat::telemetry::Journal::WriteLogsResponse::Transient err)
          (:wat::telemetry::Span::CloseResponse::Transient err))
        ((:wat::telemetry::Journal::WriteLogsResponse::Fatal err)
          (:wat::telemetry::Span::CloseResponse::Fatal err))
        ((:wat::telemetry::Journal::WriteLogsResponse::RequestTooLarge bytes cap)
          (:wat::telemetry::Span::CloseResponse::RequestTooLarge bytes cap))
        ((:wat::telemetry::Journal::WriteLogsResponse::RequestMalformed mpath mexpected mgot)
          (:wat::telemetry::Span::CloseResponse::RequestMalformed mpath mexpected mgot))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::telemetry::Span::CloseResponse::Fatal
        (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::telemetry::Span::CloseResponse::Fatal
        (:wat::query::Fatal :reason (:wat::query::Fault :message "span.wat: stop requested mid-call — the journal sink peer was ALIVE"))))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::telemetry::Span::CloseResponse::Fatal
        (:wat::query::Fatal :reason (:wat::query::Fault :message "span.wat: journal sink peer closed"))))))

(:wat::core::defn :wat::telemetry::span::close-response->flush-response
  [c <- :wat::telemetry::Span::CloseResponse] -> :wat::telemetry::Span::FlushResponse
  (:wat::core::match c
    ((:wat::telemetry::Span::CloseResponse::Done)
      (:wat::telemetry::Span::FlushResponse::Done))
    ((:wat::telemetry::Span::CloseResponse::Constraint err)
      (:wat::telemetry::Span::FlushResponse::Constraint err))
    ((:wat::telemetry::Span::CloseResponse::Transient err)
      (:wat::telemetry::Span::FlushResponse::Transient err))
    ((:wat::telemetry::Span::CloseResponse::Fatal err)
      (:wat::telemetry::Span::FlushResponse::Fatal err))
    ((:wat::telemetry::Span::CloseResponse::RequestTooLarge bytes cap)
      (:wat::telemetry::Span::FlushResponse::RequestTooLarge bytes cap))
    ((:wat::telemetry::Span::CloseResponse::RequestMalformed mpath mexpected mgot)
      (:wat::telemetry::Span::FlushResponse::RequestMalformed mpath mexpected mgot))))

;; ONE emit-and-reset path for logs. Called by the logs size trigger, -flush-logs, and
;; flush-accumulators (close/flush). A second builder here is stone A's double-count.
(:wat::core::defn :wat::telemetry::span::flush-logs
  [s <- :wat::telemetry::span::State]
  -> (:wat::core::Tuple :- [:wat::telemetry::span::State :wat::telemetry::Span::CloseResponse])
  (:wat::core::let
    [rec  (:wat::telemetry::span::State/durable s)
     sink (:wat::telemetry::span::State/sink s)
     logs (:wat::telemetry::span::Record/logs rec)]
    (:wat::core::if (:wat::core::= (:wat::core::count logs) 0)
      (:wat::core::Tuple s (:wat::telemetry::Span::CloseResponse::Done))
      (:wat::core::let
        [cresp (:wat::telemetry::span::map-write-logs-recv
                 (:wat::telemetry::Journal/write-logs sink
                   (:wat::telemetry::Journal::WriteLogsRequest logs)))]
        (:wat::core::match cresp
          ((:wat::telemetry::Span::CloseResponse::Done)
            (:wat::core::Tuple
              (:wat::telemetry::span::State
                :durable (:wat::telemetry::span::record-accum rec
                           (:wat::telemetry::span::Record/counters rec)
                           (:wat::telemetry::span::Record/durations rec)
                           (:wat::core::Vector :- [:wat::telemetry::Log]))
                :sink sink)
              (:wat::telemetry::Span::CloseResponse::Done)))
          (_ (:wat::core::Tuple s cresp)))))))

;; ONE emit-and-reset path for counters+durations. Called by the metrics size trigger,
;; -flush-metrics, and flush-accumulators (close/flush).
(:wat::core::defn :wat::telemetry::span::flush-metrics
  [s <- :wat::telemetry::span::State]
  -> (:wat::core::Tuple :- [:wat::telemetry::span::State :wat::telemetry::Span::CloseResponse])
  (:wat::core::let
    [rec  (:wat::telemetry::span::State/durable s)
     sink (:wat::telemetry::span::State/sink s)
     now  (:wat::time::epoch-nanos (:wat::time::now))
     metrics (:wat::telemetry::span::build-metrics rec now)]
    (:wat::core::if (:wat::core::= (:wat::core::count metrics) 0)
      (:wat::core::Tuple s (:wat::telemetry::Span::CloseResponse::Done))
      (:wat::core::let
        [cresp (:wat::telemetry::span::map-write-metrics-recv
                 (:wat::telemetry::Journal/write-metrics sink
                   (:wat::telemetry::Journal::WriteMetricsRequest metrics)))]
        (:wat::core::match cresp
          ((:wat::telemetry::Span::CloseResponse::Done)
            (:wat::core::Tuple
              (:wat::telemetry::span::State
                :durable (:wat::telemetry::span::record-accum rec
                           (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                           (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                           (:wat::telemetry::span::Record/logs rec))
                :sink sink)
              (:wat::telemetry::Span::CloseResponse::Done)))
          (_ (:wat::core::Tuple s cresp)))))))

;; close / Span/flush: both groups, logs then metrics. Still one builder per group.
(:wat::core::defn :wat::telemetry::span::flush-accumulators
  [s <- :wat::telemetry::span::State]
  -> (:wat::core::Tuple :- [:wat::telemetry::span::State :wat::telemetry::Span::CloseResponse])
  (:wat::core::let
    [p1 (:wat::telemetry::span::flush-logs s)
     c1 (:wat::core::second p1)]
    (:wat::core::match c1
      ((:wat::telemetry::Span::CloseResponse::Done)
        (:wat::core::let
          [p2 (:wat::telemetry::span::flush-metrics (:wat::core::first p1))]
          p2))
      (_ p1))))

;; ── the call-site macros (STONE Span.3) ──────────────────────────────────────────
;; `timed` — the timing widget (Clojure `time` idiom): read the clock, run the body, feed
;; name + elapsed-nanos to the PURE `Span/timed` op, return the body's value untouched. No closure
;; enters the actor. `Span/timed` (the op) ≠ `:wat::telemetry'::timed` (this macro) — FQDN.
(:wat::core::defmacro :wat::telemetry::timed
  [span <- :wat::WatAST  name <- :wat::WatAST  body <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [start-sym   (:wat::core::fresh-symbol "start")
     ret-sym     (:wat::core::fresh-symbol "ret")
     elapsed-sym (:wat::core::fresh-symbol "elapsed")
     t-sym       (:wat::core::fresh-symbol "t")]
    `(:wat::core::let
       [~start-sym   (:wat::time::epoch-nanos (:wat::time::now))
        ~ret-sym     ~body
        ~elapsed-sym (:wat::core::- (:wat::time::epoch-nanos (:wat::time::now)) ~start-sym)
        ~t-sym       (:wat::telemetry::Span/timed ~span
                       (:wat::telemetry::Span::TimedRequest :name ~name :nanos ~elapsed-sym))]
       ~ret-sym)))

;; `log` — the client call-site widget: bake emitted-from at the (log …) line via macro-call-site,
;; edn::write the message opaque (Stone B), issue the span's `log` op. POSITIONAL [span level message]
;; — same grain as `timed`/`with-span` above; the message is a record the widget edn::write's (opaque,
;; Stone B) so the caller never serializes by hand. `:wat::telemetry::log` (this macro) ≠ `Span/log`
;; (the op) — FQDN disambiguates.
(:wat::core::defmacro :wat::telemetry::log
  [span <- :wat::WatAST  level <- :wat::WatAST  message <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::telemetry::Span/log ~span
     (:wat::telemetry::Span::LogRequest
       :emitted-from ~(:wat::kernel::macro-call-site)
       :level ~level
       :message (:wat::edn::write ~message))))

;; `with-span` — acquire / use / guaranteed close, INLINE (the scope law: the span' handle must
;; share one lexical scope with its use + close). binding = [span-name sink-addr namespace tags];
;; mints uuid + start-time at the call site, starts + dials span', runs the body, closes.
;; (Close-on-error needs a wat unwind primitive — a named follow-on; the happy path always closes.)
(:wat::core::defmacro :wat::telemetry::with-span
  [span-name <- :wat::WatAST  sink-addr <- :wat::WatAST
   namespace <- :wat::WatAST  tags <- :wat::WatAST  body <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [uuid-sym   (:wat::core::fresh-symbol "uuid")
     start-sym  (:wat::core::fresh-symbol "start")
     rec-sym    (:wat::core::fresh-symbol "rec")
     h-sym      (:wat::core::fresh-symbol "h")
     result-sym (:wat::core::fresh-symbol "result")
     close-sym  (:wat::core::fresh-symbol "close")]
    `(:wat::core::let
       [~uuid-sym  (:wat::uuid::v4)
        ~start-sym (:wat::time::epoch-nanos (:wat::time::now))
        ~rec-sym   (:wat::telemetry::span::Record
                     :namespace ~namespace :uuid ~uuid-sym :tags ~tags :start-time-ns ~start-sym
                     :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                     :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                     :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                     :logs-flush-after-ms :wat::telemetry::span::DEFAULT-LOGS-FLUSH-AFTER-MS
                     :metrics-flush-after-ms :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS)
        ~h-sym     (:wat::telemetry::span/start :locus (:wat::spawn::thread)
                     :record ~rec-sym :sink-addr ~sink-addr)
        ;; arc 278 the connect'-outcome wall — the generated dial faces all four arms;
        ;; ::Connected → the span sink Peer'; failure arms → assertion-failed! (fatal,
        ;; preserving the pre-wall raise-unwind). Arm-local p/c don't escape to ~body.
        ~span-name (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr ~h-sym))
                     ((:wat::kernel::ConnectOutcome::Connected p) p)
                     ((:wat::kernel::ConnectOutcome::Refused c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                     ((:wat::kernel::ConnectOutcome::Rejected c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                     ((:wat::kernel::ConnectOutcome::Failed c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
        ~result-sym ~body
        ~close-sym (:wat::telemetry::Span/close ~span-name (:wat::telemetry::Span::CloseRequest))]
       ~result-sym)))
