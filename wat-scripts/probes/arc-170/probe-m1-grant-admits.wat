;; probe-m1-grant-admits.wat — DISCONFIRMING PROBE for M1's teeth (arc 170).
;;
;; Isolates the ONE load-bearing unknown for the M1 live-prober strike:
;;   (A) peer-pid reads a spawn-program' (process) prober's kernel pid  → (Some p)
;;   (B) a MANUAL grant of that pid into a defservice A's allow-set ADMITS the prober's
;;       RAW connect' — the prober is a SEPARATE process whose pid ∉ A's birth-seed, so it
;;       is served ONLY because we granted it. This is grant enabling a real dial that the
;;       birth-seed alone would refuse (the admit half of the teeth).
;;
;; The refuse half (revoke → bounced → EOF → raise) is already PROVEN by
;; tests/services/probe_arc209_c0b3bb_bounced.rs::stranger_is_bounced — this probe does NOT
;; re-prove it; it isolates only the un-grounded admit-via-grant path.
;;
;; EXPECT (green):  echo:hi        (the granted prober dialed A and was served)
;; If grant does NOT admit → the prober's echo recv' EOFs → the prober DIES → the owner's
;; recv' on the prober RAISES → the program errors out (non-zero exit, a raise) on that line.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea  (:probe::echo::Handle/addr eh)
     ;; the prober — a SEPARATE process; receives A's addr (down), dials, echoes the reply UP.
     prober (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                ;; the child evals in a FRESH world — it must re-declare the surface it dials
                ;; (deterministic derivation → wire-identical Op/Reply; arc-054 idempotent).
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer
                            :wat::core::String
                            (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]))
                     addr (:wat::kernel::recv self)
                     c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                     er   (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                    nil))))
     ;; capture the prober's kernel pid and grant it into A's allow-set (ack'd: PeersAllowed).
     _   (:wat::core::match (:wat::kernel::peer-pid prober) 
           ((:wat::core::Some p)
             (:probe::echo/grant eh (:wat::core::Vector :- [:wat::core::i64] p)))
           (:wat::core::None
             (:wat::kernel::assertion-failed! "peer-pid returned None on a process prober"
               :wat::core::None :wat::core::None)))
     ;; hand A's addr down; the prober dials — served ONLY because we granted its pid.
     _   (:wat::core::match (:wat::kernel::send prober ea) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     out (:wat::kernel::recv prober)]
    (:wat::kernel::println out)))
