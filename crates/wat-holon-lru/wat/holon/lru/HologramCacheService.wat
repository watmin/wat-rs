;; :wat::holon::lru::HologramCacheService — queue-addressed wrapper for
;; HologramCache. A long-running spawned program that owns a cache
;; instance and serves requests via per-slot request queues. Each
;; client gets a Handle = (ReqTx, ReplyRx); the driver holds the
;; matching DriverPair = (ReqRx, ReplyTx) at the same index.
;;
;; Arc 078: ported from the lab's :trading::cache::Service. Nothing
;; in the cache service shape is trader-specific — the Request enum,
;; the Reporter contract, the cadence-gated metrics — all of it is
;; generic substrate machinery built atop HologramCache. The trader
;; (and any other consumer) merely USES it.
;;
;; Arc 119: symmetric batch protocol. Request is an enum (Get | Put).
;;   Get carries Vec<HolonAST> probes,   returns Vec<Option<HolonAST>> via Reply::GetResult
;;   Put carries Vec<Entry>,             returns unit                  via Reply::PutAck
;;
;; Arc 130: pair-by-index. spawn pre-allocates N (ReqChannel, ReplyChannel)
;; pairs. HandlePool holds N Handle = (ReqTx, ReplyRx). Driver holds
;; N DriverPair = (ReqRx, ReplyTx). select fires at index i; same
;; index locates the ReplyTx. No per-call channel allocation.
;;
;; K = V = :wat::holon::HolonAST throughout (concrete, not parametric).
;; The new typealiases (Reply, ReplyTx, ReplyRx, ReplyChannel, Handle,
;; DriverPair) carry no type parameters — the wat-lru template's
;; <K,V> heads collapse to bare names here.
;;
;; The Reporter + MetricsCadence + null-* + typed Report enum pattern
;; documented here is the canonical service-contract idiom for
;; queue-addressed substrate services. Future stdlib services follow
;; this shape; see CONVENTIONS.md "Service contract" section.
;;
;; Surface:
;;   - Request: Get(probes) | Put(entries)
;;   - Reply:   GetResult(results) | PutAck
;;   - Handle:  (ReqTx, ReplyRx) — popped from HandlePool at spawn
;;   - State:   HologramCache + Stats (cache + per-window counters)
;;   - Telemetry: caller-supplied (reporter, metrics-cadence) pair.
;;     Both are non-negotiable: caller must pass both. Pass
;;     :wat::holon::lru::HologramCacheService/null-reporter and
;;     (:wat::holon::lru::HologramCacheService/null-metrics-cadence)
;;     for the explicit "no reporting" choice.
;;
;; Pattern mirrors archive's programs/stdlib/cache.rs::cache(can_emit,
;; emit) — same callback-injection idea, lifted to wat's stateful-
;; values-up shape: the cadence's tick is `:wat::core::Fn(G, Stats) -> :(G, bool)`
;; so the user threads time / counters / whatever through the loop
;; without reaching for Mutex.
;;
;; Arc 076 + 077: slot routing inferred from the form's structure
;; (the substrate does it inside HologramCache); no caller-supplied
;; pos. Filter is bound at HologramCache/make time.
;;
;; See docs/CONVENTIONS.md § "Batch convention" and
;; docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md.

;; ─── Entry typealias ────────────────────────────────────────────
;;
;; Entry = (HolonAST, HolonAST) — the batch-element name (arc 119).
;; Concrete K=V=HolonAST; no type parameters.
(:wat::core::typealias :wat::holon::lru::HologramCacheService::Entry
  :(wat::holon::HolonAST,wat::holon::HolonAST))

;; ─── Reply enum (arc 130) ───────────────────────────────────────
;;
;; Unified enum: Get returns GetResult carrying Vec<Option<HolonAST>>;
;; Put returns PutAck carrying unit. Both verbs share ONE reply channel
;; per slot (pair-by-index via HandlePool). Replaces the old per-verb
;; channel families (PutAck* + GetReply*).
(:wat::core::enum :wat::holon::lru::HologramCacheService::Reply
  (GetResult (results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>))
  (PutAck))

;; ─── Reply* — pair-by-index reply channel family (arc 130) ──────
;;
;; Concrete (no <V>) — HolonLRU is K=V=HolonAST. ReplyTx widens from
;; the old per-verb Senders (Sender<Vec<Option<HolonAST>>> for Get,
;; Sender<unit> for Put) to a single Sender<Reply> so Get + Put share
;; one channel per slot.
(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReplyTx
  :wat::kernel::Sender<wat::holon::lru::HologramCacheService::Reply>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReplyRx
  :wat::kernel::Receiver<wat::holon::lru::HologramCacheService::Reply>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReplyChannel
  :(wat::holon::lru::HologramCacheService::ReplyTx,wat::holon::lru::HologramCacheService::ReplyRx))

;; ─── Request enum ───────────────────────────────────────────────
;;
;; Arc 119: enum-based (Get | Put). Single-probe/key/val tagged-tuple retires.
;; Arc 130: embedded reply-tx/ack-tx removed; reply routing is by pair index.
;;   Get carries Vec<HolonAST> probes; driver replies via indexed ReplyTx.
;;   Put carries Vec<Entry> entries; driver replies PutAck via same.
(:wat::core::enum :wat::holon::lru::HologramCacheService::Request
  (Get  (probes   :wat::core::Vector<wat::holon::HolonAST>))
  (Put  (entries  :wat::core::Vector<wat::holon::lru::HologramCacheService::Entry>)))

;; ─── Per-client request channel typealiases ─────────────────────

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReqTx
  :wat::kernel::Sender<wat::holon::lru::HologramCacheService::Request>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReqRx
  :wat::kernel::Receiver<wat::holon::lru::HologramCacheService::Request>)

;; The (ReqTx, ReqRx) pair as a single name. Used by the spawn body
;; to keep nested `<>` depth tractable when iterating bounded-queue
;; pairs.
(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReqChannel
  :(wat::holon::lru::HologramCacheService::ReqTx,wat::holon::lru::HologramCacheService::ReqRx))

;; ─── Handle / DriverPair (arc 130) ──────────────────────────────
;;
;; Handle = (ReqTx, ReplyRx) — the client's view of one slot. Pop one
;; from the pool; pass to :wat::holon::lru::HologramCacheService/get
;; or /put. No per-call channel allocation.
;;
;; DriverPair = (ReqRx, ReplyTx) — the driver's view of one slot.
;; select fires at index i; driver-pairs[i].second is the ReplyTx for
;; the matching client.
(:wat::core::typealias :wat::holon::lru::HologramCacheService::Handle
  :(wat::holon::lru::HologramCacheService::ReqTx,wat::holon::lru::HologramCacheService::ReplyRx))

(:wat::core::typealias :wat::holon::lru::HologramCacheService::DriverPair
  :(wat::holon::lru::HologramCacheService::ReqRx,wat::holon::lru::HologramCacheService::ReplyTx))

;; ─── Spawn return shape ─────────────────────────────────────────
;;
;; What `:wat::holon::lru::HologramCacheService/spawn` returns: the
;; HandlePool of per-client Handles ((ReqTx, ReplyRx) pairs) + the
;; driver's Thread handle (arc 114). Caller pops N handles, finishes
;; the pool, scoped-drops at end → driver exits.
(:wat::core::typealias :wat::holon::lru::HologramCacheService::Spawn
  :(wat::kernel::HandlePool<wat::holon::lru::HologramCacheService::Handle>,wat::kernel::Thread<wat::core::nil,wat::core::nil>))

;; ─── Reporting contract — non-negotiable ───────────────────────
;;
;; Mirrors the archive's request/reply service contract (treasury-
;; program.rs's TreasuryRequest grew from SubmitPaper alone to five
;; variants as needs arose; same shape here, flipped: the SERVICE is
;; the producer, the user's Reporter fn is the consumer):
;;
;;   1. The service DECLARES the typed messages it emits via the
;;      `:wat::holon::lru::HologramCacheService::Report` enum.
;;      Producer-defined.
;;   2. The user provides a `Reporter` fn that match-dispatches the
;;      variants to whatever backend they want (sqlite, CloudWatch,
;;      stdout, /dev/null). Consumer-defined.
;;   3. Producer/consumer agree on the variant set; new variants are
;;      additive — the function signature never changes, only the
;;      match grows arms.
;;
;; Service/spawn DEMANDS two injection points:
;;
;;   1. reporter        :Reporter           — (Report) -> ()
;;   2. metrics-cadence :MetricsCadence<G>  — gate + tick fn
;;
;; The cadence gates the `Metrics` variant specifically — when it
;; fires, the service emits `(Report::Metrics stats)` and resets stats.
;; Future ungated variants (Error / Evicted / Started / Stopped) ride
;; the same Reporter callback whenever the service decides to emit.
;;
;; The user picks G (the cadence's gate type). Common shapes:
;;
;;   G = :()                   null-metrics-cadence (never fires)
;;   G = :wat::time::Instant   wall-clock rate gate via tick-gate
;;   G = :wat::core::i64                  counter-mod-N gate
;;
;; Both injection points are required. Pass null-reporter and
;; (null-metrics-cadence) for the explicit "no reporting" choice.

(:wat::core::struct :wat::holon::lru::HologramCacheService::Stats
  (lookups :wat::core::i64)        ;; total Gets in this window
  (hits :wat::core::i64)           ;; Gets returning Some
  (misses :wat::core::i64)         ;; Gets returning :None
  (puts :wat::core::i64)           ;; total Puts in this window
  (cache-size :wat::core::i64))    ;; HologramCache/len at gate-fire time

;; Report — discriminated outbound messages the cache emits.
;; Slice 1 ships ONE variant (Metrics, gated by metrics-cadence). Future
;; variants extend the enum without breaking any consumer:
;;   - lifecycle (Started, Stopped)
;;   - errors    (SendFailed, EncodeFailed)
;;   - evictions (LRUEvicted)
;; Each new variant earns its slot when the service has a concrete
;; reason to communicate it. Producer/consumer agree on the variant
;; set; consumers add an arm to their match when a new one ships.
(:wat::core::enum :wat::holon::lru::HologramCacheService::Report
  (Metrics (stats :wat::holon::lru::HologramCacheService::Stats)))

;; MetricsCadence<G> — stateful rate gate. Holds the gate state
;; (G, picked by the user) AND the tick function that advances it.
;; Service/tick-window calls (tick gate stats) → (gate', fired?), then
;; rebuilds the cadence with the new gate. The cadence threads gate
;; through the loop; the tick function itself is invariant.
(:wat::core::struct :wat::holon::lru::HologramCacheService::MetricsCadence<G>
  (gate :G)
  (tick :wat::core::Fn(G,wat::holon::lru::HologramCacheService::Stats)->(G,wat::core::bool)))

(:wat::core::typealias :wat::holon::lru::HologramCacheService::Reporter
  :wat::core::Fn(wat::holon::lru::HologramCacheService::Report)->wat::core::nil)

;; null-metrics-cadence — fresh `MetricsCadence<()>` whose tick
;; never fires. Use when metrics are a deliberate opt-out.
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/null-metrics-cadence
    -> :wat::holon::lru::HologramCacheService::MetricsCadence<wat::core::nil>)
  (:wat::holon::lru::HologramCacheService::MetricsCadence/new
    :wat::core::nil
    (:wat::core::fn
      [gate <- :wat::core::nil _stats <- :wat::holon::lru::HologramCacheService::Stats] -> :(wat::core::nil,wat::core::bool)
      (:wat::core::Tuple gate false))))

;; null-reporter — discards every Report variant.
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/null-reporter
    (_report :wat::holon::lru::HologramCacheService::Report) -> :wat::core::nil)
  :wat::core::nil)

;; Fresh zero-counters Stats. Used at startup and after each
;; gate-fire (window-rolling reset, matching the archive's
;; `stats = CacheStats::default()` after emit).
(:wat::core::define
  (:wat::holon::lru::HologramCacheService::Stats/zero
    -> :wat::holon::lru::HologramCacheService::Stats)
  (:wat::holon::lru::HologramCacheService::Stats/new 0 0 0 0 0))

;; ─── Service state — cache + running stats ─────────────────────
;;
;; Threaded through Service/loop alongside the metrics-cadence's gate.
;; The cache mutates in place (HologramCache is thread-owned mutable);
;; Stats rebuilds each iteration (values-up). Gate is independent
;; of State — caller-typed.

(:wat::core::struct :wat::holon::lru::HologramCacheService::State
  (cache :wat::holon::lru::HologramCache)
  (stats :wat::holon::lru::HologramCacheService::Stats))

;; One loop-step's outputs: the post-dispatch State paired with the
;; advanced MetricsCadence. Service/loop and Service/tick-window both
;; thread this shape; the alias caps angle-bracket density at the
;; Service layer.
(:wat::core::typealias :wat::holon::lru::HologramCacheService::Step<G>
  :(wat::holon::lru::HologramCacheService::State,wat::holon::lru::HologramCacheService::MetricsCadence<G>))

;; ─── Per-variant request handler ────────────────────────────────
;;
;; Get: batch-lookup probes via HologramCache/get; reply with
;;      Reply::GetResult on reply-tx; stats: lookups += len(probes),
;;      hits/misses counted from result vec.
;; Put: batch-insert entries via HologramCache/put; reply Reply::PutAck
;;      on reply-tx after whole batch persisted; stats: puts += len(entries).
;;      Note: HologramCache/put returns :unit (not Option eviction).
;;
;; Returns the new State (cache pointer unchanged — mutates in
;; place; stats rebuilt).
;;
;; Arc 130: reply-tx is supplied by the driver loop (looked up via the
;; pair index from select). The Request no longer carries it.

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/handle
    (req :wat::holon::lru::HologramCacheService::Request)
    (reply-tx :wat::holon::lru::HologramCacheService::ReplyTx)
    (state :wat::holon::lru::HologramCacheService::State)
    -> :wat::holon::lru::HologramCacheService::State)
  (:wat::core::let
    [cache
      (:wat::holon::lru::HologramCacheService::State/cache state)
     stats
      (:wat::holon::lru::HologramCacheService::State/stats state)]
    (:wat::core::match req -> :wat::holon::lru::HologramCacheService::State
      ((:wat::holon::lru::HologramCacheService::Request::Get probes)
        (:wat::core::let
          [results
            (:wat::core::map probes
              (:wat::core::fn [probe <- :wat::holon::HolonAST] -> :wat::core::Option<wat::holon::HolonAST>
                (:wat::holon::lru::HologramCache/get cache probe)))
           hit-count
            (:wat::list::reduce results 0
              (:wat::core::fn
                [acc <- :wat::core::i64 slot <- :wat::core::Option<wat::holon::HolonAST>] -> :wat::core::i64
                (:wat::core::match slot -> :wat::core::i64
                  ((:wat::core::Some _) (:wat::core::i64::+,2 acc 1))
                  (:wat::core::None acc))))
           n (:wat::core::Vector/length probes)
           miss-count (:wat::core::i64::-,2 n hit-count)
           ;; Arc 110: in-memory peer-death is catastrophic; panic with a
           ;; meaningful message rather than silently dropping the reply.
           ;; Arc 130: send Reply::GetResult variant on the slot's reply-tx.
           _send
            (:wat::core::Result/expect -> :wat::core::nil
              (:wat::kernel::send reply-tx (:wat::holon::lru::HologramCacheService::Reply::GetResult results))
              "HologramCacheService/handle: reply-tx disconnected — client died mid-request?")
           stats'
            (:wat::holon::lru::HologramCacheService::Stats/new
              (:wat::core::i64::+,2 (:wat::holon::lru::HologramCacheService::Stats/lookups stats) n)
              (:wat::core::i64::+,2 (:wat::holon::lru::HologramCacheService::Stats/hits stats) hit-count)
              (:wat::core::i64::+,2 (:wat::holon::lru::HologramCacheService::Stats/misses stats) miss-count)
              (:wat::holon::lru::HologramCacheService::Stats/puts stats)
              (:wat::holon::lru::HologramCacheService::Stats/cache-size stats))]
          (:wat::holon::lru::HologramCacheService::State/new cache stats')))
      ((:wat::holon::lru::HologramCacheService::Request::Put entries)
        (:wat::core::let
          [;; HologramCache/put returns :unit (not Option eviction).
           ;; Map entries, discard results (all units).
           _
            (:wat::core::map entries
              (:wat::core::fn
                [entry <- :wat::holon::lru::HologramCacheService::Entry] -> :wat::core::nil
                (:wat::core::let
                  [k (:wat::core::first entry)
                   v (:wat::core::second entry)]
                  (:wat::holon::lru::HologramCache/put cache k v))))
           n (:wat::core::Vector/length entries)
           ;; Arc 110: same discipline — driver dying mid-protocol is
           ;; catastrophic; panic with a meaningful message.
           ;; Arc 130: send Reply::PutAck variant on the slot's reply-tx.
           _send
            (:wat::core::Result/expect -> :wat::core::nil
              (:wat::kernel::send reply-tx (:wat::holon::lru::HologramCacheService::Reply::PutAck))
              "HologramCacheService/handle: reply-tx disconnected — client died mid-request?")
           stats'
            (:wat::holon::lru::HologramCacheService::Stats/new
              (:wat::holon::lru::HologramCacheService::Stats/lookups stats)
              (:wat::holon::lru::HologramCacheService::Stats/hits stats)
              (:wat::holon::lru::HologramCacheService::Stats/misses stats)
              (:wat::core::i64::+,2 (:wat::holon::lru::HologramCacheService::Stats/puts stats) n)
              (:wat::holon::lru::HologramCacheService::Stats/cache-size stats))]
          (:wat::holon::lru::HologramCacheService::State/new cache stats'))))))

;; ─── Tick the metrics window — advance gate, emit+reset on fire ──
;;
;; Always: pull stats from State, tick the cadence (gate → gate'),
;; rebuild the cadence struct with the advanced gate. The cadence
;; never freezes; every call moves it forward.
;;
;; On fire: stamp cache-size onto the stats, send
;; `(Report::Metrics final-stats)` through the reporter, reset the
;; running stats. Returns the post-emit State + advanced cadence.
;;
;; On no-fire: state unchanged, cadence advanced. The window stays
;; open; counters keep accumulating.

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/tick-window<G>
    (state :wat::holon::lru::HologramCacheService::State)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :wat::holon::lru::HologramCacheService::Step<G>)
  (:wat::core::let
    [stats
      (:wat::holon::lru::HologramCacheService::State/stats state)
     gate
      (:wat::holon::lru::HologramCacheService::MetricsCadence/gate metrics-cadence)
     tick-fn
      (:wat::holon::lru::HologramCacheService::MetricsCadence/tick metrics-cadence)
     tick (tick-fn gate stats)
     gate' (:wat::core::first tick)
     fired (:wat::core::second tick)
     cadence'
      (:wat::holon::lru::HologramCacheService::MetricsCadence/new gate' tick-fn)]
    (:wat::core::if fired -> :wat::holon::lru::HologramCacheService::Step<G>
      (:wat::core::let
        [cache
          (:wat::holon::lru::HologramCacheService::State/cache state)
         final-stats
          (:wat::holon::lru::HologramCacheService::Stats/new
            (:wat::holon::lru::HologramCacheService::Stats/lookups stats)
            (:wat::holon::lru::HologramCacheService::Stats/hits stats)
            (:wat::holon::lru::HologramCacheService::Stats/misses stats)
            (:wat::holon::lru::HologramCacheService::Stats/puts stats)
            (:wat::holon::lru::HologramCache/len cache))
         _ (reporter (:wat::holon::lru::HologramCacheService::Report::Metrics final-stats))
         state'
          (:wat::holon::lru::HologramCacheService::State/new
            cache (:wat::holon::lru::HologramCacheService::Stats/zero))]
        (:wat::core::Tuple state' cadence'))
      (:wat::core::Tuple state cadence'))))

;; --- Helper — dispatch req to handle + send Reply on pairs[idx].second ---
;;
;; Lifted out of loop-step to keep loop-step's outer let one-let-deep
;; per `feedback_simple_forms_per_func`. Looks up the DriverPair at idx,
;; extracts the ReplyTx, calls handle (which sends the reply on reply-tx),
;; ticks the window, recurses.
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/reply-at<G>
    (driver-pairs :wat::core::Vector<wat::holon::lru::HologramCacheService::DriverPair>)
    (idx :wat::core::i64)
    (req :wat::holon::lru::HologramCacheService::Request)
    (state :wat::holon::lru::HologramCacheService::State)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :wat::core::nil)
  (:wat::core::match (:wat::core::get driver-pairs idx) -> :wat::core::nil
    ((:wat::core::Some pair)
      (:wat::core::let
        [reply-tx
          (:wat::core::second pair)
         after-handle
          (:wat::holon::lru::HologramCacheService/handle req reply-tx state)
         step
          (:wat::holon::lru::HologramCacheService/tick-window
            after-handle reporter metrics-cadence)
         next-state
          (:wat::core::first step)
         cadence'
          (:wat::core::second step)]
        (:wat::holon::lru::HologramCacheService/loop-step
          next-state driver-pairs reporter cadence')))
    (:wat::core::None :wat::core::nil)))

;; ─── Driver entry — allocates the cache INSIDE the driver thread ──
;;
;; HologramCache's underlying LocalCache is thread-owned (lives in a
;; ThreadOwnedCell), so the cache MUST stay on the worker thread.
;; Allocate inside the driver fn; delegate to loop-step.
;;
;; Arc 130: takes driver-pairs Vec<DriverPair> instead of bare req-rxs.
;; The driver uses the pair index to locate the matching ReplyTx after
;; select fires.
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/loop<G>
    (cap :wat::core::i64)
    (driver-pairs :wat::core::Vector<wat::holon::lru::HologramCacheService::DriverPair>)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :wat::core::nil)
  (:wat::core::let
    [cache
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        cap)
     initial
      (:wat::holon::lru::HologramCacheService::State/new
        cache (:wat::holon::lru::HologramCacheService::Stats/zero))]
    (:wat::holon::lru::HologramCacheService/loop-step
      initial driver-pairs reporter metrics-cadence)))

;; Recursive inner loop. Owns the cache for the duration of the driver
;; thread's lifetime; select across request receivers (projected from
;; driver-pairs); index i → driver-pairs[i].second is the ReplyTx for
;; routing. After every dispatch, tick the metrics window.
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/loop-step<G>
    (state :wat::holon::lru::HologramCacheService::State)
    (driver-pairs :wat::core::Vector<wat::holon::lru::HologramCacheService::DriverPair>)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :wat::core::nil)
  (:wat::core::if (:wat::core::empty? driver-pairs) -> :wat::core::nil
    :wat::core::nil
    (:wat::core::let
      [req-rxs
        (:wat::core::map driver-pairs
          (:wat::core::fn
            [p <- :wat::holon::lru::HologramCacheService::DriverPair] -> :wat::holon::lru::HologramCacheService::ReqRx
            (:wat::core::first p)))
       chosen
        (:wat::kernel::select req-rxs)
       idx (:wat::core::first chosen)
       maybe
        (:wat::core::second chosen)]
      (:wat::core::match maybe -> :wat::core::nil
        ((:wat::core::Ok (:wat::core::Some req))
          (:wat::holon::lru::HologramCacheService/reply-at driver-pairs idx req state reporter metrics-cadence))
        ((:wat::core::Ok :wat::core::None)
          (:wat::holon::lru::HologramCacheService/loop-step
            state
            (:wat::std::list::remove-at driver-pairs idx)
            reporter metrics-cadence))
        ((:wat::core::Err _died) :wat::core::nil)))))

;; ─── Client helpers ──────────────────────────────────────────
;;
;; Arc 130: helper verbs take a single Handle (pair-by-index).
;; No per-call channel allocation. The channels are pre-allocated by
;; spawn and owned by the Handle; the driver holds the matching
;; DriverPair indexed the same way.
;;
;; Arc 119: get takes Vec<HolonAST> probes, returns Vec<Option<HolonAST>>.
;;          put takes Vec<Entry>, returns unit after PutAck.
;;
;; Recv pattern (two nested levels per arc 111+113):
;;   Result/expect unwraps the outer Result (ThreadDiedError on peer death).
;;   Option/expect unwraps the inner Option (None = clean channel close).

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/get
    (handle :wat::holon::lru::HologramCacheService::Handle)
    (probes :wat::core::Vector<wat::holon::HolonAST>)
    -> :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
  (:wat::core::let
    [req-tx
      (:wat::core::first handle)
     reply-rx
      (:wat::core::second handle)
     ;; Arc 110: in-memory peer-death is catastrophic; cache driver
     ;; dying means our state-of-the-world claim is invalid. Panic
     ;; with a meaningful message rather than silently returning
     ;; :None and pretending we got a "miss."
     _send
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send req-tx (:wat::holon::lru::HologramCacheService::Request::Get probes))
        "HologramCacheService/get: req-tx disconnected — driver died?")
     reply
      (:wat::core::Option/expect -> :wat::holon::lru::HologramCacheService::Reply
        (:wat::core::Result/expect -> :wat::core::Option<wat::holon::lru::HologramCacheService::Reply>
          (:wat::kernel::recv reply-rx)
          "HologramCacheService/get: reply-rx disconnected — driver died mid-request?")
        "HologramCacheService/get: reply channel closed — driver dropped reply-tx?")]
    (:wat::core::match reply -> :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>
      ((:wat::holon::lru::HologramCacheService::Reply::GetResult results) results)
      ((:wat::holon::lru::HologramCacheService::Reply::PutAck)
        (:wat::core::panic! "HologramCacheService/get: driver sent PutAck on Get reply channel")))))

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/put
    (handle :wat::holon::lru::HologramCacheService::Handle)
    (entries :wat::core::Vector<wat::holon::lru::HologramCacheService::Entry>)
    -> :wat::core::nil)
  (:wat::core::let
    [req-tx
      (:wat::core::first handle)
     reply-rx
      (:wat::core::second handle)
     ;; Arc 110: same as HologramCacheService/get — driver dying mid-protocol
     ;; is catastrophic; panic with a meaningful message rather than
     ;; silently absorbing the disconnect.
     _send
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send req-tx (:wat::holon::lru::HologramCacheService::Request::Put entries))
        "HologramCacheService/put: req-tx disconnected — driver died?")
     reply
      (:wat::core::Option/expect -> :wat::holon::lru::HologramCacheService::Reply
        (:wat::core::Result/expect -> :wat::core::Option<wat::holon::lru::HologramCacheService::Reply>
          (:wat::kernel::recv reply-rx)
          "HologramCacheService/put: reply-rx disconnected — driver died mid-request?")
        "HologramCacheService/put: reply channel closed — driver dropped reply-tx?")]
    (:wat::core::match reply -> :wat::core::nil
      ((:wat::holon::lru::HologramCacheService::Reply::PutAck) :wat::core::nil)
      ((:wat::holon::lru::HologramCacheService::Reply::GetResult _)
        (:wat::core::panic! "HologramCacheService/put: driver sent GetResult on Put reply channel")))))

;; ─── Service/spawn — the constructor ─────────────────────────
;;
;; Arc 130: Creates N bounded(1) request queues + N bounded(1) reply
;; queues in lock-step. The index of the request pair matches the index
;; of the reply pair — this is what makes pair-by-index reply routing
;; possible inside loop-step. Builds N Handle tuples (client's view =
;; (ReqTx, ReplyRx)) and N DriverPair tuples (driver's view =
;; (ReqRx, ReplyTx)).
;;
;; Both reporter + metrics-cadence are required; pass
;; :wat::holon::lru::HologramCacheService/null-reporter and
;; (:wat::holon::lru::HologramCacheService/null-metrics-cadence) for the
;; explicit "no reporting" choice. See CONVENTIONS.md "Service contract".

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/spawn<G>
    (count :wat::core::i64)
    (cap :wat::core::i64)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :wat::holon::lru::HologramCacheService::Spawn)
  (:wat::core::let
    ;; N request pairs and N reply pairs in lock-step. The pair index
    ;; is preserved so Handle[i] and DriverPair[i] correspond to the
    ;; same slot.
    [req-pairs
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::fn [_i <- :wat::core::i64] -> :wat::holon::lru::HologramCacheService::ReqChannel
          (:wat::kernel::make-bounded-channel :wat::holon::lru::HologramCacheService::Request 1)))
     reply-pairs
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::fn [_i <- :wat::core::i64] -> :wat::holon::lru::HologramCacheService::ReplyChannel
          (:wat::kernel::make-bounded-channel :wat::holon::lru::HologramCacheService::Reply 1)))
     ;; Client-side: Handle = (ReqTx, ReplyRx).
     handles
      (:wat::std::list::zip
        (:wat::core::map req-pairs
          (:wat::core::fn [p <- :wat::holon::lru::HologramCacheService::ReqChannel] -> :wat::holon::lru::HologramCacheService::ReqTx
            (:wat::core::first p)))
        (:wat::core::map reply-pairs
          (:wat::core::fn [p <- :wat::holon::lru::HologramCacheService::ReplyChannel] -> :wat::holon::lru::HologramCacheService::ReplyRx
            (:wat::core::second p))))
     ;; Driver-side: DriverPair = (ReqRx, ReplyTx) at matching index.
     driver-pairs
      (:wat::std::list::zip
        (:wat::core::map req-pairs
          (:wat::core::fn [p <- :wat::holon::lru::HologramCacheService::ReqChannel] -> :wat::holon::lru::HologramCacheService::ReqRx
            (:wat::core::second p)))
        (:wat::core::map reply-pairs
          (:wat::core::fn [p <- :wat::holon::lru::HologramCacheService::ReplyChannel] -> :wat::holon::lru::HologramCacheService::ReplyTx
            (:wat::core::first p))))
     pool
      (:wat::kernel::HandlePool::new "hologram-cache-service" handles)
     driver
      (:wat::kernel::spawn-thread
        (:wat::core::fn
          [_in <- :rust::crossbeam_channel::Receiver<wat::core::nil>
           _out <- :rust::crossbeam_channel::Sender<wat::core::nil>]
           -> :wat::core::nil
          (:wat::holon::lru::HologramCacheService/loop
            cap driver-pairs reporter metrics-cadence)))]
    (:wat::core::Tuple pool driver)))
