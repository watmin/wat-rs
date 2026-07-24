;; Co-located fixture for probe_arc278_span_surface.rs — arc 278 stone Span.1 acceptance gate.
;;
;; A throwaway toy `:wat::telemetry'::Span` satisfier (NOT `span'`, that is stone Span.2) proving the
;; surface freezes, is satisfiable via `:satisfies`, and all four ops reply through the wire. Mirrors
;; probe_arc278_journal_surface's toy `Journal` satisfier. The toy holds no sink and accumulates no
;; state — each op just replies its Ok/Done.

;; a trivial payload record the producer `edn::write`s into the opaque log message String (Stone B).
(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::service::defservice :probe::toy-span'
  :satisfies :wat::telemetry::Span
  :durable   []
  :ephemeral []
  :impls
  [(incr  [s req] (:wat::service::Outcome::Reply s (:wat::telemetry::Span::IncrResponse::Ok)))
   (timed [s req] (:wat::service::Outcome::Reply s (:wat::telemetry::Span::TimedResponse::Ok)))
   (log   [s req] (:wat::service::Outcome::Reply s (:wat::telemetry::Span::LogResponse::Ok)))
   (close [s req] (:wat::service::Outcome::Reply s (:wat::telemetry::Span::CloseResponse::Done)))])

;; :user::compute — start the toy on a thread, dial it, drive all four ops, return 1 iff close -> Done.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h    (:probe::toy-span'/start :locus (:wat::spawn::thread) :record (:probe::toy-span'::Record))
     span (:wat::core::match (:wat::kernel::connect' (:probe::toy-span'::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _i   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _t   (:wat::telemetry::Span/timed span
            (:wat::telemetry::Span::TimedRequest :name :fetch :nanos 100))
     _l   (:wat::telemetry::Span/log span
            (:wat::telemetry::Span::LogRequest :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
              :message (:wat::edn::write (:probe::Note :text "hello"))))
     c    (:wat::telemetry::Span/close span (:wat::telemetry::Span::CloseRequest))]
    (:wat::core::match c ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Span::CloseResponse::Done) 1)
      (_ 0))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
