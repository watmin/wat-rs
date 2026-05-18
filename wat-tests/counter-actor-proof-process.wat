;; wat-tests/counter-actor-proof-process.wat — Counter actor pattern, process tier.
;;
;; Arc 170 pre-D3 verification artifact. Proves the Counter actor pattern
;; inscribed in INTERSTITIAL-REALIZATIONS.md § 2026-05-16 (late) Kay-OOP
;; entry + § 2026-05-16 (deeper) control-channels entry — at the process tier.
;;
;; What this proves:
;;   - Enum types can be declared independently in parent + subprocess (same
;;     type names → same EDN serialization → interoperable across process boundary)
;;   - Server-side dispatch uses ambient readln/println (no peer struct; no typed
;;     channel params) — the process boundary IS the isolation
;;   - Server's :user::main calls :counter/dispatch initial (direct entry; no spawn)
;;   - ProcessPeer/new constructed from Receiver/from-pipe(Process/stdout) +
;;     Sender/from-pipe(Process/stdin) — verbose-is-honest composition
;;   - Client wrappers use Process/println peer! (request) + Process/readln peer!
;;   - Same body shape as thread tier (same operations, same assertions)
;;   - State recovery via Final variant — captured from Shutdown response
;;   - Process exits cleanly via Process/drain-and-join
;;
;; Honest deltas from inscribed pattern (BRIEF § Honest deltas):
;;   1. Enum unit variants use `(VariantName)` with parens per substrate.
;;   2. Enum payload variant uses named field: `(Increment (n :wat::core::i64))`.
;;   3. Enum variant constructors use `::` separator, not `/`.
;;   4. ProcessPeer/new takes (rx, tx) where rx = Receiver/from-pipe(stdout),
;;      tx = Sender/from-pipe(stdin). The inscribed BRIEF shows argument order
;;      (Process/stdout proc, Process/stdin proc) — actual construction must
;;      wrap these in Receiver/from-pipe + Sender/from-pipe first, then pass
;;      to ProcessPeer/new. No constructor verb is minted (verbose-is-honest).
;;   5. spawn-process does not allow capturing parent types; the subprocess
;;      declares its own independent copy of the counter enum types.
;;   6. Client wrappers take ProcessPeer<counter::Response, counter::Request>
;;      (reads responses from process stdout; sends requests to process stdin).
;;   7. readln at the server side uses `(:wat::kernel::readln -> :counter::Request)`
;;      for typed deserialization from EDN. The ambient println encodes to EDN.
;;
;; Deftest prelude format per arc 170 slice 4a-γ-flip:
;;   (:wat::test::deftest :name (prelude-forms...) body)
;; Prelude forms are spliced at top-level under (:wat::core::do ...) at freeze.
;; The body runs in a cheap in-process thread via :wat::test::run-thread.

(:wat::test::deftest :counter-actor::process-proof
  (;; ─── Parent-side type declarations ──────────────────────────────────
   ;;
   ;; Same enum names as the thread tier. The subprocess independently
   ;; declares the same types. EDN serialization uses the same tag format
   ;; (#counter/Request/Get nil, #counter/Request/Increment {:n 5}, etc.)
   ;; so values round-trip across the process boundary without any shared
   ;; type registry.
   (:wat::core::enum :counter::Request
     (Get)
     (Increment (n :wat::core::i64))
     (Reset)
     (Shutdown))

   (:wat::core::enum :counter::Response
     (Value (v :wat::core::i64))
     (Ok    (v :wat::core::i64))
     (Final (v :wat::core::i64)))

   ;; ─── Client-side wrappers (ProcessPeer tier) ─────────────────────────
   ;;
   ;; Parallel to the thread-tier wrappers but using Process/println and
   ;; Process/readln. The ProcessPeer type is:
   ;;   ProcessPeer<counter::Response, counter::Request>
   ;; where:
   ;;   peer.rx = Receiver (reads counter::Response from process stdout)
   ;;   peer.tx = Sender   (writes counter::Request to process stdin)
   ;;
   ;; Arc 208 slice 2 conversion: Result/expect replaced with honest match-on-Err.
   ;; These wrappers return bare i64 (no ServiceError type in this proof-of-concept);
   ;; Err arms use assertion-failed! (structurally honest; same panic semantics).
   ;; Walker accepts Process/println + Process/readln in match-value position.

   (:wat::core::defn :counter-proc/get
     [peer! <- :wat::kernel::ProcessPeer<counter::Response,counter::Request>]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::Process/println peer! (:counter::Request::Get))
       -> :wat::core::i64
       ((:wat::core::Ok _)
         (:wat::core::match (:wat::kernel::Process/readln peer!)
           -> :wat::core::i64
           ((:wat::core::Ok resp)
             (:wat::core::match resp -> :wat::core::i64
               ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::core::Err _chain)
             (:wat::kernel::assertion-failed! "Process/readln failed: subprocess died" :wat::core::None :wat::core::None))))
       ((:wat::core::Err _chain)
         (:wat::kernel::assertion-failed! "Process/println failed: subprocess died" :wat::core::None :wat::core::None))))

   (:wat::core::defn :counter-proc/increment
     [peer! <- :wat::kernel::ProcessPeer<counter::Response,counter::Request>
      n     <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::Process/println peer! (:counter::Request::Increment n))
       -> :wat::core::i64
       ((:wat::core::Ok _)
         (:wat::core::match (:wat::kernel::Process/readln peer!)
           -> :wat::core::i64
           ((:wat::core::Ok resp)
             (:wat::core::match resp -> :wat::core::i64
               ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::core::Err _chain)
             (:wat::kernel::assertion-failed! "Process/readln failed: subprocess died" :wat::core::None :wat::core::None))))
       ((:wat::core::Err _chain)
         (:wat::kernel::assertion-failed! "Process/println failed: subprocess died" :wat::core::None :wat::core::None))))

   (:wat::core::defn :counter-proc/reset
     [peer! <- :wat::kernel::ProcessPeer<counter::Response,counter::Request>]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::Process/println peer! (:counter::Request::Reset))
       -> :wat::core::i64
       ((:wat::core::Ok _)
         (:wat::core::match (:wat::kernel::Process/readln peer!)
           -> :wat::core::i64
           ((:wat::core::Ok resp)
             (:wat::core::match resp -> :wat::core::i64
               ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::core::Err _chain)
             (:wat::kernel::assertion-failed! "Process/readln failed: subprocess died" :wat::core::None :wat::core::None))))
       ((:wat::core::Err _chain)
         (:wat::kernel::assertion-failed! "Process/println failed: subprocess died" :wat::core::None :wat::core::None))))

   (:wat::core::defn :counter-proc/shutdown
     [peer! <- :wat::kernel::ProcessPeer<counter::Response,counter::Request>]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::Process/println peer! (:counter::Request::Shutdown))
       -> :wat::core::i64
       ((:wat::core::Ok _)
         (:wat::core::match (:wat::kernel::Process/readln peer!)
           -> :wat::core::i64
           ((:wat::core::Ok resp)
             (:wat::core::match resp -> :wat::core::i64
               ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::core::Err _chain)
             (:wat::kernel::assertion-failed! "Process/readln failed: subprocess died" :wat::core::None :wat::core::None))))
       ((:wat::core::Err _chain)
         (:wat::kernel::assertion-failed! "Process/println failed: subprocess died" :wat::core::None :wat::core::None)))))

  ;; ─── Test body ───────────────────────────────────────────────────────
  ;;
  ;; Spawn the counter server as a subprocess using (:wat::core::forms ...).
  ;; The subprocess program is self-contained: declares the counter enum types
  ;; independently, defines the dispatch fn using ambient readln/println, and
  ;; exposes :user::main as the process entry point calling dispatch with 10.
  ;;
  ;; ProcessPeer construction per Stone C2 substrate-composition proof:
  ;;   rx = Receiver/from-pipe(Process/stdout proc)   ← reads process stdout
  ;;   tx = Sender/from-pipe(Process/stdin proc)      ← writes process stdin
  ;;   peer = ProcessPeer/new(rx, tx)
  ;;
  ;; This is the verbose-is-honest form (per feedback_verbose_is_honest):
  ;; the three-step construction reveals what the bracket macro will eventually
  ;; hide. No constructor verb is minted — Stone D's run-processes is the
  ;; user-facing surface.
  (:wat::core::let
    [proc
       (:wat::kernel::spawn-process
         (:wat::core::forms
           ;; Subprocess type declarations — independent from parent's types.
           ;; Same names → same EDN tags → interoperable across process boundary.
           (:wat::core::enum :counter::Request
             (Get)
             (Increment (n :wat::core::i64))
             (Reset)
             (Shutdown))
           (:wat::core::enum :counter::Response
             (Value (v :wat::core::i64))
             (Ok    (v :wat::core::i64))
             (Final (v :wat::core::i64)))
           ;; Server-side dispatch — uses ambient readln/println (tier-honest).
           ;; Reads one counter::Request from stdin, dispatches, sends
           ;; counter::Response to stdout. Recurs on all non-terminal arms.
           ;; Shutdown arm sends Final and returns nil → process exits.
           (:wat::core::defn :counter/dispatch
             [state <- :wat::core::i64]
             -> :wat::core::nil
             (:wat::core::match (:wat::kernel::readln -> :counter::Request)
               -> :wat::core::nil
               ;; Read — no state change; reply current value; recur
               ((:counter::Request::Get)
                  (:wat::core::do
                    (:wat::kernel::println (:counter::Response::Value state))
                    (:counter/dispatch state)))
               ;; Mutate-computed — let-bind new state; reply + recur
               ((:counter::Request::Increment n)
                  (:wat::core::let [new-n (:wat::core::i64::+'2 state n)]
                    (:wat::kernel::println (:counter::Response::Ok new-n))
                    (:counter/dispatch new-n)))
               ;; Mutate-literal — reply 0; recur with literal
               ((:counter::Request::Reset)
                  (:wat::core::do
                    (:wat::kernel::println (:counter::Response::Ok 0))
                    (:counter/dispatch 0)))
               ;; Terminal — send Final; return nil; process exits
               ((:counter::Request::Shutdown)
                  (:wat::kernel::println (:counter::Response::Final state)))))
           ;; Entry point — the substrate calls :user::main when the subprocess
           ;; starts. Per user 2026-05-16: "processes must always define
           ;; :user::main ... there is no :user::main-process".
           (:wat::core::define (:user::main -> :wat::core::nil)
             (:counter/dispatch 10))))
     ;; Build ProcessPeer — verbose-is-honest composition.
     ;; Receiver/from-pipe reads what the subprocess prints to stdout.
     ;; Sender/from-pipe writes to the subprocess's stdin (what it reads).
     rx        (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))
     tx        (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  proc))
     peer!     (:wat::kernel::ProcessPeer/new rx tx)
     ;; Same operations + assertions as thread tier (BRIEF § "same body shape").
     after-inc-5  (:counter-proc/increment peer! 5)
     _            (:wat::test::assert-eq after-inc-5 15)
     val          (:counter-proc/get peer!)
     _            (:wat::test::assert-eq val 15)
     after-inc-7  (:counter-proc/increment peer! 7)
     _            (:wat::test::assert-eq after-inc-7 22)
     after-reset  (:counter-proc/reset peer!)
     _            (:wat::test::assert-eq after-reset 0)
     after-inc-3  (:counter-proc/increment peer! 3)
     _            (:wat::test::assert-eq after-inc-3 3)
     final-state  (:counter-proc/shutdown peer!)
     _            (:wat::test::assert-eq final-state 3)
     _drained     (:wat::kernel::Process/drain-and-join proc)]
    :wat::core::nil))
