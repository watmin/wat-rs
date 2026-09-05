;; probe-arc278-reap-serve-event.wat — WHICH ServiceEvent terminates the serve loop
;; when the owner's env is dropped by the tail-call trampoline?
;;
;; A hand-rolled poll' loop (same shape defservice generates) that PRINTS the event it
;; got. The owner tail-calls, so its env — holding the spawn handle (the lineage peer) —
;; is dropped before the tail callee runs. The printed event is the answer:
;;
;;   "SERVE: Shutdown"  => poll' index 0 (self-peer / lineage channel) EOF'd
;;                         (runtime.rs eval_poll_prime, index.0 == 0, Err(_) arm)
;;   a panic / silence  => poll' index 1 (listener) fired instead
;;                         ("poll': listener recv failed — address was dropped")

(:wat::core::defn :se::serve
  [self  <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])
   l     <- (:wat::kernel::Listener :- [:wat::core::i64 :wat::core::i64])
   peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])])]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::poll self l peers)
    ;; THE REPORT CHANNEL: the spawned thread CANNOT println (stdio services are not
    ;; routed into a hand-spawned thread — see the SVC LOST row this probe first
    ;; produced), so each event reports itself by sending a distinct sentinel DOWN to
    ;; the connected client, which the owner reads. 999 = Shutdown reached.
    (:wat::spawn::ServiceEvent::Shutdown
      (:wat::core::match (:wat::kernel::try-send (:wat::core::nth peers 0) 999)
        (:wat::kernel::TrySendOutcome::Sent nil)
        (:wat::kernel::TrySendOutcome::WouldBlock nil)
        (:wat::kernel::TrySendOutcome::Closed nil)
        ((:wat::kernel::TrySendOutcome::Lost _c) nil)))
    ((:wat::spawn::ServiceEvent::Admin _m) (:se::serve self l peers))
    ((:wat::spawn::ServiceEvent::Connection peer)
      (:se::serve self l (:wat::core::conj peers peer)))
    ((:wat::spawn::ServiceEvent::Message idx msg)
      (:wat::core::do
        (:wat::core::match (:wat::kernel::send (:wat::core::nth peers idx) msg)
          (:wat::kernel::SendOutcome::Sent nil)
          (:wat::kernel::SendOutcome::Closed nil)
          ;; the world-stopping fact is caught above at the ServiceEvent::Shutdown arm
          ;; (poll' index 0), not here — this is one client's reply-send, so a stop mid
          ;; -send is discarded just like its Sent/Closed siblings; the unconditional
          ;; recurse below still runs either way (this probe measures REAP, not stop).
          (:wat::kernel::SendOutcome::Stopped nil)
          ((:wat::kernel::SendOutcome::Lost _c) nil))
        (:se::serve self l peers)))
    ((:wat::spawn::ServiceEvent::Closed idx)
      (:se::serve self l (:wat::seq::remove-at peers idx)))
    ((:wat::spawn::ServiceEvent::Lost idx _cause)
      (:se::serve self l (:wat::seq::remove-at peers idx)))
    (_ nil)))

(:wat::core::defn :se::try [c <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])
                           label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::kernel::send c 7)
      (:wat::kernel::SendOutcome::Sent nil)
      (:wat::kernel::SendOutcome::Closed (:wat::kernel::println (:wat::string::concat label " send => CLOSED")))
      (:wat::kernel::SendOutcome::Stopped (:wat::kernel::println (:wat::string::concat label " send => STOPPED")))
      ((:wat::kernel::SendOutcome::Lost _c) nil))
    (:wat::core::match (:wat::kernel::recv c)
      ((:wat::kernel::RecvOutcome::Message m)
        (:wat::kernel::println (:wat::string::concat label
          (:wat::string::concat " => Message " (:wat::core::i64/to-string m)))))
      ((:wat::kernel::RecvOutcome::Lost _cause)
        (:wat::kernel::println (:wat::string::concat label " => LOST")))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::println (:wat::string::concat label " => STOPPED")))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::println (:wat::string::concat label " => CLOSED"))) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::println (:wat::string::concat label " => LOST"))))
    nil))

(:wat::core::defn :se::row-tail [] -> :wat::core::nil
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l    (:wat::spawn::Bound/listener pair)
     a    (:wat::spawn::Bound/address pair)
     svc  (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])]
              -> :wat::core::nil
              (:se::serve self l (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])]))))
     c    (:wat::core::match (:wat::kernel::connect a)
            ((:wat::kernel::ConnectOutcome::Connected p) p)
            ((:wat::kernel::ConnectOutcome::Refused _f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Rejected _f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Failed _f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))
     _    (:se::try c "row non-tail")]
    ;; TAIL: the caller's env (holding `svc`, the lineage Thread' peer) is dropped by the
    ;; trampoline BEFORE this runs. `Thread::drop` → drain_and_join → the serve loop's
    ;; poll' index 0 EOFs. If the loop reaches the Shutdown arm, the client reads 999.
    (:se::try c "row TAIL    ")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do (:se::row-tail) nil))
