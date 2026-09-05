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
;;   - spawn-program' (process) returns the process peer directly (arc 278 IPC
;;     de-prime) — no Receiver/from-pipe + Sender/from-pipe + ProcessPeer/new dance
;;   - Client wrappers use send' peer! (request) + recv' peer! (response) over the
;;     peer wire; SendOutcome / RecvOutcome are matched; a Lost cause is a
;;     LociDiedError surfaced (never swallowed)
;;   - Same body shape as thread tier (same operations, same assertions)
;;   - State recovery via Final variant — captured from Shutdown response
;;   - Process exits cleanly; recv-all' drains the peer to a clean Closed
;;
;; Honest deltas from inscribed pattern (BRIEF § Honest deltas):
;;   1. Enum unit variants use `(VariantName)` with parens per substrate.
;;   2. Enum payload variant uses named field: `(Increment (n :wat::core::i64))`.
;;   3. Enum variant constructors use `::` separator, not `/`.
;;   4. spawn-program' (process) hands back a (Peer' :- [counter::Request counter::Response])
;;      directly — no rx/tx pipe wrapping, no ProcessPeer/new (arc 278 IPC de-prime).
;;   5. spawn-program' does not allow capturing parent types; the subprocess
;;      declares its own independent copy of the counter enum types.
;;   6. Client wrappers take (Peer' :- [counter::Request counter::Response])
;;      (recv' responses from the peer; send' requests to the peer).
;;   7. readln at the server side uses `(:wat::kernel::readln -> :counter::Request)`
;;      for typed deserialization from EDN. The ambient println encodes to EDN.
;;
;; Deftest prelude format per arc 170 slice 4a-γ-flip:
;;   (:wat::test::deftest :name (prelude-forms...) body)
;; Prelude forms are spliced at top-level under (:wat::core::do ...) at freeze.
;; The body runs in a cheap in-process thread via :wat::test::run-thread.

;; ─── Parent-side type declarations ──────────────────────────────────
   ;;
   ;; Same enum names as the thread tier. The subprocess independently
   ;; declares the same types. EDN serialization uses the same tag format
   ;; (#counter/Request/Get nil, #counter/Request/Increment {:n 5}, etc.)
   ;; so values round-trip across the process boundary without any shared
   ;; type registry.
   ;; Stone 241.9 — migrated from :wat::core::enum to :wat::core::defenum (HARD CUT).
   (:wat::core::defenum :counter::Request :wat::enum::Pure
     :Get
     :Increment [n <- :wat::core::i64]
     :Reset
     :Shutdown)

   (:wat::core::defenum :counter::Response :wat::enum::Pure
     :Value [v <- :wat::core::i64]
     :Ok    [v <- :wat::core::i64]
     :Final [v <- :wat::core::i64])

   ;; ─── Client-side wrappers (Peer' tier) ───────────────────────────────
   ;;
   ;; Parallel to the thread-tier wrappers but over the process peer wire using
   ;; send' + recv'. The peer type is:
   ;;   (Peer' :- [counter::Request counter::Response])
   ;; where the parent recv's counter::Response from the peer and send's
   ;; counter::Request to it (arc 278 IPC de-prime).
   ;;
   ;; Arc 278 outcome walls: send' returns a SendOutcome (Sent/Closed/Lost),
   ;; recv' a RecvOutcome (Message/Lost/Closed) — both matched. These wrappers
   ;; return bare i64 (no ServiceError type in this proof-of-concept); terminal
   ;; arms use assertion-failed! (a Lost cause is a LociDiedError, surfaced).

   (:wat::core::defn :counter-proc::get
     [peer! <- (:wat::kernel::Peer :- [:counter::Request :counter::Response])]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::send peer! :counter::Request::Get)
       (:wat::kernel::SendOutcome::Sent
         (:wat::core::match (:wat::kernel::recv peer!)
           ((:wat::kernel::RecvOutcome::Message resp)
             (:wat::core::match resp ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': subprocess closed before replying" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
       (:wat::kernel::SendOutcome::Closed
         (:wat::kernel::assertion-failed! "send': subprocess closed" :wat::core::None :wat::core::None))
       (:wat::kernel::SendOutcome::Stopped
         (:wat::kernel::assertion-failed! "send': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
       ((:wat::kernel::SendOutcome::Lost cause)
         (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))))

   (:wat::core::defn :counter-proc::increment
     [peer! <- (:wat::kernel::Peer :- [:counter::Request :counter::Response])
      n     <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::send peer! (:counter::Request::Increment n))
       (:wat::kernel::SendOutcome::Sent
         (:wat::core::match (:wat::kernel::recv peer!)
           ((:wat::kernel::RecvOutcome::Message resp)
             (:wat::core::match resp ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': subprocess closed before replying" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
       (:wat::kernel::SendOutcome::Closed
         (:wat::kernel::assertion-failed! "send': subprocess closed" :wat::core::None :wat::core::None))
       (:wat::kernel::SendOutcome::Stopped
         (:wat::kernel::assertion-failed! "send': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
       ((:wat::kernel::SendOutcome::Lost cause)
         (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))))

   (:wat::core::defn :counter-proc::reset
     [peer! <- (:wat::kernel::Peer :- [:counter::Request :counter::Response])]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::send peer! :counter::Request::Reset)
       (:wat::kernel::SendOutcome::Sent
         (:wat::core::match (:wat::kernel::recv peer!)
           ((:wat::kernel::RecvOutcome::Message resp)
             (:wat::core::match resp ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': subprocess closed before replying" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
       (:wat::kernel::SendOutcome::Closed
         (:wat::kernel::assertion-failed! "send': subprocess closed" :wat::core::None :wat::core::None))
       (:wat::kernel::SendOutcome::Stopped
         (:wat::kernel::assertion-failed! "send': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
       ((:wat::kernel::SendOutcome::Lost cause)
         (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))))

   (:wat::core::defn :counter-proc::shutdown
     [peer! <- (:wat::kernel::Peer :- [:counter::Request :counter::Response])]
     -> :wat::core::i64
     (:wat::core::match (:wat::kernel::send peer! :counter::Request::Shutdown)
       (:wat::kernel::SendOutcome::Sent
         (:wat::core::match (:wat::kernel::recv peer!)
           ((:wat::kernel::RecvOutcome::Message resp)
             (:wat::core::match resp ((:counter::Response::Value v) v)
               ((:counter::Response::Ok    v) v)
               ((:counter::Response::Final v) v)))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': subprocess closed before replying" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
       (:wat::kernel::SendOutcome::Closed
         (:wat::kernel::assertion-failed! "send': subprocess closed" :wat::core::None :wat::core::None))
       (:wat::kernel::SendOutcome::Stopped
         (:wat::kernel::assertion-failed! "send': stopped — the substrate was asked to stop; the subprocess was ALIVE and the channel open" :wat::core::None :wat::core::None))
       ((:wat::kernel::SendOutcome::Lost cause)
         (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))))


(:wat::test::deftest :counter-actor::process-proof
  
  ;; ─── Test body ───────────────────────────────────────────────────────
  ;;
  ;; Spawn the counter server as a subprocess using (:wat::core::forms ...).
  ;; The subprocess program is self-contained: declares the counter enum types
  ;; independently, defines the dispatch fn using ambient readln/println, and
  ;; exposes :user::main as the process entry point calling dispatch with 10.
  ;;
  ;; Arc 278 IPC de-prime: spawn-program' (process) returns the process peer
  ;; directly — no Receiver/from-pipe + Sender/from-pipe + ProcessPeer/new
  ;; construction. The peer is a (Peer' :- [counter::Request counter::Response]);
  ;; the wrappers send' requests and recv' responses over it.
  (:wat::core::let
    [peer!
       (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           ;; Subprocess type declarations — independent from parent's types.
           ;; Same names → same EDN tags → interoperable across process boundary.
           ;; Stone 241.9 — migrated from :wat::core::enum to :wat::core::defenum (HARD CUT).
           (:wat::core::defenum :counter::Request :wat::enum::Pure
             :Get
             :Increment [n <- :wat::core::i64]
             :Reset
             :Shutdown)
           (:wat::core::defenum :counter::Response :wat::enum::Pure
             :Value [v <- :wat::core::i64]
             :Ok    [v <- :wat::core::i64]
             :Final [v <- :wat::core::i64])
           ;; Server-side dispatch — uses ambient readln/println (tier-honest).
           ;; Reads one counter::Request from stdin, dispatches, sends
           ;; counter::Response to stdout. Recurs on all non-terminal arms.
           ;; Shutdown arm sends Final and returns nil → process exits.
           (:wat::core::defn :counter::dispatch
             [state <- :wat::core::i64]
             -> :wat::core::nil
             (:wat::core::match (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                
               ;; Read — no state change; reply current value; recur
               (:counter::Request::Get
                  (:wat::core::do
                    (:wat::kernel::println (:counter::Response::Value state))
                    (:counter::dispatch state)))
               ;; Mutate-computed — let-bind new state; reply + recur
               ((:counter::Request::Increment n)
                  (:wat::core::let [new-n (:wat::i64::+ state n)]
                    (:wat::kernel::println (:counter::Response::Ok new-n))
                    (:counter::dispatch new-n)))
               ;; Mutate-literal — reply 0; recur with literal
               (:counter::Request::Reset
                  (:wat::core::do
                    (:wat::kernel::println (:counter::Response::Ok 0))
                    (:counter::dispatch 0)))
               ;; Terminal — send Final; return nil; process exits
               (:counter::Request::Shutdown
                  (:wat::kernel::println (:counter::Response::Final state)))))
           ;; Entry point — the substrate calls :user::main when the subprocess
           ;; starts. Per user 2026-05-16: "processes must always define
           ;; :user::main ... there is no :user::main-process".
           (:wat::core::defn :user::main [] -> :wat::core::nil (:counter::dispatch 10))))
     ;; spawn-program' (process) returns the peer directly (arc 278 IPC de-prime) —
     ;; no Receiver/from-pipe + Sender/from-pipe + ProcessPeer/new construction needed.
     ;; Same operations + assertions as thread tier (BRIEF § "same body shape").
     after-inc-5  (:counter-proc::increment peer! 5)
     _            (:wat::test::assert-eq after-inc-5 15)
     val          (:counter-proc::get peer!)
     _            (:wat::test::assert-eq val 15)
     after-inc-7  (:counter-proc::increment peer! 7)
     _            (:wat::test::assert-eq after-inc-7 22)
     after-reset  (:counter-proc::reset peer!)
     _            (:wat::test::assert-eq after-reset 0)
     after-inc-3  (:counter-proc::increment peer! 3)
     _            (:wat::test::assert-eq after-inc-3 3)
     final-state  (:counter-proc::shutdown peer!)
     _            (:wat::test::assert-eq final-state 3)
     ;; Drain the peer to a clean close (arc 278 IPC de-prime: recv-all' replaces
     ;; Process/drain-and-join). The peer's death rides in the Err — surfaced, never swallowed.
     _drained     (:wat::core::match (:wat::kernel::recv-all peer!)
                    ((:wat::core::Ok _) nil)
                    ((:wat::core::Err cause)
                      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None)))]
    nil))
