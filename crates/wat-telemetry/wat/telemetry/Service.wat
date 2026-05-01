;; :wat::telemetry::* — generic queue-fronted destination service
;; for structured records.
;;
;; Arc 080 (initial design) + arc 089 slice 2-3 (drain-all +
;; per-batch dispatch) + arc 095 (paired channels, this protocol)
;; + arc 109 slice K.telemetry (Service grouping noun retired;
;; verbs and typealiases live at the namespace level per § K's
;; "/ requires a real Type" doctrine).
;;
;; Channel-naming family: Pattern A (Request + Ack — data forward,
;; release back, server matches by index). See INVENTORY § K.
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
;;   1. Caller `(:wat::telemetry::spawn count dispatcher translator cadence)`
;;      → `(HandlePool<E>, ProgramHandle<()>)`.
;;   2. Driver loop opens nothing — substrate has no resources to
;;      manage; the dispatcher closes over whatever the consumer
;;      supplies (db handle, console-handle, etc).
;;   3. Caller pops Handles, distributes, finishes the pool.
;;   4. Each client `(:wat::telemetry::batch-log req-tx ack-rx entries)`
;;      sends + acks per arc 029's Q10 ("confirmed batch + ack").
;;      Two channel ends. No reply-tx-in-request.
;;   5. Driver `select`s over rx side of pairs; on fire, pulls the
;;      matching ack-tx by index, dispatches, acks back through
;;      that ack-tx.
;;   6. Clients drop their Handles. Driver loop converges, exits,
;;      `(join driver)` confirms clean exit.

;; ─── Self-heartbeat contract — Stats + MetricsCadence ────────────

(:wat::core::struct :wat::telemetry::Stats
  (batches :wat::core::i64)
  (entries :wat::core::i64)
  (max-batch-size :wat::core::i64))

(:wat::core::struct :wat::telemetry::MetricsCadence<G>
  (gate :G)
  (tick :fn(G,wat::telemetry::Stats)->(G,wat::core::bool)))

(:wat::core::define
  (:wat::telemetry::null-metrics-cadence
    -> :wat::telemetry::MetricsCadence<wat::core::unit>)
  (:wat::telemetry::MetricsCadence/new
    ()
    (:wat::core::lambda
      ((gate :wat::core::unit) (_stats :wat::telemetry::Stats) -> :(wat::core::unit,wat::core::bool))
      (:wat::core::Tuple gate false))))

(:wat::core::define
  (:wat::telemetry::Stats/zero
    -> :wat::telemetry::Stats)
  (:wat::telemetry::Stats/new 0 0 0))


;; ─── Protocol typealiases (arc 095) ──────────────────────────────

;; Ack channel — unit signal. Same shape both sides; the (tx, rx)
;; pair is split between server and client, NOT bundled on either.
(:wat::core::typealias :wat::telemetry::AckTx
  :wat::kernel::Sender<wat::core::unit>)
(:wat::core::typealias :wat::telemetry::AckRx
  :wat::kernel::Receiver<wat::core::unit>)
(:wat::core::typealias :wat::telemetry::AckChannel
  :(wat::telemetry::AckTx,wat::telemetry::AckRx))

;; Request — just the batch of entries. The client's reply address
;; is no longer in the wire payload (retired arc 095); the server
;; holds the matching ack-tx in its paired DriverPair vector.
(:wat::core::typealias :wat::telemetry::Request<E>
  :wat::core::Vector<E>)

(:wat::core::typealias :wat::telemetry::ReqTx<E>
  :wat::kernel::Sender<wat::telemetry::Request<E>>)
(:wat::core::typealias :wat::telemetry::ReqRx<E>
  :wat::kernel::Receiver<wat::telemetry::Request<E>>)

(:wat::core::typealias :wat::telemetry::ReqChannel<E>
  :(wat::telemetry::ReqTx<E>,wat::telemetry::ReqRx<E>))

;; A complete client/server connection — one ReqChannel and one
;; AckChannel that the spawn step distributes between Handle (client
;; side) and DriverPair (server side). Aliased so spawn's zip-and-map
;; doesn't smear the verbose tuple form across every lambda body.
(:wat::core::typealias :wat::telemetry::Connection<E>
  :(wat::telemetry::ReqChannel<E>,wat::telemetry::AckChannel))

;; Client-side Handle — what the consumer pops from the pool.
;; Two opposite ends: req-tx to write, ack-rx to read.
(:wat::core::typealias :wat::telemetry::Handle<E>
  :(wat::telemetry::ReqTx<E>,wat::telemetry::AckRx))

;; Server-side pair — what the worker holds in parallel by index.
;; Two opposite ends: req-rx to read, ack-tx to write.
(:wat::core::typealias :wat::telemetry::DriverPair<E>
  :(wat::telemetry::ReqRx<E>,wat::telemetry::AckTx))

;; A DriverPair tagged with its index in the server's pairs vector.
;; Used by drain-rest's foldl to skip the rx select already
;; consumed (first-idx) and look up the matching ack-tx by position.
(:wat::core::typealias :wat::telemetry::IndexedDriverPair<E>
  :(wat::telemetry::DriverPair<E>,wat::core::i64))

(:wat::core::typealias :wat::telemetry::HandlePool<E>
  :wat::kernel::HandlePool<wat::telemetry::Handle<E>>)

(:wat::core::typealias :wat::telemetry::Spawn<E>
  :(wat::telemetry::HandlePool<E>,wat::kernel::Thread<wat::core::unit,wat::core::unit>))

(:wat::core::typealias :wat::telemetry::Step<G>
  :(wat::telemetry::Stats,wat::telemetry::MetricsCadence<G>))


;; ─── Tick the heartbeat window ───────────────────────────────────

(:wat::core::define
  (:wat::telemetry::tick-window<E,G>
    (stats :wat::telemetry::Stats)
    (cadence :wat::telemetry::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<E>)
    -> :wat::telemetry::Step<G>)
  (:wat::core::let*
    (((gate :G)
      (:wat::telemetry::MetricsCadence/gate cadence))
     ((tick-fn :fn(G,wat::telemetry::Stats)->(G,wat::core::bool))
      (:wat::telemetry::MetricsCadence/tick cadence))
     ((tick :(G,wat::core::bool)) (tick-fn gate stats))
     ((gate' :G) (:wat::core::first tick))
     ((fired :wat::core::bool) (:wat::core::second tick))
     ((cadence' :wat::telemetry::MetricsCadence<G>)
      (:wat::telemetry::MetricsCadence/new gate' tick-fn)))
    (:wat::core::if fired
      -> :wat::telemetry::Step<G>
      (:wat::core::let*
        (((entries :wat::core::Vector<E>) (stats-translator stats))
         ((_dispatch :wat::core::unit) (dispatcher entries)))
        (:wat::core::Tuple
          (:wat::telemetry::Stats/zero) cadence'))
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
(:wat::core::typealias :wat::telemetry::Pending<E>
  :(wat::core::Vector<E>,wat::core::Vector<wat::telemetry::AckTx>))


;; Add one client's contribution into the Pending accumulator.
;; Entries come from the Request payload; ack-tx comes from the
;; server's paired DriverPair.
(:wat::core::define
  (:wat::telemetry::extend<E>
    (acc :wat::telemetry::Pending<E>)
    (req-entries :wat::core::Vector<E>)
    (ack :wat::telemetry::AckTx)
    -> :wat::telemetry::Pending<E>)
  (:wat::core::let*
    (((entries :wat::core::Vector<E>) (:wat::core::first acc))
     ((acks :wat::core::Vector<wat::telemetry::AckTx>) (:wat::core::second acc))
     ((entries' :wat::core::Vector<E>) (:wat::core::concat entries req-entries))
     ((acks' :wat::core::Vector<wat::telemetry::AckTx>)
      (:wat::core::concat acks
        (:wat::core::Vector :wat::telemetry::AckTx ack))))
    (:wat::core::Tuple entries' acks')))


;; Merge one indexed pair into the accumulator. On the first-idx
;; pair, attach `first-entries` (already drained by select) +
;; pair.ack. On every other pair, try-recv pair.rx; on a hit,
;; attach entries + pair.ack. The single foldl over ALL pairs
;; eliminates the prior split between "first" and "rest" and the
;; need for an out-of-band lookup.
(:wat::core::define
  (:wat::telemetry::maybe-merge<E>
    (acc :wat::telemetry::Pending<E>)
    (first-idx :wat::core::i64)
    (first-entries :wat::core::Vector<E>)
    (indexed :wat::telemetry::IndexedDriverPair<E>)
    -> :wat::telemetry::Pending<E>)
  (:wat::core::let*
    (((pair :wat::telemetry::DriverPair<E>) (:wat::core::first indexed))
     ((idx :wat::core::i64) (:wat::core::second indexed))
     ((rx :wat::telemetry::ReqRx<E>) (:wat::core::first pair))
     ((ack :wat::telemetry::AckTx) (:wat::core::second pair)))
    (:wat::core::if (:wat::core::= idx first-idx)
      -> :wat::telemetry::Pending<E>
      (:wat::telemetry::extend acc first-entries ack)
      (:wat::core::match (:wat::kernel::try-recv rx)
        -> :wat::telemetry::Pending<E>
        ((:wat::core::Ok (:wat::core::Some req-entries))
          (:wat::telemetry::extend acc req-entries ack))
        ((:wat::core::Ok :wat::core::None) acc)
        ((:wat::core::Err _died) acc)))))


;; Drain — single foldl over all pairs. The first-idx pair gets
;; first-entries from select; every other pair tries try-recv.
(:wat::core::define
  (:wat::telemetry::drain-pairs<E>
    (pairs :wat::core::Vector<wat::telemetry::DriverPair<E>>)
    (first-idx :wat::core::i64)
    (first-entries :wat::core::Vector<E>)
    (init :wat::telemetry::Pending<E>)
    -> :wat::telemetry::Pending<E>)
  (:wat::core::let*
    (((indices :wat::core::Vector<wat::core::i64>)
      (:wat::core::range 0 (:wat::core::length pairs)))
     ((indexed :wat::core::Vector<wat::telemetry::IndexedDriverPair<E>>)
      (:wat::std::list::zip pairs indices)))
    (:wat::core::foldl indexed init
      (:wat::core::lambda
        ((acc :wat::telemetry::Pending<E>)
         (pair :wat::telemetry::IndexedDriverPair<E>)
         -> :wat::telemetry::Pending<E>)
        (:wat::telemetry::maybe-merge acc first-idx first-entries pair)))))


;; Send () on every contributing client's ack-tx.
(:wat::core::define
  (:wat::telemetry::ack-all
    (ack-txs :wat::core::Vector<wat::telemetry::AckTx>)
    -> :wat::core::unit)
  (:wat::core::foldl ack-txs ()
    (:wat::core::lambda
      ((_acc :wat::core::unit) (tx :wat::telemetry::AckTx) -> :wat::core::unit)
      (:wat::core::match (:wat::kernel::send tx ()) -> :wat::core::unit
        ((:wat::core::Ok _) ())
        ((:wat::core::Err _) ())))))


(:wat::core::define
  (:wat::telemetry::bump-stats
    (stats :wat::telemetry::Stats)
    (batch-size :wat::core::i64)
    -> :wat::telemetry::Stats)
  (:wat::core::let*
    (((max-prev :wat::core::i64)
      (:wat::telemetry::Stats/max-batch-size stats))
     ((max' :wat::core::i64)
      (:wat::core::if (:wat::core::> batch-size max-prev) -> :wat::core::i64
        batch-size
        max-prev)))
    (:wat::telemetry::Stats/new
      (:wat::core::+ (:wat::telemetry::Stats/batches stats) 1)
      (:wat::core::+ (:wat::telemetry::Stats/entries stats) batch-size)
      max')))


;; Extract the wat::core::Vector<ReqRx> half of pairs for the kernel select.
(:wat::core::define
  (:wat::telemetry::pair-rxs<E>
    (pairs :wat::core::Vector<wat::telemetry::DriverPair<E>>)
    -> :wat::core::Vector<wat::telemetry::ReqRx<E>>)
  (:wat::core::map pairs
    (:wat::core::lambda
      ((p :wat::telemetry::DriverPair<E>)
       -> :wat::telemetry::ReqRx<E>)
      (:wat::core::first p))))


;; One drain-and-dispatch cycle. drain-pairs handles BOTH first-idx
;; (which gets first-entries from select) and the rest (which try-recv).
;; No separate first-pair lookup needed.
(:wat::core::define
  (:wat::telemetry::loop-step<E,G>
    (pairs :wat::core::Vector<wat::telemetry::DriverPair<E>>)
    (first-idx :wat::core::i64)
    (first-entries :wat::core::Vector<E>)
    (stats :wat::telemetry::Stats)
    (cadence :wat::telemetry::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((init :wat::telemetry::Pending<E>)
      (:wat::core::Tuple
        (:wat::core::Vector :E)
        (:wat::core::Vector :wat::telemetry::AckTx)))
     ((pending :wat::telemetry::Pending<E>)
      (:wat::telemetry::drain-pairs pairs first-idx first-entries init))
     ((entries :wat::core::Vector<E>) (:wat::core::first pending))
     ((ack-txs :wat::core::Vector<wat::telemetry::AckTx>)
      (:wat::core::second pending))
     ((_apply :wat::core::unit) (dispatcher entries))
     ((_ack :wat::core::unit) (:wat::telemetry::ack-all ack-txs))
     ((batch-size :wat::core::i64) (:wat::core::length entries))
     ((stats' :wat::telemetry::Stats)
      (:wat::telemetry::bump-stats stats batch-size))
     ((step :wat::telemetry::Step<G>)
      (:wat::telemetry::tick-window
        stats' cadence dispatcher stats-translator))
     ((stats'' :wat::telemetry::Stats) (:wat::core::first step))
     ((cadence' :wat::telemetry::MetricsCadence<G>)
      (:wat::core::second step)))
    (:wat::telemetry::loop
      pairs stats'' cadence' dispatcher stats-translator)))


(:wat::core::define
  (:wat::telemetry::loop<E,G>
    (pairs :wat::core::Vector<wat::telemetry::DriverPair<E>>)
    (stats :wat::telemetry::Stats)
    (cadence :wat::telemetry::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::empty? pairs) -> :wat::core::unit
    ()
    (:wat::core::let*
      (((rxs :wat::core::Vector<wat::telemetry::ReqRx<E>>)
        (:wat::telemetry::pair-rxs pairs))
       ((chosen :wat::kernel::Chosen<wat::telemetry::Request<E>>)
        (:wat::kernel::select rxs))
       ((idx :wat::core::i64) (:wat::core::first chosen))
       ((maybe :wat::kernel::CommResult<wat::telemetry::Request<E>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :wat::core::unit
        ((:wat::core::Ok (:wat::core::Some first-entries))
          (:wat::telemetry::loop-step
            pairs idx first-entries stats cadence dispatcher stats-translator))
        ((:wat::core::Ok :wat::core::None)
          (:wat::telemetry::loop
            (:wat::std::list::remove-at pairs idx)
            stats cadence dispatcher stats-translator))
        ((:wat::core::Err _died) ())))))


;; ─── Client helper — single primitive, batch + ack ───────────────
;;
;; Two channel ends. Block-write the entries; block-read the ack.
;; Single-entry callers wrap in a one-element vec.

(:wat::core::define
  (:wat::telemetry::batch-log<E>
    (req-tx :wat::telemetry::ReqTx<E>)
    (ack-rx :wat::telemetry::AckRx)
    (entries :wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((_send :wat::core::unit)
      (:wat::core::Result/expect -> :wat::core::unit
        (:wat::kernel::send req-tx entries)
        "Service/batch-log: req-tx disconnected — telemetry service died?"))
     ((_recv :wat::core::Option<wat::core::unit>)
      (:wat::core::Result/expect -> :wat::core::Option<wat::core::unit>
        (:wat::kernel::recv ack-rx)
        "Service/batch-log: ack-rx disconnected — telemetry service died mid-flush?")))
    ()))


;; ─── Worker entry — initial Stats + enter loop ──────────────────

(:wat::core::define
  (:wat::telemetry::run<E,G>
    (pairs :wat::core::Vector<wat::telemetry::DriverPair<E>>)
    (cadence :wat::telemetry::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<E>)
    -> :wat::core::unit)
  (:wat::telemetry::loop
    pairs
    (:wat::telemetry::Stats/zero)
    cadence dispatcher stats-translator))


;; ─── Setup — spawn driver, return (HandlePool, driver) ───────────
;;
;; For each of the N connections, allocate ONE Request channel and
;; ONE Ack channel. The client gets (req-tx, ack-rx) — its Handle.
;; The server gets (req-rx, ack-tx) — its DriverPair. Pool hands
;; out Handles; worker thread carries the Vec of DriverPairs.

(:wat::core::define
  (:wat::telemetry::spawn<E,G>
    (count :wat::core::i64)
    (cadence :wat::telemetry::MetricsCadence<G>)
    (dispatcher :fn(wat::core::Vector<E>)->wat::core::unit)
    (stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<E>)
    -> :wat::telemetry::Spawn<E>)
  (:wat::core::let*
    (((req-pairs :wat::core::Vector<wat::telemetry::ReqChannel<E>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :wat::core::i64) -> :wat::telemetry::ReqChannel<E>)
          (:wat::kernel::make-bounded-channel
            :wat::telemetry::Request<E> 1))))
     ((ack-pairs :wat::core::Vector<wat::telemetry::AckChannel>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :wat::core::i64) -> :wat::telemetry::AckChannel)
          (:wat::kernel::make-bounded-channel :wat::core::unit 1))))
     ((handles :wat::core::Vector<wat::telemetry::Handle<E>>)
      (:wat::core::map
        (:wat::std::list::zip req-pairs ack-pairs)
        (:wat::core::lambda
          ((rp+ap :wat::telemetry::Connection<E>)
           -> :wat::telemetry::Handle<E>)
          (:wat::core::let*
            (((rp :wat::telemetry::ReqChannel<E>) (:wat::core::first rp+ap))
             ((ap :wat::telemetry::AckChannel) (:wat::core::second rp+ap))
             ((req-tx :wat::telemetry::ReqTx<E>) (:wat::core::first rp))
             ((ack-rx :wat::telemetry::AckRx) (:wat::core::second ap)))
            (:wat::core::Tuple req-tx ack-rx)))))
     ((driver-pairs :wat::core::Vector<wat::telemetry::DriverPair<E>>)
      (:wat::core::map
        (:wat::std::list::zip req-pairs ack-pairs)
        (:wat::core::lambda
          ((rp+ap :wat::telemetry::Connection<E>)
           -> :wat::telemetry::DriverPair<E>)
          (:wat::core::let*
            (((rp :wat::telemetry::ReqChannel<E>) (:wat::core::first rp+ap))
             ((ap :wat::telemetry::AckChannel) (:wat::core::second rp+ap))
             ((req-rx :wat::telemetry::ReqRx<E>) (:wat::core::second rp))
             ((ack-tx :wat::telemetry::AckTx) (:wat::core::first ap)))
            (:wat::core::Tuple req-rx ack-tx)))))
     ((pool :wat::telemetry::HandlePool<E>)
      (:wat::kernel::HandlePool::new "telemetry::Service" handles))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::telemetry::run
            driver-pairs cadence dispatcher stats-translator)))))
    (:wat::core::Tuple pool driver)))
