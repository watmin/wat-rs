;; :wat::telemetry::Service<E,G> — generic queue-fronted
;; destination service for structured records.
;;
;; Arc 080 (initial design) + arc 089 slice 2-3 (drain-all +
;; per-batch dispatch) + arc 095 (paired channels, this protocol).
;;
;; Generic over:
;;
;;   E — the consumer's entry type. Substrate ships ZERO entry
;;       variants per arc 080's discipline ("the LogEntry must be
;;       user defined").
;;   G — the cadence gate type. Same as arc 078's MetricsCadence<G>
;;       contract; users pick `()`/i64/Instant/etc. by domain.
;;
;; Channel topology (arc 095): each client gets a Handle pair —
;; `(ReqTx<E>, AckRx)`. The server holds the matched halves —
;; `wat::core::Vector<DriverPair<E>>` where each `DriverPair = (ReqRx<E>, AckTx)`
;; paired by index. Client uses two opposite ends (write req, read
;; ack); server uses two opposite ends (read req, write ack). The
;; embedded-ack-tx-in-request pattern from before retired — the
;; user flagged it as "extremely messy" mid-arc-091 and arc 095
;; closed it.
;;
;; Lifecycle:
;;   1. Caller `(Service/spawn count dispatcher translator cadence)`
;;      → `(HandlePool<E>, ProgramHandle<()>)`.
;;   2. Driver loop opens nothing — substrate has no resources to
;;      manage; the dispatcher closes over whatever the consumer
;;      supplies (db handle, console-handle, etc).
;;   3. Caller pops Handles, distributes, finishes the pool.
;;   4. Each client `(Service/batch-log req-tx ack-rx entries)`
;;      sends + acks per arc 029's Q10 ("confirmed batch + ack").
;;      Two channel ends. No reply-tx-in-request.
;;   5. Driver `select`s over rx side of pairs; on fire, pulls the
;;      matching ack-tx by index, dispatches, acks back through
;;      that ack-tx.
;;   6. Clients drop their Handles. Driver loop converges, exits,
;;      `(join driver)` confirms clean exit.

;; ─── Self-heartbeat contract — Stats + MetricsCadence ────────────

(:wat::core::struct :wat::telemetry::Service::Stats
  (batches :wat::core::i64)
  (entries :wat::core::i64)
  (max-batch-size :wat::core::i64))

(:wat::core::struct :wat::telemetry::Service::MetricsCadence<G>
  (gate :G)
  (tick :fn(G,wat::telemetry::Service::Stats)->(G,wat::core::bool)))

(:wat::core::define
  (:wat::telemetry::Service/null-metrics-cadence
    -> :wat::telemetry::Service::MetricsCadence<wat::core::unit>)
  (:wat::telemetry::Service::MetricsCadence/new
    ()
    (:wat::core::lambda
      ((gate :wat::core::unit) (_stats :wat::telemetry::Service::Stats) -> :(wat::core::unit,wat::core::bool))
      (:wat::core::Tuple gate false))))

(:wat::core::define
  (:wat::telemetry::Service::Stats/zero
    -> :wat::telemetry::Service::Stats)
  (:wat::telemetry::Service::Stats/new 0 0 0))


;; ─── Protocol typealiases (arc 095) ──────────────────────────────

;; Ack channel — unit signal. Same shape both sides; the (tx, rx)
;; pair is split between server and client, NOT bundled on either.
(:wat::core::typealias :wat::telemetry::Service::AckTx
  :wat::kernel::QueueSender<wat::core::unit>)
(:wat::core::typealias :wat::telemetry::Service::AckRx
  :wat::kernel::QueueReceiver<wat::core::unit>)
(:wat::core::typealias :wat::telemetry::Service::AckChannel
  :(wat::telemetry::Service::AckTx,wat::telemetry::Service::AckRx))

;; Request — just the batch of entries. The client's reply address
;; is no longer in the wire payload (retired arc 095); the server
;; holds the matching ack-tx in its paired DriverPair vector.
(:wat::core::typealias :wat::telemetry::Service::Request<E>
  :wat::core::Vector<E>)

(:wat::core::typealias :wat::telemetry::Service::ReqTx<E>
  :wat::kernel::QueueSender<wat::telemetry::Service::Request<E>>)
(:wat::core::typealias :wat::telemetry::Service::ReqRx<E>
  :wat::kernel::QueueReceiver<wat::telemetry::Service::Request<E>>)

(:wat::core::typealias :wat::telemetry::Service::ReqChannel<E>
  :(wat::telemetry::Service::ReqTx<E>,wat::telemetry::Service::ReqRx<E>))

;; A complete client/server connection — one ReqChannel and one
;; AckChannel that the spawn step distributes between Handle (client
;; side) and DriverPair (server side). Aliased so spawn's zip-and-map
;; doesn't smear the verbose tuple form across every lambda body.
(:wat::core::typealias :wat::telemetry::Service::Connection<E>
  :(wat::telemetry::Service::ReqChannel<E>,wat::telemetry::Service::AckChannel))

;; Client-side Handle — what the consumer pops from the pool.
;; Two opposite ends: req-tx to write, ack-rx to read.
(:wat::core::typealias :wat::telemetry::Service::Handle<E>
  :(wat::telemetry::Service::ReqTx<E>,wat::telemetry::Service::AckRx))

;; Server-side pair — what the worker holds in parallel by index.
;; Two opposite ends: req-rx to read, ack-tx to write.
(:wat::core::typealias :wat::telemetry::Service::DriverPair<E>
  :(wat::telemetry::Service::ReqRx<E>,wat::telemetry::Service::AckTx))

;; A DriverPair tagged with its index in the server's pairs vector.
;; Used by drain-rest's foldl to skip the rx select already
;; consumed (first-idx) and look up the matching ack-tx by position.
(:wat::core::typealias :wat::telemetry::Service::IndexedDriverPair<E>
  :(wat::telemetry::Service::DriverPair<E>,wat::core::i64))

(:wat::core::typealias :wat::telemetry::Service::HandlePool<E>
  :wat::kernel::HandlePool<wat::telemetry::Service::Handle<E>>)

(:wat::core::typealias :wat::telemetry::Service::Spawn<E>
  :(wat::telemetry::Service::HandlePool<E>,wat::kernel::Thread<wat::core::unit,wat::core::unit>))

(:wat::core::typealias :wat::telemetry::Service::Step<G>
  :(wat::telemetry::Service::Stats,wat::telemetry::Service::MetricsCadence<G>))


;; ─── Tick the heartbeat window ───────────────────────────────────

(:wat::core::define
  (:wat::telemetry::Service/tick-window<E,G>
    (stats :wat::telemetry::Service::Stats)
    (cadence :wat::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Service::Stats)->wat::core::Vector<E>)
    -> :wat::telemetry::Service::Step<G>)
  (:wat::core::let*
    (((gate :G)
      (:wat::telemetry::Service::MetricsCadence/gate cadence))
     ((tick-fn :fn(G,wat::telemetry::Service::Stats)->(G,wat::core::bool))
      (:wat::telemetry::Service::MetricsCadence/tick cadence))
     ((tick :(G,wat::core::bool)) (tick-fn gate stats))
     ((gate' :G) (:wat::core::first tick))
     ((fired :wat::core::bool) (:wat::core::second tick))
     ((cadence' :wat::telemetry::Service::MetricsCadence<G>)
      (:wat::telemetry::Service::MetricsCadence/new gate' tick-fn)))
    (:wat::core::if fired
      -> :wat::telemetry::Service::Step<G>
      (:wat::core::let*
        (((entries :wat::core::Vector<E>) (stats-translator stats))
         ((_dispatch :wat::core::unit) (dispatcher entries)))
        (:wat::core::Tuple
          (:wat::telemetry::Service::Stats/zero) cadence'))
      (:wat::core::Tuple stats cadence'))))


;; ─── Driver loop (arc 089 drain-all + arc 095 paired channels) ──
;;
;; Per-iteration order:
;;   1. Extract rxs from pairs (select needs the wat::core::Vector<Receiver>)
;;   2. select; blocks until ANY rx has data
;;   3. on :None — remove pairs[idx] (drops both ReqRx and AckTx of
;;      the disconnected client), recurse
;;   4. on Some(first-entries) — drain every OTHER rx via try-recv;
;;      on each hit, accumulate entries + the matching ack-tx from
;;      pairs[j].second
;;   5. dispatch via the per-batch dispatcher
;;   6. ack-all — release every contributing client's batch-log
;;      (preserves the "in-memory TCP" discipline)
;;   7. update Stats with combined batch size + tick window
;;   8. recurse with (stats', cadence')

;; Pending — accumulator threaded through drain-rest. (entries,
;; ack-txs). The ack-txs come from the server's paired vector
;; lookup, not from any request payload.
(:wat::core::typealias :wat::telemetry::Service::Pending<E>
  :(wat::core::Vector<E>,wat::core::Vector<wat::telemetry::Service::AckTx>))


;; Add one client's contribution into the Pending accumulator.
;; Entries come from the Request payload; ack-tx comes from the
;; server's paired DriverPair.
(:wat::core::define
  (:wat::telemetry::Service/extend<E>
    (acc :wat::telemetry::Service::Pending<E>)
    (req-entries :wat::core::Vector<E>)
    (ack :wat::telemetry::Service::AckTx)
    -> :wat::telemetry::Service::Pending<E>)
  (:wat::core::let*
    (((entries :wat::core::Vector<E>) (:wat::core::first acc))
     ((acks :wat::core::Vector<wat::telemetry::Service::AckTx>) (:wat::core::second acc))
     ((entries' :wat::core::Vector<E>) (:wat::core::concat entries req-entries))
     ((acks' :wat::core::Vector<wat::telemetry::Service::AckTx>)
      (:wat::core::concat acks
        (:wat::core::Vector :wat::telemetry::Service::AckTx ack))))
    (:wat::core::Tuple entries' acks')))


;; Merge one indexed pair into the accumulator. On the first-idx
;; pair, attach `first-entries` (already drained by select) +
;; pair.ack. On every other pair, try-recv pair.rx; on a hit,
;; attach entries + pair.ack. The single foldl over ALL pairs
;; eliminates the prior split between "first" and "rest" and the
;; need for an out-of-band lookup.
(:wat::core::define
  (:wat::telemetry::Service/maybe-merge<E>
    (acc :wat::telemetry::Service::Pending<E>)
    (first-idx :wat::core::i64)
    (first-entries :wat::core::Vector<E>)
    (indexed :wat::telemetry::Service::IndexedDriverPair<E>)
    -> :wat::telemetry::Service::Pending<E>)
  (:wat::core::let*
    (((pair :wat::telemetry::Service::DriverPair<E>) (:wat::core::first indexed))
     ((idx :wat::core::i64) (:wat::core::second indexed))
     ((rx :wat::telemetry::Service::ReqRx<E>) (:wat::core::first pair))
     ((ack :wat::telemetry::Service::AckTx) (:wat::core::second pair)))
    (:wat::core::if (:wat::core::= idx first-idx)
      -> :wat::telemetry::Service::Pending<E>
      (:wat::telemetry::Service/extend acc first-entries ack)
      (:wat::core::match (:wat::kernel::try-recv rx)
        -> :wat::telemetry::Service::Pending<E>
        ((Ok (Some req-entries))
          (:wat::telemetry::Service/extend acc req-entries ack))
        ((Ok :None) acc)
        ((Err _died) acc)))))


;; Drain — single foldl over all pairs. The first-idx pair gets
;; first-entries from select; every other pair tries try-recv.
(:wat::core::define
  (:wat::telemetry::Service/drain-pairs<E>
    (pairs :wat::core::Vector<wat::telemetry::Service::DriverPair<E>>)
    (first-idx :wat::core::i64)
    (first-entries :wat::core::Vector<E>)
    (init :wat::telemetry::Service::Pending<E>)
    -> :wat::telemetry::Service::Pending<E>)
  (:wat::core::let*
    (((indices :wat::core::Vector<wat::core::i64>)
      (:wat::core::range 0 (:wat::core::length pairs)))
     ((indexed :wat::core::Vector<wat::telemetry::Service::IndexedDriverPair<E>>)
      (:wat::std::list::zip pairs indices)))
    (:wat::core::foldl indexed init
      (:wat::core::lambda
        ((acc :wat::telemetry::Service::Pending<E>)
         (pair :wat::telemetry::Service::IndexedDriverPair<E>)
         -> :wat::telemetry::Service::Pending<E>)
        (:wat::telemetry::Service/maybe-merge acc first-idx first-entries pair)))))


;; Send () on every contributing client's ack-tx.
(:wat::core::define
  (:wat::telemetry::Service/ack-all
    (ack-txs :wat::core::Vector<wat::telemetry::Service::AckTx>)
    -> :wat::core::unit)
  (:wat::core::foldl ack-txs ()
    (:wat::core::lambda
      ((_acc :wat::core::unit) (tx :wat::telemetry::Service::AckTx) -> :wat::core::unit)
      (:wat::core::match (:wat::kernel::send tx ()) -> :wat::core::unit
        ((Ok _) ())
        ((Err _) ())))))


(:wat::core::define
  (:wat::telemetry::Service/bump-stats
    (stats :wat::telemetry::Service::Stats)
    (batch-size :wat::core::i64)
    -> :wat::telemetry::Service::Stats)
  (:wat::core::let*
    (((max-prev :wat::core::i64)
      (:wat::telemetry::Service::Stats/max-batch-size stats))
     ((max' :wat::core::i64)
      (:wat::core::if (:wat::core::> batch-size max-prev) -> :wat::core::i64
        batch-size
        max-prev)))
    (:wat::telemetry::Service::Stats/new
      (:wat::core::+ (:wat::telemetry::Service::Stats/batches stats) 1)
      (:wat::core::+ (:wat::telemetry::Service::Stats/entries stats) batch-size)
      max')))


;; Extract the wat::core::Vector<ReqRx> half of pairs for the kernel select.
(:wat::core::define
  (:wat::telemetry::Service/pair-rxs<E>
    (pairs :wat::core::Vector<wat::telemetry::Service::DriverPair<E>>)
    -> :wat::core::Vector<wat::telemetry::Service::ReqRx<E>>)
  (:wat::core::map pairs
    (:wat::core::lambda
      ((p :wat::telemetry::Service::DriverPair<E>)
       -> :wat::telemetry::Service::ReqRx<E>)
      (:wat::core::first p))))


;; One drain-and-dispatch cycle. drain-pairs handles BOTH first-idx
;; (which gets first-entries from select) and the rest (which try-recv).
;; No separate first-pair lookup needed.
(:wat::core::define
  (:wat::telemetry::Service/loop-step<E,G>
    (pairs :wat::core::Vector<wat::telemetry::Service::DriverPair<E>>)
    (first-idx :wat::core::i64)
    (first-entries :wat::core::Vector<E>)
    (stats :wat::telemetry::Service::Stats)
    (cadence :wat::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Service::Stats)->wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((init :wat::telemetry::Service::Pending<E>)
      (:wat::core::Tuple
        (:wat::core::Vector :E)
        (:wat::core::Vector :wat::telemetry::Service::AckTx)))
     ((pending :wat::telemetry::Service::Pending<E>)
      (:wat::telemetry::Service/drain-pairs pairs first-idx first-entries init))
     ((entries :wat::core::Vector<E>) (:wat::core::first pending))
     ((ack-txs :wat::core::Vector<wat::telemetry::Service::AckTx>)
      (:wat::core::second pending))
     ((_apply :wat::core::unit) (dispatcher entries))
     ((_ack :wat::core::unit) (:wat::telemetry::Service/ack-all ack-txs))
     ((batch-size :wat::core::i64) (:wat::core::length entries))
     ((stats' :wat::telemetry::Service::Stats)
      (:wat::telemetry::Service/bump-stats stats batch-size))
     ((step :wat::telemetry::Service::Step<G>)
      (:wat::telemetry::Service/tick-window
        stats' cadence dispatcher stats-translator))
     ((stats'' :wat::telemetry::Service::Stats) (:wat::core::first step))
     ((cadence' :wat::telemetry::Service::MetricsCadence<G>)
      (:wat::core::second step)))
    (:wat::telemetry::Service/loop
      pairs stats'' cadence' dispatcher stats-translator)))


(:wat::core::define
  (:wat::telemetry::Service/loop<E,G>
    (pairs :wat::core::Vector<wat::telemetry::Service::DriverPair<E>>)
    (stats :wat::telemetry::Service::Stats)
    (cadence :wat::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Service::Stats)->wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::empty? pairs) -> :wat::core::unit
    ()
    (:wat::core::let*
      (((rxs :wat::core::Vector<wat::telemetry::Service::ReqRx<E>>)
        (:wat::telemetry::Service/pair-rxs pairs))
       ((chosen :wat::kernel::Chosen<wat::telemetry::Service::Request<E>>)
        (:wat::kernel::select rxs))
       ((idx :wat::core::i64) (:wat::core::first chosen))
       ((maybe :wat::kernel::CommResult<wat::telemetry::Service::Request<E>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :wat::core::unit
        ((Ok (Some first-entries))
          (:wat::telemetry::Service/loop-step
            pairs idx first-entries stats cadence dispatcher stats-translator))
        ((Ok :None)
          (:wat::telemetry::Service/loop
            (:wat::std::list::remove-at pairs idx)
            stats cadence dispatcher stats-translator))
        ((Err _died) ())))))


;; ─── Client helper — single primitive, batch + ack ───────────────
;;
;; Two channel ends. Block-write the entries; block-read the ack.
;; Single-entry callers wrap in a one-element vec.

(:wat::core::define
  (:wat::telemetry::Service/batch-log<E>
    (req-tx :wat::telemetry::Service::ReqTx<E>)
    (ack-rx :wat::telemetry::Service::AckRx)
    (entries :wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((_send :wat::core::unit)
      (:wat::core::result::expect -> :wat::core::unit
        (:wat::kernel::send req-tx entries)
        "Service/batch-log: req-tx disconnected — telemetry service died?"))
     ((_recv :wat::core::Option<wat::core::unit>)
      (:wat::core::result::expect -> :wat::core::Option<wat::core::unit>
        (:wat::kernel::recv ack-rx)
        "Service/batch-log: ack-rx disconnected — telemetry service died mid-flush?")))
    ()))


;; ─── Worker entry — initial Stats + enter loop ──────────────────

(:wat::core::define
  (:wat::telemetry::Service/run<E,G>
    (pairs :wat::core::Vector<wat::telemetry::Service::DriverPair<E>>)
    (cadence :wat::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Service::Stats)->wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::telemetry::Service/loop
    pairs
    (:wat::telemetry::Service::Stats/zero)
    cadence dispatcher stats-translator))


;; ─── Setup — spawn driver, return (HandlePool, driver) ───────────
;;
;; For each of the N connections, allocate ONE Request channel and
;; ONE Ack channel. The client gets (req-tx, ack-rx) — its Handle.
;; The server gets (req-rx, ack-tx) — its DriverPair. Pool hands
;; out Handles; worker thread carries the Vec of DriverPairs.

(:wat::core::define
  (:wat::telemetry::Service/spawn<E,G>
    (count :wat::core::i64)
    (cadence :wat::telemetry::Service::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Service::Stats)->wat::core::Vector<E>)
    -> :wat::telemetry::Service::Spawn<E>)
  (:wat::core::let*
    (((req-pairs :wat::core::Vector<wat::telemetry::Service::ReqChannel<E>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :wat::core::i64) -> :wat::telemetry::Service::ReqChannel<E>)
          (:wat::kernel::make-bounded-queue
            :wat::telemetry::Service::Request<E> 1))))
     ((ack-pairs :wat::core::Vector<wat::telemetry::Service::AckChannel>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :wat::core::i64) -> :wat::telemetry::Service::AckChannel)
          (:wat::kernel::make-bounded-queue :wat::core::unit 1))))
     ((handles :wat::core::Vector<wat::telemetry::Service::Handle<E>>)
      (:wat::core::map
        (:wat::std::list::zip req-pairs ack-pairs)
        (:wat::core::lambda
          ((rp+ap :wat::telemetry::Service::Connection<E>)
           -> :wat::telemetry::Service::Handle<E>)
          (:wat::core::let*
            (((rp :wat::telemetry::Service::ReqChannel<E>) (:wat::core::first rp+ap))
             ((ap :wat::telemetry::Service::AckChannel) (:wat::core::second rp+ap))
             ((req-tx :wat::telemetry::Service::ReqTx<E>) (:wat::core::first rp))
             ((ack-rx :wat::telemetry::Service::AckRx) (:wat::core::second ap)))
            (:wat::core::Tuple req-tx ack-rx)))))
     ((driver-pairs :wat::core::Vector<wat::telemetry::Service::DriverPair<E>>)
      (:wat::core::map
        (:wat::std::list::zip req-pairs ack-pairs)
        (:wat::core::lambda
          ((rp+ap :wat::telemetry::Service::Connection<E>)
           -> :wat::telemetry::Service::DriverPair<E>)
          (:wat::core::let*
            (((rp :wat::telemetry::Service::ReqChannel<E>) (:wat::core::first rp+ap))
             ((ap :wat::telemetry::Service::AckChannel) (:wat::core::second rp+ap))
             ((req-rx :wat::telemetry::Service::ReqRx<E>) (:wat::core::second rp))
             ((ack-tx :wat::telemetry::Service::AckTx) (:wat::core::first ap)))
            (:wat::core::Tuple req-rx ack-tx)))))
     ((pool :wat::telemetry::Service::HandlePool<E>)
      (:wat::kernel::HandlePool::new "telemetry::Service" handles))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::telemetry::Service/run
            driver-pairs cadence dispatcher stats-translator)))))
    (:wat::core::Tuple pool driver)))
