(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
             ;; ── the service loop (named recursion) — poll' multiplexes the self-peer
             ;; (owner link → :Shutdown), the socket listener (new connections), and the
             ;; connected socket client peers (requests) over ONE process::Select ring. ──
             (:wat::core::defn :user::serve
               [self    <- :wat::kernel::Peer'<wat::kernel::Address'<wat::core::i64,wat::core::i64>,wat::core::i64>
                l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
                clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>]
               -> :wat::core::nil
               (:wat::core::match (:wat::kernel::poll' self l clients) 
                 ;; SHUTDOWN — owner dropped the handle; RAII drain EOF'd the self-peer.
                 ;; Return nil → the loop exits, clients drop, the child ends, join completes.
                 (:wat::spawn::ServiceEvent::Shutdown nil)
                 ;; GROW — poll' accepted the dialing client; conj the new peer.
                 ((:wat::spawn::ServiceEvent::Connection peer)
                   (:user::serve self l (:wat::core::conj clients peer)))
                 ;; SERVE — an i64 arrived from clients[idx]; reply n+100.
                 ((:wat::spawn::ServiceEvent::Message idx n)
                   (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                          (:wat::core::+ n 100))]
                     (:user::serve self l clients)))
                 ;; SHRINK — clients[idx] left gracefully.
                 ((:wat::spawn::ServiceEvent::Closed idx)
                   (:user::serve self l (:wat::std::list::remove-at clients idx)))
                 ;; SHRINK — clients[idx]'s transport broke (remote tier; cause is a Failure).
                 ((:wat::spawn::ServiceEvent::Lost idx _cause)
                   (:user::serve self l (:wat::std::list::remove-at clients idx)))
                 ;; Admin wildcard — arc 291 new variant; not exercised by this probe.
                 (_ nil)))
             ;; the child entry: autobind (no name — unguessable capability), hand the minted
             ;; address to the parent over the self-peer (arc 272 capability handoff), then serve.
             ;; The self-peer carries Address'<i64,i64> up to the parent (S), i64 down from parent (R).
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                  self (:wat::program::self-peer
                          :wat::kernel::Address'<wat::core::i64,wat::core::i64> :wat::core::i64)
                  _    (:wat::kernel::send' self (:wat::spawn::Bound/address b))]
                 (:user::serve self (:wat::spawn::Bound/listener b)
                   (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>))))))
     ;; recv' the child's minted capability over the lineage channel (blocks until the child sends it).
     addr (:wat::core::match (:wat::kernel::recv' svc)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': svc closed before sending the capability" :wat::core::None :wat::core::None)))
     ;; dial the capability — the child is guaranteed listening (it sent AFTER listen()).
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c 5)
     got  (:wat::core::match (:wat::kernel::recv' c)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': c closed before replying" :wat::core::None :wat::core::None)))]
    ;; No Stop op. Scope-exit drops `svc` → the child's input pipe EOFs → the self-peer's
    ;; Recv{0} fires → serve's poll' returns :Shutdown → the child exits → join completes.
    ;; (If this hangs, poll' isn't watching the self-peer over the socket tier — STOP-1.)
    got))
