;; :wat::std::telemetry::Service<E,G> — generic queue-fronted
;; destination service for structured records.
;;
;; Arc 080. The substrate's contribution to the data-not-text
;; observability rebuild. Lifted-and-generalized from the lab's
;; :trading::rundb::Service (and its arc-080-prep retrofit that
;; threaded Stats + MetricsCadence). Generic over:
;;
;;   E — the consumer's entry type. Substrate ships ZERO entry
;;       variants per arc 080's discipline ("the LogEntry must be
;;       user defined"). Each consumer defines its own entry enum
;;       (Trader: PaperResolved + Metric; MTG: future variants).
;;   G — the cadence gate type. Same as arc 078's MetricsCadence<G>
;;       contract; users pick `()`/i64/Instant/etc. by domain.
;;
;; The Service shell is pure-wat composition. No Rust shim. No
;; sqlite-specificity. The DISPATCHER is the consumer's closure that
;; knows where each entry lives (sqlite write, console line, file
;; append, …). The STATS-TRANSLATOR is the consumer's closure that
;; converts the Service's own counters to entries of E so the
;; self-heartbeat lands through the same dispatcher path.
;;
;; Lifecycle (mirrors the canonical Step-3 lockstep + arc 078 contract):
;;   1. Caller `(Service/spawn count dispatcher translator cadence)`
;;      → `(HandlePool<ReqTx<E>>, ProgramHandle<()>)`.
;;   2. Driver loop opens nothing — substrate has no resources to
;;      manage; the dispatcher closes over whatever the consumer
;;      supplies (db handle, console-tx, etc).
;;   3. Caller pops handles, distributes, finishes the pool.
;;   4. Each client `(Service/batch-log req-tx ack-tx ack-rx entries)`
;;      sends + acks per arc 029's Q10 ("confirmed batch + ack").
;;   5. Driver dispatches each entry through the closure; acks; updates
;;      Stats; ticks the cadence; on fire, builds Vec<E> via
;;      translator and dispatches each through the SAME closure.
;;   6. Clients drop their handles. Driver loop converges, exits,
;;      `(join driver)` confirms clean exit.

;; ─── Self-heartbeat contract — Stats + MetricsCadence ────────────
;;
;; Same shape arc 078 codified for HologramCacheService. Three
;; counters: batches received, total entries committed, and the
;; high-water-mark batch size (a useful gauge that doesn't fit
;; cleanly in a counter alone).

(:wat::core::struct :wat::std::telemetry::Service::Stats
  (batches :i64)
  (entries :i64)
  (max-batch-size :i64))

(:wat::core::struct :wat::std::telemetry::Service::MetricsCadence<G>
  (gate :G)
  (tick :fn(G,wat::std::telemetry::Service::Stats)->(G,bool)))

;; null-metrics-cadence — fresh `MetricsCadence<()>` whose tick
;; never fires. The opt-out for self-heartbeat.
(:wat::core::define
  (:wat::std::telemetry::Service/null-metrics-cadence
    -> :wat::std::telemetry::Service::MetricsCadence<()>)
  (:wat::std::telemetry::Service::MetricsCadence/new
    ()
    (:wat::core::lambda
      ((gate :()) (_stats :wat::std::telemetry::Service::Stats) -> :((),bool))
      (:wat::core::tuple gate false))))

;; Fresh zero-counters Stats. Used at startup and after each
;; gate-fire (window-rolling reset).
(:wat::core::define
  (:wat::std::telemetry::Service::Stats/zero
    -> :wat::std::telemetry::Service::Stats)
  (:wat::std::telemetry::Service::Stats/new 0 0 0))


;; ─── Protocol typealiases ────────────────────────────────────────

(:wat::core::typealias :wat::std::telemetry::Service::AckTx
  :rust::crossbeam_channel::Sender<()>)
(:wat::core::typealias :wat::std::telemetry::Service::AckRx
  :rust::crossbeam_channel::Receiver<()>)
(:wat::core::typealias :wat::std::telemetry::Service::AckChannel
  :(wat::std::telemetry::Service::AckTx,wat::std::telemetry::Service::AckRx))

;; A Request is a batch of entries + the client's ack channel.
(:wat::core::typealias :wat::std::telemetry::Service::Request<E>
  :(Vec<E>,wat::std::telemetry::Service::AckTx))

(:wat::core::typealias :wat::std::telemetry::Service::ReqTx<E>
  :rust::crossbeam_channel::Sender<wat::std::telemetry::Service::Request<E>>)
(:wat::core::typealias :wat::std::telemetry::Service::ReqRx<E>
  :rust::crossbeam_channel::Receiver<wat::std::telemetry::Service::Request<E>>)

(:wat::core::typealias :wat::std::telemetry::Service::ReqChannel<E>
  :(wat::std::telemetry::Service::ReqTx<E>,wat::std::telemetry::Service::ReqRx<E>))

(:wat::core::typealias :wat::std::telemetry::Service::ReqTxPool<E>
  :wat::kernel::HandlePool<wat::std::telemetry::Service::ReqTx<E>>)

(:wat::core::typealias :wat::std::telemetry::Service::Spawn<E>
  :(wat::std::telemetry::Service::ReqTxPool<E>,wat::kernel::ProgramHandle<()>))

;; One loop-iteration's outputs (Step alias keeps the loop signature
;; flat per the arc 077 type-alias-density rule).
(:wat::core::typealias :wat::std::telemetry::Service::Step<G>
  :(wat::std::telemetry::Service::Stats,wat::std::telemetry::Service::MetricsCadence<G>))


;; ─── Tick the heartbeat window ───────────────────────────────────

;; Always: tick the cadence; rebuild the cadence struct with the
;; advanced gate. On fire: build Vec<E> via translator, dispatch
;; each entry through the SAME closure that handles client batches,
;; reset stats. On no-fire: stats unchanged; cadence advanced.

(:wat::core::define
  (:wat::std::telemetry::Service/tick-window<E,G>
    (stats :wat::std::telemetry::Service::Stats)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(E)->())
    (stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<E>)
    -> :wat::std::telemetry::Service::Step<G>)
  (:wat::core::let*
    (((gate :G)
      (:wat::std::telemetry::Service::MetricsCadence/gate cadence))
     ((tick-fn :fn(G,wat::std::telemetry::Service::Stats)->(G,bool))
      (:wat::std::telemetry::Service::MetricsCadence/tick cadence))
     ((tick :(G,bool)) (tick-fn gate stats))
     ((gate' :G) (:wat::core::first tick))
     ((fired :bool) (:wat::core::second tick))
     ((cadence' :wat::std::telemetry::Service::MetricsCadence<G>)
      (:wat::std::telemetry::Service::MetricsCadence/new gate' tick-fn)))
    (:wat::core::if fired
      -> :wat::std::telemetry::Service::Step<G>
      (:wat::core::let*
        (((entries :Vec<E>) (stats-translator stats))
         ((_dispatch :())
          (:wat::core::foldl entries ()
            (:wat::core::lambda
              ((acc :()) (e :E) -> :())
              (dispatcher e)))))
        (:wat::core::tuple
          (:wat::std::telemetry::Service::Stats/zero) cadence'))
      (:wat::core::tuple stats cadence'))))


;; ─── Driver loop ─────────────────────────────────────────────────
;;
;; Per-iteration order (per arc 029 Q10 batch + ack discipline plus
;; arc 078 cadence threading):
;;   1. select; on Some(req): foldl-dispatch the batch's entries
;;   2. ack the client (release their batch-log call ASAP)
;;   3. update Stats with this batch's contribution
;;   4. tick-window — advance cadence; on fire, emit self-telemetry
;;      via translator + dispatcher, reset stats
;;   5. recurse with (stats', cadence')
;;
;; On :None: prune the disconnected receiver, recurse with stats +
;; cadence unchanged. Loop exits when rxs is empty.

(:wat::core::define
  (:wat::std::telemetry::Service/loop<E,G>
    (rxs :Vec<wat::std::telemetry::Service::ReqRx<E>>)
    (stats :wat::std::telemetry::Service::Stats)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(E)->())
    (stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<E>)
    -> :())
  (:wat::core::if (:wat::core::empty? rxs) -> :()
    ()
    (:wat::core::let*
      (((chosen :(i64,Option<wat::std::telemetry::Service::Request<E>>))
        (:wat::kernel::select rxs))
       ((idx :i64) (:wat::core::first chosen))
       ((maybe :Option<wat::std::telemetry::Service::Request<E>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :()
        ((Some req)
          (:wat::core::let*
            (((entries :Vec<E>) (:wat::core::first req))
             ((ack-tx :wat::std::telemetry::Service::AckTx)
              (:wat::core::second req))
             ;; Apply each entry. Caller's dispatcher closes over
             ;; whatever destination state it needs.
             ((_apply :())
              (:wat::core::foldl entries ()
                (:wat::core::lambda ((acc :()) (e :E) -> :())
                  (dispatcher e))))
             ;; Ack first — release client's batch-log call before
             ;; running heartbeat tick.
             ((_ack :Option<()>) (:wat::kernel::send ack-tx ()))
             ;; Update Stats with this batch's contribution.
             ((batch-size :i64) (:wat::core::length entries))
             ((stats' :wat::std::telemetry::Service::Stats)
              (:wat::std::telemetry::Service::Stats/new
                (:wat::core::+
                  (:wat::std::telemetry::Service::Stats/batches stats) 1)
                (:wat::core::+
                  (:wat::std::telemetry::Service::Stats/entries stats) batch-size)
                (:wat::core::if
                  (:wat::core::> batch-size
                    (:wat::std::telemetry::Service::Stats/max-batch-size stats))
                  -> :i64
                  batch-size
                  (:wat::std::telemetry::Service::Stats/max-batch-size stats))))
             ;; Tick window — advance cadence; fire emits self-rows.
             ((step :wat::std::telemetry::Service::Step<G>)
              (:wat::std::telemetry::Service/tick-window
                stats' cadence dispatcher stats-translator))
             ((stats'' :wat::std::telemetry::Service::Stats)
              (:wat::core::first step))
             ((cadence' :wat::std::telemetry::Service::MetricsCadence<G>)
              (:wat::core::second step)))
            (:wat::std::telemetry::Service/loop
              rxs stats'' cadence' dispatcher stats-translator)))
        (:None
          (:wat::std::telemetry::Service/loop
            (:wat::std::list::remove-at rxs idx)
            stats cadence dispatcher stats-translator))))))


;; ─── Client helper — single primitive, batch + ack ───────────────
;;
;; Sends the batch + ack-tx on req-tx, blocks on ack-rx until the
;; driver signals commit. Single-entry callers wrap in a one-element
;; vec — same convention as arc 029's rundb-Service/batch-log.

(:wat::core::define
  (:wat::std::telemetry::Service/batch-log<E>
    (req-tx :wat::std::telemetry::Service::ReqTx<E>)
    (ack-tx :wat::std::telemetry::Service::AckTx)
    (ack-rx :wat::std::telemetry::Service::AckRx)
    (entries :Vec<E>)
    -> :())
  (:wat::core::let*
    (((req :wat::std::telemetry::Service::Request<E>)
      (:wat::core::tuple entries ack-tx))
     ((_send :Option<()>) (:wat::kernel::send req-tx req))
     ((_recv :Option<()>) (:wat::kernel::recv ack-rx)))
    ()))


;; ─── Worker entry — initial Stats + enter loop ──────────────────

(:wat::core::define
  (:wat::std::telemetry::Service/run<E,G>
    (rxs :Vec<wat::std::telemetry::Service::ReqRx<E>>)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(E)->())
    (stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<E>)
    -> :())
  (:wat::std::telemetry::Service/loop
    rxs
    (:wat::std::telemetry::Service::Stats/zero)
    cadence dispatcher stats-translator))


;; ─── Setup — spawn driver, return (HandlePool, driver) ───────────

(:wat::core::define
  (:wat::std::telemetry::Service/spawn<E,G>
    (count :i64)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(E)->())
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
      (:wat::kernel::HandlePool::new "telemetry::Service" req-txs))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::telemetry::Service/run
        req-rxs cadence dispatcher stats-translator)))
    (:wat::core::tuple pool driver)))
