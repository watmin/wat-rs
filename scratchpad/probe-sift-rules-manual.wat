;; Hand-expanded replica of what sift-rules-defsvc emits, WITH debug printlns threaded through
;; the op body, to bisect where the thread dies (macroexpand-1 showed the generated code LOOKS
;; right; something at runtime inside :init or the op handler panics — this isolates it without
;; the ~40s cargo-install cycle the macro's baked-stdlib edit loop requires).

(:wat::core::defsurface :usr::my-sift :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
   (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
   (:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])
   (:wat::core::defrecord :usr::my-sift::SiftRulesRequest
     [namespace <- :wat::core::String time-lo <- :wat::core::i64 time-hi <- :wat::core::i64 limit <- :wat::core::i64])
   (:wat::core::defenum :usr::my-sift::SiftRulesResponse :wat::enum::Pure
     :Deductions [items <- :wat::core::PersistentVector<wat::core::Value>]
     :Fatal [err <- :wat::query::Fault])]
  :features
  [(sift-rules [self <- :usr::my-sift req <- :usr::my-sift::SiftRulesRequest] -> :usr::my-sift::SiftRulesResponse)])

(:wat::service::defservice :usr::my-sift'
  :satisfies :usr::my-sift
  :durable []
  :ephemeral [journal <- :wat::kernel::Peer'<wat::telemetry::Journal::Op,wat::telemetry::Journal::Reply>
              template <- :wat::rete::Session]
  :peers [:wat::telemetry::Journal]
  :init (:wat::core::fn
          [record <- :usr::my-sift'::Record
           journal-addr <- :wat::kernel::Address'<wat::telemetry::Journal::Op,wat::telemetry::Journal::Reply>]
          -> :usr::my-sift'::State
          (:wat::core::let
            [_i0 (:wat::kernel::println "init: entered")
             j   (:wat::kernel::connect' journal-addr)
             _i1 (:wat::kernel::println "init: connected to journal")
             t   (:wat::rete::compile
                   (:wat::core::PersistentVector
                     (:wat::rete::make-rule "usr::hot-rule"
                       (:wat::core::quote [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))])
                       (:wat::core::quote [(:wat::rete::insert (:usr::Hot :c ?c))]))
                     (:wat::rete::make-rule "usr::warn-rule"
                       (:wat::core::quote [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))])
                       (:wat::core::quote [(:wat::rete::insert (:usr::Warn :c ?c))]))))
             _i2 (:wat::kernel::println "init: compiled template")]
            (:usr::my-sift'::State :durable record :journal j :template t)))
  :impls
  [(sift-rules [s req]
     (:wat::service::Outcome::Reply s
       (:wat::core::let
         [journal (:usr::my-sift'::State/journal s)
          template (:usr::my-sift'::State/template s)
          _p0 (:wat::kernel::println "op: got state")
          qresp (:wat::telemetry::Journal/query-logs journal
                  (:wat::telemetry::Journal::QueryLogsRequest
                    :namespace (:usr::my-sift::SiftRulesRequest/namespace req)
                    :time-lo (:usr::my-sift::SiftRulesRequest/time-lo req)
                    :time-hi (:usr::my-sift::SiftRulesRequest/time-hi req)
                    :limit (:usr::my-sift::SiftRulesRequest/limit req)
                    :cursor :wat::core::None))
          _p1 (:wat::kernel::println "op: got qresp")]
         (:wat::core::match qresp -> :usr::my-sift::SiftRulesResponse
           ((:wat::telemetry::Journal::QueryLogsResponse::Success logs _cur)
             (:wat::core::let
               [_p2 (:wat::kernel::println (:wat::core::string::concat "op: logs count=" (:wat::core::str (:wat::core::count logs))))
                ok? (:wat::core::foldl
                      (:wat::core::fn [ok <- :wat::core::bool log <- :wat::telemetry::Log]
                        -> :wat::core::bool
                        (:wat::core::if ok
                          (:wat::core::Vector/contains?
                            (:wat::core::Vector :wat::core::String "usr::Temp" "usr::Hot" "usr::Warn")
                            (:wat::edn::ForeignRecord/class
                              (:wat::edn::read-foreign (:wat::telemetry::Log/message log))))
                          false))
                      true
                      logs)
                _p3 (:wat::kernel::println (:wat::core::string::concat "op: ok?=" (:wat::core::str ok?)))]
               (:wat::core::if ok?
                 (:wat::core::let
                   [items (:wat::core::foldl
                            (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Value>
                                             log <- :wat::telemetry::Log]
                              -> :wat::core::PersistentVector<wat::core::Value>
                              (:wat::core::concat acc
                                (:wat::core::let
                                  [fired (:wat::rete::fire-rules
                                           (:wat::rete::insert template
                                             (:wat::edn::read (:wat::telemetry::Log/message log))))]
                                  (:wat::core::concat
                                    (:wat::rete::query fired :usr::Hot)
                                    (:wat::rete::query fired :usr::Warn)))))
                            (:wat::core::PersistentVector)
                            logs)
                    _p4 (:wat::kernel::println (:wat::core::string::concat "op: items count=" (:wat::core::str (:wat::core::length items))))]
                   (:usr::my-sift::SiftRulesResponse::Deductions items))
                 (:usr::my-sift::SiftRulesResponse::Fatal
                   (:wat::query::Fault :message "sift-rules: a Log message type is not among :defs"))))
             )
           (_ (:usr::my-sift::SiftRulesResponse::Fatal
                (:wat::query::Fault :message "sift-rules: journal query-logs failed"))))))) ])

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
     _p    (:wat::kernel::println "main: wrote logs")
     sh    (:usr::my-sift'/start :locus (:wat::spawn::thread)
             :record (:usr::my-sift'::Record) :journal-addr jaddr)
     _p2   (:wat::kernel::println "main: started sift svc")
     svc   (:wat::kernel::connect' (:usr::my-sift'::Handle/addr sh))
     _p3   (:wat::kernel::println "main: connected")
     resp  (:usr::my-sift/sift-rules svc
             (:usr::my-sift::SiftRulesRequest :namespace "sift-ns" :time-lo 0 :time-hi 100000 :limit 50))]
    (:wat::core::match resp -> :wat::core::nil
      ((:usr::my-sift::SiftRulesResponse::Deductions items)
        (:wat::kernel::println (:wat::core::string::concat "deductions=" (:wat::core::str (:wat::core::length items)))))
      ((:usr::my-sift::SiftRulesResponse::Fatal err)
        (:wat::kernel::println (:wat::core::string::concat "FATAL: " (:wat::query::Fault/message err)))))))
