;; Probe: sift-rules-defsvc end to end on THREAD locus. mem-store' <- journal' <- my-sift'
;; (macro-generated). Write 10 Logs (5 hot usr::Temp c=60, 5 cold c=10); call sift-rules; expect
;; 10 deductions (5 hot x 2 rules = 10; cold contributes 0).

(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])

(:wat::query::sift-rules-defsvc
  :name :usr::my-sift
  :defs [(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
         (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
         (:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])]
  :rules [(:wat::rete::defrule :usr::hot-rule
            :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
            :then (:wat::rete::insert (:usr::Hot :c ?c)))
          (:wat::rete::defrule :usr::warn-rule
            :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
            :then (:wat::rete::insert (:usr::Warn :c ?c)))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::kernel::connect' jaddr)
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     idxs  (:wat::core::range 0 10)
     logs  (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log
                 (:wat::core::let
                   [hot? (:wat::core::i64::< (:wat::core::mod i 2) 1)
                    c    (:wat::core::if hot? 60 10)
                    msg  (:wat::edn::write (:usr::Temp :c c))]
                   (:wat::telemetry::Log :namespace "sift-ns" :uuid (:wat::core::Uuid/nil) :tags tags
                     :time-ns (:wat::core::i64::+ i 1) :caller :probe
                     :level :wat::telemetry::Level::Info :message msg)))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs))
     sh    (:usr::my-sift'/start :locus (:wat::spawn::thread)
             :record (:usr::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::kernel::connect' (:usr::my-sift'::Handle/addr sh))
     resp  (:usr::my-sift/sift-rules svc
             (:usr::my-sift::SiftRulesRequest :namespace "sift-ns" :time-lo 0 :time-hi 100000 :limit 50))]
    (:wat::core::match resp -> :wat::core::nil
      ((:usr::my-sift::SiftRulesResponse::Deductions items)
        (:wat::kernel::println (:wat::core::string::concat "deductions=" (:wat::core::str (:wat::core::length items)))))
      ((:usr::my-sift::SiftRulesResponse::Fatal err)
        (:wat::kernel::println (:wat::core::string::concat "FATAL: " (:wat::query::Fault/message err)))))))
