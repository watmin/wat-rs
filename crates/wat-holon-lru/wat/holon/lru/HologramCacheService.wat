;; :wat::holon::lru::HologramCacheService — queue-addressed wrapper for
;; HologramCache. A long-running spawned program that owns a cache
;; instance and serves requests via a request queue. Each client gets
;; a per-client reply channel for Get; Put is fire-and-forget.
;;
;; Arc 078: ported from the lab's :trading::cache::Service. Nothing
;; in the cache service shape is trader-specific — the Request enum,
;; the Reporter contract, the cadence-gated metrics — all of it is
;; generic substrate machinery built atop HologramCache. The trader
;; (and any other consumer) merely USES it.
;;
;; The Reporter + MetricsCadence + null-* + typed Report enum pattern
;; documented here is the canonical service-contract idiom for
;; queue-addressed substrate services. Future stdlib services follow
;; this shape; see CONVENTIONS.md "Service contract" section.
;;
;; Surface:
;;   - Request: Get(probe, reply-tx) | Put(key, val)
;;   - State:   HologramCache + Stats (cache + per-window counters)
;;   - Reply:   Option<HolonAST> sent on reply-tx
;;   - Telemetry: caller-supplied (reporter, metrics-cadence) pair.
;;     Both are non-negotiable: caller must pass both. Pass
;;     :wat::holon::lru::HologramCacheService/null-reporter and
;;     (:wat::holon::lru::HologramCacheService/null-metrics-cadence)
;;     for the explicit "no reporting" choice.
;;
;; Pattern mirrors archive's programs/stdlib/cache.rs::cache(can_emit,
;; emit) — same callback-injection idea, lifted to wat's stateful-
;; values-up shape: the cadence's tick is `:fn(G, Stats) -> :(G, bool)`
;; so the user threads time / counters / whatever through the loop
;; without reaching for Mutex.
;;
;; Arc 076 + 077: slot routing inferred from the form's structure
;; (the substrate does it inside HologramCache); no caller-supplied
;; pos. Filter is bound at HologramCache/make time.

;; ─── Reply channel typealiases ──────────────────────────────────

(:wat::core::typealias :wat::holon::lru::HologramCacheService::GetReplyTx
  :wat::kernel::QueueSender<Option<wat::holon::HolonAST>>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::GetReplyRx
  :wat::kernel::QueueReceiver<Option<wat::holon::HolonAST>>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::GetReplyPair
  :wat::kernel::QueuePair<Option<wat::holon::HolonAST>>)

;; ─── Request enum ───────────────────────────────────────────────

(:wat::core::enum :wat::holon::lru::HologramCacheService::Request
  (Get
    (probe :wat::holon::HolonAST)
    (reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx))
  (Put
    (key :wat::holon::HolonAST)
    (val :wat::holon::HolonAST)))

;; ─── Per-client channel typealiases ─────────────────────────────

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReqTx
  :wat::kernel::QueueSender<wat::holon::lru::HologramCacheService::Request>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReqRx
  :wat::kernel::QueueReceiver<wat::holon::lru::HologramCacheService::Request>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReqTxPool
  :wat::kernel::HandlePool<wat::holon::lru::HologramCacheService::ReqTx>)

(:wat::core::typealias :wat::holon::lru::HologramCacheService::Spawn
  :(wat::holon::lru::HologramCacheService::ReqTxPool,wat::kernel::ProgramHandle<()>))

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
  (tick :fn(G,wat::holon::lru::HologramCacheService::Stats)->(G,bool)))

(:wat::core::typealias :wat::holon::lru::HologramCacheService::Reporter
  :fn(wat::holon::lru::HologramCacheService::Report)->())

;; null-metrics-cadence — fresh `MetricsCadence<()>` whose tick
;; never fires. Use when metrics are a deliberate opt-out.
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/null-metrics-cadence
    -> :wat::holon::lru::HologramCacheService::MetricsCadence<()>)
  (:wat::holon::lru::HologramCacheService::MetricsCadence/new
    ()
    (:wat::core::lambda
      ((gate :()) (_stats :wat::holon::lru::HologramCacheService::Stats) -> :((),bool))
      (:wat::core::tuple gate false))))

;; null-reporter — discards every Report variant.
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/null-reporter
    (_report :wat::holon::lru::HologramCacheService::Report) -> :())
  ())

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
;; Get: filtered-argmax via HologramCache/get; send Option<AST> on
;;      reply-tx. Stats: lookups++, then hits++ or misses++.
;; Put: insert into HologramCache; no reply. Stats: puts++.
;;
;; Returns the new State (cache pointer unchanged — mutates in
;; place; stats rebuilt).

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/handle
    (req :wat::holon::lru::HologramCacheService::Request)
    (state :wat::holon::lru::HologramCacheService::State)
    -> :wat::holon::lru::HologramCacheService::State)
  (:wat::core::let*
    (((cache :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCacheService::State/cache state))
     ((stats :wat::holon::lru::HologramCacheService::Stats)
      (:wat::holon::lru::HologramCacheService::State/stats state)))
    (:wat::core::match req -> :wat::holon::lru::HologramCacheService::State
      ((:wat::holon::lru::HologramCacheService::Request::Get probe reply-tx)
        (:wat::core::let*
          (((result :Option<wat::holon::HolonAST>)
            (:wat::holon::lru::HologramCache/get cache probe))
           ((_send :())
            (:wat::core::result::expect -> :()
              (:wat::kernel::send reply-tx result)
              "HologramCacheService::Request::Get reply-tx disconnected — caller died?"))
           ((hit-delta :wat::core::i64)
            (:wat::core::match result -> :wat::core::i64
              ((Some _) 1)
              (:None 0)))
           ((miss-delta :wat::core::i64)
            (:wat::core::i64::- 1 hit-delta))
           ((stats' :wat::holon::lru::HologramCacheService::Stats)
            (:wat::holon::lru::HologramCacheService::Stats/new
              (:wat::core::i64::+ (:wat::holon::lru::HologramCacheService::Stats/lookups stats) 1)
              (:wat::core::i64::+ (:wat::holon::lru::HologramCacheService::Stats/hits stats) hit-delta)
              (:wat::core::i64::+ (:wat::holon::lru::HologramCacheService::Stats/misses stats) miss-delta)
              (:wat::holon::lru::HologramCacheService::Stats/puts stats)
              (:wat::holon::lru::HologramCacheService::Stats/cache-size stats))))
          (:wat::holon::lru::HologramCacheService::State/new cache stats')))
      ((:wat::holon::lru::HologramCacheService::Request::Put key val)
        (:wat::core::let*
          (((_ :()) (:wat::holon::lru::HologramCache/put cache key val))
           ((stats' :wat::holon::lru::HologramCacheService::Stats)
            (:wat::holon::lru::HologramCacheService::Stats/new
              (:wat::holon::lru::HologramCacheService::Stats/lookups stats)
              (:wat::holon::lru::HologramCacheService::Stats/hits stats)
              (:wat::holon::lru::HologramCacheService::Stats/misses stats)
              (:wat::core::i64::+ (:wat::holon::lru::HologramCacheService::Stats/puts stats) 1)
              (:wat::holon::lru::HologramCacheService::Stats/cache-size stats))))
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
  (:wat::core::let*
    (((stats :wat::holon::lru::HologramCacheService::Stats)
      (:wat::holon::lru::HologramCacheService::State/stats state))
     ((gate :G)
      (:wat::holon::lru::HologramCacheService::MetricsCadence/gate metrics-cadence))
     ((tick-fn :fn(G,wat::holon::lru::HologramCacheService::Stats)->(G,bool))
      (:wat::holon::lru::HologramCacheService::MetricsCadence/tick metrics-cadence))
     ((tick :(G,bool)) (tick-fn gate stats))
     ((gate' :G) (:wat::core::first tick))
     ((fired :wat::core::bool) (:wat::core::second tick))
     ((cadence' :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
      (:wat::holon::lru::HologramCacheService::MetricsCadence/new gate' tick-fn)))
    (:wat::core::if fired -> :wat::holon::lru::HologramCacheService::Step<G>
      (:wat::core::let*
        (((cache :wat::holon::lru::HologramCache)
          (:wat::holon::lru::HologramCacheService::State/cache state))
         ((final-stats :wat::holon::lru::HologramCacheService::Stats)
          (:wat::holon::lru::HologramCacheService::Stats/new
            (:wat::holon::lru::HologramCacheService::Stats/lookups stats)
            (:wat::holon::lru::HologramCacheService::Stats/hits stats)
            (:wat::holon::lru::HologramCacheService::Stats/misses stats)
            (:wat::holon::lru::HologramCacheService::Stats/puts stats)
            (:wat::holon::lru::HologramCache/len cache)))
         ((_ :()) (reporter (:wat::holon::lru::HologramCacheService::Report::Metrics final-stats)))
         ((state' :wat::holon::lru::HologramCacheService::State)
          (:wat::holon::lru::HologramCacheService::State/new
            cache (:wat::holon::lru::HologramCacheService::Stats/zero))))
        (:wat::core::tuple state' cadence'))
      (:wat::core::tuple state cadence'))))

;; ─── Driver loop — select + dispatch + gate-check ──────────────
;;
;; Empty rxs → exit with final state. Otherwise: select; on Some(req)
;; dispatch + tick-window + recurse; on :None prune the closed channel
;; and recurse. The cadence's gate updates each iteration via
;; MetricsCadence/new with the new gate value; the tick function
;; itself is invariant across the loop.

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/loop<G>
    (req-rxs :Vec<wat::holon::lru::HologramCacheService::ReqRx>)
    (state :wat::holon::lru::HologramCacheService::State)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :wat::holon::lru::HologramCacheService::State)
  (:wat::core::if (:wat::core::empty? req-rxs)
    -> :wat::holon::lru::HologramCacheService::State
    state
    (:wat::core::let*
      (((chosen :wat::kernel::Chosen<wat::holon::lru::HologramCacheService::Request>)
        (:wat::kernel::select req-rxs))
       ((idx :wat::core::i64) (:wat::core::first chosen))
       ((maybe :wat::kernel::CommResult<wat::holon::lru::HologramCacheService::Request>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :wat::holon::lru::HologramCacheService::State
        ((Ok (Some req))
          (:wat::core::let*
            (((after-handle :wat::holon::lru::HologramCacheService::State)
              (:wat::holon::lru::HologramCacheService/handle req state))
             ((step :wat::holon::lru::HologramCacheService::Step<G>)
              (:wat::holon::lru::HologramCacheService/tick-window
                after-handle reporter metrics-cadence))
             ((next-state :wat::holon::lru::HologramCacheService::State)
              (:wat::core::first step))
             ((cadence' :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
              (:wat::core::second step)))
            (:wat::holon::lru::HologramCacheService/loop
              req-rxs next-state reporter cadence')))
        ((Ok :None)
          (:wat::holon::lru::HologramCacheService/loop
            (:wat::std::list::remove-at req-rxs idx)
            state reporter metrics-cadence))
        ((Err _died) state)))))

;; ─── Worker entry — owns the cache for its full lifetime ──────
;;
;; HologramCache's underlying LocalCache is thread-owned (lives in a
;; ThreadOwnedCell), so the cache MUST stay on the worker thread.
;; Service/run wraps Service/loop so the spawned handle resolves
;; to :() — caller-friendly type.

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/run<G>
    (req-rxs :Vec<wat::holon::lru::HologramCacheService::ReqRx>)
    (cap :wat::core::i64)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :())
  (:wat::core::let*
    (((cache :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        cap))
     ((initial :wat::holon::lru::HologramCacheService::State)
      (:wat::holon::lru::HologramCacheService::State/new
        cache (:wat::holon::lru::HologramCacheService::Stats/zero)))
     ((_final :wat::holon::lru::HologramCacheService::State)
      (:wat::holon::lru::HologramCacheService/loop
        req-rxs initial reporter metrics-cadence)))
    ()))

;; ─── Service/spawn — the constructor ─────────────────────────
;;
;; Build N bounded request channels (capacity 1 each — back-pressure
;; under load), pool the senders (HandlePool's orphan detector
;; surfaces over/under-claim at finish), spawn the driver with a
;; fresh HologramCache and the user-supplied (reporter, metrics-cadence)
;; pair.
;;
;; Both injection points are non-negotiable. Pass
;; :wat::holon::lru::HologramCacheService/null-reporter and
;; (:wat::holon::lru::HologramCacheService/null-metrics-cadence) for
;; the explicit "no reporting" choice; pass real values for real
;; reporting (e.g., a Reporter that match-dispatches Report variants
;; to sqlite / CloudWatch + a tick-gate-shaped MetricsCadence with an
;; Instant gate).

(:wat::core::define
  (:wat::holon::lru::HologramCacheService/spawn<G>
    (count :wat::core::i64)
    (cap :wat::core::i64)
    (reporter :wat::holon::lru::HologramCacheService::Reporter)
    (metrics-cadence :wat::holon::lru::HologramCacheService::MetricsCadence<G>)
    -> :wat::holon::lru::HologramCacheService::Spawn)
  (:wat::core::let*
    (((pairs :Vec<wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :wat::core::i64)
           -> :wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>)
          (:wat::kernel::make-bounded-queue
            :wat::holon::lru::HologramCacheService::Request 1))))
     ((req-txs :Vec<wat::holon::lru::HologramCacheService::ReqTx>)
      (:wat::core::map pairs
        (:wat::core::lambda
          ((p :wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>)
           -> :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::core::first p))))
     ((req-rxs :Vec<wat::holon::lru::HologramCacheService::ReqRx>)
      (:wat::core::map pairs
        (:wat::core::lambda
          ((p :wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>)
           -> :wat::holon::lru::HologramCacheService::ReqRx)
          (:wat::core::second p))))
     ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
      (:wat::kernel::HandlePool::new "hologram-cache-service" req-txs))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::holon::lru::HologramCacheService/run
        req-rxs cap reporter metrics-cadence)))
    (:wat::core::tuple pool driver)))
