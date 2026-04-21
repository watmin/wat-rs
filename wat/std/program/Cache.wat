;; :wat::std::program::Cache<K,V> — the L2 tier of 058 FOUNDATION's
;; caching stack (lines 1527-1565). A program that owns its own
;; LocalCache<K,V> behind a select loop; clients send requests with
;; their own reply address attached so the driver routes responses
;; without a sender-index map.
;;
;; Generic over K,V — type params propagate through every define via
;; wat's `<K,V>` declaration syntax, same pattern LocalCache.wat uses.
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
;; The four parts above are typealiases declared below, registered
;; through the stdlib's privileged `register_stdlib_types` path so
;; they can live under `:wat::std::program::Cache::*`. Every signature
;; in the file uses the alias names; `reduce` (src/check.rs) walks
;; through at unification + shape-inspection sites, so
;; `:wat::std::program::Cache::Request<i64,String>` and its tuple
;; expansion are interchangeable everywhere.

(:wat::core::use! :rust::crossbeam_channel::Sender)
(:wat::core::use! :rust::crossbeam_channel::Receiver)

;; --- Protocol typealiases ---
(:wat::core::typealias :wat::std::program::Cache::Body<K,V>
  :(i64,K,Option<V>))
(:wat::core::typealias :wat::std::program::Cache::ReplyTx<V>
  :rust::crossbeam_channel::Sender<Option<V>>)
(:wat::core::typealias :wat::std::program::Cache::Request<K,V>
  :(wat::std::program::Cache::Body<K,V>,wat::std::program::Cache::ReplyTx<V>))
(:wat::core::typealias :wat::std::program::Cache::ReqTx<K,V>
  :rust::crossbeam_channel::Sender<wat::std::program::Cache::Request<K,V>>)
(:wat::core::typealias :wat::std::program::Cache::ReqRx<K,V>
  :rust::crossbeam_channel::Receiver<wat::std::program::Cache::Request<K,V>>)

;; Driver entry — allocates the LocalCache INSIDE the driver thread
;; (LocalCache is thread-owned; creating it in the caller and passing
;; across threads would trip the thread-id guard and wedge the
;; driver). Then delegates to `Cache/loop-step` for the recursion.
(:wat::core::define
  (:wat::std::program::Cache/loop<K,V>
    (capacity :i64)
    (req-rxs :Vec<wat::std::program::Cache::ReqRx<K,V>>)
    -> :())
  (:wat::core::let*
    (((cache :wat::std::LocalCache<K,V>)
      (:wat::std::LocalCache::new capacity)))
    (:wat::std::program::Cache/loop-step cache req-rxs)))

;; Recursive inner loop. Owns the cache for the duration of the
;; driver thread's lifetime; select across request receivers; each
;; request carries its reply-to sender for routing.
(:wat::core::define
  (:wat::std::program::Cache/loop-step<K,V>
    (cache :wat::std::LocalCache<K,V>)
    (req-rxs :Vec<wat::std::program::Cache::ReqRx<K,V>>)
    -> :())
  (:wat::core::if (:wat::core::empty? req-rxs) -> :()
    ()
    (:wat::core::let*
      (((chosen :(i64,Option<wat::std::program::Cache::Request<K,V>>))
        (:wat::kernel::select req-rxs))
       ((idx :i64) (:wat::core::first chosen))
       ((maybe :Option<wat::std::program::Cache::Request<K,V>>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :()
        ((Some req)
          (:wat::core::let*
            (((body :wat::std::program::Cache::Body<K,V>) (:wat::core::first req))
             ((reply-to :wat::std::program::Cache::ReplyTx<V>)
              (:wat::core::second req))
             ((tag :i64) (:wat::core::first body))
             ((key :K) (:wat::core::second body))
             ((put-val :Option<V>) (:wat::core::third body))
             ((resp :Option<V>)
              (:wat::core::if (:wat::core::= tag 0) -> :Option<V>
                (:wat::std::LocalCache::get cache key)
                (:wat::core::match put-val -> :Option<V>
                  ((Some v)
                    (:wat::core::let*
                      (((_ :()) (:wat::std::LocalCache::put cache key v)))
                      :None))
                  (:None :None))))
             ;; reply-to may have been dropped (client no longer
             ;; interested). `send` now returns :Option<()>; either
             ;; outcome leaves the driver free to carry on — we
             ;; swallow :None.
             ((_ :Option<()>) (:wat::kernel::send reply-to resp)))
            (:wat::std::program::Cache/loop-step cache req-rxs)))
        (:None
          (:wat::std::program::Cache/loop-step
            cache
            (:wat::std::list::remove-at req-rxs idx)))))))

;; --- Client helpers ---
;;
;; A client creates its response channel once at setup and reuses it
;; for every request. Cache/get and Cache/put package the request,
;; send it, and block on the response.

(:wat::core::define
  (:wat::std::program::Cache/get<K,V>
    (req-tx :wat::std::program::Cache::ReqTx<K,V>)
    (reply-tx :wat::std::program::Cache::ReplyTx<V>)
    (reply-rx :rust::crossbeam_channel::Receiver<Option<V>>)
    (key :K)
    -> :Option<V>)
  (:wat::core::let*
    (((body :wat::std::program::Cache::Body<K,V>)
      (:wat::core::tuple 0 key :None))
     ((req :wat::std::program::Cache::Request<K,V>)
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
  (:wat::std::program::Cache/put<K,V>
    (req-tx :wat::std::program::Cache::ReqTx<K,V>)
    (reply-tx :wat::std::program::Cache::ReplyTx<V>)
    (reply-rx :rust::crossbeam_channel::Receiver<Option<V>>)
    (key :K)
    (value :V)
    -> :())
  (:wat::core::let*
    (((body :wat::std::program::Cache::Body<K,V>)
      (:wat::core::tuple 1 key (Some value)))
     ((req :wat::std::program::Cache::Request<K,V>)
      (:wat::core::tuple body reply-tx))
     ;; Same swallow as Cache/get above: either send lands and the
     ;; recv acks, or the driver is gone and both short-circuit
     ;; through :None. Callers receive :() either way.
     ((_ :Option<()>) (:wat::kernel::send req-tx req))
     ((_ :Option<Option<V>>) (:wat::kernel::recv reply-rx)))
    ()))

;; --- Cache setup ---
;;
;; Creates N bounded(1) request queues, wraps senders in a HandlePool,
;; spawns one driver thread that owns a fresh LocalCache<K,V> of the
;; given capacity and fans in all request receivers. Returns the
;; (pool, driver-handle) pair.
(:wat::core::define
  (:wat::std::program::Cache<K,V>
    (capacity :i64)
    (count :i64)
    -> :(wat::kernel::HandlePool<wat::std::program::Cache::ReqTx<K,V>>,wat::kernel::ProgramHandle<()>))
  (:wat::core::let*
    (((pairs :Vec<(wat::std::program::Cache::ReqTx<K,V>,wat::std::program::Cache::ReqRx<K,V>)>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda ((_i :i64) -> :(wat::std::program::Cache::ReqTx<K,V>,wat::std::program::Cache::ReqRx<K,V>))
          (:wat::kernel::make-bounded-queue :wat::std::program::Cache::Request<K,V> 1))))
     ((req-txs :Vec<wat::std::program::Cache::ReqTx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :(wat::std::program::Cache::ReqTx<K,V>,wat::std::program::Cache::ReqRx<K,V>))
                            -> :wat::std::program::Cache::ReqTx<K,V>)
          (:wat::core::first p))))
     ((req-rxs :Vec<wat::std::program::Cache::ReqRx<K,V>>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :(wat::std::program::Cache::ReqTx<K,V>,wat::std::program::Cache::ReqRx<K,V>))
                            -> :wat::std::program::Cache::ReqRx<K,V>)
          (:wat::core::second p))))
     ((pool :wat::kernel::HandlePool<wat::std::program::Cache::ReqTx<K,V>>)
      (:wat::kernel::HandlePool::new "Cache" req-txs))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::program::Cache/loop capacity req-rxs)))
    (:wat::core::tuple pool driver)))
