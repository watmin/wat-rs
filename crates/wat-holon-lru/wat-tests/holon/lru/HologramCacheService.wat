;; arc-130 sweep 2b — wat-holon-lru HologramCacheService stepping-stone tests.
;;
;; Top-down dependency graph in ONE file per /complectens.
;; Each layer adds ONE new thing; each layer carries its own deftest.
;; Failure trace names the broken layer by helper-function name.
;;
;; Substrate shape (post-arc-130 sweep 2a):
;;   spawn returns Spawn = (HandlePool<Handle>, Thread<unit,unit>)
;;   Handle      = (ReqTx, ReplyRx)
;;   DriverPair  = (ReqRx, ReplyTx)
;;   Request     = Get(Vec<HolonAST>) | Put(Vec<Entry>)
;;   Reply       = GetResult(Vec<Option<HolonAST>>) | PutAck
;;
;; Concrete typing — K = V = HolonAST throughout (no <K,V> parameters).
;; The Reply enum unifies what used to be per-verb ack/reply channels;
;; helper verbs :wat::holon::lru::HologramCacheService::get / ::put do
;; send-AND-recv internally per arc 110 — tests use them as the primary
;; interface. NO raw kernel::send / kernel::recv in test code.
;;
;; Layer order — top-down, no forward refs:
;;   Layer 0 — :test::hcs-spawn-and-drop       lifecycle, no requests
;;   Layer 1 — :test::hcs-helper-get-empty     one get round trip (empty probes)
;;   Layer 2 — :test::hcs-helper-put-one       one put round trip (single entry)
;;   Layer 3 — :test::hcs-helper-put-then-get  put-one + get-same-key round trip
;;   Layer 4 — :test::hcs-helper-get-many-keys multi-key probe alignment
;;   Layer 5 — :test::hcs-eviction              cap=2; put 3; first key evicts
;;   Layer 6 — :test::hcs-multi-client          spawn count=2; two handles, two keys

;; ─── Prelude — all layered helpers ──────────────────────────────────
;; Helpers spliced into each deftest via make-deftest because deftest's
;; sandbox does not capture outer-scope defines. ONE prelude — the
;; post-arc-130 substrate has no channel-pair-deadlock pattern, so
;; the two-prelude split that the prior test file used is no longer
;; required.

(:wat::test::make-deftest :deftest-hcs
  (
   ;; ─── Layer 0 helper — spawn → pop → finish → drop → join ─────────
   ;;
   ;; Narrowest proof: post-arc-130 spawn/shutdown lifecycle works.
   ;; Pop one handle (required before finish per HandlePool contract);
   ;; finish pool; let handle drop at inner scope exit → driver sees
   ;; disconnect → outer Thread/join-result unblocks. Outer scope owns
   ;; only the driver Thread; inner scope owns spawn/pool/handle so
   ;; their Sender clones drop before outer join (arc 117/126 check).
   (:wat::core::define
     (:test::hcs-spawn-and-drop -> :wat::core::nil)
     (:wat::core::let
       ((driver
         (:wat::core::let
           ((spawn
             (:wat::holon::lru::HologramCacheService/spawn 1 16
               :wat::holon::lru::HologramCacheService/null-reporter
               (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
            (pool
             (:wat::core::first spawn))
            (d
             (:wat::core::second spawn))
            (_handle
             (:wat::kernel::HandlePool::pop pool))
            (_finish
             (:wat::kernel::HandlePool::finish pool)))
           d)))
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::nil
         ((:wat::core::Ok _) :wat::core::nil)
         ((:wat::core::Err _) (:wat::test::assert-eq "hcs-spawn-and-drop-died" "")))))

   ;; ─── Layer 1 helper — spawn → pop → get(empty) → finish → drop → join
   ;;
   ;; One :wat::holon::lru::HologramCacheService::get call with an empty
   ;; probes vec. The helper verb does send-AND-recv internally per arc
   ;; 110's contract — no raw send/recv in test code. The inner let
   ;; owns the pool + handle so their Sender clones drop before the
   ;; outer join; only (driver, n) survive to the outer scope. Returns
   ;; the result-vec length.
   (:wat::core::define
     (:test::hcs-helper-get-empty -> :wat::core::i64)
     (:wat::core::let
       ((driver-and-n
         (:wat::core::let
           ((spawn
             (:wat::holon::lru::HologramCacheService/spawn 1 16
               :wat::holon::lru::HologramCacheService/null-reporter
               (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
            (pool
             (:wat::core::first spawn))
            (d
             (:wat::core::second spawn))
            (handle
             (:wat::kernel::HandlePool::pop pool))
            (results
             (:wat::holon::lru::HologramCacheService/get handle
               (:wat::core::Vector :wat::holon::HolonAST)))
            (_finish
             (:wat::kernel::HandlePool::finish pool)))
           (:wat::core::Tuple d (:wat::core::Vector/length results))))
        (driver
         (:wat::core::first driver-and-n))
        (n (:wat::core::second driver-and-n)))
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) n)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "hcs-helper-get-empty-died" "")
             n)))))

   ;; ─── Layer 2 helper — spawn → pop → put(one) → finish → drop → join
   ;;
   ;; One :wat::holon::lru::HologramCacheService::put call with a single
   ;; Entry. Helper-verb does send-AND-recv internally; driver replies
   ;; Reply::PutAck. Returns 1 on success — the deftest body asserts on
   ;; that constant so a missing PutAck (driver died, wrong reply variant)
   ;; trips up.
   (:wat::core::define
     (:test::hcs-helper-put-one -> :wat::core::i64)
     (:wat::core::let
       ((driver
         (:wat::core::let
           ((spawn
             (:wat::holon::lru::HologramCacheService/spawn 1 16
               :wat::holon::lru::HologramCacheService/null-reporter
               (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
            (pool
             (:wat::core::first spawn))
            (d
             (:wat::core::second spawn))
            (handle
             (:wat::kernel::HandlePool::pop pool))
            (_put
             (:wat::holon::lru::HologramCacheService/put handle
               (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry
                 (:wat::core::Tuple
                   (:wat::holon::leaf :alpha)
                   (:wat::holon::leaf :av)))))
            (_finish
             (:wat::kernel::HandlePool::finish pool)))
           d)))
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) 1)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "hcs-helper-put-one-died" "")
             0)))))

   ;; ─── Layer 3a sub-helper — score Option<HolonAST> slot vs presence ─
   ;;
   ;; Encodes "is this slot Some(Some(_))? then 1 else 0" so the
   ;; round-trip + multi-key deftests can check presence by index via
   ;; integer arithmetic. HolonAST values aren't trivially i64-comparable;
   ;; presence-pattern reduces the assertion surface to packed digits.
   (:wat::core::define
     (:test::hcs-slot-presence
       (slot :wat::core::Option<wat::holon::HolonAST>) -> :wat::core::i64)
     (:wat::core::match slot -> :wat::core::i64
       ((:wat::core::Some _) 1)
       (:wat::core::None 0)))

   ;; ─── Layer 3b sub-helper — put one entry then get the same key ────
   ;;
   ;; Pure handle-level work; no spawn/finish/join scaffolding. Puts
   ;; (k, v) then reads back results[0] from a single-key get. Returns
   ;; 1 on hit, 0 on miss. Panics with a named message if the slot is
   ;; missing from the result vec entirely.
   (:wat::core::define
     (:test::hcs-put-then-get-on-handle
       (handle :wat::holon::lru::HologramCacheService::Handle)
       (k :wat::holon::HolonAST)
       (v :wat::holon::HolonAST)
       -> :wat::core::i64)
     (:wat::core::let
       ((_put
         (:wat::holon::lru::HologramCacheService/put handle
           (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry
             (:wat::core::Tuple k v))))
        (results
         (:wat::holon::lru::HologramCacheService/get handle
           (:wat::core::Vector :wat::holon::HolonAST k)))
        (slot
         (:wat::core::Option/expect -> :wat::core::Option<wat::holon::HolonAST>
           (:wat::core::get results 0)
           "hcs-put-then-get-on-handle: results vec is empty")))
       (:test::hcs-slot-presence slot)))

   ;; ─── Layer 3 helper — put-then-get round trip with full lifecycle ──
   ;;
   ;; THE HAPPY-PATH PROOF. Spawns the service, pops a handle, calls
   ;; the Layer 3b sub-helper (put → get → score), tears the pool
   ;; down. Returns 1 on Some hit, 0 on miss / Err join.
   (:wat::core::define
     (:test::hcs-helper-put-then-get -> :wat::core::i64)
     (:wat::core::let
       ((driver-and-p
         (:wat::core::let
           ((spawn
             (:wat::holon::lru::HologramCacheService/spawn 1 16
               :wat::holon::lru::HologramCacheService/null-reporter
               (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
            (pool
             (:wat::core::first spawn))
            (d
             (:wat::core::second spawn))
            (handle
             (:wat::kernel::HandlePool::pop pool))
            (p
             (:test::hcs-put-then-get-on-handle handle
               (:wat::holon::leaf :alpha)
               (:wat::holon::leaf :av)))
            (_finish
             (:wat::kernel::HandlePool::finish pool)))
           (:wat::core::Tuple d p)))
        (driver
         (:wat::core::first driver-and-p))
        (p (:wat::core::second driver-and-p)))
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) p)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "hcs-helper-put-then-get-died" "")
             0)))))

   ;; ─── Layer 4a sub-helper — put two entries then probe three keys ──
   ;;
   ;; Returns the presence pattern as a packed i64: 100*p[0] + 10*p[1]
   ;; + p[2] where p[i] is :test::hcs-slot-presence. Deftest asserts
   ;; against the literal 110 (Some, Some, None for "alpha","beta","gamma").
   (:wat::core::define
     (:test::hcs-probe-three-on-handle
       (handle :wat::holon::lru::HologramCacheService::Handle)
       -> :wat::core::i64)
     (:wat::core::let
       ((_put
         (:wat::holon::lru::HologramCacheService/put handle
           (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry
             (:wat::core::Tuple (:wat::holon::leaf :alpha) (:wat::holon::leaf :av))
             (:wat::core::Tuple (:wat::holon::leaf :beta)  (:wat::holon::leaf :bv)))))
        (results
         (:wat::holon::lru::HologramCacheService/get handle
           (:wat::core::Vector :wat::holon::HolonAST
             (:wat::holon::leaf :alpha)
             (:wat::holon::leaf :beta)
             (:wat::holon::leaf :gamma))))
        (p0 (:test::hcs-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::holon::HolonAST>
             (:wat::core::get results 0)
             "hcs-probe-three-on-handle: results[0] missing")))
        (p1 (:test::hcs-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::holon::HolonAST>
             (:wat::core::get results 1)
             "hcs-probe-three-on-handle: results[1] missing")))
        (p2 (:test::hcs-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::holon::HolonAST>
             (:wat::core::get results 2)
             "hcs-probe-three-on-handle: results[2] missing"))))
       (:wat::core::i64::+,2
         (:wat::core::i64::+,2 (:wat::core::i64::*,2 p0 100) (:wat::core::i64::*,2 p1 10))
         p2)))

   ;; ─── Layer 4 helper — multi-key probe with full lifecycle ─────────
   ;;
   ;; Spawns the service, pops a handle, calls the Layer 4a sub-helper
   ;; (put-2 → probe-3 → score), tears the pool down. Returns the
   ;; packed presence pattern — the deftest body asserts against 110.
   (:wat::core::define
     (:test::hcs-helper-get-many-keys -> :wat::core::i64)
     (:wat::core::let
       ((driver-and-pat
         (:wat::core::let
           ((spawn
             (:wat::holon::lru::HologramCacheService/spawn 1 16
               :wat::holon::lru::HologramCacheService/null-reporter
               (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
            (pool
             (:wat::core::first spawn))
            (d
             (:wat::core::second spawn))
            (handle
             (:wat::kernel::HandlePool::pop pool))
            (pat
             (:test::hcs-probe-three-on-handle handle))
            (_finish
             (:wat::kernel::HandlePool::finish pool)))
           (:wat::core::Tuple d pat)))
        (driver
         (:wat::core::first driver-and-pat))
        (pat (:wat::core::second driver-and-pat)))
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) pat)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "hcs-helper-get-many-keys-died" "")
             0)))))

   ;; ─── Layer 5a sub-helper — put 3 keys at cap=2, probe all 3 ───────
   ;;
   ;; LRU eviction: with cap=2, putting k1, k2, k3 evicts k1 (the
   ;; oldest). Probe ["alpha","beta","gamma"]; presence pattern packed
   ;; as 100*p[0] + 10*p[1] + p[2] = 011 (None Some Some).
   (:wat::core::define
     (:test::hcs-eviction-on-handle
       (handle :wat::holon::lru::HologramCacheService::Handle)
       -> :wat::core::i64)
     (:wat::core::let
       ((_put
         (:wat::holon::lru::HologramCacheService/put handle
           (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry
             (:wat::core::Tuple (:wat::holon::leaf :alpha) (:wat::holon::leaf :av))
             (:wat::core::Tuple (:wat::holon::leaf :beta)  (:wat::holon::leaf :bv))
             (:wat::core::Tuple (:wat::holon::leaf :gamma) (:wat::holon::leaf :gv)))))
        (results
         (:wat::holon::lru::HologramCacheService/get handle
           (:wat::core::Vector :wat::holon::HolonAST
             (:wat::holon::leaf :alpha)
             (:wat::holon::leaf :beta)
             (:wat::holon::leaf :gamma))))
        (p0 (:test::hcs-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::holon::HolonAST>
             (:wat::core::get results 0)
             "hcs-eviction-on-handle: results[0] missing")))
        (p1 (:test::hcs-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::holon::HolonAST>
             (:wat::core::get results 1)
             "hcs-eviction-on-handle: results[1] missing")))
        (p2 (:test::hcs-slot-presence
           (:wat::core::Option/expect -> :wat::core::Option<wat::holon::HolonAST>
             (:wat::core::get results 2)
             "hcs-eviction-on-handle: results[2] missing"))))
       (:wat::core::i64::+,2
         (:wat::core::i64::+,2 (:wat::core::i64::*,2 p0 100) (:wat::core::i64::*,2 p1 10))
         p2)))

   ;; ─── Layer 5 helper — eviction at cap=2 with full lifecycle ───────
   ;;
   ;; Spawns the service at cap=2, pops a handle, calls Layer 5a's
   ;; eviction sub-helper (put-3 → probe-3 → score), tears down.
   ;; Returns the packed presence pattern — deftest body asserts 011.
   (:wat::core::define
     (:test::hcs-eviction -> :wat::core::i64)
     (:wat::core::let
       ((driver-and-pat
         (:wat::core::let
           ((spawn
             (:wat::holon::lru::HologramCacheService/spawn 1 2
               :wat::holon::lru::HologramCacheService/null-reporter
               (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
            (pool
             (:wat::core::first spawn))
            (d
             (:wat::core::second spawn))
            (handle
             (:wat::kernel::HandlePool::pop pool))
            (pat (:test::hcs-eviction-on-handle handle))
            (_finish
             (:wat::kernel::HandlePool::finish pool)))
           (:wat::core::Tuple d pat)))
        (driver
         (:wat::core::first driver-and-pat))
        (pat (:wat::core::second driver-and-pat)))
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) pat)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "hcs-eviction-died" "")
             0)))))

   ;; ─── Layer 6a sub-helper — put + get one key on a handle, score ───
   ;;
   ;; Pure handle-level work for the multi-client scenario. Same shape
   ;; as Layer 3b but takes its own (k, v) so two clients can each do
   ;; their own put+get on distinct keys.
   (:wat::core::define
     (:test::hcs-client-put-get
       (handle :wat::holon::lru::HologramCacheService::Handle)
       (k :wat::holon::HolonAST)
       (v :wat::holon::HolonAST)
       -> :wat::core::i64)
     (:test::hcs-put-then-get-on-handle handle k v))

   ;; ─── Layer 6 helper — two clients on one service, each puts+gets ──
   ;;
   ;; Spawns the service with count=2, pops two handles, each handle
   ;; does its own put-then-get. Returns 10*pa + pb where pa/pb are the
   ;; two clients' hit/miss bits. Both hit → 11.
   (:wat::core::define
     (:test::hcs-multi-client -> :wat::core::i64)
     (:wat::core::let
       ((driver-and-pat
         (:wat::core::let
           ((spawn
             (:wat::holon::lru::HologramCacheService/spawn 2 16
               :wat::holon::lru::HologramCacheService/null-reporter
               (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
            (pool
             (:wat::core::first spawn))
            (d
             (:wat::core::second spawn))
            (handle-a
             (:wat::kernel::HandlePool::pop pool))
            (handle-b
             (:wat::kernel::HandlePool::pop pool))
            (pa
             (:test::hcs-client-put-get handle-a
               (:wat::holon::leaf :alpha) (:wat::holon::leaf :av)))
            (pb
             (:test::hcs-client-put-get handle-b
               (:wat::holon::leaf :beta)  (:wat::holon::leaf :bv)))
            (_finish
             (:wat::kernel::HandlePool::finish pool)))
           (:wat::core::Tuple d (:wat::core::i64::+,2 (:wat::core::i64::*,2 pa 10) pb))))
        (driver
         (:wat::core::first driver-and-pat))
        (pat (:wat::core::second driver-and-pat)))
       (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::i64
         ((:wat::core::Ok _) pat)
         ((:wat::core::Err _)
           (:wat::core::do
             (:wat::test::assert-eq "hcs-multi-client-died" "")
             0)))))
   ))

;; ─── Layer 0 — :test::hcs-spawn-and-drop ─────────────────────────────
;;
;; Narrowest proof: post-arc-130 spawn/shutdown lifecycle works.
;; No request, no send, no recv. Pop one handle; finish pool; let
;; handle drop at inner scope exit → driver sees disconnect → outer
;; Thread/join-result unblocks.

(:wat::test::time-limit "200ms")
(:deftest-hcs :wat-tests::holon::lru::HologramCacheService::test-hcs-spawn-and-drop
  (:test::hcs-spawn-and-drop))

;; ─── Layer 1 — :test::hcs-helper-get-empty ──────────────────────────
;;
;; Helper-verb call site. :wat::holon::lru::HologramCacheService::get
;; on an empty probes vec returns an empty Vec<Option<HolonAST>>. The
;; helper-verb internally sends Request::Get and recvs Reply::GetResult
;; — driver gets a clean shutdown after pool finish + handle drop.

(:wat::test::time-limit "200ms")
(:deftest-hcs :wat-tests::holon::lru::HologramCacheService::test-hcs-helper-get-empty
  (:wat::test::assert-eq (:test::hcs-helper-get-empty) 0))

;; ─── Layer 2 — :test::hcs-helper-put-one ────────────────────────────
;;
;; Helper-verb call site for :wat::holon::lru::HologramCacheService::put.
;; Single-entry batch round trips Request::Put → Reply::PutAck. Helper
;; returns 1 on Ok join, 0 on Err join (after surfacing the death).

(:wat::test::time-limit "200ms")
(:deftest-hcs :wat-tests::holon::lru::HologramCacheService::test-hcs-helper-put-one
  (:wat::test::assert-eq (:test::hcs-helper-put-one) 1))

;; ─── Layer 3 — :test::hcs-helper-put-then-get ────────────────────────
;;
;; The happy-path round trip. Composes :wat::holon::lru::HologramCacheService::put
;; + ::get on a single handle. Put one (k, v); get k back; assert
;; results[0] = Some(_) (presence = 1). Proves Reply enum routing for
;; both variants on the same channel.

(:wat::test::time-limit "200ms")
(:deftest-hcs :wat-tests::holon::lru::HologramCacheService::test-hcs-helper-put-then-get
  (:wat::test::assert-eq (:test::hcs-helper-put-then-get) 1))

;; ─── Layer 4 — :test::hcs-helper-get-many-keys ────────────────────────
;;
;; Multi-key probe alignment. Put alpha=av + beta=bv; probe ["alpha","beta","gamma"];
;; presence pattern as packed digits = 110 (Some Some None). Proves
;; result-vec aligns with probe-vec by index — the contract Reply's
;; GetResult variant carries.

(:wat::test::time-limit "200ms")
(:deftest-hcs :wat-tests::holon::lru::HologramCacheService::test-hcs-helper-get-many-keys
  (:wat::test::assert-eq (:test::hcs-helper-get-many-keys) 110))

;; ─── Layer 5 — :test::hcs-eviction ──────────────────────────────────
;;
;; LRU eviction visible through the helper-verb interface. cap=2; put
;; alpha, beta, gamma in order — alpha evicts. Probe ["alpha","beta","gamma"];
;; presence pattern packed = 011 (None Some Some). Preserves the
;; eviction coverage from the prior file's test-step6 / test-hcs-spawn-put-3-eviction
;; without the channel-pair-deadlock plumbing.

(:wat::test::time-limit "200ms")
(:deftest-hcs :wat-tests::holon::lru::HologramCacheService::test-hcs-eviction
  (:wat::test::assert-eq (:test::hcs-eviction) 11))

;; ─── Layer 6 — :test::hcs-multi-client ──────────────────────────────
;;
;; Multi-client fan-in via HandlePool. spawn count=2 produces two
;; Handles; each handle does its own put+get on a distinct key. Both
;; hit → packed 11. Preserves the multi-client coverage from the prior
;; file's test-step5 / test-hcs-spawn-2clients-put-get-verify.

(:wat::test::time-limit "200ms")
(:deftest-hcs :wat-tests::holon::lru::HologramCacheService::test-hcs-multi-client
  (:wat::test::assert-eq (:test::hcs-multi-client) 11))
