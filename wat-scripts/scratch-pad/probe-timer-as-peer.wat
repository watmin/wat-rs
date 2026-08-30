;; wat-scripts/scratch-pad/probe-timer-as-peer.wat — arc 278 Stone 1 DISCONFIRMING probe.
;;
;; THE CLAIM: `(:wat::kernel::after peer-kind d msg)` now builds a UNIFIED `(Peer' :- [nil O])`
;; (arc 278 Stone 1 — the timer relocated to the CORRECT location), so a timer drops into
;; a service's `poll'` set BY CONSTRUCTION and `poll'` delivers its `msg` as a
;; `ServiceEvent::Message`, exactly like an accepted client connection.
;;
;; This walks the REAL `poll'`/`Peer'` path (`eval_poll_prime` downcasts every peers-element
;; to the unified `PEER_TYPE_PATH` and errors on anything else) — NOT `select'` (the adjacent
;; path). A one-element `[timer]` peers set alongside the self-peer + listener IS a real
;; `poll'` set; the timer is the client-peer element the multiplexer watches. (A reply-ing
;; client cannot yet share the homogeneous `(Vector :- [(Peer' :- [I O])])` with a `(Peer' :- [nil O])` timer —
;; that is Stone 2's `<service>::Op` superset — so the minimal real set is self+listener+timer.)
;;
;; RED before the stone: `after` built a tier-specific `(Timer' :- [O])` (`THREAD_PEER_TYPE_PATH` /
;; `PROCESS_PEER_TYPE_PATH`), which `poll'` rejects (`peers[i] must be a Peer'`). GREEN after:
;; `after` builds `PEER_TYPE_PATH`, so `poll'` accepts + delivers it. BOTH tiers.
;;
;; Run:  cargo wat ./wat-scripts/scratch-pad/probe-timer-as-peer.wat   (asserts; exit 0 = GREEN)

;; ── THREAD tier ───────────────────────────────────────────────────────────────
;; A hand-rolled poll' loop. The timer is armed INSIDE the spawned thread's closure
;; (the unified Peer's ThreadOwnedCell owner-thread invariant demands its owner be the
;; thread that poll's it). On the timer's Message, forward the msg UP the self-peer and
;; exit; the parent recv's it off the lineage handle.
(:wat::core::defn :probe::serve-thread
  [self  <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::keyword :wat::core::nil])
   l     <- (:wat::kernel::Listener :- [:wat::core::keyword :wat::core::nil])
   peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])])]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::poll self l peers) 
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection peer)
      (:probe::serve-thread self l (:wat::core::conj peers peer)))
    ;; THE PROOF: poll' delivered the timer's msg as a peer Message. Forward it up, then exit.
    ((:wat::spawn::ServiceEvent::Message _idx msg)
      (:wat::core::let [_ (:wat::core::match (:wat::kernel::send self msg) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))] nil))
    ((:wat::spawn::ServiceEvent::Closed idx)
      (:probe::serve-thread self l (:wat::seq::remove-at peers idx)))
    ((:wat::spawn::ServiceEvent::Lost idx _cause)
      (:probe::serve-thread self l (:wat::seq::remove-at peers idx)))
    (_ nil)))

(:wat::core::defn :probe::thread-timer-in-poll [] -> :wat::core::keyword
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :wat::core::keyword :wat::core::nil)
     l    (:wat::spawn::Bound/listener pair)
     svc  (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::keyword :wat::core::nil])]
              -> :wat::core::nil
              (:wat::core::let
                [t (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond 30) :tick)]
                (:probe::serve-thread self l
                  (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])] t)))))
     got  (:wat::core::match (:wat::kernel::recv svc)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; svc was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': svc closed" :wat::core::None :wat::core::None)))]
    got))

;; ── PROCESS tier ──────────────────────────────────────────────────────────────
;; Same shape, forked child universe: listener' + self-peer + timer all built in the child.
(:wat::core::defn :probe::process-timer-in-poll [] -> :wat::core::keyword
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :probe::serve-proc
               [self  <- (:wat::kernel::Peer :- [:wat::core::keyword :wat::core::nil])
                l     <- (:wat::kernel::Listener :- [:wat::core::keyword :wat::core::nil])
                peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])])]
               -> :wat::core::nil
               (:wat::core::match (:wat::kernel::poll self l peers) 
                 (:wat::spawn::ServiceEvent::Shutdown nil)
                 ((:wat::spawn::ServiceEvent::Connection peer)
                   (:probe::serve-proc self l (:wat::core::conj peers peer)))
                 ((:wat::spawn::ServiceEvent::Message _idx msg)
                   (:wat::core::let [_ (:wat::core::match (:wat::kernel::send self msg) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))] nil))
                 ((:wat::spawn::ServiceEvent::Closed idx)
                   (:probe::serve-proc self l (:wat::seq::remove-at peers idx)))
                 ((:wat::spawn::ServiceEvent::Lost idx _cause)
                   (:probe::serve-proc self l (:wat::seq::remove-at peers idx)))
                 (_ nil)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [b    (:wat::kernel::listener (:wat::spawn::process) :wat::core::keyword :wat::core::nil)
                  self (:wat::program::self-peer :wat::core::keyword :wat::core::nil)
                  t    (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Millisecond 30) :tick)]
                 (:probe::serve-proc self (:wat::spawn::Bound/listener b)
                   (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])] t))))))
     got (:wat::core::match (:wat::kernel::recv svc)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; svc was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': svc closed" :wat::core::None :wat::core::None)))]
    got))

;; ── the assertion — both tiers deliver the timer's :tick through poll' ─────────
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq (:probe::thread-timer-in-poll) :tick)
    (:wat::test::assert-eq (:probe::process-timer-in-poll) :tick)))
