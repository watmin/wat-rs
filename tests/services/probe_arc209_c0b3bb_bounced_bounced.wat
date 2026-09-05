;; Proof 2: a real stranger child (pid ≠ owner) is bounced.
;; The owner recv's the service capability, then spawns a SEPARATE stranger process and HANDS
;; the (leaked) service address DOWN to the stranger over the stranger's lineage channel.

;; arc 278 VALUE-CONTRACT: the owner FACES the child's death as a matchable RecvOutcome VALUE and
;; RETURNS this enum — never re-raises it with assertion-failed! (which panic_any's past apply_function).
(:wat::core::defenum :probe::Outcome :wat::enum::Pure
  :Bounced []    ;; the stranger was refused → crashed on the bounce → owner saw Lost
  :Served  [])   ;; the stranger was served (the gate FAILED/regressed) → clean exit → owner saw Closed

(:wat::core::defn :user::compute [] -> :probe::Outcome
  (:wat::core::let
    [svc     (:wat::test::spawn-peer (:wat::spawn::process)
               (:wat::core::forms
                (:wat::core::defn :user::serve
                  [self    <- (:wat::kernel::Peer :- [(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64])
                   l       <- (:wat::kernel::Listener :- [:wat::core::i64 :wat::core::i64])
                   clients <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])])]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::poll self l clients) 
                    (:wat::spawn::ServiceEvent::Shutdown nil)
                    ((:wat::spawn::ServiceEvent::Connection peer)
                      (:user::serve self l (:wat::core::conj clients peer)))
                    ((:wat::spawn::ServiceEvent::Message idx n)
                      (:wat::core::let [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth clients idx)
                                             (:wat::core::+ n 100)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                        (:user::serve self l clients)))
                    ((:wat::spawn::ServiceEvent::Closed idx)
                      (:user::serve self l (:wat::seq::remove-at clients idx)))
                    ((:wat::spawn::ServiceEvent::Lost idx _cause)
                      (:user::serve self l (:wat::seq::remove-at clients idx)))
                    ;; Admin wildcard — arc 291 new variant; not exercised by this probe.
                    (_ nil)))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [b    (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                     self (:wat::program::self-peer
                             (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64)
                     _    (:wat::core::match (:wat::kernel::send self (:wat::spawn::Bound/address b)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                    (:user::serve self (:wat::spawn::Bound/listener b)
                      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])]))))))
     ;; recv' the service's minted capability (blocks until service sends it).
     svc-addr (:wat::core::match (:wat::kernel::recv svc)
                ((:wat::kernel::RecvOutcome::Message m) m)
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::kernel::assertion-failed! "recv': svc closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     ;; A SEPARATE process child — its pid ≠ the owner's → NOT in the birth-seeded allow-set.
     ;; The owner hands the (leaked) service address DOWN to the stranger via its lineage channel.
     ;; stranger self-peer: S=i64 (would send up — never does), R=(Address' :- [i64 i64]) (receives cap).
     stranger (:wat::test::spawn-peer (:wat::spawn::process)
                (:wat::core::forms
                  (:wat::core::defn :user::main [] -> :wat::core::nil
                    (:wat::core::let
                      ;; receive the leaked service address from the owner via our lineage channel.
                      [self (:wat::program::self-peer
                               :wat::core::i64
                               (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]))
                       addr (:wat::core::match (:wat::kernel::recv self)  ;; blocks until parent sends the cap
                              ((:wat::kernel::RecvOutcome::Message m) m)
                              ((:wat::kernel::RecvOutcome::Lost cause)
                                (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                              (:wat::kernel::RecvOutcome::Stopped
                                (:wat::kernel::assertion-failed! "recv': stopped before the owner sent the cap — the peer was ALIVE" :wat::core::None :wat::core::None))
                              (:wat::kernel::RecvOutcome::Closed
                                (:wat::kernel::assertion-failed! "recv': owner closed (cap handoff)" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
                       c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                       _    (:wat::core::match (:wat::kernel::send c 7) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                       ;; 3b-b: the stranger is bounced → the service drops the stream → this recv'
                       ;; sees Closed (EOF on the bounce) → we RAISE → the stranger process DIES.
                       ;; arc 278 #73 — a stop here is likewise not a served reply; RAISE the same way
                       ;; Closed does, worded distinctly — JUDGEMENT CALL, flagged for review.
                       _got (:wat::core::match (:wat::kernel::recv c)
                              ((:wat::kernel::RecvOutcome::Message m) m)
                              ((:wat::kernel::RecvOutcome::Lost cause)
                                (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                              (:wat::kernel::RecvOutcome::Stopped
                                (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                              (:wat::kernel::RecvOutcome::Closed
                                (:wat::kernel::assertion-failed! "stranger bounced: service closed the stream" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
                      nil))))
     ;; hand the (leaked) service capability DOWN to the stranger.
     _   (:wat::core::match (:wat::kernel::send stranger svc-addr) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))  ;; arc 278 #73 — the recv' below already faces the stop
     got (:wat::core::match (:wat::kernel::recv stranger)  ;; owner FACES the outcome as a VALUE, returns the enum
           ((:wat::kernel::RecvOutcome::Lost cause) (:probe::Outcome::Bounced))   ;; child crashed on bounce = correct
           ;; arc 278 #73 — a stop is neither a crash nor a clean exit; this enum has no third arm,
           ;; and Bounced is the closer read (does not falsely claim the regression) — JUDGEMENT
           ;; CALL, flagged for review.
           (:wat::kernel::RecvOutcome::Stopped (:probe::Outcome::Bounced))
           (:wat::kernel::RecvOutcome::Closed (:probe::Outcome::Served))          ;; stranger served then exited cleanly = regression
           ((:wat::kernel::RecvOutcome::Message m) (:probe::Outcome::Served)) (:wat::kernel::RecvOutcome::TimedOut (:probe::Outcome::Bounced)))]   ;; defensive (stranger never sends up)
    got))
