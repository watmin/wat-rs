;; wat-tests/counter-actor-proof-thread.wat — Counter actor pattern, thread tier.
;;
;; Arc 170 pre-D3 verification artifact. Proves the Counter actor pattern
;; inscribed in INTERSTITIAL-REALIZATIONS.md § 2026-05-16 (late) Kay-OOP
;; entry + § 2026-05-16 (deeper) control-channels entry — at the thread tier.
;;
;; What this proves:
;;   - Enums declare cleanly (counter::Request with unit + payload variants;
;;     counter::Response with payload variants)
;;   - :counter/spawn constructor returns a Thread<counter::Request, counter::Response>
;;   - :counter/dispatch recursive handler with all four shapes operates correctly:
;;       Read            — Get  → returns current value, recurs
;;       Mutate-computed — Increment → let-binds new state, recurs
;;       Mutate-literal  — Reset → recurs with literal 0
;;       Terminal        — Shutdown → sends Final, returns nil
;;   - Client wrappers (:counter/get, :counter/increment, :counter/reset,
;;     :counter/shutdown) round-trip per the mini-TCP lockstep
;;   - State recovery via Final<i64> — captured from Shutdown response
;;   - Thread exits cleanly via Thread/drain-and-join
;;
;; Honest deltas from inscribed pattern (BRIEF § Honest deltas):
;;   1. Enum unit variants use `(VariantName)` syntax with parens per substrate.
;;      The inscribed `Get`, `Reset`, `Shutdown` (bare symbols) must be
;;      `(Get)`, `(Reset)`, `(Shutdown)`.
;;   2. Enum payload variant uses named field: `(Increment (n :wat::core::i64))`
;;      not positional `(Increment :wat::core::i64)`.
;;   3. Enum variant CONSTRUCTORS use `::` separator, not `/`.
;;      `:counter::Request::Get` not `:counter::Request/Get`.
;;   4. Dispatch loop uses Thread/readln + Thread/println on a ThreadPeer
;;      (NOT bare recv/send on raw channels). recv returns Result<Option<T>>;
;;      Thread/readln returns bare T — simpler dispatch.
;;      The factory fn wraps server-rx! + server-tx! in ThreadPeer/new.
;;   5. Type names use :: separator throughout: :counter::Request, :counter::Response.
;;   6. Client wrappers take ThreadPeer<counter::Response, counter::Request>
;;      (coordinator reads responses via Thread/readln; sends requests via Thread/println).
;;
;; Deftest prelude format per arc 170 slice 4a-γ-flip:
;;   (:wat::test::deftest :name (prelude-forms...) body)
;; Prelude forms are spliced at top-level under (:wat::core::do ...) at freeze.
;; The body runs in a cheap in-process thread via :wat::test::run-thread.

(:wat::test::deftest :counter-actor::thread-proof
  (;; ─── Type declarations ───────────────────────────────────────────────
   ;;
   ;; counter::Request — the actor's input enum. Four variants:
   ;;   Get       — read-only query; reply is current value
   ;;   Increment — mutate by adding n; reply is new value
   ;;   Reset     — mutate to 0; reply is 0
   ;;   Shutdown  — convention (INTERSTITIAL § control-channels): terminal;
   ;;               reply is Final carrying the last state; thread exits.
   (:wat::core::enum :counter::Request
     (Get)
     (Increment (n :wat::core::i64))
     (Reset)
     (Shutdown))

   ;; counter::Response — the actor's output enum. Three variants:
   ;;   Value — reply to Get; carries the current (unchanged) state
   ;;   Ok    — reply to Increment and Reset; carries the new state
   ;;   Final — convention: reply to Shutdown; carries the terminal state
   (:wat::core::enum :counter::Response
     (Value (v :wat::core::i64))
     (Ok    (v :wat::core::i64))
     (Final (v :wat::core::i64)))

   ;; ─── Dispatch loop ───────────────────────────────────────────────────
   ;;
   ;; :counter/dispatch — the actor's message loop.
   ;;
   ;; Reads one request via Thread/readln (bare :counter::Request; no
   ;; Result wrapping). Dispatches per the four handler shapes. Recurs
   ;; (TCO per ITERATION-PATTERNS.md pattern 6) on all non-terminal arms.
   ;; Shutdown arm does NOT recur — sends Final, returns nil, thread exits.
   ;;
   ;; Takes the server-side ThreadPeer (reads requests, sends responses).
   ;; Honest delta vs inscription: peer! takes ThreadPeer<counter::Request,
   ;; counter::Response> (server reads requests via Thread/readln → Request;
   ;; server sends responses via Thread/println → Response).
   (:wat::core::defn :counter/dispatch
     [peer!  <- :wat::kernel::ThreadPeer<counter::Request,counter::Response>
      state  <- :wat::core::i64]
     -> :wat::core::nil
     (:wat::core::match (:wat::kernel::Thread/readln peer!)
       -> :wat::core::nil

       ;; Read — no state change; reply current value; recur same state
       ((:counter::Request::Get)
          (:wat::core::do
            (:wat::kernel::Thread/println peer! (:counter::Response::Value state))
            (:counter/dispatch peer! state)))

       ;; Mutate-computed — let-bind new state; reply + recur with new state
       ((:counter::Request::Increment n)
          (:wat::core::let [new-n (:wat::core::i64::+'2 state n)]
            (:wat::kernel::Thread/println peer! (:counter::Response::Ok new-n))
            (:counter/dispatch peer! new-n)))

       ;; Mutate-literal — reply 0; recur with literal 0
       ((:counter::Request::Reset)
          (:wat::core::do
            (:wat::kernel::Thread/println peer! (:counter::Response::Ok 0))
            (:counter/dispatch peer! 0)))

       ;; Terminal — send Final with last state; do NOT recur; thread exits
       ((:counter::Request::Shutdown)
          (:wat::kernel::Thread/println peer! (:counter::Response::Final state)))))

   ;; ─── Constructor ─────────────────────────────────────────────────────
   ;;
   ;; :counter/spawn — the actor constructor.
   ;;
   ;; Spawns a thread; the fn body receives the typed channel pair
   ;; (server-rx! = Receiver<counter::Request>, server-tx! =
   ;; Sender<counter::Response>), wraps them in ThreadPeer/new, and calls
   ;; the dispatch loop with the initial state.
   ;;
   ;; Thread<I,O>: I = counter::Request (parent writes into thread),
   ;;              O = counter::Response (thread writes out to parent).
   (:wat::core::defn :counter/spawn
     [initial <- :wat::core::i64]
     -> :wat::kernel::Thread<counter::Request,counter::Response>
     (:wat::kernel::spawn-thread
       (:wat::core::fn
         [server-rx! <- :wat::kernel::Receiver<counter::Request>
          server-tx! <- :wat::kernel::Sender<counter::Response>]
         -> :wat::core::nil
         (:counter/dispatch
           (:wat::kernel::ThreadPeer/new server-rx! server-tx!)
           initial))))

   ;; ─── Client-side wrappers ────────────────────────────────────────────
   ;;
   ;; Each wrapper takes the client-side ThreadPeer (reads responses, sends
   ;; requests). Honest delta vs inscription: type is
   ;; ThreadPeer<counter::Response, counter::Request> — reads Response
   ;; (O from coordinator perspective), writes Request (I to thread).
   ;;
   ;; The ThreadPeer/new in the test body is:
   ;;   (ThreadPeer/new (Thread/output thread) (Thread/input thread))
   ;; = ThreadPeer/new(Receiver<counter::Response>, Sender<counter::Request>)
   ;; Thread/readln client-peer → counter::Response
   ;; Thread/println client-peer (counter::Request::Get) → sends request

   (:wat::core::defn :counter/get
     [peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>]
     -> :wat::core::i64
     (:wat::kernel::Thread/println peer! (:counter::Request::Get))
     (:wat::core::match (:wat::kernel::Thread/readln peer!)
       -> :wat::core::i64
       ((:counter::Response::Value v) v)
       ((:counter::Response::Ok    v) v)
       ((:counter::Response::Final v) v)))

   (:wat::core::defn :counter/increment
     [peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>
      n     <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::kernel::Thread/println peer! (:counter::Request::Increment n))
     (:wat::core::match (:wat::kernel::Thread/readln peer!)
       -> :wat::core::i64
       ((:counter::Response::Value v) v)
       ((:counter::Response::Ok    v) v)
       ((:counter::Response::Final v) v)))

   (:wat::core::defn :counter/reset
     [peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>]
     -> :wat::core::i64
     (:wat::kernel::Thread/println peer! (:counter::Request::Reset))
     (:wat::core::match (:wat::kernel::Thread/readln peer!)
       -> :wat::core::i64
       ((:counter::Response::Value v) v)
       ((:counter::Response::Ok    v) v)
       ((:counter::Response::Final v) v)))

   (:wat::core::defn :counter/shutdown
     [peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>]
     -> :wat::core::i64
     (:wat::kernel::Thread/println peer! (:counter::Request::Shutdown))
     (:wat::core::match (:wat::kernel::Thread/readln peer!)
       -> :wat::core::i64
       ((:counter::Response::Value v) v)
       ((:counter::Response::Ok    v) v)
       ((:counter::Response::Final v) v))))

  ;; ─── Test body ───────────────────────────────────────────────────────
  ;;
  ;; Spawn the counter with initial state 10.
  ;; Build the client-side ThreadPeer from Thread/output + Thread/input.
  ;; Exercise all four handler shapes with assertions.
  ;; Shutdown + capture final state; assert equals expected.
  ;; Drain and join the thread.
  (:wat::core::let
    [thread       (:counter/spawn 10)
     peer!        (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread)
                    (:wat::kernel::Thread/input  thread))
     after-inc-5  (:counter/increment peer! 5)
     _            (:wat::test::assert-eq after-inc-5 15)
     val          (:counter/get peer!)
     _            (:wat::test::assert-eq val 15)
     after-inc-7  (:counter/increment peer! 7)
     _            (:wat::test::assert-eq after-inc-7 22)
     after-reset  (:counter/reset peer!)
     _            (:wat::test::assert-eq after-reset 0)
     after-inc-3  (:counter/increment peer! 3)
     _            (:wat::test::assert-eq after-inc-3 3)
     final-state  (:counter/shutdown peer!)
     _            (:wat::test::assert-eq final-state 3)
     _drained     (:wat::kernel::Thread/drain-and-join thread)]
    :wat::core::nil))
