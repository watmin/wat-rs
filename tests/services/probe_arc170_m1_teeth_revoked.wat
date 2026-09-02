;; probe_arc170_m1_teeth_revoked.wat — arc 170 M1-teeth, the TEETH: revoke deterministically bites.
;;
;; A TWO-PHASE prober (a SEPARATE process). The owner:
;;   1. grants the prober's pid into A's allow-set (ack'd: PeersAllowed);
;;   2. hands A's addr down → dial #1 is ADMITTED (echo:hi reported UP);
;;   3. REVOKES the prober's pid (ack'd: PeersDenied — the pid is provably GONE);
;;   4. ONLY THEN sends the re-dial signal (a 2nd addr) → dial #2 is REFUSED → the prober's
;;      echo recv' EOFs → the prober RAISES → dies → the owner's recv' surfaces the death →
;;      :user::compute RAISES.
;;
;; DETERMINISM: the re-dial signal is sent only AFTER echo'/revoke returns (it blocks on the
;; PeersDenied ack). So revoke happens-before re-dial happens-before dial #2. NO race.
;;
;; The Rust harness asserts Err (compute raised) == the revoked dial #2 was bounced.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok
                (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

;; arc 278 VALUE-CONTRACT: the owner FACES the prober's death as a matchable RecvOutcome VALUE and
;; RETURNS this enum — never re-raises it with assertion-failed! (which panic_any's past apply_function).
;; Served carries dial-#2's reply to preserve the DISCRIMINATE intent (a regressed revoke → a real reply).
(:wat::core::defenum :probe::Outcome :wat::enum::Pure
  :Bounced []                          ;; dial #2 refused → prober crashed → owner saw Lost/Closed
  :Served  [reply <- :wat::core::String]) ;; dial #2 ADMITTED (revoke regressed) → prober replied → owner saw Message

(:wat::core::defn :user::compute [] -> :probe::Outcome
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea  (:probe::echo::Handle/addr eh)
     ;; the TWO-PHASE prober — a SEPARATE process; dials once (admitted), reports UP, blocks for
     ;; a re-dial signal, then dials again (which after revoke is refused → EOF → RAISE → die).
     prober (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                ;; the child evals in a FRESH world — it must re-declare the surface it dials.
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
                     :Ok              [reply <- :wat::core::String]
                     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String
                             (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]))
                     addr (:wat::core::match (:wat::kernel::recv self)               ;; A's addr (down)
                            ((:wat::kernel::RecvOutcome::Message m) m)
                            ((:wat::kernel::RecvOutcome::Lost cause)
                              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Stopped
                              (:wat::kernel::assertion-failed! "recv': stopped before the owner sent A's addr — the peer was ALIVE" :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Closed
                              (:wat::kernel::assertion-failed! "recv': owner closed (addr handoff)" :wat::core::None :wat::core::None)))
                     c1   (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest :msg "hi"))     ;; dial #1 — ADMITTED
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er1 ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
                              ((:probe::Echo::EchoResponse::Ok reply) reply)
                              ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                                (:wat::kernel::assertion-failed! "prober dial #1: unexpected RequestTooLarge"
                                  :wat::core::None :wat::core::None))
                              ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
                                (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil)) ;; report "echo:hi" UP
                     _sig (:wat::core::match (:wat::kernel::recv self)               ;; BLOCK for re-dial (2nd addr)
                            ((:wat::kernel::RecvOutcome::Message m) m)
                            ((:wat::kernel::RecvOutcome::Lost cause)
                              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Stopped
                              (:wat::kernel::assertion-failed! "recv': stopped before the owner sent the re-dial signal — the peer was ALIVE" :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Closed
                              (:wat::kernel::assertion-failed! "recv': owner closed (re-dial signal)" :wat::core::None :wat::core::None)))
                     c2   (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest :msg "hi"))     ;; dial #2 — after revoke: BOUNCED → RAISE → die (before the send below)
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er2 ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
                              ((:probe::Echo::EchoResponse::Ok reply) reply)
                              ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                                (:wat::kernel::assertion-failed! "prober dial #2: unexpected RequestTooLarge"
                                  :wat::core::None :wat::core::None))
                              ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
                                (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))] ;; dial #2 reply UP — ONLY reached if ADMITTED. makes the test DISCRIMINATE: if the revoke ever regressed, dial #2 admits, this fires, the owner's r2 = "echo:hi" → compute Ok → the test (asserts Err) goes RED. without it, the prober's clean exit ALSO disconnects the channel → recv' raises → Err either way (vacuous).
                    nil))))
     r2  (:wat::core::match (:wat::kernel::peer-pid prober) 
           ((:wat::core::Some p)
             (:wat::core::let
               [_  (:probe::echo/grant  eh (:wat::core::Vector :- [:wat::core::i64] p)) ;; ack'd PeersAllowed
                _  (:wat::core::match (:wat::kernel::send prober ea) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))                                   ;; arc 278 #73 — the recv' below already faces the stop ;; give addr → dial #1
                r1 (:wat::core::match (:wat::kernel::recv prober)                    ;; "echo:hi" (dial #1 admitted); ::Message passes m through so the DISCRIMINATE assert holds
                     ((:wat::kernel::RecvOutcome::Message m) m)
                     ((:wat::kernel::RecvOutcome::Lost cause)
                       (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Stopped
                       (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Closed
                       (:wat::kernel::assertion-failed! "recv': prober closed" :wat::core::None :wat::core::None)))
                _  (:probe::echo/revoke eh (:wat::core::Vector :- [:wat::core::i64] p)) ;; ack'd PeersDenied — pid GONE
                _  (:wat::core::match (:wat::kernel::send prober ea) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))                                   ;; arc 278 #73 — the recv' below already faces the stop ;; re-dial signal (AFTER revoke ack)
                r2 (:wat::core::match (:wat::kernel::recv prober)                    ;; owner FACES the outcome as a VALUE, returns the enum
                     ((:wat::kernel::RecvOutcome::Message m) (:probe::Outcome::Served m))  ;; dial #2 admitted (the regression) — carries the reply
                     ((:wat::kernel::RecvOutcome::Lost cause) (:probe::Outcome::Bounced))  ;; prober crashed on bounce = correct
                     ;; arc 278 #73 — a stop is neither a bounce nor a serve; this enum has no third
                     ;; arm, and Bounced is the closer read ("not served" holds under Stopped too) —
                     ;; JUDGEMENT CALL, flagged for review.
                     (:wat::kernel::RecvOutcome::Stopped (:probe::Outcome::Bounced))
                     (:wat::kernel::RecvOutcome::Closed (:probe::Outcome::Bounced)))]      ;; prober closed without a reply = not served
               r2))                                                                  ;; the enum outcome
           (:wat::core::None
             (:wat::kernel::assertion-failed! "peer-pid None on process prober"
               :wat::core::None :wat::core::None)))]
    r2))
