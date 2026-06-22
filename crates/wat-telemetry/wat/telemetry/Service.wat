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

(:wat::core::defstruct :wat::telemetry::Stats
  [batches        <- :wat::core::i64
   entries        <- :wat::core::i64
   max-batch-size <- :wat::core::i64])

(:wat::core::defstruct :wat::telemetry::MetricsCadence<G>
  [gate <- :G
   tick <- :wat::core::Fn(G,wat::telemetry::Stats)->(G,wat::core::bool)])

(:wat::core::defn :wat::telemetry::null-metrics-cadence
  [] -> :wat::telemetry::MetricsCadence<wat::core::nil>
  (:wat::telemetry::MetricsCadence/new
    nil
    (:wat::core::fn
      [gate <- :wat::core::nil _stats <- :wat::telemetry::Stats] -> :(wat::core::nil,wat::core::bool)
      (:wat::core::Tuple gate false))))

(:wat::core::defn :wat::telemetry::Stats/zero
  [] -> :wat::telemetry::Stats
  (:wat::telemetry::Stats/new 0 0 0))


;; ─── Protocol typealiases (arc 095) ──────────────────────────────

;; Ack channel — unit signal. Same shape both sides; the (tx, rx)
;; pair is split between server and client, NOT bundled on either.
(:wat::core::typealias :wat::telemetry::AckTx
  :wat::kernel::Sender<wat::core::nil>)
(:wat::core::typealias :wat::telemetry::AckRx
  :wat::kernel::Receiver<wat::core::nil>)
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
;; doesn't smear the verbose tuple form across every fn body.
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
  :(wat::telemetry::HandlePool<E>,wat::kernel::Thread<wat::core::nil,wat::core::nil>))

(:wat::core::typealias :wat::telemetry::Step<G>
  :(wat::telemetry::Stats,wat::telemetry::MetricsCadence<G>))


;; ─── Tick the heartbeat window ───────────────────────────────────

(:wat::core::defn :wat::telemetry::tick-window<E,G>
  [stats <- :wat::telemetry::Stats
   cadence <- :wat::telemetry::MetricsCadence<G>
   dispatcher <- :wat::core::Fn(wat::core::Vector<E>)->wat::core::nil
   stats-translator <- :wat::core::Fn(wat::telemetry::Stats)->wat::core::Vector<E>]
  -> :wat::telemetry::Step<G>
  (:wat::core::let
    [gate
      (:wat::telemetry::MetricsCadence/gate cadence)
     tick-fn
      (:wat::telemetry::MetricsCadence/tick cadence)
     tick (tick-fn gate stats)
     gate' (:wat::core::first tick)
     fired (:wat::core::second tick)
     cadence'
      (:wat::telemetry::MetricsCadence/new gate' tick-fn)]
    (:wat::core::if fired
      -> :wat::telemetry::Step<G>
      (:wat::core::let
        [entries (stats-translator stats)
         _dispatch (dispatcher entries)]
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
;;   4. on Some(first-entries) — that ONE select-chosen client is the
;;      tick's contribution (arc 214 ε: try-recv is dead, so there is
;;      no opportunistic peek of the other rxs; each ready client is
;;      handled on its own select, which fires immediately while its
;;      fd stays readable)
;;   5. dispatch via the per-batch dispatcher
;;   6. ack — release the contributing client's batch-log
;;      (preserves the "in-memory TCP" discipline)
;;   7. update Stats with batch size + tick window
;;   8. recurse with (stats', cadence')

;; Pending — accumulator threaded through drain-rest. (entries,
;; ack-txs). The ack-txs come from the server's paired vector
;; lookup, not from any request payload.
(:wat::core::typealias :wat::telemetry::Pending<E>
  :(wat::core::Vector<E>,wat::core::Vector<wat::telemetry::AckTx>))


;; Add one client's contribution into the Pending accumulator.
;; Entries come from the Request payload; ack-tx comes from the
;; server's paired DriverPair.
(:wat::core::defn :wat::telemetry::extend<E>
  [acc <- :wat::telemetry::Pending<E>
   req-entries <- :wat::core::Vector<E>
   ack <- :wat::telemetry::AckTx]
  -> :wat::telemetry::Pending<E>
  (:wat::core::let
    [entries (:wat::core::first acc)
     acks (:wat::core::second acc)
     entries' (:wat::core::concat entries req-entries)
     acks'
      (:wat::core::concat acks
        (:wat::core::Vector :wat::telemetry::AckTx ack))]
    (:wat::core::Tuple entries' acks')))


;; Merge one indexed pair into the accumulator. On the first-idx
;; pair — the one `select` woke on — attach `first-entries` (already
;; drained by select) + pair.ack. Every OTHER pair is skipped this
;; tick: arc 214 ε annihilated `try-recv` (the non-blocking peek), so
;; there is no opportunistic cross-channel drain. The lock-step model
;; handles exactly the select-chosen client per tick; any other pair
;; with data is picked up on the next `select` (which returns
;; immediately while its fd stays readable). The foldl still locates
;; the contributing pair + its ack by index — no out-of-band lookup.
(:wat::core::defn :wat::telemetry::maybe-merge<E>
  [acc <- :wat::telemetry::Pending<E>
   first-idx <- :wat::core::i64
   first-entries <- :wat::core::Vector<E>
   indexed <- :wat::telemetry::IndexedDriverPair<E>]
  -> :wat::telemetry::Pending<E>
  (:wat::core::let
    [pair (:wat::core::first indexed)
     idx (:wat::core::second indexed)
     ack (:wat::core::second pair)]
    (:wat::core::if (:wat::core::= idx first-idx)
      -> :wat::telemetry::Pending<E>
      (:wat::telemetry::extend acc first-entries ack)
      acc)))


;; Drain — single foldl over all pairs. Only the first-idx pair (the
;; one select woke on) contributes its first-entries; every other pair
;; is skipped this tick (no try-recv peek — see maybe-merge).
(:wat::core::defn :wat::telemetry::drain-pairs<E>
  [pairs <- :wat::core::Vector<wat::telemetry::DriverPair<E>>
   first-idx <- :wat::core::i64
   first-entries <- :wat::core::Vector<E>
   init <- :wat::telemetry::Pending<E>]
  -> :wat::telemetry::Pending<E>
  (:wat::core::let
    [indices
      (:wat::core::range 0 (:wat::core::length pairs))
     indexed
      (:wat::std::list::zip pairs indices)]
    (:wat::core::foldl
      (:wat::core::fn
        [acc <- :wat::telemetry::Pending<E>
         pair <- :wat::telemetry::IndexedDriverPair<E>]
         -> :wat::telemetry::Pending<E>
        (:wat::telemetry::maybe-merge acc first-idx first-entries pair))
      init
      indexed)))


;; Send () on every contributing client's ack-tx.
(:wat::core::defn :wat::telemetry::ack-all
  [ack-txs <- :wat::core::Vector<wat::telemetry::AckTx>]
  -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn
      [_acc <- :wat::core::nil tx <- :wat::telemetry::AckTx] -> :wat::core::nil
      (:wat::core::match (:wat::kernel::send tx nil) -> :wat::core::nil
        ((:wat::core::Ok _) nil)
        ((:wat::core::Err _) nil)))
    nil
    ack-txs))


(:wat::core::defn :wat::telemetry::bump-stats
  [stats <- :wat::telemetry::Stats
   batch-size <- :wat::core::i64]
  -> :wat::telemetry::Stats
  (:wat::core::let
    [max-prev
      (:wat::telemetry::Stats/max-batch-size stats)
     max'
      (:wat::core::if (:wat::core::> batch-size max-prev) -> :wat::core::i64
        batch-size
        max-prev)]
    (:wat::telemetry::Stats/new
      (:wat::core::+ (:wat::telemetry::Stats/batches stats) 1)
      (:wat::core::+ (:wat::telemetry::Stats/entries stats) batch-size)
      max')))


;; Extract the wat::core::Vector<ReqRx> half of pairs for the kernel select.
(:wat::core::defn :wat::telemetry::pair-rxs<E>
  [pairs <- :wat::core::Vector<wat::telemetry::DriverPair<E>>]
  -> :wat::core::Vector<wat::telemetry::ReqRx<E>>
  (:wat::core::map
    (:wat::core::fn
      [p <- :wat::telemetry::DriverPair<E>]
       -> :wat::telemetry::ReqRx<E>
      (:wat::core::first p))
    pairs))


;; One drain-and-dispatch cycle. drain-pairs contributes the first-idx
;; pair (which gets first-entries from select); every other pair is
;; skipped this tick and re-selected next loop. No separate first-pair
;; lookup needed.
(:wat::core::defn :wat::telemetry::loop-step<E,G>
  [pairs <- :wat::core::Vector<wat::telemetry::DriverPair<E>>
   first-idx <- :wat::core::i64
   first-entries <- :wat::core::Vector<E>
   stats <- :wat::telemetry::Stats
   cadence <- :wat::telemetry::MetricsCadence<G>
   dispatcher <- :wat::core::Fn(wat::core::Vector<E>)->wat::core::nil
   stats-translator <- :wat::core::Fn(wat::telemetry::Stats)->wat::core::Vector<E>]
  -> :wat::core::nil
  (:wat::core::let
    [init
      (:wat::core::Tuple
        (:wat::core::Vector :E)
        (:wat::core::Vector :wat::telemetry::AckTx))
     pending
      (:wat::telemetry::drain-pairs pairs first-idx first-entries init)
     entries (:wat::core::first pending)
     ack-txs
      (:wat::core::second pending)
     _apply (dispatcher entries)
     _ack (:wat::telemetry::ack-all ack-txs)
     batch-size (:wat::core::length entries)
     stats'
      (:wat::telemetry::bump-stats stats batch-size)
     step
      (:wat::telemetry::tick-window
        stats' cadence dispatcher stats-translator)
     stats'' (:wat::core::first step)
     cadence'
      (:wat::core::second step)]
    (:wat::telemetry::loop
      pairs stats'' cadence' dispatcher stats-translator)))


(:wat::core::defn :wat::telemetry::loop<E,G>
  [pairs <- :wat::core::Vector<wat::telemetry::DriverPair<E>>
   stats <- :wat::telemetry::Stats
   cadence <- :wat::telemetry::MetricsCadence<G>
   dispatcher <- :wat::core::Fn(wat::core::Vector<E>)->wat::core::nil
   stats-translator <- :wat::core::Fn(wat::telemetry::Stats)->wat::core::Vector<E>]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? pairs) -> :wat::core::nil
    nil
    (:wat::core::let
      [rxs
        (:wat::telemetry::pair-rxs pairs)
       chosen
        (:wat::kernel::select rxs)
       idx (:wat::core::first chosen)
       maybe
        (:wat::core::second chosen)]
      (:wat::core::match maybe -> :wat::core::nil
        ((:wat::core::Ok (:wat::core::Some first-entries))
          (:wat::telemetry::loop-step
            pairs idx first-entries stats cadence dispatcher stats-translator))
        ((:wat::core::Ok :wat::core::None)
          (:wat::telemetry::loop
            (:wat::std::list::remove-at pairs idx)
            stats cadence dispatcher stats-translator))
        ((:wat::core::Err _died) nil)))))


;; ─── Client helper — single primitive, batch + ack ───────────────
;;
;; Two channel ends. Block-write the entries; block-read the ack.
;; Single-entry callers wrap in a one-element vec.

(:wat::core::defn :wat::telemetry::batch-log<E>
  [req-tx <- :wat::telemetry::ReqTx<E>
   ack-rx <- :wat::telemetry::AckRx
   entries <- :wat::core::Vector<E>]
  -> :wat::core::nil
  (:wat::core::let
    [_send
      (:wat::core::Result/expect  
        (:wat::kernel::send req-tx entries)
        "Service/batch-log: req-tx disconnected — telemetry service died?")
     _recv
      (:wat::core::Result/expect  
        (:wat::kernel::recv ack-rx)
        "Service/batch-log: ack-rx disconnected — telemetry service died mid-flush?")]
    nil))


;; ─── Worker entry — initial Stats + enter loop ──────────────────

(:wat::core::defn :wat::telemetry::run<E,G>
  [pairs <- :wat::core::Vector<wat::telemetry::DriverPair<E>>
   cadence <- :wat::telemetry::MetricsCadence<G>
   dispatcher <- :wat::core::Fn(wat::core::Vector<E>)->wat::core::nil
   stats-translator <- :wat::core::Fn(wat::telemetry::Stats)->wat::core::Vector<E>]
  -> :wat::core::nil
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

(:wat::core::defn :wat::telemetry::spawn<E,G>
  [count <- :wat::core::i64
   cadence <- :wat::telemetry::MetricsCadence<G>
   dispatcher <- :wat::core::Fn(wat::core::Vector<E>)->wat::core::nil
   stats-translator <- :wat::core::Fn(wat::telemetry::Stats)->wat::core::Vector<E>]
  -> :wat::telemetry::Spawn<E>
  (:wat::core::let
    [req-pairs
      (:wat::core::map
        (:wat::core::fn
          [_i <- :wat::core::i64] -> :wat::telemetry::ReqChannel<E>
          (:wat::kernel::make-channel
            :wat::telemetry::Request<E>))
        (:wat::core::range 0 count))
     ack-pairs
      (:wat::core::map
        (:wat::core::fn
          [_i <- :wat::core::i64] -> :wat::telemetry::AckChannel
          (:wat::kernel::make-channel :wat::core::nil))
        (:wat::core::range 0 count))
     handles
      (:wat::core::map
        (:wat::core::fn
          [rp+ap <- :wat::telemetry::Connection<E>]
           -> :wat::telemetry::Handle<E>
          (:wat::core::let
            [rp (:wat::core::first rp+ap)
             ap (:wat::core::second rp+ap)
             req-tx (:wat::core::first rp)
             ack-rx (:wat::core::second ap)]
            (:wat::core::Tuple req-tx ack-rx)))
        (:wat::std::list::zip req-pairs ack-pairs))
     driver-pairs
      (:wat::core::map
        (:wat::core::fn
          [rp+ap <- :wat::telemetry::Connection<E>]
           -> :wat::telemetry::DriverPair<E>
          (:wat::core::let
            [rp (:wat::core::first rp+ap)
             ap (:wat::core::second rp+ap)
             req-rx (:wat::core::second rp)
             ack-tx (:wat::core::first ap)]
            (:wat::core::Tuple req-rx ack-tx)))
        (:wat::std::list::zip req-pairs ack-pairs))
     pool
      (:wat::kernel::HandlePool::new "telemetry::Service" handles)
     driver
      (:wat::kernel::spawn-thread
        (:wat::core::fn
          [_in <- :rust::crossbeam_channel::Receiver<wat::core::nil>
           _out <- :rust::crossbeam_channel::Sender<wat::core::nil>]
           -> :wat::core::nil
          (:wat::telemetry::run
            driver-pairs cadence dispatcher stats-translator)))]
    (:wat::core::Tuple pool driver)))
