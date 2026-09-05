;; Co-located fixture for probe_arc278_span_surface.rs — arc 278 stone Span.1 acceptance gate.
;;
;; A throwaway toy `:wat::telemetry'::Span` satisfier (NOT `span'`, that is stone Span.2) proving the
;; surface freezes, is satisfiable via `:satisfies`, and all four ops reply through the wire. Mirrors
;; probe_arc278_journal_surface's toy `Journal` satisfier. The toy holds no sink and accumulates no
;; state — each op just replies its Ok/Done.

;; a trivial payload record the producer `edn::write`s into the opaque log message String (Stone B).
(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::service::defservice :probe::toy-span
  :satisfies :wat::telemetry::Span
  :durable   []
  :ephemeral []
  :impls
  [(incr  [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::telemetry::Span::Reply::Incr (:wat::telemetry::Span::IncrResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::telemetry::Span::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::toy-span::Op])])))
   (timed [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::telemetry::Span::Reply::Timed (:wat::telemetry::Span::TimedResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::telemetry::Span::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::toy-span::Op])])))
   (log   [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::telemetry::Span::Reply::Log (:wat::telemetry::Span::LogResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::telemetry::Span::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::toy-span::Op])])))
   ;; item (c) stone A — `flush` joined the surface. This toy satisfier compiled WITHOUT it,
   ;; because serve-op-arms folds over `:impls` and an unimplemented surface op simply gets no arm:
   ;; nothing checks `:impls` against the surface's `:features`. See NOTE-impls-completeness-is-
   ;; unenforced.md. Implemented here so this gate keeps meaning "EVERY declared op replies".
   (flush [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::telemetry::Span::Reply::Flush (:wat::telemetry::Span::FlushResponse::Done))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::telemetry::Span::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::toy-span::Op])])))
   (close [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::telemetry::Span::Reply::Close (:wat::telemetry::Span::CloseResponse::Done))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::telemetry::Span::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::toy-span::Op])])))])

;; :user::compute — start the toy on a thread, dial it, drive all four ops, return 1 iff close -> Done.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h    (:probe::toy-span/start :locus (:wat::spawn::thread) :record (:probe::toy-span::Record))
     span (:wat::core::match (:wat::kernel::connect (:probe::toy-span::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _i   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _t   (:wat::telemetry::Span/timed span
            (:wat::telemetry::Span::TimedRequest :name :fetch :nanos 100))
     _l   (:wat::telemetry::Span/log span
            (:wat::telemetry::Span::LogRequest :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
              :message (:wat::edn::write (:probe::Note :text "hello"))))
     _f   (:wat::telemetry::Span/flush span (:wat::telemetry::Span::FlushRequest))
     c    (:wat::telemetry::Span/close span (:wat::telemetry::Span::CloseRequest))]
    (:wat::core::match c ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Span::CloseResponse::Done) 1)
      (_ 0))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
