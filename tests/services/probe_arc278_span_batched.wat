;; Co-located fixture for probe_arc278_span_batched.rs — arc 278 item (b).
;; An over-cap buffer must drain by fragmenting into cap-fitting writes. Partial
;; progress is exact. A single over-cap item is RequestTooLarge, not a hang.

(:wat::service::defservice :probe::script-journal
  :satisfies :wat::telemetry::Journal
  :max-frame-bytes 10485760
  :durable   [fail-on <- (:wat::core::Vector :- [:wat::core::i64])
              seen    <- :wat::core::i64
              stored  <- (:wat::core::Vector :- [:wat::telemetry::Log])]
  :ephemeral []
  :impls
  [(write-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::WriteMetricsResponse::Success)))
   (write-logs [s ctx req]
     (:wat::core::let
       [rec  (:probe::script-journal::State/durable s)
        seen (:wat::i64::+ (:probe::script-journal::Record/seen rec) 1)
        fail? (:probe::contains-i64 (:probe::script-journal::Record/fail-on rec) seen)
        rec'  (:probe::script-journal::Record
                :fail-on (:probe::script-journal::Record/fail-on rec)
                :seen seen
                :stored (:probe::script-journal::Record/stored rec))]
       (:wat::core::if fail?
         (:wat::service::Outcome::Reply
           (:probe::script-journal::State :durable rec')
           (:wat::telemetry::Journal::WriteLogsResponse::Fatal
             (:wat::query::Fatal :reason (:wat::query::Fault :message "probe: scripted fail"))))
         (:wat::core::let
           [stored' (:wat::core::foldl
                      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::telemetry::Log])
                                       l   <- :wat::telemetry::Log]
                        -> (:wat::core::Vector :- [:wat::telemetry::Log])
                        (:wat::core::conj acc l))
                      (:probe::script-journal::Record/stored rec)
                      (:wat::telemetry::Journal::WriteLogsRequest/batch req))
            rec2 (:probe::script-journal::Record
                   :fail-on (:probe::script-journal::Record/fail-on rec)
                   :seen seen
                   :stored stored')]
           (:wat::service::Outcome::Reply
             (:probe::script-journal::State :durable rec2)
             (:wat::telemetry::Journal::WriteLogsResponse::Success))))))
   (query-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::QueryMetricsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Metric]) :wat::core::None)))
   (query-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::QueryLogsResponse::Success
         (:probe::script-journal::Record/stored
           (:probe::script-journal::State/durable s))
         :wat::core::None)))
   (sift-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::SiftMetricsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Metric]) :wat::core::None)))
   (sift-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::SiftLogsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Log]) :wat::core::None)))])

(:wat::core::defn :probe::contains-i64
  [v <- (:wat::core::Vector :- [:wat::core::i64])  x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  n <- :wat::core::i64] -> :wat::core::bool
      (:wat::core::or acc (:wat::core::= n x)))
    false v))

(:wat::core::defn :probe::double-n
  [s <- :wat::core::String  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    s
    (:probe::double-n (:wat::string::concat s s) (:wat::i64::- n 1))))

(:wat::core::defn :probe::repeat-x
  [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    ""
    (:wat::core::if (:wat::core::= n 1)
      "x"
      (:wat::core::let
        [half (:probe::repeat-x (:wat::i64::/ n 2))
         s    (:wat::string::concat half half)]
        (:wat::core::if (:wat::core::= n (:wat::i64::* (:wat::i64::/ n 2) 2))
          s
          (:wat::string::concat s "x"))))))

(:wat::core::defn :probe::fat-tags [] -> (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String]
    :pad (:probe::double-n "x" 20)))

(:wat::core::defn :probe::huge-tags [] -> (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
  ;; 2^24 = 16 MiB — one Log's WriteLogsRequest exceeds the 10 MiB cap.
  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String]
    :pad (:probe::double-n "x" 24)))

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

(:wat::core::defn :probe::connect-journal
  [addr <- (:wat::kernel::Address :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply])]
  -> (:wat::kernel::Peer :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply])
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::span-rec
  [ns <- :wat::core::String  tags <- (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])]
  -> :wat::telemetry::span::Record
  (:wat::telemetry::span::Record
    :namespace ns :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
    :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
    :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
    :logs (:wat::core::Vector :- [:wat::telemetry::Log])
    :logs-flush-after-ms 600000
    :metrics-flush-after-ms 600000))

(:wat::core::defn :probe::script-rec
  [fail-on <- (:wat::core::Vector :- [:wat::core::i64])] -> :probe::script-journal::Record
  (:probe::script-journal::Record
    :fail-on fail-on :seen 0
    :stored (:wat::core::Vector :- [:wat::telemetry::Log])))

(:wat::core::defn :probe::classify-log
  [r <- (:wat::kernel::RecvOutcome :- [:wat::telemetry::Span::LogResponse])] -> :wat::core::i64
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::telemetry::Span::LogResponse::Ok) 0)
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

(:wat::core::defn :probe::classify-flush
  [r <- (:wat::kernel::RecvOutcome :- [:wat::telemetry::Span::FlushResponse])] -> :wat::core::i64
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::telemetry::Span::FlushResponse::Done) 0)
        ((:wat::telemetry::Span::FlushResponse::Constraint _err) 1)
        ((:wat::telemetry::Span::FlushResponse::Transient _err) 2)
        ((:wat::telemetry::Span::FlushResponse::Fatal _err) 3)
        ((:wat::telemetry::Span::FlushResponse::RequestTooLarge _bytes _cap) 4)
        ((:wat::telemetry::Span::FlushResponse::RequestMalformed _mpath _mexpected _mgot) 5)))
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

(:wat::core::defn :probe::stored-count
  [journal <- (:wat::kernel::Peer :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply])]
  -> :wat::core::i64
  (:wat::core::match
    (:wat::telemetry::Journal/query-logs journal
      (:wat::telemetry::Journal::QueryLogsRequest
        :namespace "" :time-lo 0 :time-hi 0 :limit 1000 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::telemetry::Journal::QueryLogsResponse::Success rows _cursor)
          (:wat::core::count rows))
        ((:wat::telemetry::Journal::QueryLogsResponse::Transient _err) -1)
        ((:wat::telemetry::Journal::QueryLogsResponse::Fatal _err) -1)
        ((:wat::telemetry::Journal::QueryLogsResponse::RequestTooLarge _b _c) -1)
        ((:wat::telemetry::Journal::QueryLogsResponse::RequestMalformed _p _e _g) -1)))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::do-flush
  [span <- (:wat::kernel::Peer :- [:wat::telemetry::Span::Op :wat::telemetry::Span::Reply])]
  -> :wat::core::i64
  (:probe::classify-flush
    (:wat::telemetry::Span/flush span (:wat::telemetry::Span::FlushRequest))))

;; Row 1: over-cap buffer drains against a working sink (first write fails to create the over-cap).
(:wat::core::defn :user::overcap-drains [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::script-journal/start :locus (:wat::spawn::thread)
             :record (:probe::script-rec (:wat::core::Vector :- [:wat::core::i64] 1)))
     journal (:probe::connect-journal (:probe::script-journal::Handle/addr jh))
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::span-rec "drain-ns" (:probe::fat-tags))
             :sink-addr (:probe::script-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     n     (:probe::drive-log span 40 0)
     _f    (:probe::do-flush span)
     got   (:probe::stored-count journal)]
    (:wat::core::if (:wat::core::and (:wat::i64::>= n 2) (:wat::core::= got n)) 1 got)))

;; Row 2: first chunk lands, second is refused; a later drain lands exactly the suffix (no dup, no loss).
(:wat::core::defn :user::partial-exact [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::script-journal/start :locus (:wat::spawn::thread)
             :record (:probe::script-rec (:wat::core::Vector :- [:wat::core::i64] 1 3)))
     journal (:probe::connect-journal (:probe::script-journal::Handle/addr jh))
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::span-rec "partial-ns" (:probe::fat-tags))
             :sink-addr (:probe::script-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     n     (:probe::drive-log span 40 0)
     _f1   (:probe::do-flush span)
     _f2   (:probe::do-flush span)
     got   (:probe::stored-count journal)]
    (:wat::core::if (:wat::core::and (:wat::i64::>= n 2) (:wat::core::= got n)) 1 got)))

;; Row 3: one item whose encoding alone exceeds the cap → RequestTooLarge, flush returns.
(:wat::core::defn :user::one-item-rtl [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::script-journal/start :locus (:wat::spawn::thread)
             :record (:probe::script-rec (:wat::core::Vector :- [:wat::core::i64])))
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::span-rec "rtl-ns" (:probe::huge-tags))
             :sink-addr (:probe::script-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     _l    (:probe::classify-log
             (:wat::telemetry::Span/log span
               (:wat::telemetry::Span::LogRequest
                 :emitted-from (:wat::kernel::call-site)
                 :level :wat::telemetry::Level::Info
                 :message "x")))
     code  (:probe::do-flush span)]
    (:wat::core::if (:wat::core::= code 4) 1 code)))

;; Row 5: under-cap buffer is exactly one write (a second write would be scripted to fail).
(:wat::core::defn :user::undercap-one-write [] -> :wat::core::i64
  (:wat::core::let
    [jh    (:probe::script-journal/start :locus (:wat::spawn::thread)
             :record (:probe::script-rec (:wat::core::Vector :- [:wat::core::i64] 2)))
     journal (:probe::connect-journal (:probe::script-journal::Handle/addr jh))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record (:probe::span-rec "under-ns" tags)
             :sink-addr (:probe::script-journal::Handle/addr jh))
     span  (:probe::connect-span (:wat::telemetry::span::Handle/addr sph))
     _l1   (:probe::classify-log
             (:wat::telemetry::Span/log span
               (:wat::telemetry::Span::LogRequest
                 :emitted-from (:wat::kernel::call-site)
                 :level :wat::telemetry::Level::Info :message "a")))
     _l2   (:probe::classify-log
             (:wat::telemetry::Span/log span
               (:wat::telemetry::Span::LogRequest
                 :emitted-from (:wat::kernel::call-site)
                 :level :wat::telemetry::Level::Info :message "b")))
     _l3   (:probe::classify-log
             (:wat::telemetry::Span/log span
               (:wat::telemetry::Span::LogRequest
                 :emitted-from (:wat::kernel::call-site)
                 :level :wat::telemetry::Level::Info :message "c")))
     _f    (:probe::do-flush span)
     got   (:probe::stored-count journal)]
    (:wat::core::if (:wat::core::= got 3) 1 got)))

(:wat::core::defn :probe::tiny-log
  [msg <- :wat::core::String] -> :wat::telemetry::Log
  (:wat::telemetry::Log
    :namespace "e" :uuid (:wat::uuid::nil)
    :tags (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
    :time-ns 0 :event-id (:wat::uuid::nil)
    :emitted-from (:wat::kernel::call-site)
    :level :wat::telemetry::Level::Info
    :message msg))

(:wat::core::defn :probe::req-bytes
  [msg <- :wat::core::String] -> :wat::core::i64
  (:wat::telemetry::write-logs-request-bytes
    (:wat::core::Vector :- [:wat::telemetry::Log] (:probe::tiny-log msg))))

;; Largest message length whose 1-item request is still <= cap.
(:wat::core::defn :probe::search-fit
  [lo <- :wat::core::i64  hi <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= (:wat::i64::- hi lo) 1)
    lo
    (:wat::core::let
      [mid (:wat::i64::/ (:wat::core::+ lo hi) 2)
       b   (:probe::req-bytes (:probe::repeat-x mid))
       cap :wat::telemetry::Journal::WRITE-LOGS-MAX-REQUEST-BYTES]
      (:wat::core::if (:wat::i64::> b cap)
        (:probe::search-fit lo mid)
        (:probe::search-fit mid hi)))))

;; Row 4: a 1-item request sized exactly to the cap is SENT (cut at >, not >=).
(:wat::core::defn :user::exact-cap-sent [] -> :wat::core::i64
  (:wat::core::let
    [cap  :wat::telemetry::Journal::WRITE-LOGS-MAX-REQUEST-BYTES
     n    (:probe::search-fit 0 cap)
     msg  (:probe::repeat-x n)
     b    (:probe::req-bytes msg)]
    (:wat::core::if (:wat::core::= b cap)
      (:wat::core::let
        [jh    (:probe::script-journal/start :locus (:wat::spawn::thread)
                 :record (:probe::script-rec (:wat::core::Vector :- [:wat::core::i64])))
         journal (:probe::connect-journal (:probe::script-journal::Handle/addr jh))
         pair  (:wat::telemetry::write-logs-batched journal
                 (:wat::core::Vector :- [:wat::telemetry::Log] (:probe::tiny-log msg)))
         written (:wat::core::first pair)
         recv    (:wat::core::second pair)
         ok?     (:wat::core::match recv
                   ((:wat::kernel::RecvOutcome::Message sresp)
                     (:wat::core::match sresp
                       ((:wat::telemetry::Journal::WriteLogsResponse::Success) 1)
                       ((:wat::telemetry::Journal::WriteLogsResponse::Constraint _e) 0)
                       ((:wat::telemetry::Journal::WriteLogsResponse::Transient _e) 0)
                       ((:wat::telemetry::Journal::WriteLogsResponse::Fatal _e) 0)
                       ((:wat::telemetry::Journal::WriteLogsResponse::RequestTooLarge _b _c) 0)
                       ((:wat::telemetry::Journal::WriteLogsResponse::RequestMalformed _p _x _g) 0)))
                   ((:wat::kernel::RecvOutcome::Lost _c) 0)
                   (:wat::kernel::RecvOutcome::Stopped 0)
                   (:wat::kernel::RecvOutcome::Closed 0))
         got (:probe::stored-count journal)]
        (:wat::core::if (:wat::core::and (:wat::core::= ok? 1)
                         (:wat::core::and (:wat::core::= written 1) (:wat::core::= got 1)))
          1 0))
      -5)))
