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

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok
                (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [eh  (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea  (:probe::echo'::Handle/addr eh)
     ;; the TWO-PHASE prober — a SEPARATE process; dials once (admitted), reports UP, blocks for
     ;; a re-dial signal, then dials again (which after revoke is refused → EOF → RAISE → die).
     prober (:wat::kernel::spawn-program' (:wat::spawn::process)
              (:wat::core::forms
                ;; the child evals in a FRESH world — it must re-declare the surface it dials.
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
                     :Ok              [reply <- :wat::core::String]
                     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String
                             :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>)
                     addr (:wat::kernel::recv' self)                                  ;; A's addr (down)
                     c1   (:wat::kernel::connect' addr)
                     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest :msg "hi"))     ;; dial #1 — ADMITTED
                     _    (:wat::kernel::send' self (:wat::core::match er1 -> :wat::core::String
                              ((:probe::Echo::EchoResponse::Ok reply) reply)
                              ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                                (:wat::kernel::assertion-failed! "prober dial #1: unexpected RequestTooLarge"
                                  :wat::core::None :wat::core::None)))) ;; report "echo:hi" UP
                     _sig (:wat::kernel::recv' self)                                  ;; BLOCK for re-dial (2nd addr)
                     c2   (:wat::kernel::connect' addr)
                     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest :msg "hi"))     ;; dial #2 — after revoke: BOUNCED → RAISE → die (before the send below)
                     _    (:wat::kernel::send' self (:wat::core::match er2 -> :wat::core::String
                              ((:probe::Echo::EchoResponse::Ok reply) reply)
                              ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                                (:wat::kernel::assertion-failed! "prober dial #2: unexpected RequestTooLarge"
                                  :wat::core::None :wat::core::None))))] ;; dial #2 reply UP — ONLY reached if ADMITTED. makes the test DISCRIMINATE: if the revoke ever regressed, dial #2 admits, this fires, the owner's r2 = "echo:hi" → compute Ok → the test (asserts Err) goes RED. without it, the prober's clean exit ALSO disconnects the channel → recv' raises → Err either way (vacuous).
                    nil))))
     r2  (:wat::core::match (:wat::kernel::peer-pid prober) -> :wat::core::String
           ((:wat::core::Some p)
             (:wat::core::let
               [_  (:probe::echo'/grant  eh (:wat::core::Vector :wat::core::i64 p)) ;; ack'd PeersAllowed
                _  (:wat::kernel::send' prober ea)                                   ;; give addr → dial #1
                r1 (:wat::kernel::recv' prober)                                      ;; "echo:hi" (dial #1 admitted)
                _  (:probe::echo'/revoke eh (:wat::core::Vector :wat::core::i64 p)) ;; ack'd PeersDenied — pid GONE
                _  (:wat::kernel::send' prober ea)                                   ;; re-dial signal (AFTER revoke ack)
                r2 (:wat::kernel::recv' prober)]                                     ;; dial #2 → prober dies → RAISES
               r2))                                                                  ;; unreached; compute raises
           (:wat::core::None
             (:wat::kernel::assertion-failed! "peer-pid None on process prober"
               :wat::core::None :wat::core::None)))]
    r2))
