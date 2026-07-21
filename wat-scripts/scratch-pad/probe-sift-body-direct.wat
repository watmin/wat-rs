;; Direct (non-service, non-IPC) call of the EXACT body sift-rules-defsvc generates for its op,
;; to get a real diagnostic instead of the generic "peer closed" a crashed service thread gives a
;; blind client. Mirrors probe-sift-rules-svc.wat's setup (journal + mem-store on thread, 10 Logs,
;; 5 hot / 5 cold usr::Temp), but runs the op body as a plain :user:: fn — no defservice, no
;; connect'/send'/recv' — so any crash surfaces its FULL RuntimeError directly.

(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])

(:wat::core::defn :usr::template [] -> :wat::rete::Session
  (:wat::rete::compile
    (:wat::core::PersistentVector
      (:wat::rete::make-rule "usr::hot-rule"
        (:wat::core::quote [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))])
        (:wat::core::quote [(:wat::rete::insert (:usr::Hot :c ?c))]))
      (:wat::rete::make-rule "usr::warn-rule"
        (:wat::core::quote [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))])
        (:wat::core::quote [(:wat::rete::insert (:usr::Warn :c ?c))])))))

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
     qr    (:wat::telemetry::Journal/query-logs journal
             (:wat::telemetry::Journal::QueryLogsRequest :namespace "sift-ns" :time-lo 0 :time-hi 100000 :limit 50 :cursor :wat::core::None))]
    (:wat::core::match qr -> :wat::core::nil
      ((:wat::telemetry::Journal::QueryLogsResponse::Success qlogs _cur)
        (:wat::core::let
          [class-ok (:wat::core::foldl
                      (:wat::core::fn [ok <- :wat::core::bool log <- :wat::telemetry::Log] -> :wat::core::bool
                        (:wat::core::if ok
                          (:wat::core::Vector/contains?
                            (:wat::core::Vector :wat::core::String "usr::Temp" "usr::Hot" "usr::Warn")
                            (:wat::core::type (:wat::edn::read-foreign (:wat::telemetry::Log/message log))))
                          false))
                      true
                      qlogs)
           _p1 (:wat::kernel::println (:wat::core::string::concat "class-ok=" (:wat::core::str class-ok)))
           tmpl (:usr::template)
           deds (:wat::core::foldl
                  (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Value> log <- :wat::telemetry::Log]
                    -> :wat::core::PersistentVector<wat::core::Value>
                    (:wat::core::concat acc
                      (:wat::core::let
                        [fired (:wat::rete::fire-rules
                                 (:wat::rete::insert tmpl (:wat::edn::read (:wat::telemetry::Log/message log))))]
                        (:wat::core::concat (:wat::rete::query fired :usr::Hot) (:wat::rete::query fired :usr::Warn)))))
                  (:wat::core::PersistentVector)
                  qlogs)
           _p2 (:wat::kernel::println (:wat::core::string::concat "deds=" (:wat::core::str (:wat::core::length deds))))]
          nil))
      (_ (:wat::kernel::println "query-logs failed")))))
