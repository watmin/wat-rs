;; Proof 2: a real stranger child (pid ≠ owner) is bounced.
;; The owner recv's the service capability, then spawns a SEPARATE stranger process and HANDS
;; the (leaked) service address DOWN to the stranger over the stranger's lineage channel.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc     (:wat::kernel::spawn-program' (:wat::spawn::process)
               (:wat::core::forms
                (:wat::core::defn :user::serve
                  [self    <- :wat::kernel::Peer'<wat::kernel::Address'<wat::core::i64,wat::core::i64>,wat::core::i64>
                   l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
                   clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                    (:wat::spawn::ServiceEvent::Shutdown nil)
                    ((:wat::spawn::ServiceEvent::Connection peer)
                      (:user::serve self l (:wat::core::conj clients peer)))
                    ((:wat::spawn::ServiceEvent::Message idx n)
                      (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                             (:wat::core::+ n 100))]
                        (:user::serve self l clients)))
                    ((:wat::spawn::ServiceEvent::Closed idx)
                      (:user::serve self l (:wat::std::list::remove-at clients idx)))
                    ((:wat::spawn::ServiceEvent::Lost idx _cause)
                      (:user::serve self l (:wat::std::list::remove-at clients idx)))
                    ;; Admin wildcard — arc 291 new variant; not exercised by this probe.
                    (_ nil)))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                     self (:wat::program::self-peer
                             :wat::kernel::Address'<wat::core::i64,wat::core::i64> :wat::core::i64)
                     _    (:wat::kernel::send' self (:wat::spawn::Bound/address b))]
                    (:user::serve self (:wat::spawn::Bound/listener b)
                      (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>))))))
     ;; recv' the service's minted capability (blocks until service sends it).
     svc-addr (:wat::kernel::recv' svc)
     ;; A SEPARATE process child — its pid ≠ the owner's → NOT in the birth-seeded allow-set.
     ;; The owner hands the (leaked) service address DOWN to the stranger via its lineage channel.
     ;; stranger self-peer: S=i64 (would send up — never does), R=Address'<i64,i64> (receives cap).
     stranger (:wat::kernel::spawn-program' (:wat::spawn::process)
                (:wat::core::forms
                  (:wat::core::defn :user::main [] -> :wat::core::nil
                    (:wat::core::let
                      ;; receive the leaked service address from the owner via our lineage channel.
                      [self (:wat::program::self-peer
                               :wat::core::i64
                               :wat::kernel::Address'<wat::core::i64,wat::core::i64>)
                       addr (:wat::kernel::recv' self)    ;; blocks until parent sends the cap
                       c    (:wat::kernel::connect' addr)
                       _    (:wat::kernel::send' c 7)
                       _got (:wat::kernel::recv' c)]      ;; 3b-b: EOF on the bounce → RAISES → die
                      nil))))
     ;; hand the (leaked) service capability DOWN to the stranger.
     _   (:wat::kernel::send' stranger svc-addr)
     got (:wat::kernel::recv' stranger)]                  ;; HEAD: stranger served; 3b-b: stranger died → RAISES
    got))
