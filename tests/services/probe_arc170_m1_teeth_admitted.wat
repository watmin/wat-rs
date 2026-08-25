;; probe_arc170_m1_teeth_admitted.wat — arc 170 M1-teeth, the ADMIT-via-grant control.
;;
;; Promoted from scratchpad/probe-m1-grant-admits.wat (GREEN): the ONE load-bearing path is
;;   (A) peer-pid reads a spawn-program' (process) prober's kernel pid  → (Some p)
;;   (B) a MANUAL grant of that pid into defservice A's allow-set ADMITS the prober's RAW
;;       connect' — the prober is a SEPARATE process whose pid ∉ A's birth-seed, so it is
;;       served ONLY because we granted it (the admit half of the teeth).
;;
;; This fixture returns the echoed reply so the Rust harness asserts Ok "echo:hi".

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
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok
                (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defn :user::compute [] -> :wat::core::String
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
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
                     :Ok              [reply <- :wat::core::String]
                     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer
                            :wat::core::String
                            (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]))
                     addr (:wat::core::match (:wat::kernel::recv self)
                            ((:wat::kernel::RecvOutcome::Message m) m)
                            ((:wat::kernel::RecvOutcome::Lost cause)
                              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Stopped
                              (:wat::kernel::assertion-failed! "recv': stopped before the owner sent A's addr — the peer was ALIVE" :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Closed
                              (:wat::kernel::assertion-failed! "recv': self closed before the owner sent A's addr" :wat::core::None :wat::core::None)))
                     c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                     er   (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
                              ((:probe::Echo::EchoResponse::Ok reply) reply)
                              ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                                (:wat::kernel::assertion-failed! "prober dial: unexpected RequestTooLarge"
                                  :wat::core::None :wat::core::None))
                              ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
                                (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                    nil))))
     ;; capture the prober's kernel pid and grant it into A's allow-set (ack'd: PeersAllowed).
     _   (:wat::core::match (:wat::kernel::peer-pid prober) 
           ((:wat::core::Some p)
             (:probe::echo/grant eh (:wat::core::Vector :wat::core::i64 p)))
           (:wat::core::None
             (:wat::kernel::assertion-failed! "peer-pid returned None on a process prober"
               :wat::core::None :wat::core::None)))
     ;; hand A's addr down; the prober dials — served ONLY because we granted its pid.
     _   (:wat::core::match (:wat::kernel::send prober ea) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))  ;; arc 278 #73 — the recv' below already faces the stop
     out (:wat::core::match (:wat::kernel::recv prober)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped before reporting the echo reply — the peer was ALIVE" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': prober closed before reporting the echo reply" :wat::core::None :wat::core::None)))]
    out))
