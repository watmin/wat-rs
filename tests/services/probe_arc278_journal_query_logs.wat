;; Co-located fixture for probe_arc278_journal_query_logs.rs — arc 278 T2: query-logs (thread).
;; Symmetric to journal_query (metrics): write 2 Logs (t=1s,2s), query-logs BROAD [0,3s] -> 2,
;; NARROW [1.5s,3s] -> 1. Returns 2*10+1 = 21. Closes the un-probed query-logs gap.

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store'::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store'::Handle/addr msh)
     jh    (:wat::telemetry'::journal'/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry'::journal'::Record) :store-addr maddr)
     journal (:wat::kernel::connect' (:wat::telemetry'::journal'::Handle/addr jh))
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     l1    (:wat::telemetry'::Log :namespace "probe-ns" :uuid (:wat::core::Uuid/nil) :tags tags
             :time-ns 1000000000 :caller :c1 :level :wat::telemetry'::Level::Info
             :message (:probe::Note :text "one"))
     l2    (:wat::telemetry'::Log :namespace "probe-ns" :uuid (:wat::core::Uuid/nil) :tags tags
             :time-ns 2000000000 :caller :c2 :level :wat::telemetry'::Level::Warn
             :message (:probe::Note :text "two"))
     _wr   (:wat::telemetry'::Journal/write-logs journal
             (:wat::telemetry'::Journal::WriteLogsRequest (:wat::core::Vector :wat::telemetry'::Log l1 l2)))
     bq    (:wat::telemetry'::Journal/query-logs journal
             (:wat::telemetry'::Journal::QueryLogsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 3000000000 :limit 100 :cursor :wat::core::None))
     nq    (:wat::telemetry'::Journal/query-logs journal
             (:wat::telemetry'::Journal::QueryLogsRequest :namespace "probe-ns"
               :time-lo 1500000000 :time-hi 3000000000 :limit 100 :cursor :wat::core::None))
     bc    (:wat::core::match bq -> :wat::core::i64
             ((:wat::telemetry'::Journal::QueryLogsResponse::Success ls _c) (:wat::core::count ls))
             (_ -1))
     nc    (:wat::core::match nq -> :wat::core::i64
             ((:wat::telemetry'::Journal::QueryLogsResponse::Success ls _c) (:wat::core::count ls))
             (_ -1))]
    (:wat::core::+ (:wat::core::* bc 10) nc)))
