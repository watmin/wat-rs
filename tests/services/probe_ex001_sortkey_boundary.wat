;; Co-located fixture for probe_ex001_sortkey_boundary.rs — excursus 001 SORTKEY STOP-2.
;;
;; A SortKey record renders longer than a bare timestamp, so a row at exactly time-hi must
;; still fall inside sk-hi. If the maximal sentinel is not actually maximal, query-metrics
;; silently drops the newest data and every existing fixture still passes — none of them
;; queries a boundary. Demonstrate it; do not argue it.
;;
;; Two demonstrations in one summary:
;;   1. query-metrics [T, T] RETURNS the rows at T, including one whose event-id is NOT nil
;;      (the silent-failure case: a nil max-sentinel would exclude it). A row at T+1 is out.
;;   2. the all-f uuid sentinel is lexicographically >= every other SortKey at the same Instant
;;      (nil, a mid uuid, a high-but-not-f uuid) and < a SortKey at T+1. sort-key-hi(T) equals
;;      write(SortKey T all-f).

(:wat::core::defn :user::eid [s <- :wat::core::String] -> :wat::core::Uuid
  (:wat::core::Option/expect (:wat::uuid::from-string s) "canonical uuid"))

(:wat::core::defn :user::b01 [b <- :wat::core::bool] -> :wat::core::String
  (:wat::core::if b "1" "0"))

(:wat::core::defn :user::sk [ns <- :wat::core::i64  u <- :wat::core::Uuid] -> :wat::core::String
  (:wat::edn::write
    (:wat::telemetry::SortKey :time (:wat::time::at-nanos ns) :event-id u)))

(:wat::core::defn :user::metric
  [name <- :wat::core::keyword  ns <- :wat::core::i64  u <- :wat::core::Uuid]
  -> :wat::telemetry::Metric
  (:wat::core::let [tags (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])]
    (:wat::telemetry::Metric
      :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
      :time-ns ns :event-id u
      :start-time-ns 0 :name name
      :value (:wat::telemetry::Numeric::I64 1) :unit :wat::telemetry::Unit::Count)))

(:wat::core::defn :user::qcount
  [journal <- :wat::telemetry::Journal  lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [q (:wat::telemetry::Journal/query-metrics journal
         (:wat::telemetry::Journal::QueryMetricsRequest
           :namespace "probe-ns" :time-lo lo :time-hi hi :limit 10 :cursor :wat::core::None))]
    (:wat::core::match q
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:wat::telemetry::Journal::QueryMetricsResponse::Success ms _c) (:wat::core::count ms))
          (_ -1)))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [T     2000000000
     Tp    2000000001
     nil-u (:wat::uuid::nil)
     mid-u (:user::eid "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
     hi-u  (:user::eid "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee")
     max-u (:user::eid "ffffffff-ffff-ffff-ffff-ffffffffffff")
     sk-nil (:user::sk T nil-u)
     sk-mid (:user::sk T mid-u)
     sk-hi  (:user::sk T hi-u)
     sk-max (:user::sk T max-u)
     sk-nxt (:user::sk Tp nil-u)
     helper (:wat::telemetry::sort-key-hi T)
     msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh))
               ((:wat::kernel::ConnectOutcome::Connected p) p)
               ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
               ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
               ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _wr   (:wat::telemetry::Journal/write-metrics journal
             (:wat::telemetry::Journal::WriteMetricsRequest
               (:wat::core::Vector :- [:wat::telemetry::Metric]
                 (:user::metric :boundary T mid-u)
                 (:user::metric :maxed T max-u)
                 (:user::metric :after Tp nil-u))))
     at-hi (:user::qcount journal T T)
     wide  (:user::qcount journal T Tp)]
    (:wat::core::format
      "hi={hi};wide={wide};nil<=max={n};mid<=max={m};high<=max={h};next>max={x};helper={k}"
      :hi at-hi
      :wide wide
      :n (:user::b01 (:wat::core::<= sk-nil sk-max))
      :m (:user::b01 (:wat::core::<= sk-mid sk-max))
      :h (:user::b01 (:wat::core::<= sk-hi sk-max))
      :x (:user::b01 (:wat::core::> sk-nxt sk-max))
      :k (:user::b01 (:wat::core::= helper sk-max)))))
