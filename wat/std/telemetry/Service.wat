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
;; advanced gate. On fire: build Vec<E> via translator, hand it to
;; the per-batch dispatcher (arc 089 slice 3), reset stats. On
;; no-fire: stats unchanged; cadence advanced. The translated
;; vector goes through the SAME dispatcher that handles client
;; batches — sinks see one cohesive batch shape regardless of
;; whether it originated from clients or self-heartbeat.

(:wat::core::define
  (:wat::std::telemetry::Service/tick-window<E,G>
    (stats :wat::std::telemetry::Service::Stats)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(Vec<E>)->())
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
         ((_dispatch :()) (dispatcher entries)))
        (:wat::core::tuple
          (:wat::std::telemetry::Service::Stats/zero) cadence'))
      (:wat::core::tuple stats cadence'))))


;; ─── Driver loop (arc 089 slice 2 — drain all clients) ──────────
;;
;; Mirrors the archive's pattern at
;; `archived/pre-wat-native/src/programs/stdlib/database.rs:127-211`.
;; Per-iteration order:
;;   1. select; blocks until ANY rx has data
;;   2. on :None — remove the disconnected rx, recurse
;;   3. on Some(first-req) — drain every OTHER rx via try-recv
;;      (each rx is bounded(1), so at most one queued; the same
;;      idx the select already consumed is empty until the producer
;;      sends again, and they can't until we ack)
;;   4. dispatch each entry through the per-entry dispatcher
;;      (slice-3 will change this to per-batch)
;;   5. ack EVERY contributing client — release their batch-log
;;      calls (preserves the archive's "in-memory TCP" discipline:
;;      producer's batch-log unblocks only after the work is done)
;;   6. update Stats with combined batch size + tick window
;;   7. recurse with (stats', cadence')

;; Pending — accumulator threaded through drain-rest. The first
;; tuple slot collects entries from all draining clients; the
;; second collects their ack-tx handles. After dispatch, every
;; ack-tx is released.
(:wat::core::typealias :wat::std::telemetry::Service::Pending<E>
  :(Vec<E>,Vec<wat::std::telemetry::Service::AckTx>))


;; Merge one Request into the Pending accumulator. Extends entries
;; with req.entries; appends req.ack-tx to the ack-txs list.
(:wat::core::define
  (:wat::std::telemetry::Service/extend<E>
    (acc :wat::std::telemetry::Service::Pending<E>)
    (req :wat::std::telemetry::Service::Request<E>)
    -> :wat::std::telemetry::Service::Pending<E>)
  (:wat::core::let*
    (((entries :Vec<E>) (:wat::core::first acc))
     ((acks :Vec<wat::std::telemetry::Service::AckTx>) (:wat::core::second acc))
     ((req-entries :Vec<E>) (:wat::core::first req))
     ((req-ack :wat::std::telemetry::Service::AckTx) (:wat::core::second req))
     ((entries' :Vec<E>) (:wat::core::concat entries req-entries))
     ((acks' :Vec<wat::std::telemetry::Service::AckTx>)
      (:wat::core::concat acks
        (:wat::core::vec :wat::std::telemetry::Service::AckTx req-ack))))
    (:wat::core::tuple entries' acks')))


;; If pair.idx == first-idx, skip — select already consumed that rx
;; and we already extended with first-req. Otherwise try-recv
;; (non-blocking); on Some, extend acc; on None, leave acc alone.
(:wat::core::define
  (:wat::std::telemetry::Service/maybe-merge<E>
    (acc :wat::std::telemetry::Service::Pending<E>)
    (first-idx :i64)
    (pair :(wat::std::telemetry::Service::ReqRx<E>,i64))
    -> :wat::std::telemetry::Service::Pending<E>)
  (:wat::core::let*
    (((rx :wat::std::telemetry::Service::ReqRx<E>) (:wat::core::first pair))
     ((idx :i64) (:wat::core::second pair)))
    (:wat::core::if (:wat::core::= idx first-idx)
      -> :wat::std::telemetry::Service::Pending<E>
      acc
      (:wat::core::match (:wat::kernel::try-recv rx)
        -> :wat::std::telemetry::Service::Pending<E>
        ((Some req) (:wat::std::telemetry::Service/extend acc req))
        (:None acc)))))


;; Drain rest — try-recv each rx (other than first-idx). Returns
;; the Pending accumulator with all in-flight batches merged in.
(:wat::core::define
  (:wat::std::telemetry::Service/drain-rest<E>
    (rxs :Vec<wat::std::telemetry::Service::ReqRx<E>>)
    (first-idx :i64)
    (init :wat::std::telemetry::Service::Pending<E>)
    -> :wat::std::telemetry::Service::Pending<E>)
  (:wat::core::let*
    (((indices :Vec<i64>)
      (:wat::core::range 0 (:wat::core::length rxs)))
     ((pairs :Vec<(wat::std::telemetry::Service::ReqRx<E>,i64)>)
      (:wat::std::list::zip rxs indices)))
    (:wat::core::foldl pairs init
      (:wat::core::lambda
        ((acc :wat::std::telemetry::Service::Pending<E>)
         (pair :(wat::std::telemetry::Service::ReqRx<E>,i64))
         -> :wat::std::telemetry::Service::Pending<E>)
        (:wat::std::telemetry::Service/maybe-merge acc first-idx pair)))))


;; Send () on every contributing client's ack-tx. Per-call swallow
;; on disconnect (caller may have dropped while we were dispatching).
(:wat::core::define
  (:wat::std::telemetry::Service/ack-all
    (ack-txs :Vec<wat::std::telemetry::Service::AckTx>)
    -> :())
  (:wat::core::foldl ack-txs ()
    (:wat::core::lambda
      ((_acc :()) (tx :wat::std::telemetry::Service::AckTx) -> :())
      (:wat::core::match (:wat::kernel::send tx ()) -> :()
        ((Some _) ())
        (:None ())))))


;; Update Stats with the combined batch's contribution. Lifted out
;; of the loop body to keep the outer let* scannable.
(:wat::core::define
  (:wat::std::telemetry::Service/bump-stats
    (stats :wat::std::telemetry::Service::Stats)
    (batch-size :i64)
    -> :wat::std::telemetry::Service::Stats)
  (:wat::core::let*
    (((max-prev :i64)
      (:wat::std::telemetry::Service::Stats/max-batch-size stats))
     ((max' :i64)
      (:wat::core::if (:wat::core::> batch-size max-prev) -> :i64
        batch-size
        max-prev)))
    (:wat::std::telemetry::Service::Stats/new
      (:wat::core::+ (:wat::std::telemetry::Service::Stats/batches stats) 1)
      (:wat::core::+ (:wat::std::telemetry::Service::Stats/entries stats) batch-size)
      max')))


;; One drain-and-dispatch cycle. Caller passes the rx-idx and the
;; first Request select returned; we drain the rest, dispatch,
;; ack everyone, tick the cadence, recurse into Service/loop.
(:wat::core::define
  (:wat::std::telemetry::Service/loop-step<E,G>
    (rxs :Vec<wat::std::telemetry::Service::ReqRx<E>>)
    (first-idx :i64)
    (first-req :wat::std::telemetry::Service::Request<E>)
    (stats :wat::std::telemetry::Service::Stats)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(Vec<E>)->())
    (stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<E>)
    -> :())
  (:wat::core::let*
    (((seed :wat::std::telemetry::Service::Pending<E>)
      (:wat::std::telemetry::Service/extend
        (:wat::core::tuple
          (:wat::core::vec :E)
          (:wat::core::vec :wat::std::telemetry::Service::AckTx))
        first-req))
     ((pending :wat::std::telemetry::Service::Pending<E>)
      (:wat::std::telemetry::Service/drain-rest rxs first-idx seed))
     ((entries :Vec<E>) (:wat::core::first pending))
     ((ack-txs :Vec<wat::std::telemetry::Service::AckTx>)
      (:wat::core::second pending))
     ((_apply :()) (dispatcher entries))
     ((_ack :()) (:wat::std::telemetry::Service/ack-all ack-txs))
     ((batch-size :i64) (:wat::core::length entries))
     ((stats' :wat::std::telemetry::Service::Stats)
      (:wat::std::telemetry::Service/bump-stats stats batch-size))
     ((step :wat::std::telemetry::Service::Step<G>)
      (:wat::std::telemetry::Service/tick-window
        stats' cadence dispatcher stats-translator))
     ((stats'' :wat::std::telemetry::Service::Stats) (:wat::core::first step))
     ((cadence' :wat::std::telemetry::Service::MetricsCadence<G>)
      (:wat::core::second step)))
    (:wat::std::telemetry::Service/loop
      rxs stats'' cadence' dispatcher stats-translator)))


(:wat::core::define
  (:wat::std::telemetry::Service/loop<E,G>
    (rxs :Vec<wat::std::telemetry::Service::ReqRx<E>>)
    (stats :wat::std::telemetry::Service::Stats)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(Vec<E>)->())
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
        ((Some first-req)
          (:wat::std::telemetry::Service/loop-step
            rxs idx first-req stats cadence dispatcher stats-translator))
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
    (dispatcher :fn(Vec<E>)->())
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
    (dispatcher :fn(Vec<E>)->())
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
