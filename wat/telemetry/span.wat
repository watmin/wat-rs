;; wat/telemetry/span.wat — arc 278 STONE Span.2: :wat::telemetry'::span' — the PRODUCER service.
;;
;; A short-lived `:satisfies :wat::telemetry'::Span` actor, one per unit of work. It HOLDS a
;; `:wat::telemetry'::Journal` peer (S4d `:peers` — the sink, given at start) and threads PURE
;; accumulating state (counters + duration samples) through its serve loop. On `close` it emits the
;; accumulated state as Metrics to the sink (each counter -> 1 Metric; each duration name -> a
;; `<name>/count` + a `<name>/duration` Metric) and passes the sink's write outcome through.
;;
;; Provisioning is INLINE at the call site (the scope law: start+connect+use in one lexical scope),
;; done by the `with-span` macro (stone Span.3); the durable Record carries namespace/uuid/tags/
;; start-time-ns (minted at the call site) + empty counters/durations.
;;
;; Loads after wat/telemetry/journal.wat (needs Journal + Metric/Log/Scope + time).

;; ── the service ─────────────────────────────────────────────────────────────────
(:wat::service::defservice :wat::telemetry::span
  :satisfies :wat::telemetry::Span
  :durable   [namespace     <- :wat::core::String
              uuid          <- :wat::core::Uuid
              tags          <- :wat::telemetry::Tags
              start-time-ns <- :wat::core::i64
              counters      <- (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
              durations     <- (:wat::core::HashMap :wat::core::keyword :wat::telemetry::Samples)]
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
  [;; incr — PURE: counters[name] + 1, thread new state.
   (incr [s ctx req]
     (:wat::core::let
       [name (:wat::telemetry::Span::IncrRequest/name req)
        rec  (:wat::telemetry::span::State/durable s)
        cs   (:wat::telemetry::span::Record/counters rec)
        next (:wat::core::match (:wat::core::HashMap/get cs name) 
               (:wat::core::None 1)
               ((:wat::core::Some v) (:wat::core::+ v 1)))
        rec' (:wat::telemetry::span::Record
               :namespace (:wat::telemetry::span::Record/namespace rec)
               :uuid (:wat::telemetry::span::Record/uuid rec)
               :tags (:wat::telemetry::span::Record/tags rec)
               :start-time-ns (:wat::telemetry::span::Record/start-time-ns rec)
               :counters (:wat::core::HashMap/assoc cs name next)
               :durations (:wat::telemetry::span::Record/durations rec))]
       (:wat::service::Outcome::Reply
         (:wat::telemetry::span::State :durable rec' :sink (:wat::telemetry::span::State/sink s))
         (:wat::telemetry::Span::IncrResponse::Ok))))

   ;; timed — PURE: durations[name] ++ nanos, thread new state.
   (timed [s ctx req]
     (:wat::core::let
       [name  (:wat::telemetry::Span::TimedRequest/name req)
        nanos (:wat::telemetry::Span::TimedRequest/nanos req)
        rec   (:wat::telemetry::span::State/durable s)
        ds    (:wat::telemetry::span::Record/durations rec)
        samples (:wat::core::match (:wat::core::HashMap/get ds name) 
                  (:wat::core::None (:wat::core::Vector :wat::core::i64))
                  ((:wat::core::Some v) v))
        rec'  (:wat::telemetry::span::Record
                :namespace (:wat::telemetry::span::Record/namespace rec)
                :uuid (:wat::telemetry::span::Record/uuid rec)
                :tags (:wat::telemetry::span::Record/tags rec)
                :start-time-ns (:wat::telemetry::span::Record/start-time-ns rec)
                :counters (:wat::telemetry::span::Record/counters rec)
                :durations (:wat::core::HashMap/assoc ds name (:wat::core::conj samples nanos)))]
       (:wat::service::Outcome::Reply
         (:wat::telemetry::span::State :durable rec' :sink (:wat::telemetry::span::State/sink s))
         (:wat::telemetry::Span::TimedResponse::Ok))))

   ;; log — build a Log from this span's scope, write it through the sink NOW; state unchanged.
   (log [s ctx req]
     (:wat::core::let
       [rec (:wat::telemetry::span::State/durable s)
        now (:wat::time::epoch-nanos (:wat::time::now))
        l   (:wat::telemetry::Log
              :namespace (:wat::telemetry::span::Record/namespace rec)
              :uuid (:wat::telemetry::span::Record/uuid rec)
              :tags (:wat::telemetry::span::Record/tags rec)
              :time-ns now
              :emitted-from (:wat::telemetry::Span::LogRequest/emitted-from req)
              :level (:wat::telemetry::Span::LogRequest/level req)
              :message (:wat::telemetry::Span::LogRequest/message req))
        _w  (:wat::telemetry::Journal/write-logs (:wat::telemetry::span::State/sink s)
              (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :wat::telemetry::Log l)))]
       (:wat::service::Outcome::Reply s (:wat::telemetry::Span::LogResponse::Ok))))

   ;; close — emit counters + durations as Metrics to the sink; pass the write outcome through.
   (close [s ctx req]
     (:wat::core::let
       [rec (:wat::telemetry::span::State/durable s)
        ns    (:wat::telemetry::span::Record/namespace rec)
        uuid  (:wat::telemetry::span::Record/uuid rec)
        tags  (:wat::telemetry::span::Record/tags rec)
        start (:wat::telemetry::span::Record/start-time-ns rec)
        cs    (:wat::telemetry::span::Record/counters rec)
        ds    (:wat::telemetry::span::Record/durations rec)
        now   (:wat::time::epoch-nanos (:wat::time::now))
        ;; counter metrics: one per counter key.
        counter-metrics
        (:wat::core::foldl
          (:wat::core::fn [acc <- (:wat::core::Vector :wat::telemetry::Metric) name <- :wat::core::keyword]
            -> (:wat::core::Vector :wat::telemetry::Metric)
            (:wat::core::conj acc
              (:wat::telemetry::Metric :namespace ns :uuid uuid :tags tags :time-ns now
                :start-time-ns start :name name
                :value (:wat::telemetry::Numeric::I64
                         (:wat::core::Option/expect (:wat::core::HashMap/get cs name) "counter present"))
                :unit :wat::telemetry::Unit::Count)))
          (:wat::core::Vector :wat::telemetry::Metric)
          (:wat::core::HashMap/keys cs))
        ;; duration metrics: <name>/count + <name>/duration per duration key, folded onto the counters.
        all-metrics
        (:wat::core::foldl
          (:wat::core::fn [acc <- (:wat::core::Vector :wat::telemetry::Metric) name <- :wat::core::keyword]
            -> (:wat::core::Vector :wat::telemetry::Metric)
            (:wat::core::let
              [samples (:wat::core::Option/expect (:wat::core::HashMap/get ds name) "duration present")
               cnt (:wat::core::count samples)
               total (:wat::core::foldl
                       (:wat::core::fn [a <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ a x))
                       0 samples)
               base (:wat::core::keyword/to-string name)
               count-name (:wat::core::keyword/from-string (:wat::core::format "{base}/count" :base base))
               dur-name   (:wat::core::keyword/from-string (:wat::core::format "{base}/duration" :base base))]
              (:wat::core::conj
                (:wat::core::conj acc
                  (:wat::telemetry::Metric :namespace ns :uuid uuid :tags tags :time-ns now
                    :start-time-ns start :name count-name
                    :value (:wat::telemetry::Numeric::I64 cnt) :unit :wat::telemetry::Unit::Count))
                (:wat::telemetry::Metric :namespace ns :uuid uuid :tags tags :time-ns now
                  :start-time-ns start :name dur-name
                  :value (:wat::telemetry::Numeric::I64 total) :unit :wat::telemetry::Unit::Nanos))))
          counter-metrics
          (:wat::core::HashMap/keys ds))
        resp (:wat::telemetry::Journal/write-metrics (:wat::telemetry::span::State/sink s)
               (:wat::telemetry::Journal::WriteMetricsRequest all-metrics))
        cresp (:wat::core::match resp
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
                    ;; wire-breach at the sink peer propagates outward as our own op's breach.
                    ((:wat::telemetry::Journal::WriteMetricsResponse::RequestTooLarge bytes cap)
                      (:wat::telemetry::Span::CloseResponse::RequestTooLarge bytes cap))
                    ((:wat::telemetry::Journal::WriteMetricsResponse::RequestMalformed mpath mexpected mgot)
                      (:wat::telemetry::Span::CloseResponse::RequestMalformed mpath mexpected mgot))))
                ;; a lost/closed sink peer must NOT kill this span service — map to our own Fatal
                ;; response value and KEEP SERVING (the client-triggerable-DoS arc forbids raise).
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::telemetry::Span::CloseResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
                ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape
                ;; (the operation cannot complete either way) with the TRUE reason: the
                ;; journal sink peer was alive and the substrate was asked to stop.
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::telemetry::Span::CloseResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "span.wat: stop requested mid-call — the journal sink peer was ALIVE"))))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::telemetry::Span::CloseResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "span.wat: journal sink peer closed")))))]
       (:wat::service::Outcome::Reply s cresp)))])

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
       [~uuid-sym  (:wat::core::Uuid/v4)
        ~start-sym (:wat::time::epoch-nanos (:wat::time::now))
        ~rec-sym   (:wat::telemetry::span::Record
                     :namespace ~namespace :uuid ~uuid-sym :tags ~tags :start-time-ns ~start-sym
                     :counters (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
                     :durations (:wat::core::HashMap :wat::core::keyword :wat::telemetry::Samples))
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
