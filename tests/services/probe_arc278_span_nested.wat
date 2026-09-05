;; Co-located fixture for probe_arc278_span_nested.rs — arc 278: nested units of work.
;;
;; Nesting works TODAY via nested with-span (no Nest op): an inner with-span closing over the same
;; sink is a child unit of work — its own span, own uuid, own metrics, closes independently. Here
;; outer (ns "outer-ns", incr :o) wraps inner (ns "inner-ns", incr :i). After both close, each
;; namespace holds exactly its own 1 counter Metric. Returns outer*10 + inner = 11.
;;
;; NOTE: this is UN-correlated nesting (child has its own uuid, no parent link). Parent->child
;; correlation (a parent-uuid on the scope, the trace tree) is the enhancement still to build.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     _ws   (:wat::telemetry::with-span outer jaddr "outer-ns" tags
             (:wat::core::do
               (:wat::core::match (:wat::telemetry::Span/incr outer (:wat::telemetry::Span::IncrRequest :name :o))
                 ((:wat::kernel::RecvOutcome::Message _resp) nil)
                 ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
               (:wat::telemetry::with-span inner jaddr "inner-ns" tags
                 (:wat::telemetry::Span/incr inner (:wat::telemetry::Span::IncrRequest :name :i)))))
     jclient (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     oq    (:wat::telemetry::Journal/query-metrics jclient
             (:wat::telemetry::Journal::QueryMetricsRequest :namespace "outer-ns"
               :time-lo 0 :time-hi 9000000000000000000 :limit 100 :cursor :wat::core::None))
     iq    (:wat::telemetry::Journal/query-metrics jclient
             (:wat::telemetry::Journal::QueryMetricsRequest :namespace "inner-ns"
               :time-lo 0 :time-hi 9000000000000000000 :limit 100 :cursor :wat::core::None))
     oc    (:wat::core::match oq ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
             ((:wat::telemetry::Journal::QueryMetricsResponse::Success ms _c) (:wat::core::count ms))
             (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     ic    (:wat::core::match iq ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
             ((:wat::telemetry::Journal::QueryMetricsResponse::Success ms _c) (:wat::core::count ms))
             (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::core::+ (:wat::core::* oc 10) ic)))
