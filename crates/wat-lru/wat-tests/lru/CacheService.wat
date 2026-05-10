;; arc-130 slice 1 reland — wat-lru CacheService stepping-stone tests.
;;
;; Top-down dependency graph in ONE file per /complectens.
;; Each layer adds ONE new thing; each layer carries its own deftest.
;; Failure trace names the broken layer by helper-function name.
;;
;; Substrate shape (post-arc-130):
;;   spawn returns Spawn<K,V> = (HandlePool<Handle<K,V>>, Thread<(),()>)
;;   Handle<K,V>  = (ReqTx<K,V>, ReplyRx<V>)
;;   Request<K,V> = Get(Vec<K>) | Put(Vec<Entry<K,V>>)
;;   Reply<V>     = GetResult(Vec<Option<V>>) | PutAck
;;
;; No make-bounded-channel in test code — arc 130 owns the channels.
;; Helper verbs :wat::lru::get / :wat::lru::put do send-AND-recv per
;; arc 110's contract; tests use them as the primary interface.
;; Layer order — top-down, no forward refs:
;;   Layer 0 — :test::lru-spawn-and-drop       lifecycle, no requests
;;   Layer 1 — :test::lru-helper-get-empty     one get round trip (empty probes)
;;   Layer 2 — :test::lru-helper-put-one       one put round trip (single entry)
;;   Layer 3 — :test::lru-helper-put-then-get  put-one + get-same-key round trip
;;   Layer 4 — :test::lru-helper-get-many-keys multi-key probe alignment

;; ─── Prelude — all layered helpers ──────────────────────────────────
;; Helpers spliced into each deftest via make-deftest because deftest's
;; sandbox does not capture outer-scope defines.

(:wat::test::make-deftest :deftest-lru
  (
   ;; ─── Layer 0 helper — spawn → pop → finish → drop → join ─────────
   (:wat::core::define
     (:test::lru-spawn-and-drop -> :wat::core::nil)
     (:wat::core::let
       [driver
         (:wat::core::let
           [spawn
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence))
            pool
             (:wat::core::first spawn)
            d
             (:wat::core::second spawn)
            _handle
             (:wat::kernel::HandlePool::pop pool)
            _finish
             (:wat::kernel::HandlePool::finish pool)]
           d)]
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::nil
         ((:wat::core::Ok _) :wat::core::nil)
         ((:wat::core::Err _) (:wat::test::assert-eq "lru-spawn-and-drop-died" "")))))

   ;; ─── Layer 1 helper — spawn → pop → get(empty) → finish → drop → join
   ;;
   ;; One :wat::lru::get call with an empty probes vec. The helper verb
   ;; does send-AND-recv internally per arc 110's contract — no raw
   ;; send/recv in test code. The inner let owns the pool + handle so
   ;; their Sender clones drop before the outer join; only (driver, n)
   ;; survive to the outer scope. Returns the result-vec length.
   (:wat::core::define
     (:test::lru-helper-get-empty -> :wat::core::i64)
     (:wat::core::let
       [driver-and-n
         (:wat::core::let
           [spawn
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence))
            pool
             (:wat::core::first spawn)
            d
             (:wat::core::second spawn)
            handle
             (:wat::kernel::HandlePool::pop pool)
            results
             (:wat::lru::get handle (:wat::core::Vector :wat::core::String))
            _finish
             (:wat::kernel::HandlePool::finish pool)]
           (:wat::core::Tuple d (:wat::core::Vector/length results)))
        driver
         (:wat::core::first driver-and-n)
        n (:wat::core::second driver-and-n)]
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) n)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "lru-helper-get-empty-died" "")
             n)))))

   ;; ─── Layer 2 helper — spawn → pop → put(one) → finish → drop → join
   ;;
   ;; One :wat::lru::put call with a single Entry. Helper-verb does
   ;; send-AND-recv internally; driver replies Reply::PutAck (unit).
   ;; Returns 1 on success — the deftest body asserts on that constant
   ;; so a missing PutAck (driver died, wrong reply variant) trips up.
   (:wat::core::define
     (:test::lru-helper-put-one -> :wat::core::i64)
     (:wat::core::let
       [driver
         (:wat::core::let
           [spawn
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence))
            pool
             (:wat::core::first spawn)
            d
             (:wat::core::second spawn)
            handle
             (:wat::kernel::HandlePool::pop pool)
            _put
             (:wat::lru::put handle
               (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>
                 (:wat::core::Tuple "k1" 42)))
            _finish
             (:wat::kernel::HandlePool::finish pool)]
           d)]
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) 1)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "lru-helper-put-one-died" "")
             0)))))

   ;; ─── Layer 3a sub-helper — put one entry then get the same key ───
   ;;
   ;; Pure handle-level work; no spawn/finish/join scaffolding. Puts
   ;; (k, v) then reads back results[0] from a single-key get. Panics
   ;; with named messages if the slot is missing or None.
   (:wat::core::define
     (:test::lru-put-then-get-on-handle
       (handle :wat::lru::Handle<wat::core::String,wat::core::i64>)
       (k :wat::core::String)
       (v :wat::core::i64)
       -> :wat::core::i64)
     (:wat::core::let
       [_put
         (:wat::lru::put handle
           (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>
             (:wat::core::Tuple k v)))
        results
         (:wat::lru::get handle (:wat::core::Vector :wat::core::String k))
        slot
         (:wat::core::Option/expect -> :wat::core::Option<wat::core::i64>
           (:wat::core::get results 0)
           "lru-put-then-get-on-handle: results vec is empty")]
       (:wat::core::Option/expect -> :wat::core::i64
         slot
         "lru-put-then-get-on-handle: get returned None for the put key")))

   ;; ─── Layer 3 helper — put-then-get round trip with full lifecycle ──
   ;;
   ;; THE HAPPY-PATH PROOF. Spawns the service, pops a handle, calls
   ;; the Layer 3a sub-helper (put → get → unwrap), tears the pool
   ;; down. Returns the round-tripped value (or 0 on Err join after
   ;; surfacing the death).
   (:wat::core::define
     (:test::lru-helper-put-then-get -> :wat::core::i64)
     (:wat::core::let
       [driver-and-v
         (:wat::core::let
           [spawn
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence))
            pool
             (:wat::core::first spawn)
            d
             (:wat::core::second spawn)
            handle
             (:wat::kernel::HandlePool::pop pool)
            v
             (:test::lru-put-then-get-on-handle handle "k1" 42)
            _finish
             (:wat::kernel::HandlePool::finish pool)]
           (:wat::core::Tuple d v))
        driver
         (:wat::core::first driver-and-v)
        v (:wat::core::second driver-and-v)]
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) v)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "lru-helper-put-then-get-died" "")
             0)))))

   ;; ─── Layer 4a sub-helper — score Option<i64> slot vs presence ─────
   ;;
   ;; Encodes "is this slot Some? then 1 else 0" so the multi-key
   ;; deftest can check index-by-index alignment against a known
   ;; presence pattern via a sum.
   (:wat::core::define
     (:test::lru-slot-presence
       (slot :wat::core::Option<wat::core::i64>) -> :wat::core::i64)
     (:wat::core::match slot -> :wat::core::i64
       ((:wat::core::Some _) 1)
       (:wat::core::None 0)))

   ;; ─── Layer 4b sub-helper — put two entries then probe three keys ──
   ;;
   ;; Returns the presence pattern as a packed i64: 100*p[0] + 10*p[1]
   ;; + p[2] where p[i] is :test::lru-slot-presence. Deftest asserts
   ;; against the literal 110 (Some, Some, None for "k1","k2","k3").
   (:wat::core::define
     (:test::lru-probe-three-on-handle
       (handle :wat::lru::Handle<wat::core::String,wat::core::i64>)
       -> :wat::core::i64)
     (:wat::core::let
       [_put
         (:wat::lru::put handle
           (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>
             (:wat::core::Tuple "k1" 11)
             (:wat::core::Tuple "k2" 22)))
        results
         (:wat::lru::get handle
           (:wat::core::Vector :wat::core::String "k1" "k2" "k3"))
        p0 (:test::lru-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::core::i64>
             (:wat::core::get results 0)
             "lru-probe-three-on-handle: results[0] missing"))
        p1 (:test::lru-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::core::i64>
             (:wat::core::get results 1)
             "lru-probe-three-on-handle: results[1] missing"))
        p2 (:test::lru-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::core::i64>
             (:wat::core::get results 2)
             "lru-probe-three-on-handle: results[2] missing"))]
       (:wat::core::i64::+'2
         (:wat::core::i64::+'2 (:wat::core::i64::*'2 p0 100) (:wat::core::i64::*'2 p1 10))
         p2)))

   ;; ─── Layer 4 helper — multi-key probe with full lifecycle ─────────
   ;;
   ;; Spawns the service, pops a handle, calls the Layer 4b sub-helper
   ;; (put-2 → probe-3 → score), tears the pool down. Returns the
   ;; packed presence pattern — the deftest body asserts against 110.
   (:wat::core::define
     (:test::lru-helper-get-many-keys -> :wat::core::i64)
     (:wat::core::let
       [driver-and-pat
         (:wat::core::let
           [spawn
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence))
            pool
             (:wat::core::first spawn)
            d
             (:wat::core::second spawn)
            handle
             (:wat::kernel::HandlePool::pop pool)
            pat
             (:test::lru-probe-three-on-handle handle)
            _finish
             (:wat::kernel::HandlePool::finish pool)]
           (:wat::core::Tuple d pat))
        driver
         (:wat::core::first driver-and-pat)
        pat (:wat::core::second driver-and-pat)]
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) pat)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "lru-helper-get-many-keys-died" "")
             0)))))
   ))

;; ─── Layer 0 — :test::lru-spawn-and-drop ────────────────────────────
;;
;; Narrowest proof: post-arc-130 spawn/shutdown lifecycle works.
;; No request, no send, no recv. Pop one handle (required before finish
;; per HandlePool contract); finish pool; let handle drop at inner
;; scope exit → driver sees disconnect → outer Thread/join-result
;; unblocks.

(:wat::test::time-limit "200ms")
(:deftest-lru :wat-lru::test-lru-spawn-and-drop
  (:test::lru-spawn-and-drop))

;; ─── Layer 1 — :test::lru-helper-get-empty ──────────────────────────
;;
;; Helper-verb call site. :wat::lru::get on an empty probes vec returns
;; an empty Vec<Option<V>>. The helper-verb internally sends Request::Get
;; and recvs Reply::GetResult — driver gets a clean shutdown after pool
;; finish + handle drop.

(:wat::test::time-limit "200ms")
(:deftest-lru :wat-lru::test-lru-helper-get-empty
  (:wat::test::assert-eq (:test::lru-helper-get-empty) 0))

;; ─── Layer 2 — :test::lru-helper-put-one ────────────────────────────
;;
;; Helper-verb call site for :wat::lru::put. Single-entry batch round
;; trips Request::Put → Reply::PutAck. Helper returns 1 on Ok join,
;; 0 on Err join (after surfacing the death).

(:wat::test::time-limit "200ms")
(:deftest-lru :wat-lru::test-lru-helper-put-one
  (:wat::test::assert-eq (:test::lru-helper-put-one) 1))

;; ─── Layer 3 — :test::lru-helper-put-then-get ────────────────────────
;;
;; The happy-path round trip. Composes :wat::lru::put + :wat::lru::get
;; on a single handle. v=42 goes in via Put; comes back via Get's
;; results[0] = Some(42). Proves Reply<V> enum routing for both
;; variants on the same channel.

(:wat::test::time-limit "200ms")
(:deftest-lru :wat-lru::test-lru-helper-put-then-get
  (:wat::test::assert-eq (:test::lru-helper-put-then-get) 42))

;; ─── Layer 4 — :test::lru-helper-get-many-keys ────────────────────────
;;
;; Multi-key probe alignment. Put "k1"=11 + "k2"=22; probe ["k1","k2","k3"];
;; presence pattern as packed digits = 110 (Some Some None). Proves
;; result-vec aligns with probe-vec by index — the contract Reply<V>'s
;; GetResult variant carries.

(:wat::test::time-limit "200ms")
(:deftest-lru :wat-lru::test-lru-helper-get-many-keys
  (:wat::test::assert-eq (:test::lru-helper-get-many-keys) 110))
