;; Co-located fixture for probe_arc278_span_bounded.rs — arc 278 item (c) stone D.
;; logs and duration samples are bounded in ITEMS. Overflow drops the oldest, counts
;; every drop as an ordinary counter, and tells the caller :Dropped{buffered, cap}.

(:wat::service::defservice :probe::fail-journal
  :satisfies :wat::telemetry::Journal
  :max-frame-bytes 10485760
  :durable   []
  :ephemeral []
  :impls
  [(write-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::WriteMetricsResponse::Fatal
         (:wat::query::Fatal :reason (:wat::query::Fault :message "probe: forced fail")))))
   (write-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::WriteLogsResponse::Fatal
         (:wat::query::Fatal :reason (:wat::query::Fault :message "probe: forced fail")))))
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

(:wat::core::defn :probe::connect-store
  [addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::span-rec
  [ns <- :wat::core::String  logs-max <- :wat::core::i64  samples-max <- :wat::core::i64]
  -> :wat::telemetry::span::Record
  (:wat::telemetry::span::Record
    :namespace ns :uuid (:wat::uuid::nil)
    :tags (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
    :start-time-ns 0
    :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
    :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
    :logs (:wat::core::Vector :- [:wat::telemetry::Log])
    :logs-flush-after-ms 600000
    :metrics-flush-after-ms 600000
    :logs-max logs-max
    :duration-samples-max samples-max))

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
        ((:wat::telemetry::Span::LogResponse::RequestTooLarge _bytes _c) 4)
        ((:wat::telemetry::Span::LogResponse::RequestMalformed _p _e _g) 5)))
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
        ((:wat::telemetry::Span::TimedResponse::RequestTooLarge _bytes _c) 4)
        ((:wat::telemetry::Span::TimedResponse::RequestMalformed _p _e _g) 5)))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::log-msg
  [span <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
   msg  <- :wat::core::String] -> :wat::core::i64
  (:probe::classify-log
    (:wat::telemetry::Span/log span
      (:wat::telemetry::Span::LogRequest
        :emitted-from (:wat::kernel::call-site)
        :level :wat::telemetry::Level::Info
        :message msg))))

(:wat::core::defn :probe::log-n
  [span <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
   i <- :wat::core::i64  n <- :wat::core::i64  last <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i n)
    last
    (:probe::log-n span (:wat::core::+ i 1) n
      (:probe::log-msg span (:wat::i64::to-string i)))))

(:wat::core::defn :probe::timed-n
  [span <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])
   i <- :wat::core::i64  n <- :wat::core::i64  last <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i n)
    last
    (:probe::timed-n span (:wat::core::+ i 1) n
      (:probe::classify-timed
        (:wat::telemetry::Span/timed span
          (:wat::telemetry::Span::TimedRequest :name :fetch :nanos i))))))

(:wat::core::defn :probe::metric-named
  [rows <- (:wat::core::Vector :- [:wat::query::Row])  want <- :wat::core::keyword] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  row <- :wat::query::Row] -> :wat::core::i64
      (:wat::core::let
        [m (:wat::edn::read (:wat::query::Row/data row))]
        (:wat::core::if (:wat::core::= (:wat::telemetry::Metric/name m) want)
          (:wat::core::match (:wat::telemetry::Metric/value m)
            ((:wat::telemetry::Numeric::I64 v) v)
            ((:wat::telemetry::Numeric::F64 _f) acc))
          acc)))
    -1 rows))

(:wat::core::defn :probe::scan-metrics
  [client <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   ns     <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::query::Row])
  (:wat::core::let
    [pk (:wat::edn::write (:wat::telemetry::PartitionKey :namespace ns :kind :wat::telemetry::Kind::Metric))
     resp (:wat::query::Store/scan client
            (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 50 :cursor :wat::core::None))]
    (:wat::core::match resp
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:wat::query::Store::ScanResponse::Success rows _cursor) rows)
          (_ (:wat::core::Vector :- [:wat::query::Row]))))
      (_ (:wat::core::Vector :- [:wat::query::Row])))))

(:wat::core::defn :probe::scan-logs
  [client <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   ns     <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::query::Row])
  (:wat::core::let
    [pk (:wat::edn::write (:wat::telemetry::PartitionKey :namespace ns :kind :wat::telemetry::Kind::Log))
     resp (:wat::query::Store/scan client
            (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 50 :cursor :wat::core::None))]
    (:wat::core::match resp
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:wat::query::Store::ScanResponse::Success rows _cursor) rows)
          (_ (:wat::core::Vector :- [:wat::query::Row]))))
      (_ (:wat::core::Vector :- [:wat::query::Row])))))

;; Row 1: failing sink, log far past logs-max — buffer never exceeds it.
(:wat::core::defn :user::bound-holds [] -> :wat::core::i64
  (:wat::core::let
    [jh   (:probe::fail-journal/start :locus (:wat::spawn::thread)
            :record (:probe::fail-journal::Record))
     sph  (:wat::telemetry::span/start :locus (:wat::spawn::thread)
            :record (:probe::span-rec "bound-ns" 3 4096)
            :sink-addr (:probe::fail-journal::Handle/addr jh))
     span (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     _    (:probe::log-n span 0 10 0)
     rec  (:wat::telemetry::span/stop sph)
     n    (:wat::core::count (:wat::telemetry::span::Record/logs rec))]
    (:wat::core::if (:wat::core::= n 3) 1 n)))

;; Row 3: the overflowing log returns Dropped{buffered=3, cap=3}, never Ok.
(:wat::core::defn :user::caller-told [] -> :wat::core::i64
  (:wat::core::let
    [jh   (:probe::fail-journal/start :locus (:wat::spawn::thread)
            :record (:probe::fail-journal::Record))
     sph  (:wat::telemetry::span/start :locus (:wat::spawn::thread)
            :record (:probe::span-rec "told-ns" 3 4096)
            :sink-addr (:probe::fail-journal::Handle/addr jh))
     span (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     c0   (:probe::log-msg span "0")
     c1   (:probe::log-msg span "1")
     c2   (:probe::log-msg span "2")
     c3   (:probe::log-msg span "3")]
    (:wat::core::if
      (:wat::core::and (:wat::core::= c0 0)
        (:wat::core::and (:wat::core::= c1 0)
          (:wat::core::and (:wat::core::= c2 0) (:wat::core::= c3 6))))
      1 c3)))

;; Row 4: after overflow the buffer holds the most recent logs-max, in order.
(:wat::core::defn :user::oldest-go [] -> :wat::core::i64
  (:wat::core::let
    [jh   (:probe::fail-journal/start :locus (:wat::spawn::thread)
            :record (:probe::fail-journal::Record))
     sph  (:wat::telemetry::span/start :locus (:wat::spawn::thread)
            :record (:probe::span-rec "old-ns" 3 4096)
            :sink-addr (:probe::fail-journal::Handle/addr jh))
     span (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     _    (:probe::log-n span 0 10 0)
     rec  (:wat::telemetry::span/stop sph)
     logs (:wat::telemetry::span::Record/logs rec)
     m0   (:wat::telemetry::Log/message (:wat::core::nth logs 0))
     m1   (:wat::telemetry::Log/message (:wat::core::nth logs 1))
     m2   (:wat::telemetry::Log/message (:wat::core::nth logs 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::core::count logs) 3)
        (:wat::core::and (:wat::core::= m0 "7")
          (:wat::core::and (:wat::core::= m1 "8") (:wat::core::= m2 "9"))))
      1 0)))

;; Row 2: drain against a working sink — :logs-dropped is exactly 7 (10 logged, max 3).
(:wat::core::defn :user::drop-count-exact [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::span-rec "drop-ns" 3 4096)
             :sink-addr (:wat::telemetry::journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     _     (:probe::log-n span 0 10 0)
     _f    (:wat::telemetry::Span/flush span (:wat::telemetry::Span::FlushRequest))
     client (:probe::connect-store maddr)
     rows  (:probe::scan-metrics client "drop-ns")
     got   (:probe::metric-named rows :logs-dropped)]
    (:wat::core::if (:wat::core::= got 7) 1 got)))

;; Row 5a: timed bound holds + oldest samples remain + caller told Dropped.
(:wat::core::defn :user::samples-bound [] -> :wat::core::i64
  (:wat::core::let
    [jh   (:probe::fail-journal/start :locus (:wat::spawn::thread)
            :record (:probe::fail-journal::Record))
     sph  (:wat::telemetry::span/start :locus (:wat::spawn::thread)
            :record (:probe::span-rec "samp-ns" 4096 3)
            :sink-addr (:probe::fail-journal::Handle/addr jh))
     span (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     last (:probe::timed-n span 0 10 0)
     rec  (:wat::telemetry::span/stop sph)
     ds   (:wat::telemetry::span::Record/durations rec)
     samples (:wat::core::match (:wat::hashmap::get ds :fetch)
               (:wat::core::None (:wat::core::Vector :- [:wat::core::i64]))
               ((:wat::core::Some v) v))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= last 6)
        (:wat::core::and (:wat::core::= (:wat::core::count samples) 3)
          (:wat::core::and (:wat::core::= (:wat::core::nth samples 0) 7)
            (:wat::core::and (:wat::core::= (:wat::core::nth samples 1) 8)
              (:wat::core::= (:wat::core::nth samples 2) 9)))))
      1 0)))

;; Row 5b: :samples-dropped is exactly 7 after a working-sink flush.
(:wat::core::defn :user::samples-drop-count [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::span-rec "sdrop-ns" 4096 3)
             :sink-addr (:wat::telemetry::journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     _     (:probe::timed-n span 0 10 0)
     _f    (:wat::telemetry::Span/flush span (:wat::telemetry::Span::FlushRequest))
     client (:probe::connect-store maddr)
     rows  (:probe::scan-metrics client "sdrop-ns")
     got   (:probe::metric-named rows :samples-dropped)]
    (:wat::core::if (:wat::core::= got 7) 1 got)))
