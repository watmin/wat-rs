;; :wat::telemetry::WorkUnitLog — producer-side log emitter bound
;; to a :wat::telemetry::Service<Event,_> destination.
;;
;; Arc 091 slice 5. Mirrors arc 087's ConsoleLogger pattern but
;; ships Event::Log rows through the substrate's measurement
;; service rather than tagged-stdio writes. Closure over
;; (handle, caller, now-fn). Built once per producer; passed by
;; reference into hot paths.
;;
;; Per-emission shape:
;;   - capture wall-clock nanos via the injected now-fn
;;   - pull namespace + uuid + tags from the wu (per-scope identity)
;;   - lift caller (keyword) → HolonAST → NoTag for the row
;;   - lift level   (keyword) → HolonAST → NoTag for the row
;;   - lower data  (WatAST) → HolonAST via :wat::holon::Atom (the
;;     polymorphic-Atom WatAST arm structurally lowers any quoted
;;     form per arc 057); wrap that HolonAST in :wat::edn::Tagged.
;;   - build Event::Log; ship as a single-element batch through
;;     Service/batch-log; block on ack
;;
;; Why data is :wat::WatAST (not :wat::holon::HolonAST):
;; struct values don't lift through Atom directly (arc 057's
;; polymorphism covers primitives + HolonAST + WatAST, not Struct).
;; Producers pass quoted/quasiquoted FORMS that capture the typed
;; shape with values spliced in:
;;   (/info wlog wu (:wat::core::quote :hello))
;;   (/info wlog wu (:wat::core::quasiquote
;;                   (:trading::PaperResolved/new ,run-name ...)))
;; The substrate's watast_to_holon arm of Atom does the structural
;; lowering; the resulting HolonAST round-trips through wat-edn.
;;
;; "Sync per event": each /log call is one Service/batch-log
;; round-trip. Same model as the lab archive's
;; `DatabaseHandle.send(entry)` — single-element batch + ack.
;; Mirrors ConsoleLogger's "render-and-send synchronously in the
;; producer's thread" justification: queue-fronting adds latency
;; without eliminating anything at dev/debug log volume.
;;
;; Why caller is keyword-not-HolonAST: precedent from arc 087's
;; ConsoleLogger. The two loggers stay symmetric at their producer-
;; facing surface; the substrate-internal lift to NoTag is the
;; logger's responsibility, not the caller's. (The "labels are
;; ASTs" memory still holds for `data` — that one IS HolonAST.)

(:wat::core::struct :wat::telemetry::WorkUnitLog
  ;; The Service<Event,_>::Handle the logger ships through. Same
  ;; paired (ReqTx<Event>, AckRx) tuple WorkUnit/make-scope captures.
  ;; Two loggers can share a destination by closing over clones of
  ;; the same handle.
  (handle :wat::telemetry::SinkHandles)
  ;; Producer identity — set once at construction. Stamped on every
  ;; emitted row's `caller` column. Caller-discriminator at query
  ;; time: filter rows by who emitted them.
  (caller :wat::core::keyword)
  ;; Clock injection — a closure taking unit, returning a wall-
  ;; clock Instant. Tests pass a deterministic now-fn; production
  ;; passes (lambda (_) (:wat::time::now)). Same pattern as arc 087.
  (now-fn :fn(wat::core::unit)->wat::time::Instant))


;; ─── /log — universal form (caller passes level explicitly) ─────
;;
;; Build the Event::Log row, ship it as a single-element batch,
;; block on ack. Convenience methods (/debug /info /warn /error)
;; sugar over this with the level keyword baked in.
(:wat::core::define
  (:wat::telemetry::WorkUnitLog/log
    (logger :wat::telemetry::WorkUnitLog)
    (wu     :wat::telemetry::WorkUnit)
    (level  :wat::core::keyword)
    (data   :wat::WatAST)
    -> :wat::core::unit)
  (:wat::core::let*
    (((handle :wat::telemetry::SinkHandles)
      (:wat::telemetry::WorkUnitLog/handle logger))
     ((caller :wat::core::keyword)
      (:wat::telemetry::WorkUnitLog/caller logger))
     ((now-fn :fn(wat::core::unit)->wat::time::Instant)
      (:wat::telemetry::WorkUnitLog/now-fn logger))
     ((now :wat::time::Instant) (now-fn ()))
     ((time-ns :wat::core::i64) (:wat::time::epoch-nanos now))
     ;; Per-scope identity — pulled from the wu at every emit so
     ;; each row carries the scope's uuid for cross-table joins
     ;; (Event::Log.uuid == Event::Metric.uuid for rows from the
     ;; same scope).
     ((ns    :wat::holon::HolonAST) (:wat::telemetry::WorkUnit/namespace wu))
     ((uuid  :wat::core::String)               (:wat::telemetry::WorkUnit/uuid wu))
     ((tags  :wat::telemetry::Tags) (:wat::telemetry::WorkUnit/tags wu))
     ;; Lift keyword → HolonAST → NoTag. Atom is polymorphic per
     ;; arc 057 (∀T. T → HolonAST); a runtime keyword Value lifts
     ;; to a holon-ast leaf.
     ((caller-ast :wat::holon::HolonAST) (:wat::holon::Atom caller))
     ((level-ast  :wat::holon::HolonAST) (:wat::holon::Atom level))
     ((ns-notag     :wat::edn::NoTag)  (:wat::edn::NoTag/new ns))
     ((caller-notag :wat::edn::NoTag)  (:wat::edn::NoTag/new caller-ast))
     ((level-notag  :wat::edn::NoTag)  (:wat::edn::NoTag/new level-ast))
     ;; Lower the captured form to a HolonAST (Atom's WatAST arm —
     ;; runtime.rs:6129's `watast_to_holon` — handles the structural
     ;; lowering: primitives → leaves, list-forms → Bundles).
     ((data-holon :wat::holon::HolonAST) (:wat::holon::Atom data))
     ;; Tagged-wrap so the sqlite shim writes via :wat::edn::write
     ;; (round-trip-safe; logs read back as HolonAST and pattern-
     ;; match per arc 091's design).
     ((data-tagged :wat::edn::Tagged) (:wat::edn::Tagged/new data-holon))
     ((event :wat::telemetry::Event)
      (:wat::telemetry::Event::Log
        time-ns ns-notag caller-notag level-notag uuid tags data-tagged))
     ((entries :wat::core::Vector<wat::telemetry::Event>)
      (:wat::core::Vector :wat::telemetry::Event event))
     ((req-tx :wat::telemetry::Service::ReqTx<wat::telemetry::Event>)
      (:wat::core::first handle))
     ((ack-rx :wat::telemetry::Service::AckRx)
      (:wat::core::second handle)))
    (:wat::telemetry::Service/batch-log req-tx ack-rx entries)))


;; ─── Convenience methods — level baked, /log re-routed ──────────
;;
;; All four ship through the same handle to the same Service<Event,_>
;; destination. Unlike ConsoleLogger (which routes :debug/:info to
;; stdout and :warn/:error to stderr), WorkUnitLog has ONE destination
;; — the Event::Log table. Level is a column value (queryable filter),
;; not a routing key.

(:wat::core::define
  (:wat::telemetry::WorkUnitLog/debug
    (logger :wat::telemetry::WorkUnitLog)
    (wu     :wat::telemetry::WorkUnit)
    (data   :wat::WatAST)
    -> :wat::core::unit)
  (:wat::telemetry::WorkUnitLog/log logger wu :debug data))

(:wat::core::define
  (:wat::telemetry::WorkUnitLog/info
    (logger :wat::telemetry::WorkUnitLog)
    (wu     :wat::telemetry::WorkUnit)
    (data   :wat::WatAST)
    -> :wat::core::unit)
  (:wat::telemetry::WorkUnitLog/log logger wu :info data))

(:wat::core::define
  (:wat::telemetry::WorkUnitLog/warn
    (logger :wat::telemetry::WorkUnitLog)
    (wu     :wat::telemetry::WorkUnit)
    (data   :wat::WatAST)
    -> :wat::core::unit)
  (:wat::telemetry::WorkUnitLog/log logger wu :warn data))

(:wat::core::define
  (:wat::telemetry::WorkUnitLog/error
    (logger :wat::telemetry::WorkUnitLog)
    (wu     :wat::telemetry::WorkUnit)
    (data   :wat::WatAST)
    -> :wat::core::unit)
  (:wat::telemetry::WorkUnitLog/log logger wu :error data))
