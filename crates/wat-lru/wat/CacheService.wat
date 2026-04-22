;; :user::wat::std::lru::CacheService — wat-lru's multi-client LRU
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
;; Protocol:
;;   Body<K,V>     = (tag :i64, key :K, put-val :Option<V>)
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
(:wat::core::typealias :user::wat::std::lru::CacheService::Body<K,V>
  :(i64,K,Option<V>))
(:wat::core::typealias :user::wat::std::lru::CacheService::ReplyTx<V>
  :rust::crossbeam_channel::Sender<Option<V>>)
(:wat::core::typealias :user::wat::std::lru::CacheService::Request<K,V>
  :(user::wat::std::lru::CacheService::Body<K,V>,user::wat::std::lru::CacheService::ReplyTx<V>))
(:wat::core::typealias :user::wat::std::lru::CacheService::ReqTx<K,V>
  :rust::crossbeam_channel::Sender<user::wat::std::lru::CacheService::Request<K,V>>)
(:wat::core::typealias :user::wat::std::lru::CacheService::ReqRx<K,V>
  :rust::crossbeam_channel::Receiver<user::wat::std::lru::CacheService::Request<K,V>>)

;; Driver entry — allocates the LocalCache INSIDE the driver thread
;; (LocalCache is thread-owned; creating it in the caller and passing
;; across threads would trip the thread-id guard and wedge the
;; driver). Then delegates to `CacheService/loop-step` for the recursion.
(:wat::core::define
  (:user::wat::std::lru::CacheService/loop<K,V>
    (capacity :i64)
    (req-rxs :Vec<user::wat::std::lru::CacheService::ReqRx<K,V>>)
    -> :())
  (:wat::core::let*
    (((cache :user::wat::std::lru::LocalCache<K,V>)
      (:user::wat::std::lru::LocalCache::new capacity)))
    (:user::wat::std::lru::CacheService/loop-step cache req-rxs)))

;; Recursive inner loop. Owns the cache for the duration of the
;; driver thread's lifetime; select across request receivers; each
;; request carries its reply-to sender for routing.
(:wat::core::define
  (:user::wat::std::lru::CacheService/loop-step<K,V>
    (cache :user::wat::std::lru::LocalCache<K,V>)
    (req-rxs :Vec<user::wat::std::lru::CacheService::ReqRx<K,V>>)
    -> :())
  (:wat::core::if (:wat::core::empty? req-rxs) -> :()
    ()
    (:wat::core::let*
      (((chosen :(i64,Option<user::wat::std::lru::CacheService::Request<K,V>>))
        (:wat::kernel::select req-rxs))
       ((idx :i64) (:wat::core::first chosen))
       ((maybe :Option<user::wat::std::lru::CacheService::Request<K,V>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :()
        ((Some req)
          (:wat::core::let*
            (((body :user::wat::std::lru::CacheService::Body<K,V>) (:wat::core::first req))
             ((reply-to :user::wat::std::lru::CacheService::ReplyTx<V>)
              (:wat::core::second req))
             ((tag :i64) (:wat::core::first body))
             ((key :K) (:wat::core::second body))
             ((put-val :Option<V>) (:wat::core::third body))
             ((resp :Option<V>)
              (:wat::core::if (:wat::core::= tag 0) -> :Option<V>
                (:user::wat::std::lru::LocalCache::get cache key)
                (:wat::core::match put-val -> :Option<V>
                  ((Some v)
                    (:wat::core::let*
                      (((_ :()) (:user::wat::std::lru::LocalCache::put cache key v)))
                      :None))
                  (:None :None))))
             ;; reply-to may have been dropped (client no longer
             ;; interested). `send` returns :Option<()>; either
             ;; outcome leaves the driver free to carry on — we
             ;; swallow :None.
             ((_ :Option<()>) (:wat::kernel::send reply-to resp)))
            (:user::wat::std::lru::CacheService/loop-step cache req-rxs)))
        (:None
          (:user::wat::std::lru::CacheService/loop-step
            cache
            (:wat::std::list::remove-at req-rxs idx)))))))

;; --- Client helpers ---
;;
;; A client creates its response channel once at setup and reuses it
;; for every request. CacheService/get and CacheService/put package
;; the request, send it, and block on the response.

(:wat::core::define
  (:user::wat::std::lru::CacheService/get<K,V>
    (req-tx :user::wat::std::lru::CacheService::ReqTx<K,V>)
    (reply-tx :user::wat::std::lru::CacheService::ReplyTx<V>)
    (reply-rx :rust::crossbeam_channel::Receiver<Option<V>>)
    (key :K)
    -> :Option<V>)
  (:wat::core::let*
    (((body :user::wat::std::lru::CacheService::Body<K,V>)
      (:wat::core::tuple 0 key :None))
     ((req :user::wat::std::lru::CacheService::Request<K,V>)
      (:wat::core::tuple body reply-tx))
     ;; If the driver dropped before we wrote, `send` returns :None.
     ;; The subsequent `recv` will then also see the reply-tx dropped
     ;; and return :None, so either way we fall through to the
     ;; :None arm below — caller observes "miss."
     ((_ :Option<()>) (:wat::kernel::send req-tx req)))
    (:wat::core::match (:wat::kernel::recv reply-rx) -> :Option<V>
      ((Some resp) resp)
      (:None :None))))

(:wat::core::define
  (:user::wat::std::lru::CacheService/put<K,V>
    (req-tx :user::wat::std::lru::CacheService::ReqTx<K,V>)
    (reply-tx :user::wat::std::lru::CacheService::ReplyTx<V>)
    (reply-rx :rust::crossbeam_channel::Receiver<Option<V>>)
    (key :K)
    (value :V)
    -> :())
  (:wat::core::let*
    (((body :user::wat::std::lru::CacheService::Body<K,V>)
      (:wat::core::tuple 1 key (Some value)))
     ((req :user::wat::std::lru::CacheService::Request<K,V>)
      (:wat::core::tuple body reply-tx))
     ;; Same swallow as CacheService/get above: either send lands and
     ;; the recv acks, or the driver is gone and both short-circuit
     ;; through :None. Callers receive :() either way.
     ((_ :Option<()>) (:wat::kernel::send req-tx req))
     ((_ :Option<Option<V>>) (:wat::kernel::recv reply-rx)))
    ()))

;; --- CacheService setup ---
;;
;; Creates N bounded(1) request queues, wraps senders in a HandlePool,
;; spawns one driver thread that owns a fresh LocalCache<K,V> of the
;; given capacity and fans in all request receivers. Returns the
;; (pool, driver-handle) pair.
(:wat::core::define
  (:user::wat::std::lru::CacheService<K,V>
    (capacity :i64)
    (count :i64)
    -> :(wat::kernel::HandlePool<user::wat::std::lru::CacheService::ReqTx<K,V>>,wat::kernel::ProgramHandle<()>))
  (:wat::core::let*
    (((pairs :Vec<(user::wat::std::lru::CacheService::ReqTx<K,V>,user::wat::std::lru::CacheService::ReqRx<K,V>)>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda ((_i :i64) -> :(user::wat::std::lru::CacheService::ReqTx<K,V>,user::wat::std::lru::CacheService::ReqRx<K,V>))
          (:wat::kernel::make-bounded-queue :user::wat::std::lru::CacheService::Request<K,V> 1))))
     ((req-txs :Vec<user::wat::std::lru::CacheService::ReqTx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :(user::wat::std::lru::CacheService::ReqTx<K,V>,user::wat::std::lru::CacheService::ReqRx<K,V>))
                            -> :user::wat::std::lru::CacheService::ReqTx<K,V>)
          (:wat::core::first p))))
     ((req-rxs :Vec<user::wat::std::lru::CacheService::ReqRx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :(user::wat::std::lru::CacheService::ReqTx<K,V>,user::wat::std::lru::CacheService::ReqRx<K,V>))
                            -> :user::wat::std::lru::CacheService::ReqRx<K,V>)
          (:wat::core::second p))))
     ((pool :wat::kernel::HandlePool<user::wat::std::lru::CacheService::ReqTx<K,V>>)
      (:wat::kernel::HandlePool::new "CacheService" req-txs))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :user::wat::std::lru::CacheService/loop capacity req-rxs)))
    (:wat::core::tuple pool driver)))
