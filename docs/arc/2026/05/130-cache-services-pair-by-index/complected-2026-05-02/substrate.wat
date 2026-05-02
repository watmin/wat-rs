;; :wat::lru::* — wat-lru's multi-client LRU service program.
;;
;; Repathed from wat-rs's former :wat::std::service::Cache when arc
;; 013 externalized this crate (slice 4b). The CacheService grouping
;; noun retired in arc 109 slice K.lru (2026-05-01) per § K's
;; "/ requires a real Type" doctrine — verbs and typealiases live
;; at the namespace level. Real types Stats / MetricsCadence /
;; State / Report keep their PascalCase + /methods.
;;
;; Arc 119: symmetric batch protocol. Request is now an enum
;; (Get | Put) rather than a tagged-tuple Body<K,V>. Body<K,V>
;; retires. Entry<K,V> = (K,V) is the batch-element name.
;;
;;   Get carries Vec<K> probes,   returns Vec<Option<V>>
;;   Put carries Vec<Entry<K,V>>, returns unit-ack
;;
;; Arc 130: pair-by-index via HandlePool. Request enum no longer
;; embeds reply channels; the unified Reply<V> enum (GetResult |
;; PutAck) flows on one reply channel per slot, pre-allocated at
;; spawn time. Clients pop a Handle = (ReqTx, ReplyRx); driver
;; holds matching DriverPair = (ReqRx, ReplyTx) at the same index.
;; PutAckTx / PutAckRx / PutAckChannel retire. Mirrors Console's
;; pair-by-index discipline.
;;
;; See docs/CONVENTIONS.md § "Batch convention" and
;; docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md.
;;
;; A program that owns its own LocalCache<K,V> behind a select loop;
;; clients pop a Handle from the pool and call helper verbs — no
;; per-call channel allocation needed.
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

;; Reply<V> enum — unifies Get's Vec<Option<V>> and Put's unit-ack on
;; one channel per slot (arc 130). Replaces the old per-verb ReplyTx /
;; PutAckTx split; the driver matches Request variant, sends matching
;; Reply variant on the slot's ReplyTx; the helper verb matches Reply
;; variant, returns the relevant data.
(:wat::core::enum :wat::lru::Reply<V>
  (GetResult (results :wat::core::Vector<wat::core::Option<V>>))
  (PutAck))

;; Reply channel typealiases — rebind ReplyTx/ReplyRx/ReplyChannel to
;; carry Reply<V> enum instead of the old bare Vec<Option<V>>.
;; PutAckTx/PutAckRx/PutAckChannel retire (arc 130).
(:wat::core::typealias :wat::lru::ReplyTx<V>
  :wat::kernel::Sender<wat::lru::Reply<V>>)
(:wat::core::typealias :wat::lru::ReplyRx<V>
  :wat::kernel::Receiver<wat::lru::Reply<V>>)
(:wat::core::typealias :wat::lru::ReplyChannel<V>
  :(wat::lru::ReplyTx<V>,wat::lru::ReplyRx<V>))

;; Request<K,V> — enum-based (arc 119). Embedded reply/ack channels
;; removed (arc 130): driver routes replies by DriverPair index.
(:wat::core::enum :wat::lru::Request<K,V>
  (Get  (probes  :wat::core::Vector<K>))
  (Put  (entries :wat::core::Vector<wat::lru::Entry<K,V>>)))

(:wat::core::typealias :wat::lru::ReqTx<K,V>
  :wat::kernel::Sender<wat::lru::Request<K,V>>)
(:wat::core::typealias :wat::lru::ReqRx<K,V>
  :wat::kernel::Receiver<wat::lru::Request<K,V>>)

;; The (ReqTx, ReqRx) pair as a single name. Used by the spawn body
;; to keep nested `<>` depth tractable when iterating bounded-queue
;; pairs.
(:wat::core::typealias :wat::lru::ReqChannel<K,V>
  :(wat::lru::ReqTx<K,V>,wat::lru::ReqRx<K,V>))

;; Handle<K,V> = (ReqTx, ReplyRx) — the client's view of one slot.
;; Mirrors Console::Handle = (ReqTx, AckRx). Popped from the HandlePool
;; at the client's scope; both channel-ends are pre-paired at spawn time
;; so no per-call channel allocation is needed. Arc 130.
(:wat::core::typealias :wat::lru::Handle<K,V>
  :(wat::lru::ReqTx<K,V>,wat::lru::ReplyRx<V>))

;; DriverPair<K,V> = (ReqRx, ReplyTx) — the driver's view of one slot.
;; Mirrors Console::DriverPair = (ReqRx, AckTx). The driver holds a
;; Vec<DriverPair>; select fires on the ReqRx vector; the index also
;; indexes into the DriverPair vector to find the matching ReplyTx.
;; Arc 130.
(:wat::core::typealias :wat::lru::DriverPair<K,V>
  :(wat::lru::ReqRx<K,V>,wat::lru::ReplyTx<V>))

;; --- Spawn return shape ---
;;
;; What `:wat::lru::spawn` returns: the HandlePool of
;; per-client Handles ((ReqTx, ReplyRx) pairs) + the driver's Thread
;; handle (arc 114). Caller pops N Handles, finishes the pool,
;; scoped-drops at end → driver exits. Arc 130: was HandlePool<ReqTx>.
(:wat::core::typealias :wat::lru::Spawn<K,V>
  :(wat::kernel::HandlePool<wat::lru::Handle<K,V>>,wat::kernel::Thread<wat::core::unit,wat::core::unit>))

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
;; GET: batch-lookup probes via LocalCache::get; send Reply::GetResult
;;      on reply-tx; stats: lookups += len(probes), hits/misses counted.
;; PUT: batch-insert entries via LocalCache::put; send Reply::PutAck on
;;      reply-tx after whole batch persisted; stats: puts += len(entries).
;;
;; Arc 130: reply-tx is now the slot's ReplyTx<V> passed in from the
;; driver loop (pair-by-index). Both variants send on the same reply-tx.
;;
;; Returns the new State (cache pointer unchanged — mutates in
;; place; stats rebuilt).

(:wat::core::define
  (:wat::lru::handle<K,V>
    (req :wat::lru::Request<K,V>)
    (reply-tx :wat::lru::ReplyTx<V>)
    (state :wat::lru::State<K,V>)
    -> :wat::lru::State<K,V>)
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<K,V>)
      (:wat::lru::State/cache state))
     ((stats :wat::lru::Stats)
      (:wat::lru::State/stats state)))
    (:wat::core::match req -> :wat::lru::State<K,V>
      ((:wat::lru::Request::Get probes)
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
           ;; Arc 130: send Reply::GetResult variant on the slot's reply-tx.
           ((_send :wat::core::unit)
            (:wat::core::Result/expect -> :wat::core::unit
              (:wat::kernel::send reply-tx (:wat::lru::Reply::GetResult results))
              "CacheService/handle: reply-tx disconnected — client died mid-request?"))
           ((stats' :wat::lru::Stats)
            (:wat::lru::Stats/new
              (:wat::core::i64::+ (:wat::lru::Stats/lookups stats) n)
              (:wat::core::i64::+ (:wat::lru::Stats/hits stats) hit-count)
              (:wat::core::i64::+ (:wat::lru::Stats/misses stats) miss-count)
              (:wat::lru::Stats/puts stats)
              (:wat::lru::Stats/cache-size stats))))
          (:wat::lru::State/new cache stats')))
      ((:wat::lru::Request::Put entries)
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
           ;; Arc 130: send Reply::PutAck variant on the slot's reply-tx.
           ((_send :wat::core::unit)
            (:wat::core::Result/expect -> :wat::core::unit
              (:wat::kernel::send reply-tx (:wat::lru::Reply::PutAck))
              "CacheService/handle: reply-tx disconnected — client died mid-request?"))
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
;;
;; Arc 130: takes Vec<DriverPair<K,V>> instead of Vec<ReqRx<K,V>>.
;; Each DriverPair = (ReqRx, ReplyTx); select fires on the ReqRx
;; side; the same index gives the matching ReplyTx. Mirrors
;; Console's driver pattern.
(:wat::core::define
  (:wat::lru::loop<K,V,G>
    (capacity :wat::core::i64)
    (driver-pairs :wat::core::Vector<wat::lru::DriverPair<K,V>>)
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
      initial driver-pairs reporter metrics-cadence)))

;; Recursive inner loop. Owns the cache for the duration of the
;; driver thread's lifetime; select across ReqRx vector extracted
;; from driver-pairs; index into driver-pairs to find matching
;; ReplyTx for pair-by-index routing. After every dispatch, tick
;; the metrics window (advance gate; emit on fire).
;;
;; Arc 130: select is over the ReqRx side of each DriverPair. The
;; same idx that select returns into the ReqRx vector indexes into
;; driver-pairs to recover the matching ReplyTx. Mirror of
;; Console's loop which indexes into pairs[i].second for the AckTx.
(:wat::core::define
  (:wat::lru::loop-step<K,V,G>
    (state :wat::lru::State<K,V>)
    (driver-pairs :wat::core::Vector<wat::lru::DriverPair<K,V>>)
    (reporter :wat::lru::Reporter)
    (metrics-cadence :wat::lru::MetricsCadence<G>)
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::empty? driver-pairs) -> :wat::core::unit
    ()
    (:wat::core::let*
      (((req-rxs :wat::core::Vector<wat::lru::ReqRx<K,V>>)
        (:wat::core::map driver-pairs
          (:wat::core::lambda
            ((p :wat::lru::DriverPair<K,V>)
             -> :wat::lru::ReqRx<K,V>)
            (:wat::core::first p))))
       ((chosen :wat::kernel::Chosen<wat::lru::Request<K,V>>)
        (:wat::kernel::select req-rxs))
       ((idx :wat::core::i64) (:wat::core::first chosen))
       ((maybe :wat::kernel::CommResult<wat::lru::Request<K,V>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :wat::core::unit
        ((:wat::core::Ok (:wat::core::Some req))
          (:wat::lru::reply-at driver-pairs idx req state reporter metrics-cadence))
        ((:wat::core::Ok :wat::core::None)
          (:wat::lru::loop-step
            state
            (:wat::std::list::remove-at driver-pairs idx)
            reporter metrics-cadence))
        ((:wat::core::Err _died) ())))))

;; --- Helper — dispatch req to handle + send Reply on pairs[idx].second ---
;;
;; Lifted out of loop-step for the same reason Console lifts ack-at:
;; keeps loop-step's outer let* one-let-deep per
;; `feedback_simple_forms_per_func`. Looks up the DriverPair at idx,
;; extracts the ReplyTx, calls handle, ticks the window, recurses.
(:wat::core::define
  (:wat::lru::reply-at<K,V,G>
    (driver-pairs :wat::core::Vector<wat::lru::DriverPair<K,V>>)
    (idx :wat::core::i64)
    (req :wat::lru::Request<K,V>)
    (state :wat::lru::State<K,V>)
    (reporter :wat::lru::Reporter)
    (metrics-cadence :wat::lru::MetricsCadence<G>)
    -> :wat::core::unit)
  (:wat::core::match (:wat::core::get driver-pairs idx) -> :wat::core::unit
    ((:wat::core::Some pair)
      (:wat::core::let*
        (((reply-tx :wat::lru::ReplyTx<V>)
          (:wat::core::second pair))
         ((after-handle :wat::lru::State<K,V>)
          (:wat::lru::handle req reply-tx state))
         ((step :wat::lru::Step<K,V,G>)
          (:wat::lru::tick-window
            after-handle reporter metrics-cadence))
         ((next-state :wat::lru::State<K,V>)
          (:wat::core::first step))
         ((cadence' :wat::lru::MetricsCadence<G>)
          (:wat::core::second step)))
        (:wat::lru::loop-step
          next-state driver-pairs reporter cadence')))
    (:wat::core::None ())))

;; --- Client helpers ---
;;
;; Arc 130: callers pop a Handle = (ReqTx, ReplyRx) from the pool.
;; No per-call channel allocation; the handle carries both ends of
;; the pre-paired (req, reply) channel pair. Helper verbs project
;; the Handle's two fields, send the Request variant, recv Reply,
;; and match on the Reply variant to extract the expected data.
;;
;; Arc 119: get takes Vec<K> probes, returns Vec<Option<V>>.
;;          put takes Vec<Entry<K,V>>, returns unit after ack.
;;
;; Recv pattern (two nested levels per arc 111+113):
;;   Result/expect unwraps the outer Result (ThreadDiedError on peer death).
;;   Option/expect unwraps the inner Option (None = clean channel close).

(:wat::core::define
  (:wat::lru::get<K,V>
    (handle :wat::lru::Handle<K,V>)
    (probes :wat::core::Vector<K>)
    -> :wat::core::Vector<wat::core::Option<V>>)
  (:wat::core::let*
    (((req-tx :wat::lru::ReqTx<K,V>)
      (:wat::core::first handle))
     ((reply-rx :wat::lru::ReplyRx<V>)
      (:wat::core::second handle))
     ;; Arc 110: in-memory peer-death is catastrophic; cache driver
     ;; dying means our state-of-the-world claim is invalid. Panic
     ;; with a meaningful message rather than silently returning
     ;; :None and pretending we got a "miss."
     ((_send :wat::core::unit)
      (:wat::core::Result/expect -> :wat::core::unit
        (:wat::kernel::send req-tx (:wat::lru::Request::Get probes))
        "lru::get: req-tx disconnected — driver died?"))
     ((reply :wat::lru::Reply<V>)
      (:wat::core::Option/expect -> :wat::lru::Reply<V>
        (:wat::core::Result/expect -> :wat::core::Option<wat::lru::Reply<V>>
          (:wat::kernel::recv reply-rx)
          "lru::get: reply-rx disconnected — driver died mid-request?")
        "lru::get: reply channel closed — driver dropped reply-tx?")))
    (:wat::core::match reply -> :wat::core::Vector<wat::core::Option<V>>
      ((:wat::lru::Reply::GetResult results) results)
      ((:wat::lru::Reply::PutAck)
        (:wat::core::panic! "lru/get: driver sent PutAck on Get reply channel")))))

(:wat::core::define
  (:wat::lru::put<K,V>
    (handle :wat::lru::Handle<K,V>)
    (entries :wat::core::Vector<wat::lru::Entry<K,V>>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((req-tx :wat::lru::ReqTx<K,V>)
      (:wat::core::first handle))
     ((reply-rx :wat::lru::ReplyRx<V>)
      (:wat::core::second handle))
     ;; Arc 110: same as lru::get — driver dying mid-protocol
     ;; is catastrophic; panic with a meaningful message rather than
     ;; silently absorbing the disconnect.
     ((_send :wat::core::unit)
      (:wat::core::Result/expect -> :wat::core::unit
        (:wat::kernel::send req-tx (:wat::lru::Request::Put entries))
        "lru::put: req-tx disconnected — driver died?"))
     ((reply :wat::lru::Reply<V>)
      (:wat::core::Option/expect -> :wat::lru::Reply<V>
        (:wat::core::Result/expect -> :wat::core::Option<wat::lru::Reply<V>>
          (:wat::kernel::recv reply-rx)
          "lru::put: reply-rx disconnected — driver died mid-request?")
        "lru::put: reply channel closed — driver dropped reply-tx?")))
    (:wat::core::match reply -> :wat::core::unit
      ((:wat::lru::Reply::PutAck) ())
      ((:wat::lru::Reply::GetResult _)
        (:wat::core::panic! "lru/put: driver sent GetResult on Put reply channel")))))

;; --- CacheService setup ---
;;
;; Arc 130: allocates N request channels AND N reply channels in
;; lock-step so their indices match. Builds N Handle<K,V> = (ReqTx,
;; ReplyRx) tuples for the client side; builds N DriverPair<K,V> =
;; (ReqRx, ReplyTx) tuples for the driver side. Populates a
;; HandlePool<Handle> so clients pop a single opaque Handle with no
;; per-call channel allocation. Passes Vec<DriverPair> to the driver.
;; Mirrors Console::spawn exactly (req-pairs + reply-pairs → zip
;; into handles + driver-pairs). Returns (pool, driver-handle).
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
    ;; Build N request pairs and N reply pairs in lock-step. The
    ;; index of the request pair matches the index of the reply pair —
    ;; this is what makes pair-by-index routing possible in loop-step.
    (((req-pairs :wat::core::Vector<wat::lru::ReqChannel<K,V>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda ((_i :wat::core::i64) -> :wat::lru::ReqChannel<K,V>)
          (:wat::kernel::make-bounded-channel :wat::lru::Request<K,V> 1))))
     ((reply-pairs :wat::core::Vector<wat::lru::ReplyChannel<V>>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda ((_i :wat::core::i64) -> :wat::lru::ReplyChannel<V>)
          (:wat::kernel::make-bounded-channel :wat::lru::Reply<V> 1))))
     ;; Client-side: Handle = (req-Tx, reply-Rx).
     ((handles :wat::core::Vector<wat::lru::Handle<K,V>>)
      (:wat::std::list::zip
        (:wat::core::map req-pairs
          (:wat::core::lambda ((p :wat::lru::ReqChannel<K,V>)
                              -> :wat::lru::ReqTx<K,V>)
            (:wat::core::first p)))
        (:wat::core::map reply-pairs
          (:wat::core::lambda ((p :wat::lru::ReplyChannel<V>)
                              -> :wat::lru::ReplyRx<V>)
            (:wat::core::second p)))))
     ;; Driver-side: DriverPair = (req-Rx, reply-Tx) at the matching index.
     ;; select fires for idx i; driver-pairs[i].second is the ReplyTx.
     ((driver-pairs :wat::core::Vector<wat::lru::DriverPair<K,V>>)
      (:wat::std::list::zip
        (:wat::core::map req-pairs
          (:wat::core::lambda ((p :wat::lru::ReqChannel<K,V>)
                              -> :wat::lru::ReqRx<K,V>)
            (:wat::core::second p)))
        (:wat::core::map reply-pairs
          (:wat::core::lambda ((p :wat::lru::ReplyChannel<V>)
                              -> :wat::lru::ReplyTx<V>)
            (:wat::core::first p)))))
     ((pool :wat::kernel::HandlePool<wat::lru::Handle<K,V>>)
      (:wat::kernel::HandlePool::new "CacheService" handles))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::lru::loop
            capacity driver-pairs reporter metrics-cadence)))))
    (:wat::core::Tuple pool driver)))
