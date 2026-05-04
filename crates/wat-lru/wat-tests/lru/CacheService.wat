;; arc-130 slice 1 reland — CacheService stepping-stone proofs.
;;
;; Top-down dependency graph in ONE file per /complectens.
;; Each layer adds ONE new thing. Each layer carries its own deftest.
;; Failure trace names the broken layer by function name.
;;
;; Substrate shape (post-arc-130):
;;   spawn returns Spawn<K,V> = (HandlePool<Handle<K,V>>, Thread<(),()>)
;;   Handle<K,V>  = (ReqTx<K,V>, ReplyRx<V>)
;;   Request<K,V> = Get(Vec<K>) | Put(Vec<Entry<K,V>>)
;;   Reply<V>     = GetResult(Vec<Option<V>>) | PutAck
;;
;; No make-bounded-channel in test code — arc 130 owns the channels.
;; Arc 126's check does not fire (handle's two halves come from
;; different pre-allocated channels; no single make-bounded-channel
;; anchor).

;; ─── Layer 0 — spawn → pop → finish → join ──────────────────────────
;;
;; Narrowest possible proof: the post-arc-130 spawn/shutdown lifecycle
;; works. No request, no send, no recv. Pop one handle (required before
;; finish per HandlePool contract); finish pool; let handle drop at inner
;; scope exit → driver sees disconnect → outer Thread/join-result
;; unblocks.

(:wat::test::time-limit "200ms")
(:wat::test::deftest :wat-lru::test-lru-spawn-and-drop
  ()
  (:wat::core::let*
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::lru::Spawn<wat::core::String,wat::core::i64>)
          (:wat::lru::spawn 16 1
            :wat::lru::null-reporter
            (:wat::lru::null-metrics-cadence)))
         ((pool :wat::kernel::HandlePool<wat::lru::Handle<wat::core::String,wat::core::i64>>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((_handle :wat::lru::Handle<wat::core::String,wat::core::i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit)
          (:wat::kernel::HandlePool::finish pool)))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "layer0-service-died" "")))))

;; ─── Layer 1 — raw send Request::Get, no recv ───────────────────────
;;
;; Layer 0 + one raw send of Request::Get (empty probes) on req-tx.
;; No recv of the reply. The inner scope drops the handle before the
;; driver can deliver its reply.
;;
;; DIAGNOSTIC LAYER — surfaces the substrate bug:
;;   CacheService/handle's Get branch calls :wat::core::reduce to count
;;   hit/miss — but :wat::core::reduce is not in the runtime's vocabulary.
;;   Driver panics at CacheService.wat:213 "unknown function: :wat::core::reduce".
;;   Thread/join-result returns Err; the test reveals the panic message.
;;
;; The assert-eq below will FAIL with the driver's panic message as
;; "actual" — this is the intended diagnostic output for this reland.

(:wat::test::time-limit "200ms")
(:wat::test::deftest :wat-lru::test-lru-raw-send-no-recv
  ()
  (:wat::core::let*
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::lru::Spawn<wat::core::String,wat::core::i64>)
          (:wat::lru::spawn 16 1
            :wat::lru::null-reporter
            (:wat::lru::null-metrics-cadence)))
         ((pool :wat::kernel::HandlePool<wat::lru::Handle<wat::core::String,wat::core::i64>>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((handle :wat::lru::Handle<wat::core::String,wat::core::i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit)
          (:wat::kernel::HandlePool::finish pool))
         ((_send :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send
              (:wat::core::first handle)
              (:wat::lru::Request::Get
                (:wat::core::Vector :wat::core::String)))
            "raw-send-no-recv: req-tx disconnected — driver died?")))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err errs)
       (:wat::test::assert-eq
         (:wat::kernel::ThreadDiedError/message
           (:wat::core::Option/expect -> :wat::kernel::ThreadDiedError
             (:wat::core::get errs 0)
             "no errors in Err"))
         "")))))
