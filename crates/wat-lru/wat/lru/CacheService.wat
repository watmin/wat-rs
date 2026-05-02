;; :wat::lru::* — wat-lru's multi-client LRU service program.
;;
;; Repathed from wat-rs's former :wat::std::service::Cache when arc
;; 013 externalized this crate (slice 4b). The CacheService grouping
;; noun retired in arc 109 slice K.lru (2026-05-01) per § K's
;; "/ requires a real Type" doctrine — verbs and typealiases live
;; at the namespace level. Real types Stats / MetricsCadence /
;; State / Report keep their PascalCase + /methods.
;;
;; Channel-naming family: Pattern B (Get — data-back, Reply*)
;; + Pattern A (Put — unit-ack, PutAck*). See INVENTORY § K.
;;
;; Arc 119: symmetric batch protocol. Request is now an enum
;; (Get | Put) rather than a tagged-tuple Body<K,V>. Body<K,V>
;; retires. Entry<K,V> = (K,V) is the batch-element name.
;;
;;   Get carries Vec<K> probes,   returns Vec<Option<V>>  (Pattern B back-edge)
;;   Put carries Vec<Entry<K,V>>, returns unit-ack        (Pattern A back-edge)
;;
;; Variant-scoped channel families:
;;   Reply*   (ReplyTx, ReplyRx, ReplyChannel)   — GET data-bearing back-edge
;;   PutAck*  (PutAckTx, PutAckRx, PutAckChannel) — PUT unit-ack release
;;
;; See docs/CONVENTIONS.md § "Batch convention" and
;; docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md.
;;
;; A program that owns its own LocalCache<K,V> behind a select loop;
;; clients send requests with their own reply/ack address attached so
;; the driver routes responses without a sender-index map.
;;
;; Generic over K,V — type params propagate through every define via
;; wat's `<K,V>` declaration syntax, same pattern LocalCache uses.
;; Runtime storage is canonical-string-keyed per LocalCache/HashMap
;; convention; K,V are phantom at the type-check layer.
;;
;; Arc 078: ships the canonical service contract — Reporter +
;; MetricsCadence<G> + null-helpers + typed Report enum — alongside
;; the batch Request enum protocol. spawn now demands both injection
;; points; pass null-reporter / null-metrics-cadence for the
;; explicit "no reporting" choice.

;; crossbeam_channel is wat substrate, not a wat-lru dep — the
;; runtime provides Sender<T>/Receiver<T> via :wat::kernel::
;; primitives (make-bounded-channel, etc.). `use!` declares intent
;; to consume an *external* Rust crate (a #[wat_dispatch]'d
;; library); substrate types don't need it. Only :rust::lru::LruCache
;; — the real external dep — gets a `use!` (see lru.wat).

;; --- Protocol typealiases ---

;; Entry<K,V> = (K, V) — the batch-element name (arc 119, gaze pass 3).
;; Standard cache-domain word; unambiguous across the two cache services.
(:wat::core::typealias :wat::lru::Entry<K,V>
  :(K,V))

;; PutAck* — Pattern A (unit-ack release) back-edge family for Put.
(:wat::core::typealias :wat::lru::PutAckTx
  :wat::kernel::Sender<wat::core::unit>)
(:wat::core::typealias :wat::lru::PutAckRx
  :wat::kernel::Receiver<wat::core::unit>)
(:wat::core::typealias :wat::lru::PutAckChannel
  :(wat::lru::PutAckTx,wat::lru::PutAckRx))

;; Reply* — Pattern B (data-bearing back-edge) family for Get.
;; Body widens from Sender<Option<V>> to Sender<Vec<Option<V>>> (arc 119 batch).
(:wat::core::typealias :wat::lru::ReplyTx<V>
  :wat::kernel::Sender<wat::core::Vector<wat::core::Option<V>>>)
(:wat::core::typealias :wat::lru::ReplyRx<V>
  :wat::kernel::Receiver<wat::core::Vector<wat::core::Option<V>>>)
(:wat::core::typealias :wat::lru::ReplyChannel<V>
  :(wat::lru::ReplyTx<V>,wat::lru::ReplyRx<V>))

;; Request<K,V> — enum-based (arc 119). Body<K,V> tagged-tuple retires.
;;   Get carries a Vec<K> probe batch + a reply-tx for the batch result.
;;   Put carries a Vec<Entry<K,V>> entries batch + an ack-tx for the unit release.
(:wat::core::enum :wat::lru::Request<K,V>
  (Get  (probes   :wat::core::Vector<K>)
        (reply-tx :wat::lru::ReplyTx<V>))
  (Put  (entries  :wat::core::Vector<wat::lru::Entry<K,V>>)
        (ack-tx   :wat::lru::PutAckTx)))

(:wat::core::typealias :wat::lru::ReqTx<K,V>
  :wat::kernel::Sender<wat::lru::Request<K,V>>)
(:wat::core::typealias :wat::lru::ReqRx<K,V>
  :wat::kernel::Receiver<wat::lru::Request<K,V>>)

;; The (ReqTx, ReqRx) pair as a single name. Used by the spawn body
;; to keep nested `<>` depth tractable when iterating bounded-queue
;; pairs.
(:wat::core::typealias :wat::lru::ReqChannel<K,V>
  :(wat::lru::ReqTx<K,V>,wat::lru::ReqRx<K,V>))

;; --- Spawn return shape ---
;;
;; What `:wat::lru::spawn` returns: the HandlePool of
;; client request senders + the driver's Thread handle (arc 114).
;; Caller pops N senders, finishes the pool, scoped-drops at end →
;; driver exits.
(:wat::core::typealias :wat::lru::Spawn<K,V>
  :(wat::kernel::HandlePool<wat::lru::ReqTx<K,V>>,wat::kernel::Thread<wat::core::unit,wat::core::unit>))

;; ─── Reporting contract — non-negotiable ───────────────────────
;;
;; Same shape as :wat::holon::lru::HologramCacheService — the
;; canonical wat substrate service contract per arc 078. The user
;; passes a Reporter (consumer-defined match-dispatching fn over
;; Report variants) and a MetricsCadence (gate + tick) at spawn time.
;; Both are required; pass null-reporter / null-metrics-cadence for
;; the explicit "no reporting" choice.

(:wat::core::struct :wat::lru::Stats
  (lookups :wat::core::i64)        ;; total Gets (in probe count) in this window
  (hits :wat::core::i64)           ;; probe slots returning Some
  (misses :wat::core::i64)         ;; probe slots returning :None
  (puts :wat::core::i64)           ;; total Put entries in this window
  (cache-size :wat::core::i64))    ;; LocalCache::len at gate-fire time

;; Slice 4 ships ONE variant (Metrics, gated by metrics-cadence).
;; Future variants (lifecycle, errors, evictions) extend additively
;; without breaking consumers — same grow-by-arms pattern as the
;; archive's TreasuryRequest.
(:wat::core::enum :wat::lru::Report
  (Metrics (stats :wat::lru::Stats)))

;; MetricsCadence<G> — stateful rate gate. The user picks G; the
;; cache threads the gate through each loop iteration via
;; MetricsCadence/new with the advanced gate; the tick fn itself is
;; invariant.
(:wat::core::struct :wat::lru::MetricsCadence<G>
  (gate :G)
  (tick :fn(G,wat::lru::Stats)->(G,wat::core::bool)))

(:wat::core::typealias :wat::lru::Reporter
  :fn(wat::lru::Report)->wat::core::unit)

;; null-metrics-cadence — fresh `MetricsCadence<()>` whose tick
;; never fires. Use when metrics are a deliberate opt-out.
(:wat::core::define
  (:wat::lru::null-metrics-cadence
    -> :wat::lru::MetricsCadence<wat::core::unit>)
  (:wat::lru::MetricsCadence/new
    ()
    (:wat::core::lambda
      ((gate :wat::core::unit) (_stats :wat::lru::Stats) -> :(wat::core::unit,wat::core::bool))
      (:wat::core::Tuple gate false))))

;; null-reporter — discards every Report variant.
(:wat::core::define
  (:wat::lru::null-reporter
    (_report :wat::lru::Report) -> :wat::core::unit)
  ())

;; Fresh zero-counters Stats. Used at startup and after each
;; gate-fire (window-rolling reset).
(:wat::core::define
  (:wat::lru::Stats/zero -> :wat::lru::Stats)
  (:wat::lru::Stats/new 0 0 0 0 0))

;; ─── Service state — cache + running stats ─────────────────────
;;
;; Threaded through CacheService/loop-step alongside the cadence's
;; gate. The cache mutates in place (LocalCache is thread-owned
;; mutable); Stats rebuilds each iteration (values-up).

(:wat::core::struct :wat::lru::State<K,V>
  (cache :wat::lru::LocalCache<K,V>)
  (stats :wat::lru::Stats))

;; One loop-step's outputs: the post-dispatch State paired with the
;; advanced MetricsCadence. tick-window and loop-step both thread
;; this shape.
(:wat::core::typealias :wat::lru::Step<K,V,G>
  :(wat::lru::State<K,V>,wat::lru::MetricsCadence<G>))

;; ─── Per-variant request handler ────────────────────────────────
;;
;; GET: batch-lookup probes via LocalCache::get; reply with
;;      Vec<Option<V>> on reply-tx; stats: lookups += len(probes),
;;      hits/misses counted from result vec.
;; PUT: batch-insert entries via LocalCache::put; ack unit on
;;      ack-tx after whole batch persisted; stats: puts += len(entries).
;;
;; Returns the new State (cache pointer unchanged — mutates in
;; place; stats rebuilt).

(:wat::core::define
  (:wat::lru::handle<K,V>
    (req :wat::lru::Request<K,V>)
    (state :wat::lru::State<K,V>)
    -> :wat::lru::State<K,V>)
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<K,V>)
      (:wat::lru::State/cache state))
     ((stats :wat::lru::Stats)
      (:wat::lru::State/stats state)))
    (:wat::core::match req -> :wat::lru::State<K,V>
      ((:wat::lru::Request::Get probes reply-tx)
        (:wat::core::let*
          (((results :wat::core::Vector<wat::core::Option<V>>)
            (:wat::core::map probes
              (:wat::core::lambda ((k :K) -> :wat::core::Option<V>)
                (:wat::lru::LocalCache::get cache k))))
           ((hit-count :wat::core::i64)
            (:wat::core::reduce results 0
              (:wat::core::lambda
                ((acc :wat::core::i64) (slot :wat::core::Option<V>) -> :wat::core::i64)
                (:wat::core::match slot -> :wat::core::i64
                  ((:wat::core::Some _) (:wat::core::i64::+ acc 1))
                  (:wat::core::None acc)))))
           ((n :wat::core::i64) (:wat::core::Vector/len probes))
           ((miss-count :wat::core::i64) (:wat::core::i64::- n hit-count))
           ;; Arc 110: in-memory peer-death is catastrophic; panic with a
           ;; meaningful message rather than silently dropping the reply.
           ((_send :wat::core::unit)
            (:wat::core::Result/expect -> :wat::core::unit
              (:wat::kernel::send reply-tx results)
              "CacheService/handle: reply-tx disconnected — client died mid-request?"))
           ((stats' :wat::lru::Stats)
            (:wat::lru::Stats/new
              (:wat::core::i64::+ (:wat::lru::Stats/lookups stats) n)
              (:wat::core::i64::+ (:wat::lru::Stats/hits stats) hit-count)
              (:wat::core::i64::+ (:wat::lru::Stats/misses stats) miss-count)
              (:wat::lru::Stats/puts stats)
              (:wat::lru::Stats/cache-size stats))))
          (:wat::lru::State/new cache stats')))
      ((:wat::lru::Request::Put entries ack-tx)
        (:wat::core::let*
          (((_ :wat::core::Vector<wat::core::Option<(K,V)>>)
            (:wat::core::map entries
              (:wat::core::lambda
                ((entry :wat::lru::Entry<K,V>) -> :wat::core::Option<(K,V)>)
                (:wat::core::let*
                  (((k :K) (:wat::core::first entry))
                   ((v :V) (:wat::core::second entry)))
                  (:wat::lru::LocalCache::put cache k v)))))
           ((n :wat::core::i64) (:wat::core::Vector/len entries))
           ;; Arc 110: same discipline — driver dying mid-protocol is
           ;; catastrophic; panic with a meaningful message.
           ((_send :wat::core::unit)
            (:wat::core::Result/expect -> :wat::core::unit
              (:wat::kernel::send ack-tx ())
              "CacheService/handle: ack-tx disconnected — client died mid-request?"))
           ((stats' :wat::lru::Stats)
            (:wat::lru::Stats/new
              (:wat::lru::Stats/lookups stats)
              (:wat::lru::Stats/hits stats)
              (:wat::lru::Stats/misses stats)
              (:wat::core::i64::+ (:wat::lru::Stats/puts stats) n)
              (:wat::lru::Stats/cache-size stats))))
          (:wat::lru::State/new cache stats'))))))

;; ─── Tick the metrics window — advance gate, emit+reset on fire ──

(:wat::core::define
  (:wat::lru::tick-window<K,V,G>
    (state :wat::lru::State<K,V>)
    (reporter :wat::lru::Reporter)
    (metrics-cadence :wat::lru::MetricsCadence<G>)
    -> :wat::lru::Step<K,V,G>)
  (:wat::core::let*
    (((stats :wat::lru::Stats)
      (:wat::lru::State/stats state))
     ((gate :G)
      (:wat::lru::MetricsCadence/gate metrics-cadence))
     ((tick-fn :fn(G,wat::lru::Stats)->(G,wat::core::bool))
      (:wat::lru::MetricsCadence/tick metrics-cadence))
     ((tick :(G,wat::core::bool)) (tick-fn gate stats))
     ((gate' :G) (:wat::core::first tick))
     ((fired :wat::core::bool) (:wat::core::second tick))
     ((cadence' :wat::lru::MetricsCadence<G>)
      (:wat::lru::MetricsCadence/new gate' tick-fn)))
    (:wat::core::if fired -> :wat::lru::Step<K,V,G>
      (:wat::core::let*
        (((cache :wat::lru::LocalCache<K,V>)
          (:wat::lru::State/cache state))
         ((final-stats :wat::lru::Stats)
          (:wat::lru::Stats/new
            (:wat::lru::Stats/lookups stats)
            (:wat::lru::Stats/hits stats)
            (:wat::lru::Stats/misses stats)
            (:wat::lru::Stats/puts stats)
            (:wat::lru::LocalCache::len cache)))
         ((_ :wat::core::unit) (reporter (:wat::lru::Report::Metrics final-stats)))
         ((state' :wat::lru::State<K,V>)
          (:wat::lru::State/new
            cache (:wat::lru::Stats/zero))))
        (:wat::core::Tuple state' cadence'))
      (:wat::core::Tuple state cadence'))))

;; Driver entry — allocates the LocalCache INSIDE the driver thread
;; (LocalCache is thread-owned; creating it in the caller and passing
;; across threads would trip the thread-id guard and wedge the
;; driver). Then delegates to `loop-step` for the recursion.
(:wat::core::define
  (:wat::lru::loop<K,V,G>
    (capacity :wat::core::i64)
    (req-rxs :wat::core::Vector<wat::lru::ReqRx<K,V>>)
    (reporter :wat::lru::Reporter)
    (metrics-cadence :wat::lru::MetricsCadence<G>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<K,V>)
      (:wat::lru::LocalCache::new capacity))
     ((initial :wat::lru::State<K,V>)
      (:wat::lru::State/new
        cache (:wat::lru::Stats/zero))))
    (:wat::lru::loop-step
      initial req-rxs reporter metrics-cadence)))

;; Recursive inner loop. Owns the cache for the duration of the
;; driver thread's lifetime; select across request receivers; each
;; request carries its reply-to sender for routing. After every
;; dispatch, tick the metrics window (advance gate; emit on fire).
(:wat::core::define
  (:wat::lru::loop-step<K,V,G>
    (state :wat::lru::State<K,V>)
    (req-rxs :wat::core::Vector<wat::lru::ReqRx<K,V>>)
    (reporter :wat::lru::Reporter)
    (metrics-cadence :wat::lru::MetricsCadence<G>)
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::empty? req-rxs) -> :wat::core::unit
    ()
    (:wat::core::let*
      (((chosen :wat::kernel::Chosen<wat::lru::Request<K,V>>)
        (:wat::kernel::select req-rxs))
       ((idx :wat::core::i64) (:wat::core::first chosen))
       ((maybe :wat::kernel::CommResult<wat::lru::Request<K,V>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :wat::core::unit
        ((:wat::core::Ok (:wat::core::Some req))
          (:wat::core::let*
            (((after-handle :wat::lru::State<K,V>)
              (:wat::lru::handle req state))
             ((step :wat::lru::Step<K,V,G>)
              (:wat::lru::tick-window
                after-handle reporter metrics-cadence))
             ((next-state :wat::lru::State<K,V>)
              (:wat::core::first step))
             ((cadence' :wat::lru::MetricsCadence<G>)
              (:wat::core::second step)))
            (:wat::lru::loop-step
              next-state req-rxs reporter cadence')))
        ((:wat::core::Ok :wat::core::None)
          (:wat::lru::loop-step
            state
            (:wat::std::list::remove-at req-rxs idx)
            reporter metrics-cadence))
        ((:wat::core::Err _died) ())))))

;; --- Client helpers ---
;;
;; A client creates its reply/ack channel once at setup and reuses it
;; for every request. :wat::lru::get and :wat::lru::put package the
;; batch request, send it, and block on the response.
;;
;; Arc 119: get takes Vec<K> probes, returns Vec<Option<V>>.
;;          put takes Vec<Entry<K,V>>, returns unit after ack.
;;
;; Recv pattern (two nested levels per arc 111+113):
;;   Result/expect unwraps the outer Result (ThreadDiedError on peer death).
;;   Option/expect unwraps the inner Option (None = clean channel close).

(:wat::core::define
  (:wat::lru::get<K,V>
    (req-tx :wat::lru::ReqTx<K,V>)
    (reply-tx :wat::lru::ReplyTx<V>)
    (reply-rx :wat::lru::ReplyRx<V>)
    (probes :wat::core::Vector<K>)
    -> :wat::core::Vector<wat::core::Option<V>>)
  (:wat::core::let*
    (((req :wat::lru::Request<K,V>)
      (:wat::lru::Request::Get probes reply-tx))
     ;; Arc 110: in-memory peer-death is catastrophic; cache driver
     ;; dying means our state-of-the-world claim is invalid. Panic
     ;; with a meaningful message rather than silently returning
     ;; :None and pretending we got a "miss."
     ((_send :wat::core::unit)
      (:wat::core::Result/expect -> :wat::core::unit
        (:wat::kernel::send req-tx req)
        "lru::get: req-tx disconnected — driver died?")))
    (:wat::core::Option/expect -> :wat::core::Vector<wat::core::Option<V>>
      (:wat::core::Result/expect -> :wat::core::Option<wat::core::Vector<wat::core::Option<V>>>
        (:wat::kernel::recv reply-rx)
        "lru::get: reply-rx disconnected — driver died mid-request?")
      "lru::get: reply channel closed — driver dropped reply-tx?")))

(:wat::core::define
  (:wat::lru::put<K,V>
    (req-tx :wat::lru::ReqTx<K,V>)
    (ack-tx :wat::lru::PutAckTx)
    (ack-rx :wat::lru::PutAckRx)
    (entries :wat::core::Vector<wat::lru::Entry<K,V>>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((req :wat::lru::Request<K,V>)
      (:wat::lru::Request::Put entries ack-tx))
     ;; Arc 110: same as lru::get — driver dying mid-protocol
     ;; is catastrophic; panic with a meaningful message rather than
     ;; silently absorbing the disconnect.
     ((_send :wat::core::unit)
      (:wat::core::Result/expect -> :wat::core::unit
        (:wat::kernel::send req-tx req)
        "lru::put: req-tx disconnected — driver died?"))
     ((_ :wat::core::unit)
      (:wat::core::Option/expect -> :wat::core::unit
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::unit>
          (:wat::kernel::recv ack-rx)
          "lru::put: ack-rx disconnected — driver died mid-request?")
        "lru::put: ack channel closed — driver dropped ack-tx?")))
    ()))

;; --- CacheService setup ---
;;
;; Creates N bounded(1) request queues, wraps senders in a HandlePool,
;; spawns one driver thread that owns a fresh LocalCache<K,V> of the
;; given capacity and fans in all request receivers. Returns the
;; (pool, driver-handle) pair.
;;
;; Both reporter + metrics-cadence are required; pass
;; :wat::lru::null-reporter and
;; (:wat::lru::null-metrics-cadence) for the explicit
;; "no reporting" choice. See CONVENTIONS.md "Service contract".
(:wat::core::define
  (:wat::lru::spawn<K,V,G>
    (capacity :wat::core::i64)
    (count :wat::core::i64)
    (reporter :wat::lru::Reporter)
    (metrics-cadence :wat::lru::MetricsCadence<G>)
    -> :wat::lru::Spawn<K,V>)
  (:wat::core::let*
    (((pairs :wat::core::Vector<wat::lru::ReqChannel<K,V>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda ((_i :wat::core::i64) -> :wat::lru::ReqChannel<K,V>)
          (:wat::kernel::make-bounded-channel :wat::lru::Request<K,V> 1))))
     ((req-txs :wat::core::Vector<wat::lru::ReqTx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :wat::lru::ReqChannel<K,V>)
                            -> :wat::lru::ReqTx<K,V>)
          (:wat::core::first p))))
     ((req-rxs :wat::core::Vector<wat::lru::ReqRx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :wat::lru::ReqChannel<K,V>)
                            -> :wat::lru::ReqRx<K,V>)
          (:wat::core::second p))))
     ((pool :wat::kernel::HandlePool<wat::lru::ReqTx<K,V>>)
      (:wat::kernel::HandlePool::new "CacheService" req-txs))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::lru::loop
            capacity req-rxs reporter metrics-cadence)))))
    (:wat::core::Tuple pool driver)))
