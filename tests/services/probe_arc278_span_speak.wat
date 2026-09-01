;; Co-located fixture for probe_arc278_span_speak.rs — arc 278 item (c) stone C.
;;
;; A size-triggered flush must speak. Three arms (log/incr/timed) used to throw away
;; (second pair0). Ok stays "accepted"; Constraint/Transient/Fatal are the sink's
;; failures surfaced pass-through. The arriving item is still buffered on failure.
;;
;; Proof of survival is span/stop's Record (the durable buffer). A later single
;; flush of the combined batch cannot land: the size trigger fires at would>=cap,
;; so the kept arriving item makes the buffer >=cap, and the journal server
;; rejects with `>` — that is cap arithmetic, not dropped data.

(:wat::service::defservice :probe::fail-journal
  :satisfies :wat::telemetry::Journal
  :max-frame-bytes 10485760
  :durable   []
  :ephemeral []
  :impls
  [(write-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::WriteMetricsResponse::Fatal
         (:wat::query::Fatal :reason (:wat::query::Fault :message "probe: forced write-metrics fail")))))
   (write-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::WriteLogsResponse::Fatal
         (:wat::query::Fatal :reason (:wat::query::Fault :message "probe: forced write-logs fail")))))
   (query-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::QueryMetricsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Metric]) :wat::core::None)))
   (query-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::QueryLogsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Log]) :wat::core::None)))
   (sift-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::SiftMetricsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Metric]) :wat::core::None)))
   (sift-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::SiftLogsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Log]) :wat::core::None)))])

(:wat::core::defn :probe::double-n
  [s <- :wat::core::String  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    s
    (:probe::double-n (:wat::string::concat s s) (:wat::i64::- n 1))))

(:wat::core::defn :probe::fat-tags [] -> (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
  ;; 2^20 = 1 MiB. Inflates every Log/Metric so ~10 writes cross the 10 MiB journal cap.
  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String]
    :pad (:probe::double-n "x" 20)))

(:wat::core::defn :probe::connect-span
  [addr <- (:wat::kernel::Address :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])]
  -> (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::failing-span-rec
  [ns <- :wat::core::String] -> :wat::telemetry::span::Record
  (:wat::telemetry::span::Record
    :namespace ns :uuid (:wat::uuid::nil) :tags (:probe::fat-tags) :start-time-ns 0
    :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
    :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
    :logs (:wat::core::Vector :- [:wat::telemetry::Log])
    :logs-flush-after-ms 600000
    :metrics-flush-after-ms 600000
    :logs-max :wat::telemetry::span::DEFAULT-LOGS-MAX
    :duration-samples-max :wat::telemetry::span::DEFAULT-DURATION-SAMPLES-MAX))

;; 0=Ok 1=Constraint 2=Transient 3=Fatal 4=RTL 5=Malformed. Wire death is fatal to the probe.
(:wat::core::defn :probe::classify-log
  [r <- (:wat::kernel::RecvOutcome :- [:wat::telemetry::Span::LogResponse])] -> :wat::core::i64
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::telemetry::Span::LogResponse::Ok) 0)
        ((:wat::telemetry::Span::LogResponse::Dropped _buffered _cap) 6)
        ((:wat::telemetry::Span::LogResponse::Constraint _err) 1)
        ((:wat::telemetry::Span::LogResponse::Transient _err) 2)
        ((:wat::telemetry::Span::LogResponse::Fatal _err) 3)
        ((:wat::telemetry::Span::LogResponse::RequestTooLarge _bytes _cap) 4)
        ((:wat::telemetry::Span::LogResponse::RequestMalformed _mpath _mexpected _mgot) 5)))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::classify-timed
  [r <- (:wat::kernel::RecvOutcome :- [:wat::telemetry::Span::TimedResponse])] -> :wat::core::i64
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::telemetry::Span::TimedResponse::Ok) 0)
        ((:wat::telemetry::Span::TimedResponse::Dropped _buffered _cap) 6)
        ((:wat::telemetry::Span::TimedResponse::Constraint _err) 1)
        ((:wat::telemetry::Span::TimedResponse::Transient _err) 2)
        ((:wat::telemetry::Span::TimedResponse::Fatal _err) 3)
        ((:wat::telemetry::Span::TimedResponse::RequestTooLarge _bytes _cap) 4)
        ((:wat::telemetry::Span::TimedResponse::RequestMalformed _mpath _mexpected _mgot) 5)))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::classify-incr
  [r <- (:wat::kernel::RecvOutcome :- [:wat::telemetry::Span::IncrResponse])] -> :wat::core::i64
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::telemetry::Span::IncrResponse::Ok) 0)
        ((:wat::telemetry::Span::IncrResponse::Constraint _err) 1)
        ((:wat::telemetry::Span::IncrResponse::Transient _err) 2)
        ((:wat::telemetry::Span::IncrResponse::Fatal _err) 3)
        ((:wat::telemetry::Span::IncrResponse::RequestTooLarge _bytes _cap) 4)
        ((:wat::telemetry::Span::IncrResponse::RequestMalformed _mpath _mexpected _mgot) 5)))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::failure? [code <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::and (:wat::i64::>= code 1) (:wat::i64::<= code 3)))

(:wat::core::defn :probe::drive-log
  [span <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
   attempts <- :wat::core::i64
   logged <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= attempts 0)
    -1
    (:wat::core::let
      [code (:probe::classify-log
              (:wat::telemetry::Span/log span
                (:wat::telemetry::Span::LogRequest
                  :emitted-from (:wat::kernel::call-site)
                  :level :wat::telemetry::Level::Info
                  :message "x")))
       n (:wat::core::+ logged 1)]
      (:wat::core::if (:wat::core::= code 0)
        (:probe::drive-log span (:wat::i64::- attempts 1) n)
        (:wat::core::if (:probe::failure? code) n -4)))))

(:wat::core::defn :probe::drive-timed
  [span <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
   attempts <- :wat::core::i64
   n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= attempts 0)
    -1
    (:wat::core::let
      [code (:probe::classify-timed
              (:wat::telemetry::Span/timed span
                (:wat::telemetry::Span::TimedRequest :name :fetch :nanos 1)))
       n1 (:wat::core::+ n 1)]
      (:wat::core::if (:wat::core::= code 0)
        (:probe::drive-timed span (:wat::i64::- attempts 1) n1)
        (:wat::core::if (:probe::failure? code) n1 -4)))))

(:wat::core::defn :probe::drive-incr
  [span <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
   attempts <- :wat::core::i64
   n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= attempts 0)
    -1
    (:wat::core::let
      [kw (:wat::keyword::from-string
            (:wat::string::concat "c" (:wat::i64::to-string n)))
       code (:probe::classify-incr
              (:wat::telemetry::Span/incr span
                (:wat::telemetry::Span::IncrRequest :name kw)))
       n1 (:wat::core::+ n 1)]
      (:wat::core::if (:wat::core::= code 0)
        (:probe::drive-incr span (:wat::i64::- attempts 1) n1)
        (:wat::core::if (:probe::failure? code) n1 -4)))))

(:wat::core::defn :probe::sample-count
  [rec <- :wat::telemetry::span::Record] -> :wat::core::i64
  (:wat::core::match (:wat::hashmap::get (:wat::telemetry::span::Record/durations rec) :fetch)
    (:wat::core::None 0)
    ((:wat::core::Some v) (:wat::core::count v))))

;; Row 1: size-triggered log flush against a failing sink must NOT reply Ok.
(:wat::core::defn :user::logs-speak [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::fail-journal/start :locus (:wat::spawn::thread)
             :record (:probe::fail-journal::Record))
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::failing-span-rec "speak-logs")
             :sink-addr (:probe::fail-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     n     (:probe::drive-log span 40 0)]
    (:wat::core::if (:wat::i64::>= n 2) 1 n)))

;; Row 2: the arriving log is still in the durable buffer (un-flushed batch AND the trigger).
(:wat::core::defn :user::logs-survive [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::fail-journal/start :locus (:wat::spawn::thread)
             :record (:probe::fail-journal::Record))
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::failing-span-rec "survive-logs")
             :sink-addr (:probe::fail-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     n     (:probe::drive-log span 40 0)
     rec   (:wat::telemetry::span/stop sph)
     got   (:wat::core::count (:wat::telemetry::span::Record/logs rec))]
    (:wat::core::if (:wat::core::and (:wat::i64::>= n 2) (:wat::core::= got n)) 1 got)))

;; Row 3 timed: failure on TimedResponse, arriving sample survives.
(:wat::core::defn :user::timed-speak-survive [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::fail-journal/start :locus (:wat::spawn::thread)
             :record (:probe::fail-journal::Record))
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::failing-span-rec "speak-timed")
             :sink-addr (:probe::fail-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     n     (:probe::drive-timed span 40 0)
     rec   (:wat::telemetry::span/stop sph)
     got   (:probe::sample-count rec)]
    (:wat::core::if (:wat::core::and (:wat::i64::>= n 2) (:wat::core::= got n)) 1 got)))

;; Row 3 incr: failure on IncrResponse, arriving counter survives.
(:wat::core::defn :user::incr-speak-survive [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::fail-journal/start :locus (:wat::spawn::thread)
             :record (:probe::fail-journal::Record))
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::failing-span-rec "speak-incr")
             :sink-addr (:probe::fail-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     n     (:probe::drive-incr span 40 0)
     rec   (:wat::telemetry::span/stop sph)
     got   (:wat::core::count
             (:wat::hashmap::keys (:wat::telemetry::span::Record/counters rec)))]
    (:wat::core::if (:wat::core::and (:wat::i64::>= n 2) (:wat::core::= got n)) 1 got)))

;; Row 4: a normal log, no size trigger, is Ok.
(:wat::core::defn :user::ok-accepted [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     rec   (:wat::telemetry::span::Record
             :namespace "ok-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
             :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
             :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
             :logs (:wat::core::Vector :- [:wat::telemetry::Log])
             :logs-flush-after-ms :wat::telemetry::span::DEFAULT-LOGS-FLUSH-AFTER-MS
             :metrics-flush-after-ms :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS
             :logs-max :wat::telemetry::span::DEFAULT-LOGS-MAX
             :duration-samples-max :wat::telemetry::span::DEFAULT-DURATION-SAMPLES-MAX)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record rec :sink-addr jaddr)
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     code  (:probe::classify-log
             (:wat::telemetry::Span/log span
               (:wat::telemetry::Span::LogRequest
                 :emitted-from (:wat::kernel::call-site)
                 :level :wat::telemetry::Level::Info
                 :message "hello")))]
    (:wat::core::if (:wat::core::= code 0) 1 code)))
