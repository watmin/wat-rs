;; probe-span-log-cost.wat — MEASURE the span's per-call cost, before fixing it.
;;
;; `log`/`incr`/`timed` each measure the size trigger by encoding the WHOLE would-be batch:
;;   would (conj logs0 l) / bytes (string::length (edn::write (WriteLogsRequest would)))
;; That is O(buffer) per call, so O(n^2) to fill a buffer of n. This file measures it rather than
;; asserting it: log N times into a span whose caps are large enough that NO flush fires, and report
;; elapsed nanos for N, 2N, 4N. Quadratic => roughly 4x per doubling. Linear => roughly 2x.
;;
;; The sink is a real journal' over a mem-store', so the cost measured is the span's own arm, not a
;; wire round-trip per log (there is none: the logs accumulate).

(:wat::core::defn :cost::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::select
      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
        (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done)))
    ((:wat::spawn::ServiceEvent::Message _i _m) nil)
    ((:wat::spawn::ServiceEvent::Closed _i) nil)
    ((:wat::spawn::ServiceEvent::Lost _i _c) nil)
    ((:wat::spawn::ServiceEvent::Malformed _i _c) nil)
    ((:wat::spawn::ServiceEvent::Rejected _i _c) nil)
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection _p) nil)
    ((:wat::spawn::ServiceEvent::Admin _m) nil)))

;; log `n` times through a span peer; returns nil.
(:wat::core::defrecord :cost::Note [text <- :wat::core::String])

;; use the call-site widget, which bakes :emitted-from from the macro call site.
(:wat::core::defn :cost::log-n
  [sp <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
   n  <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= n 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match
            (:wat::telemetry::log sp :wat::telemetry::Level::Info (:cost::Note :text "x"))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil))]
      (:cost::log-n sp (:wat::i64::- n 1)))))

;; time `n` logs into a FRESH span (so each run starts from an empty buffer). Returns elapsed nanos.
(:wat::core::defn :cost::time-logs [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     jh  (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
           :record (:wat::telemetry::journal::Record)
           :store-addr (:wat::query::mem-store::Handle/addr msh))
     tags (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     sph (:wat::telemetry::span/start :locus (:wat::spawn::thread)
           :record (:wat::telemetry::span::Record
                     :namespace "cost" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                     :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                     :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                     :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                     :logs-flush-after-ms 600000
                     :metrics-flush-after-ms 600000
                     :logs-max 100000
                     :duration-samples-max 100000)
           :sink-addr (:wat::telemetry::journal::Handle/addr jh))
     sp  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     t0  (:wat::time::epoch-nanos (:wat::time::now))
     _l  (:cost::log-n sp n)
     t1  (:wat::time::epoch-nanos (:wat::time::now))
     ms  (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:cost::time-logs 250)
     b (:cost::time-logs 500)
     c (:cost::time-logs 1000)]
    (:wat::kernel::println
      (:wat::string::interpolate
        "logs=250 -> {a}ms | logs=500 -> {b}ms | logs=1000 -> {c}ms  (quadratic ~= 4x per doubling)"
        :a a :b b :c c))))
