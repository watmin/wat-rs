;; probe-optA-retag.wat — arc 278 Stone 2 STEP 0 (the ⛔ RE-TAG probe; the one novel mechanism).
;;
;; THE LOAD-BEARING COMPOSITION the O-side (Option A) rests on:
;;   A `defservice` dispatches over a synthesized `<service>::Op` SUPERSET (surface variants +
;;   internal `-`-ops). But a client can only ever construct a `<surface>::Op` value — the wire type.
;;   So on the REAL decode path a client's op arrives runtime-tagged `<surface>::Op::X` while its
;;   STATIC type (from `poll'`'s `selectables` element `O`) is already `<service>::Op`. The runtime
;;   enum matcher composes `type_path::variant` (runtime.rs `try_match_pattern`), so a
;;   `<service>::Op::X` pattern would NOT fire on a surface-tagged value. The RE-TAG bridges it:
;;   `(:wat::kernel::retag-op' op :<surface>::Op :<service>::Op)` embeds the surface value into its
;;   service-Op counterpart (same variant + fields); a timer's internal op — already service-tagged —
;;   passes through unchanged.
;;
;; Here Surface and Svc are DISTINCT enums (unlike probe-selectables-homogeneity.wat's single shared
;; Op) — that split IS what STEP 0 adds: the client speaks `Surface::Op`, the serve loop speaks the
;; `Svc::Op` superset, and the re-tag turns the former into the latter at the decode boundary.
;;
;; GREEN (both tiers, exit 0) = the re-tag holds on the REAL poll'/decode path: a real client's
;;   `Surface::Op::Ping` wire frame is delivered by poll', re-tagged to `Svc::Op::Ping`, and matches a
;;   `Svc::Op::Ping` pattern; a timer's `Svc::Op::Tick` (in-process, thread; re-decoded, process)
;;   passes the re-tag through and matches `Svc::Op::Tick`. The server replies `Surface::Reply::Pong`
;;   only AFTER it has seen BOTH — so a returned `:Pong` PROVES both the re-tagged client op and the
;;   pass-through timer op dispatched from the one `Svc::Op` match. RED = the re-tag is wrong; the
;;   checker/runtime names exactly where — before the macro is built on top.

;; ── the SURFACE op/reply (the ONLY thing a client can construct — the wire type) ─────────────────
(:wat::core::defenum :probe-retag::Surface::Op :wat::enum::Pure
  :Ping [])
(:wat::core::defenum :probe-retag::Surface::Reply :wat::enum::Pure
  :Pong [])

;; ── the SVC op SUPERSET (surface :Ping counterpart + the internal :Tick) — the serve-loop's O ────
(:wat::core::defenum :probe-retag::Svc::Op :wat::enum::Pure
  :Ping []
  :Tick [])

;; ── THREAD tier ──────────────────────────────────────────────────────────────────────────────────
;; serve threads: saw-tick? + the client's idx (-1 = not yet connected). Replies :Pong to the client
;; ONLY once BOTH the timer's :Tick and the client's (re-tagged) :Ping have been delivered by poll'.
(:wat::core::defn :probe-retag::serve-thread
  [self        <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::nil :wat::core::nil])
   l           <- (:wat::kernel::Listener :- [:probe-retag::Surface::Op :probe-retag::Surface::Reply])
   selectables <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:probe-retag::Surface::Reply :probe-retag::Svc::Op])])
   saw-tick    <- :wat::core::bool
   client-idx  <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::poll self l selectables) 
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection peer)
      (:probe-retag::serve-thread self l (:wat::core::conj selectables peer) saw-tick client-idx))
    ((:wat::spawn::ServiceEvent::Message idx op)
      ;; ⛔ THE RE-TAG: a client op arrives Surface-tagged; embed it into the Svc::Op superset. A
      ;; timer's already-Svc::Op::Tick passes through unchanged.
      (:wat::core::match
          (:wat::kernel::retag-op op :probe-retag::Surface::Op :probe-retag::Svc::Op)
          
        ;; the TIMER delivered its :Tick (an internal op — passed through the re-tag):
        ((:probe-retag::Svc::Op::Tick)
          (:wat::core::if (:wat::i64::>= client-idx 0)
            (:wat::core::let
              [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth selectables client-idx) (:probe-retag::Surface::Reply::Pong)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
              nil)
            (:probe-retag::serve-thread self l selectables true client-idx)))
        ;; the CLIENT delivered its :Ping (a SURFACE op — RE-TAGGED to Svc::Op::Ping):
        ((:probe-retag::Svc::Op::Ping)
          (:wat::core::if saw-tick
            (:wat::core::let
              [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth selectables idx) (:probe-retag::Surface::Reply::Pong)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
              nil)
            (:probe-retag::serve-thread self l selectables saw-tick idx)))))
    ((:wat::spawn::ServiceEvent::Closed idx)
      (:probe-retag::serve-thread self l (:wat::seq::remove-at selectables idx) saw-tick client-idx))
    ((:wat::spawn::ServiceEvent::Lost idx _cause)
      (:probe-retag::serve-thread self l (:wat::seq::remove-at selectables idx) saw-tick client-idx))
    ((:wat::spawn::ServiceEvent::Malformed idx _cause)
      (:probe-retag::serve-thread self l selectables saw-tick client-idx))
    ((:wat::spawn::ServiceEvent::Rejected idx _cause)
      (:probe-retag::serve-thread self l (:wat::seq::remove-at selectables idx) saw-tick client-idx))
    ((:wat::spawn::ServiceEvent::Admin _m)
      (:probe-retag::serve-thread self l selectables saw-tick client-idx))))

(:wat::core::defn :probe-retag::thread-mix [] -> :probe-retag::Surface::Reply
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :probe-retag::Surface::Op :probe-retag::Surface::Reply)
     l    (:wat::spawn::Bound/listener pair)
     addr (:wat::spawn::Bound/address pair)
     _svc (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::nil :wat::core::nil])]
              -> :wat::core::nil
              (:wat::core::let
                [t (:wat::kernel::after :wat::program::PeerKind::thread
                     (:wat::time::Milliseconds 5) (:probe-retag::Svc::Op::Tick))]
                (:probe-retag::serve-thread self l
                  (:wat::core::Vector :- [(:wat::kernel::Peer :- [:probe-retag::Surface::Reply :probe-retag::Svc::Op])] t)
                  false -1))))
     c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _    (:wat::core::match (:wat::kernel::send c (:probe-retag::Surface::Op::Ping)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     r    (:wat::core::match (:wat::kernel::recv c)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; c was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': c closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    r))

;; ── PROCESS tier — IDENTICAL shape, forked child universe. The child re-declares Surface + Svc
;; (a process fork is a separate address space; the real defservice ships the surface via the
;; manifest + synthesizes Svc::Op) and sends its Address' UP the lineage self-peer. ────────────────
(:wat::core::defn :probe-retag::process-mix [] -> :probe-retag::Surface::Reply
  (:wat::core::let
    [svc  (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defenum :probe-retag::Surface::Op :wat::enum::Pure
                :Ping [])
              (:wat::core::defenum :probe-retag::Surface::Reply :wat::enum::Pure
                :Pong [])
              (:wat::core::defenum :probe-retag::Svc::Op :wat::enum::Pure
                :Ping []
                :Tick [])
              (:wat::core::defn :probe-retag::serve-proc
                [self        <- (:wat::kernel::Peer :- [(:wat::kernel::Address :- [:probe-retag::Surface::Op :probe-retag::Surface::Reply]) :wat::core::nil])
                 l           <- (:wat::kernel::Listener :- [:probe-retag::Surface::Op :probe-retag::Surface::Reply])
                 selectables <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:probe-retag::Surface::Reply :probe-retag::Svc::Op])])
                 saw-tick    <- :wat::core::bool
                 client-idx  <- :wat::core::i64]
                -> :wat::core::nil
                (:wat::core::match (:wat::kernel::poll self l selectables) 
                  (:wat::spawn::ServiceEvent::Shutdown nil)
                  ((:wat::spawn::ServiceEvent::Connection peer)
                    (:probe-retag::serve-proc self l (:wat::core::conj selectables peer) saw-tick client-idx))
                  ((:wat::spawn::ServiceEvent::Message idx op)
                    (:wat::core::match
                        (:wat::kernel::retag-op op :probe-retag::Surface::Op :probe-retag::Svc::Op)
                        
                      ((:probe-retag::Svc::Op::Tick)
                        (:wat::core::if (:wat::i64::>= client-idx 0)
                          (:wat::core::let
                            [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth selectables client-idx) (:probe-retag::Surface::Reply::Pong)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                            nil)
                          (:probe-retag::serve-proc self l selectables true client-idx)))
                      ((:probe-retag::Svc::Op::Ping)
                        (:wat::core::if saw-tick
                          (:wat::core::let
                            [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth selectables idx) (:probe-retag::Surface::Reply::Pong)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                            nil)
                          (:probe-retag::serve-proc self l selectables saw-tick idx)))))
                  ((:wat::spawn::ServiceEvent::Closed idx)
                    (:probe-retag::serve-proc self l (:wat::seq::remove-at selectables idx) saw-tick client-idx))
                  ((:wat::spawn::ServiceEvent::Lost idx _cause)
                    (:probe-retag::serve-proc self l (:wat::seq::remove-at selectables idx) saw-tick client-idx))
                  ((:wat::spawn::ServiceEvent::Malformed idx _cause)
                    (:probe-retag::serve-proc self l selectables saw-tick client-idx))
                  ((:wat::spawn::ServiceEvent::Rejected idx _cause)
                    (:probe-retag::serve-proc self l (:wat::seq::remove-at selectables idx) saw-tick client-idx))
                  ((:wat::spawn::ServiceEvent::Admin _m)
                    (:probe-retag::serve-proc self l selectables saw-tick client-idx))))
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [b2   (:wat::kernel::listener (:wat::spawn::process) :probe-retag::Surface::Op :probe-retag::Surface::Reply)
                   self (:wat::program::self-peer (:wat::kernel::Address :- [:probe-retag::Surface::Op :probe-retag::Surface::Reply]) :wat::core::nil)
                   _sa  (:wat::kernel::send self (:wat::spawn::Bound/address b2))
                   t    (:wat::kernel::after :wat::program::PeerKind::process
                          (:wat::time::Milliseconds 5) (:probe-retag::Svc::Op::Tick))]
                  (:probe-retag::serve-proc self (:wat::spawn::Bound/listener b2)
                    (:wat::core::Vector :- [(:wat::kernel::Peer :- [:probe-retag::Surface::Reply :probe-retag::Svc::Op])] t)
                    false -1)))))
     addr (:wat::core::match (:wat::kernel::recv svc)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; svc was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': svc closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _    (:wat::core::match (:wat::kernel::send c (:probe-retag::Surface::Op::Ping)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     r    (:wat::core::match (:wat::kernel::recv c)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; c was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': c closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    r))

;; ── the assertion — BOTH tiers: the client's RE-TAGGED :Ping AND the timer's pass-through :Tick ────
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:probe-retag::thread-mix) 
      ((:probe-retag::Surface::Reply::Pong) (:wat::kernel::println "thread: Pong — re-tagged client :Ping + pass-through timer :Tick both dispatched")))
    (:wat::core::match (:probe-retag::process-mix) 
      ((:probe-retag::Surface::Reply::Pong) (:wat::kernel::println "process: Pong — re-tagged client :Ping + pass-through timer :Tick both dispatched")))))
