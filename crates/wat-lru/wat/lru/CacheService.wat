;; :wat::lru::CacheService — wat-lru's multi-client LRU
;; service program.
;;
;; Repathed from wat-rs's former :wat::std::service::Cache when arc
;; 013 externalized this crate (slice 4b). A program that owns its
;; own LocalCache<K,V> behind a select loop; clients send requests
;; with their own reply address attached so the driver routes
;; responses without a sender-index map.
;;
;; Generic over K,V — type params propagate through every define via
;; wat's `<K,V>` declaration syntax, same pattern LocalCache uses.
;; Runtime storage is canonical-string-keyed per LocalCache/HashMap
;; convention; K,V are phantom at the type-check layer.
;;
;; Arc 078: ships the canonical service contract — Reporter +
;; MetricsCadence<G> + null-helpers + typed Report enum — alongside
;; the pre-existing tuple Request protocol. spawn now demands both
;; injection points; pass null-reporter / null-metrics-cadence for
;; the explicit "no reporting" choice.
;;
;; Protocol:
;;   Body<K,V>     = (tag :wat::core::i64, key :K, put-val :Option<V>)
;;   ReplyTx<V>    = :Sender<Option<V>>
;;   Request<K,V>  = (Body<K,V>, ReplyTx<V>)
;;   Response<V>   = :Option<V>
;;     body.tag 0 = GET: put-val is :None
;;     body.tag 1 = PUT: put-val is (Some v)
;;     Response:   (Some v) on GET hit, :None on GET miss, :None on PUT ack.
;;
;; The four parts above are typealiases declared below. Under the
;; user-composed stdlib tier (:user::wat::std::*), register_defines
;; applies the reserved-prefix gate — but :user::* is not reserved,
;; so these land cleanly through the normal user-pipeline path.

;; crossbeam_channel is wat substrate, not a wat-lru dep — the
;; runtime provides Sender<T>/Receiver<T> via :wat::kernel::
;; primitives (make-bounded-queue, etc.). `use!` declares intent
;; to consume an *external* Rust crate (a #[wat_dispatch]'d
;; library); substrate types don't need it. Only :rust::lru::LruCache
;; — the real external dep — gets a `use!` (see lru.wat).

;; --- Protocol typealiases ---
(:wat::core::typealias :wat::lru::CacheService::Body<K,V>
  :(i64,K,Option<V>))
(:wat::core::typealias :wat::lru::CacheService::ReplyTx<V>
  :wat::kernel::QueueSender<Option<V>>)
(:wat::core::typealias :wat::lru::CacheService::Request<K,V>
  :(wat::lru::CacheService::Body<K,V>,wat::lru::CacheService::ReplyTx<V>))
(:wat::core::typealias :wat::lru::CacheService::ReqTx<K,V>
  :wat::kernel::QueueSender<wat::lru::CacheService::Request<K,V>>)
(:wat::core::typealias :wat::lru::CacheService::ReqRx<K,V>
  :wat::kernel::QueueReceiver<wat::lru::CacheService::Request<K,V>>)

;; The (ReqTx, ReqRx) pair as a single name. Used by the spawn body
;; to keep nested `<>` depth tractable when iterating bounded-queue
;; pairs.
(:wat::core::typealias :wat::lru::CacheService::ReqPair<K,V>
  :(wat::lru::CacheService::ReqTx<K,V>,wat::lru::CacheService::ReqRx<K,V>))

;; --- Spawn return shape ---
;;
;; What `:wat::lru::CacheService/spawn` returns: the HandlePool of
;; client request senders + the driver's ProgramHandle. Caller pops
;; N senders, finishes the pool, scoped-drops at end → driver exits.
(:wat::core::typealias :wat::lru::CacheService::Spawn<K,V>
  :(wat::kernel::HandlePool<wat::lru::CacheService::ReqTx<K,V>>,wat::kernel::ProgramHandle<()>))

;; ─── Reporting contract — non-negotiable ───────────────────────
;;
;; Same shape as :wat::holon::lru::HologramCacheService — the
;; canonical wat substrate service contract per arc 078. The user
;; passes a Reporter (consumer-defined match-dispatching fn over
;; Report variants) and a MetricsCadence (gate + tick) at spawn time.
;; Both are required; pass null-reporter / null-metrics-cadence for
;; the explicit "no reporting" choice.

(:wat::core::struct :wat::lru::CacheService::Stats
  (lookups :wat::core::i64)        ;; total Gets in this window
  (hits :wat::core::i64)           ;; Gets returning Some
  (misses :wat::core::i64)         ;; Gets returning :None
  (puts :wat::core::i64)           ;; total Puts in this window
  (cache-size :wat::core::i64))    ;; LocalCache::len at gate-fire time

;; Slice 4 ships ONE variant (Metrics, gated by metrics-cadence).
;; Future variants (lifecycle, errors, evictions) extend additively
;; without breaking consumers — same grow-by-arms pattern as the
;; archive's TreasuryRequest.
(:wat::core::enum :wat::lru::CacheService::Report
  (Metrics (stats :wat::lru::CacheService::Stats)))

;; MetricsCadence<G> — stateful rate gate. The user picks G; the
;; cache threads the gate through each loop iteration via
;; MetricsCadence/new with the advanced gate; the tick fn itself is
;; invariant.
(:wat::core::struct :wat::lru::CacheService::MetricsCadence<G>
  (gate :G)
  (tick :fn(G,wat::lru::CacheService::Stats)->(G,bool)))

(:wat::core::typealias :wat::lru::CacheService::Reporter
  :fn(wat::lru::CacheService::Report)->())

;; null-metrics-cadence — fresh `MetricsCadence<()>` whose tick
;; never fires. Use when metrics are a deliberate opt-out.
(:wat::core::define
  (:wat::lru::CacheService/null-metrics-cadence
    -> :wat::lru::CacheService::MetricsCadence<()>)
  (:wat::lru::CacheService::MetricsCadence/new
    ()
    (:wat::core::lambda
      ((gate :()) (_stats :wat::lru::CacheService::Stats) -> :((),bool))
      (:wat::core::tuple gate false))))

;; null-reporter — discards every Report variant.
(:wat::core::define
  (:wat::lru::CacheService/null-reporter
    (_report :wat::lru::CacheService::Report) -> :())
  ())

;; Fresh zero-counters Stats. Used at startup and after each
;; gate-fire (window-rolling reset).
(:wat::core::define
  (:wat::lru::CacheService::Stats/zero -> :wat::lru::CacheService::Stats)
  (:wat::lru::CacheService::Stats/new 0 0 0 0 0))

;; ─── Service state — cache + running stats ─────────────────────
;;
;; Threaded through CacheService/loop-step alongside the cadence's
;; gate. The cache mutates in place (LocalCache is thread-owned
;; mutable); Stats rebuilds each iteration (values-up).

(:wat::core::struct :wat::lru::CacheService::State<K,V>
  (cache :wat::lru::LocalCache<K,V>)
  (stats :wat::lru::CacheService::Stats))

;; One loop-step's outputs: the post-dispatch State paired with the
;; advanced MetricsCadence. tick-window and loop-step both thread
;; this shape.
(:wat::core::typealias :wat::lru::CacheService::Step<K,V,G>
  :(wat::lru::CacheService::State<K,V>,wat::lru::CacheService::MetricsCadence<G>))

;; ─── Per-variant request handler ────────────────────────────────
;;
;; GET: LocalCache::get; reply with Option<V>; stats: lookups++,
;;      hits++ or misses++.
;; PUT: LocalCache::put; reply :None; stats: puts++.
;;
;; Returns the new State (cache pointer unchanged — mutates in
;; place; stats rebuilt).

(:wat::core::define
  (:wat::lru::CacheService/handle<K,V>
    (req :wat::lru::CacheService::Request<K,V>)
    (state :wat::lru::CacheService::State<K,V>)
    -> :wat::lru::CacheService::State<K,V>)
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<K,V>)
      (:wat::lru::CacheService::State/cache state))
     ((stats :wat::lru::CacheService::Stats)
      (:wat::lru::CacheService::State/stats state))
     ((body :wat::lru::CacheService::Body<K,V>) (:wat::core::first req))
     ((reply-to :wat::lru::CacheService::ReplyTx<V>) (:wat::core::second req))
     ((tag :wat::core::i64) (:wat::core::first body))
     ((key :K) (:wat::core::second body))
     ((put-val :Option<V>) (:wat::core::third body))
     ((resp :Option<V>)
      (:wat::core::if (:wat::core::= tag 0) -> :Option<V>
        (:wat::lru::LocalCache::get cache key)
        (:wat::core::match put-val -> :Option<V>
          ((Some v)
            (:wat::core::let*
              (((_ :Option<(K,V)>) (:wat::lru::LocalCache::put cache key v)))
              :None))
          (:None :None))))
     ;; Per arc 110: client dropping reply-to mid-protocol is a
     ;; protocol violation in this in-memory CSP. Panic so the
     ;; program tree learns the discipline broke instead of
     ;; silently dropping the reply.
     ((_send :())
      (:wat::core::option::expect -> :()
        (:wat::kernel::send reply-to resp)
        "CacheService/handle: reply-to disconnected — client died mid-request?"))
     ((stats' :wat::lru::CacheService::Stats)
      (:wat::core::if (:wat::core::= tag 0) -> :wat::lru::CacheService::Stats
        ;; GET — bump lookups + hits/misses
        (:wat::core::let*
          (((hit-delta :wat::core::i64)
            (:wat::core::match resp -> :wat::core::i64
              ((Some _) 1)
              (:None 0)))
           ((miss-delta :wat::core::i64)
            (:wat::core::i64::- 1 hit-delta)))
          (:wat::lru::CacheService::Stats/new
            (:wat::core::i64::+ (:wat::lru::CacheService::Stats/lookups stats) 1)
            (:wat::core::i64::+ (:wat::lru::CacheService::Stats/hits stats) hit-delta)
            (:wat::core::i64::+ (:wat::lru::CacheService::Stats/misses stats) miss-delta)
            (:wat::lru::CacheService::Stats/puts stats)
            (:wat::lru::CacheService::Stats/cache-size stats)))
        ;; PUT — bump puts
        (:wat::lru::CacheService::Stats/new
          (:wat::lru::CacheService::Stats/lookups stats)
          (:wat::lru::CacheService::Stats/hits stats)
          (:wat::lru::CacheService::Stats/misses stats)
          (:wat::core::i64::+ (:wat::lru::CacheService::Stats/puts stats) 1)
          (:wat::lru::CacheService::Stats/cache-size stats)))))
    (:wat::lru::CacheService::State/new cache stats')))

;; ─── Tick the metrics window — advance gate, emit+reset on fire ──

(:wat::core::define
  (:wat::lru::CacheService/tick-window<K,V,G>
    (state :wat::lru::CacheService::State<K,V>)
    (reporter :wat::lru::CacheService::Reporter)
    (metrics-cadence :wat::lru::CacheService::MetricsCadence<G>)
    -> :wat::lru::CacheService::Step<K,V,G>)
  (:wat::core::let*
    (((stats :wat::lru::CacheService::Stats)
      (:wat::lru::CacheService::State/stats state))
     ((gate :G)
      (:wat::lru::CacheService::MetricsCadence/gate metrics-cadence))
     ((tick-fn :fn(G,wat::lru::CacheService::Stats)->(G,bool))
      (:wat::lru::CacheService::MetricsCadence/tick metrics-cadence))
     ((tick :(G,bool)) (tick-fn gate stats))
     ((gate' :G) (:wat::core::first tick))
     ((fired :wat::core::bool) (:wat::core::second tick))
     ((cadence' :wat::lru::CacheService::MetricsCadence<G>)
      (:wat::lru::CacheService::MetricsCadence/new gate' tick-fn)))
    (:wat::core::if fired -> :wat::lru::CacheService::Step<K,V,G>
      (:wat::core::let*
        (((cache :wat::lru::LocalCache<K,V>)
          (:wat::lru::CacheService::State/cache state))
         ((final-stats :wat::lru::CacheService::Stats)
          (:wat::lru::CacheService::Stats/new
            (:wat::lru::CacheService::Stats/lookups stats)
            (:wat::lru::CacheService::Stats/hits stats)
            (:wat::lru::CacheService::Stats/misses stats)
            (:wat::lru::CacheService::Stats/puts stats)
            (:wat::lru::LocalCache::len cache)))
         ((_ :()) (reporter (:wat::lru::CacheService::Report::Metrics final-stats)))
         ((state' :wat::lru::CacheService::State<K,V>)
          (:wat::lru::CacheService::State/new
            cache (:wat::lru::CacheService::Stats/zero))))
        (:wat::core::tuple state' cadence'))
      (:wat::core::tuple state cadence'))))

;; Driver entry — allocates the LocalCache INSIDE the driver thread
;; (LocalCache is thread-owned; creating it in the caller and passing
;; across threads would trip the thread-id guard and wedge the
;; driver). Then delegates to `CacheService/loop-step` for the recursion.
(:wat::core::define
  (:wat::lru::CacheService/loop<K,V,G>
    (capacity :wat::core::i64)
    (req-rxs :Vec<wat::lru::CacheService::ReqRx<K,V>>)
    (reporter :wat::lru::CacheService::Reporter)
    (metrics-cadence :wat::lru::CacheService::MetricsCadence<G>)
    -> :())
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<K,V>)
      (:wat::lru::LocalCache::new capacity))
     ((initial :wat::lru::CacheService::State<K,V>)
      (:wat::lru::CacheService::State/new
        cache (:wat::lru::CacheService::Stats/zero))))
    (:wat::lru::CacheService/loop-step
      initial req-rxs reporter metrics-cadence)))

;; Recursive inner loop. Owns the cache for the duration of the
;; driver thread's lifetime; select across request receivers; each
;; request carries its reply-to sender for routing. After every
;; dispatch, tick the metrics window (advance gate; emit on fire).
(:wat::core::define
  (:wat::lru::CacheService/loop-step<K,V,G>
    (state :wat::lru::CacheService::State<K,V>)
    (req-rxs :Vec<wat::lru::CacheService::ReqRx<K,V>>)
    (reporter :wat::lru::CacheService::Reporter)
    (metrics-cadence :wat::lru::CacheService::MetricsCadence<G>)
    -> :())
  (:wat::core::if (:wat::core::empty? req-rxs) -> :()
    ()
    (:wat::core::let*
      (((chosen :(i64,Option<wat::lru::CacheService::Request<K,V>>))
        (:wat::kernel::select req-rxs))
       ((idx :wat::core::i64) (:wat::core::first chosen))
       ((maybe :Option<wat::lru::CacheService::Request<K,V>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :()
        ((Some req)
          (:wat::core::let*
            (((after-handle :wat::lru::CacheService::State<K,V>)
              (:wat::lru::CacheService/handle req state))
             ((step :wat::lru::CacheService::Step<K,V,G>)
              (:wat::lru::CacheService/tick-window
                after-handle reporter metrics-cadence))
             ((next-state :wat::lru::CacheService::State<K,V>)
              (:wat::core::first step))
             ((cadence' :wat::lru::CacheService::MetricsCadence<G>)
              (:wat::core::second step)))
            (:wat::lru::CacheService/loop-step
              next-state req-rxs reporter cadence')))
        (:None
          (:wat::lru::CacheService/loop-step
            state
            (:wat::std::list::remove-at req-rxs idx)
            reporter metrics-cadence))))))

;; --- Client helpers ---
;;
;; A client creates its response channel once at setup and reuses it
;; for every request. CacheService/get and CacheService/put package
;; the request, send it, and block on the response.

(:wat::core::define
  (:wat::lru::CacheService/get<K,V>
    (req-tx :wat::lru::CacheService::ReqTx<K,V>)
    (reply-tx :wat::lru::CacheService::ReplyTx<V>)
    (reply-rx :wat::kernel::QueueReceiver<Option<V>>)
    (key :K)
    -> :Option<V>)
  (:wat::core::let*
    (((body :wat::lru::CacheService::Body<K,V>)
      (:wat::core::tuple 0 key :None))
     ((req :wat::lru::CacheService::Request<K,V>)
      (:wat::core::tuple body reply-tx))
     ;; Arc 110: in-memory peer-death is catastrophic; cache driver
     ;; dying means our state-of-the-world claim is invalid. Panic
     ;; with a meaningful message rather than silently returning
     ;; :None and pretending we got a "miss."
     ((_send :())
      (:wat::core::option::expect -> :()
        (:wat::kernel::send req-tx req)
        "CacheService/get: req-tx disconnected — driver died?")))
    (:wat::core::option::expect -> :Option<V>
      (:wat::kernel::recv reply-rx)
      "CacheService/get: reply-rx disconnected — driver died mid-request?")))

(:wat::core::define
  (:wat::lru::CacheService/put<K,V>
    (req-tx :wat::lru::CacheService::ReqTx<K,V>)
    (reply-tx :wat::lru::CacheService::ReplyTx<V>)
    (reply-rx :wat::kernel::QueueReceiver<Option<V>>)
    (key :K)
    (value :V)
    -> :())
  (:wat::core::let*
    (((body :wat::lru::CacheService::Body<K,V>)
      (:wat::core::tuple 1 key (Some value)))
     ((req :wat::lru::CacheService::Request<K,V>)
      (:wat::core::tuple body reply-tx))
     ;; Arc 110: same as CacheService/get — driver dying mid-protocol
     ;; is catastrophic; panic with a meaningful message rather than
     ;; silently absorbing the disconnect.
     ((_send :())
      (:wat::core::option::expect -> :()
        (:wat::kernel::send req-tx req)
        "CacheService/put: req-tx disconnected — driver died?"))
     ((_recv :Option<V>)
      (:wat::core::option::expect -> :Option<V>
        (:wat::kernel::recv reply-rx)
        "CacheService/put: reply-rx disconnected — driver died mid-request?")))
    ()))

;; --- CacheService setup ---
;;
;; Creates N bounded(1) request queues, wraps senders in a HandlePool,
;; spawns one driver thread that owns a fresh LocalCache<K,V> of the
;; given capacity and fans in all request receivers. Returns the
;; (pool, driver-handle) pair.
;;
;; Both reporter + metrics-cadence are required; pass
;; :wat::lru::CacheService/null-reporter and
;; (:wat::lru::CacheService/null-metrics-cadence) for the explicit
;; "no reporting" choice. See CONVENTIONS.md "Service contract".
(:wat::core::define
  (:wat::lru::CacheService/spawn<K,V,G>
    (capacity :wat::core::i64)
    (count :wat::core::i64)
    (reporter :wat::lru::CacheService::Reporter)
    (metrics-cadence :wat::lru::CacheService::MetricsCadence<G>)
    -> :wat::lru::CacheService::Spawn<K,V>)
  (:wat::core::let*
    (((pairs :Vec<wat::lru::CacheService::ReqPair<K,V>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda ((_i :wat::core::i64) -> :wat::lru::CacheService::ReqPair<K,V>)
          (:wat::kernel::make-bounded-queue :wat::lru::CacheService::Request<K,V> 1))))
     ((req-txs :Vec<wat::lru::CacheService::ReqTx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :wat::lru::CacheService::ReqPair<K,V>)
                            -> :wat::lru::CacheService::ReqTx<K,V>)
          (:wat::core::first p))))
     ((req-rxs :Vec<wat::lru::CacheService::ReqRx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :wat::lru::CacheService::ReqPair<K,V>)
                            -> :wat::lru::CacheService::ReqRx<K,V>)
          (:wat::core::second p))))
     ((pool :wat::kernel::HandlePool<wat::lru::CacheService::ReqTx<K,V>>)
      (:wat::kernel::HandlePool::new "CacheService" req-txs))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::lru::CacheService/loop
        capacity req-rxs reporter metrics-cadence)))
    (:wat::core::tuple pool driver)))
