;; :wat::std::telemetry::Sqlite — substrate sqlite-backed destination
;; for :wat::std::telemetry::Service<E,G>.
;;
;; Arc 083 slice 2. Companion to arc 081's Console — same composition
;; pattern with the substrate Service shell, different sink discipline.
;;
;; Console (arc 081) is a pure dispatcher factory: takes con-tx +
;; format, returns :fn(E)->() built in the caller's thread. Sqlite
;; cannot be a pure factory — Db is thread-owned (CIRCUIT.md rule 1).
;; The worker that opens the Db must be the one that uses it. So the
;; substrate ships a worker entry (Sqlite/run) that opens the Db
;; INSIDE its thread, runs the consumer's schema-install hook against
;; that thread-local Db, then enters Service/loop with the consumer's
;; per-entry dispatcher curried over the same thread-local Db.
;;
;; TWO FLAT HOOKS — the consumer's seam:
;;
;;   schema-install :fn(Db)->()      Runs once at startup. The body
;;                                    issues `(execute-ddl db ddl)`
;;                                    calls for each schema.
;;
;;   dispatcher     :fn(Db,E)->()    Runs per entry. Reads naturally
;;                                    on the consumer side (Db +
;;                                    entry as positional args). The
;;                                    substrate curries Db before
;;                                    handing :fn(E)->() to
;;                                    Service/loop.
;;
;; A single nested hook (`:fn(Db)->fn(E)->()` returning the dispatcher
;; closure) was considered and rejected: verbose is honest. Two flat
;; hooks compose without anticipating shared state no consumer needs.

(:wat::load-file! "../../sqlite/Db.wat")

(:wat::core::use! :rust::sqlite::auto-prep)
(:wat::core::use! :rust::sqlite::auto-install-schemas)
(:wat::core::use! :rust::sqlite::auto-dispatch)


;; ─── Worker entry — opens Db, installs schemas, runs Service/loop ─

;; Top-level so :wat::kernel::spawn can route to it. Generic over E
;; (consumer's entry type) and G (substrate cadence gate). Both
;; hooks execute INSIDE this thread; the curried dispatcher closure
;; captures the thread-local Db without crossing thread boundaries.
(:wat::core::define
  (:wat::std::telemetry::Sqlite/run<E,G>
    (path :String)
    (rxs :Vec<wat::std::telemetry::Service::ReqRx<E>>)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (schema-install :fn(wat::sqlite::Db)->())
    (dispatcher :fn(wat::sqlite::Db,E)->())
    (stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<E>)
    -> :())
  (:wat::core::let*
    (((db :wat::sqlite::Db) (:wat::sqlite::open path))
     ((_install :()) (schema-install db))
     ((curried :fn(E)->())
      (:wat::core::lambda ((entry :E) -> :())
        (dispatcher db entry))))
    (:wat::std::telemetry::Service/run
      rxs cadence curried stats-translator)))


;; ─── Sqlite/spawn — caller-side wiring ──────────────────────────

;; Builds N bounded(1) Request<E> pairs, wraps senders in a
;; HandlePool, spawns Sqlite/run on a new thread, returns the
;; standard Service::Spawn<E> tuple. :user::main pops handles,
;; finishes the pool, distributes, joins the driver per the
;; CIRCUIT.md wiring discipline.
(:wat::core::define
  (:wat::std::telemetry::Sqlite/spawn<E,G>
    (path :String)
    (count :i64)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (schema-install :fn(wat::sqlite::Db)->())
    (dispatcher :fn(wat::sqlite::Db,E)->())
    (stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<E>)
    -> :wat::std::telemetry::Service::Spawn<E>)
  (:wat::core::let*
    (((pairs :Vec<wat::std::telemetry::Service::ReqChannel<E>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :i64) -> :wat::std::telemetry::Service::ReqChannel<E>)
          (:wat::kernel::make-bounded-queue
            :wat::std::telemetry::Service::Request<E> 1))))
     ((req-txs :Vec<wat::std::telemetry::Service::ReqTx<E>>)
      (:wat::core::map pairs
        (:wat::core::lambda
          ((p :wat::std::telemetry::Service::ReqChannel<E>)
           -> :wat::std::telemetry::Service::ReqTx<E>)
          (:wat::core::first p))))
     ((req-rxs :Vec<wat::std::telemetry::Service::ReqRx<E>>)
      (:wat::core::map pairs
        (:wat::core::lambda
          ((p :wat::std::telemetry::Service::ReqChannel<E>)
           -> :wat::std::telemetry::Service::ReqRx<E>)
          (:wat::core::second p))))
     ((pool :wat::std::telemetry::Service::ReqTxPool<E>)
      (:wat::kernel::HandlePool::new
        "wat::std::telemetry::Sqlite" req-txs))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::telemetry::Sqlite/run
        path req-rxs cadence
        schema-install dispatcher stats-translator)))
    (:wat::core::tuple pool driver)))


;; ─── Auto-spawn (arc 085) ───────────────────────────────────────
;;
;; Companion to Sqlite/spawn that derives schemas + INSERTs + the
;; per-entry binder from the consumer's enum decl. The consumer
;; passes the enum NAME as a keyword value — substrate looks up
;; the EnumDef through `sym.types` (capability-carrier added by
;; arc 085), walks variants, builds:
;;
;;   - one CREATE TABLE per Tagged variant (variant name PascalCase
;;     → table name snake_case; field name kebab → column snake;
;;     field type → SQLite affinity)
;;   - one cached INSERT per variant
;;   - the runtime binder that maps Value::Enum.fields to a Param vec
;;
;; The wat layer is composition over the explicit Sqlite/spawn:
;; build closures that delegate to the three Rust shims registered
;; in src/auto.rs. The closures cross thread boundaries cleanly —
;; their captures are the enum-name keyword (Send-safe String).
;;
;; Slice 1 ships with null-metrics-cadence only — auto-spawn does
;; not emit substrate self-heartbeat rows. Consumers wanting
;; heartbeat use the explicit Sqlite/spawn.

;; Empty stats-translator. Type-checks under `:Vec<E>` even though
;; the body returns an explicit empty vec — substrate's null cadence
;; never invokes this fn, so the constructed value's E is irrelevant
;; at runtime.
(:wat::core::define
  (:wat::std::telemetry::Sqlite::auto-empty-translator<E>
    (_stats :wat::std::telemetry::Service::Stats)
    -> :Vec<E>)
  (:wat::core::vec :E))


(:wat::core::define
  (:wat::std::telemetry::Sqlite/auto-spawn<E,G>
    (enum-name :wat::core::keyword)
    (path :String)
    (count :i64)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    -> :wat::std::telemetry::Service::Spawn<E>)
  (:wat::core::let*
    (((_prep :()) (:rust::sqlite::auto-prep enum-name)))
    (:wat::std::telemetry::Sqlite/spawn
      path count cadence
      (:wat::core::lambda ((db :wat::sqlite::Db) -> :())
        (:rust::sqlite::auto-install-schemas db enum-name))
      (:wat::core::lambda ((db :wat::sqlite::Db) (entry :E) -> :())
        (:rust::sqlite::auto-dispatch db enum-name entry))
      :wat::std::telemetry::Sqlite::auto-empty-translator)))
