;; wat-tests/counter-client-capability-proof.wat — Counter/Client capability proof.
;;
;; Arc 203 slice 2 — first consumer of :wat::core::struct-restricted.
;; Proves the capability pattern: a server-issued opaque handle whose
;; constructor + restricted accessors are whitelisted to :counter::* only.
;;
;; Side-by-side with wat-tests/counter-actor-proof-thread.wat:
;;   - Same observable behavior (Increment, Get, Reset, Shutdown assertions)
;;   - Different structure: bare ThreadPeer replaced by :counter::Client
;;     capability bundle issued by :counter::spawn
;;   - :counter::Client/server-id + :counter::Client/client-id are restricted
;;     accessors (only :counter::* can read them)
;;   - :counter::Client/peer! is a public accessor (anyone can read it)
;;
;; Honest deltas from BRIEF assumptions (surfaced in SCORE-SLICE-2.md):
;;   1. uuid::v4 FQDN is :wat::telemetry::uuid::v4 (not :wat::measure::),
;;      and it returns :wat::core::String (not :wat::core::keyword).
;;      This slice uses constant strings for IDs (no uuid dep needed;
;;      single-user proof; uniqueness irrelevant).
;;   2. server-id / client-id use :wat::core::String type (not keyword).
;;   3. Public field is peer! <- :wat::kernel::ThreadPeer<counter::Response,
;;      counter::Request> — cleaner than storing Sender+Receiver separately;
;;      client wrappers use Thread/println + Thread/readln on it.
;;   4. Whitelist entry [:counter/] does NOT match :counter/spawn-style FQDNs
;;      (arc 198 prefix matching only fires on entries ending in `::`, not `/`).
;;      Counter functions use :counter:: namespace (::counter::spawn, etc.) so
;;      the whitelist [:counter::] matches their FQDNs via prefix matching.
;;      This differs from counter-actor-proof-thread.wat which uses :counter/
;;      (no struct-restricted constraint). Functions within :counter:: namespace
;;      have FQDNs like :counter::spawn, :counter::dispatch, :counter::get, etc.
;;
;; Deftest prelude format per arc 170 slice 4a-γ-flip:
;;   (:wat::test::deftest :name (prelude-forms...) body)
;; Prelude forms splice at top-level under (:wat::core::do ...) at freeze.
;; The body runs in a cheap in-process thread via :wat::test::run-thread.

(:wat::test::deftest :counter-client::capability-proof
  (;; ─── Type declarations ───────────────────────────────────────────────
   ;;
   ;; :counter::Request — the actor's input enum. Four variants:
   ;;   Get       — read-only query; reply is current value
   ;;   Increment — mutate by adding n; reply is new value
   ;;   Reset     — mutate to 0; reply is 0
   ;;   Shutdown  — terminal; reply is Final carrying last state; thread exits.
   ;;
   ;; Unit variants use (VariantName) list syntax per substrate honest delta.
   ;; Payload variant uses named field per substrate honest delta.
   (:wat::core::enum :counter::Request
     (Get)
     (Increment (n :wat::core::i64))
     (Reset)
     (Shutdown))

   ;; :counter::Response — the actor's output enum. Three variants:
   ;;   Value — reply to Get; carries the current (unchanged) state
   ;;   Ok    — reply to Increment and Reset; carries the new state
   ;;   Final — convention: reply to Shutdown; carries the terminal state
   (:wat::core::enum :counter::Response
     (Value (v :wat::core::i64))
     (Ok    (v :wat::core::i64))
     (Final (v :wat::core::i64)))

   ;; :counter::Client — the capability struct.
   ;;
   ;; Minted ONLY by :counter::spawn (constructor whitelist [:counter::]).
   ;; server-id and client-id are restricted to :counter::* reads —
   ;; the server uses them for identity; callers cannot access them directly.
   ;; peer! is public — callers use it (via wrappers) to talk to the server.
   ;;
   ;; Honest delta: IDs are :wat::core::String (uuid::v4 returns String,
   ;; not keyword; and this slice uses constant strings for simplicity).
   ;; Public field peer! stores the client-side ThreadPeer; wrappers use
   ;; Thread/println + Thread/readln on it.
   ;; Honest delta: whitelist [:counter::] (NOT [:counter/]) — arc 198
   ;; prefix matching fires only for entries ending in ::; see SCORE.
   (:wat::core::struct-restricted :counter::Client
     [:counter::]
     ([:counter::] server-id <- :wat::core::String
      [:counter::] client-id <- :wat::core::String)
     (peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>))

   ;; ─── Dispatch loop ───────────────────────────────────────────────────
   ;;
   ;; :counter::dispatch — the actor's message loop. Same shape as the
   ;; counter-actor-proof-thread.wat dispatch; takes the server-side
   ;; ThreadPeer (reads Requests, sends Responses).
   ;;
   ;; Named under :counter:: namespace so the whitelist [:counter::] covers it.
   ;; Tail-calls itself on non-terminal arms (TCO per ITERATION-PATTERNS.md
   ;; pattern 6). Shutdown arm sends Final, returns nil; thread exits.
   (:wat::core::defn :counter::dispatch
     [peer!  <- :wat::kernel::ThreadPeer<counter::Request,counter::Response>
      state  <- :wat::core::i64]
     -> :wat::core::nil
     (:wat::core::match (:wat::kernel::Thread/readln peer!)
       -> :wat::core::nil

       ;; Read — no state change; reply current value; recur same state
       ((:counter::Request::Get)
          (:wat::core::do
            (:wat::kernel::Thread/println peer! (:counter::Response::Value state))
            (:counter::dispatch peer! state)))

       ;; Mutate-computed — let-bind new state; reply + recur with new state
       ((:counter::Request::Increment n)
          (:wat::core::let [new-n (:wat::core::i64::+'2 state n)]
            (:wat::kernel::Thread/println peer! (:counter::Response::Ok new-n))
            (:counter::dispatch peer! new-n)))

       ;; Mutate-literal — reply 0; recur with literal 0
       ((:counter::Request::Reset)
          (:wat::core::do
            (:wat::kernel::Thread/println peer! (:counter::Response::Ok 0))
            (:counter::dispatch peer! 0)))

       ;; Terminal — send Final with last state; do NOT recur; thread exits
       ((:counter::Request::Shutdown)
          (:wat::kernel::Thread/println peer! (:counter::Response::Final state)))))

   ;; ─── Constructor ─────────────────────────────────────────────────────
   ;;
   ;; :counter::spawn — the actor constructor.
   ;;
   ;; Named under :counter:: namespace so the whitelist [:counter::] covers it.
   ;; Spawns a dispatch thread. Builds the client-side ThreadPeer from the
   ;; Thread handle's output+input accessors. Constructs and returns a
   ;; :counter::Client capability struct.
   ;;
   ;; :counter::Client/new is restricted to [:counter::] — this is the ONLY
   ;; place where Client values can be minted. The server-id read back
   ;; below proves the restricted accessor IS accessible within :counter::*.
   ;;
   ;; Thread<I,O>: I = counter::Request (parent writes into thread),
   ;;              O = counter::Response (thread writes out to parent).
   (:wat::core::defn :counter::spawn
     [initial <- :wat::core::i64]
     -> :counter::Client
     (:wat::core::let
       [thread  (:wat::kernel::spawn-thread
                  (:wat::core::fn
                    [server-rx! <- :wat::kernel::Receiver<counter::Request>
                     server-tx! <- :wat::kernel::Sender<counter::Response>]
                    -> :wat::core::nil
                    (:counter::dispatch
                      (:wat::kernel::ThreadPeer/new server-rx! server-tx!)
                      initial)))
        ;; Build client-side peer: reads Responses, sends Requests.
        ;; Thread/output = Receiver<Response> (rx); Thread/input = Sender<Request> (tx).
        client-peer! (:wat::kernel::ThreadPeer/new
                       (:wat::kernel::Thread/output thread)
                       (:wat::kernel::Thread/input  thread))
        ;; Mint the capability struct. Constructor is restricted to :counter::*.
        ;; Constant IDs used for this single-user proof; slice 3 will use uuid::v4.
        client   (:counter::Client/new
                   "counter-server-0"
                   "counter-client-0"
                   client-peer!)
        ;; Read restricted accessors from within :counter:: namespace —
        ;; proves the accessor whitelist works for the issuing namespace.
        _sid     (:counter::Client/server-id client)
        _cid     (:counter::Client/client-id client)]
       client))

   ;; ─── Client-side wrappers ────────────────────────────────────────────
   ;;
   ;; Each wrapper takes a :counter::Client. Accesses the public peer! field
   ;; (ThreadPeer<counter::Response, counter::Request>) via the unrestricted
   ;; accessor :counter::Client/peer!. Uses Thread/println + Thread/readln
   ;; for the mini-TCP lockstep round-trip.
   ;;
   ;; Wrappers are named under :counter:: (so the whitelist [:counter::] covers
   ;; any restricted accessor reads if needed). They are CALLABLE from any
   ;; namespace — arc 198/203 restrictions apply to the CALL SITE's enclosing
   ;; defn, not to the wrapper being called. The test body (outside :counter::)
   ;; invokes these wrappers freely.

   (:wat::core::defn :counter::get
     [client! <- :counter::Client]
     -> :wat::core::i64
     (:wat::core::let [peer! (:counter::Client/peer! client!)]
       (:wat::kernel::Thread/println peer! (:counter::Request::Get))
       (:wat::core::match (:wat::kernel::Thread/readln peer!)
         -> :wat::core::i64
         ((:counter::Response::Value v) v)
         ((:counter::Response::Ok    v) v)
         ((:counter::Response::Final v) v))))

   (:wat::core::defn :counter::increment
     [client! <- :counter::Client
      n       <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::core::let [peer! (:counter::Client/peer! client!)]
       (:wat::kernel::Thread/println peer! (:counter::Request::Increment n))
       (:wat::core::match (:wat::kernel::Thread/readln peer!)
         -> :wat::core::i64
         ((:counter::Response::Value v) v)
         ((:counter::Response::Ok    v) v)
         ((:counter::Response::Final v) v))))

   (:wat::core::defn :counter::reset
     [client! <- :counter::Client]
     -> :wat::core::i64
     (:wat::core::let [peer! (:counter::Client/peer! client!)]
       (:wat::kernel::Thread/println peer! (:counter::Request::Reset))
       (:wat::core::match (:wat::kernel::Thread/readln peer!)
         -> :wat::core::i64
         ((:counter::Response::Value v) v)
         ((:counter::Response::Ok    v) v)
         ((:counter::Response::Final v) v))))

   (:wat::core::defn :counter::shutdown
     [client! <- :counter::Client]
     -> :wat::core::i64
     (:wat::core::let [peer! (:counter::Client/peer! client!)]
       (:wat::kernel::Thread/println peer! (:counter::Request::Shutdown))
       (:wat::core::match (:wat::kernel::Thread/readln peer!)
         -> :wat::core::i64
         ((:counter::Response::Value v) v)
         ((:counter::Response::Ok    v) v)
         ((:counter::Response::Final v) v)))))

  ;; ─── Test body ───────────────────────────────────────────────────────
  ;;
  ;; Spawn the counter with initial state 10.
  ;; Exercise Increment, Get, Reset, Shutdown via the Client capability.
  ;; Assert the expected state after each operation.
  ;;
  ;; The client! binding is typed as :counter::Client. The test body
  ;; is NOT in the :counter:: namespace — it cannot call
  ;; :counter::Client/new or :counter::Client/server-id directly;
  ;; those are enforced at compile time by the arc 198/203 walker.
  (:wat::core::let
    [client!        (:counter::spawn 10)
     after-inc-5    (:counter::increment client! 5)
     _              (:wat::test::assert-eq after-inc-5 15)
     after-inc-7    (:counter::increment client! 7)
     _              (:wat::test::assert-eq after-inc-7 22)
     val            (:counter::get client!)
     _              (:wat::test::assert-eq val 22)
     after-reset    (:counter::reset client!)
     _              (:wat::test::assert-eq after-reset 0)
     final-state    (:counter::shutdown client!)
     _              (:wat::test::assert-eq final-state 0)]
    :wat::core::nil))
