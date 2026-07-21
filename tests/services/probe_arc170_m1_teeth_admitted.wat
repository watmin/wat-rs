;; probe_arc170_m1_teeth_admitted.wat — arc 170 M1-teeth, the ADMIT-via-grant control.
;;
;; Promoted from scratchpad/probe-m1-grant-admits.wat (GREEN): the ONE load-bearing path is
;;   (A) peer-pid reads a spawn-program' (process) prober's kernel pid  → (Some p)
;;   (B) a MANUAL grant of that pid into defservice A's allow-set ADMITS the prober's RAW
;;       connect' — the prober is a SEPARATE process whose pid ∉ A's birth-seed, so it is
;;       served ONLY because we granted it (the admit half of the teeth).
;;
;; This fixture returns the echoed reply so the Rust harness asserts Ok "echo:hi".

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

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
     ;; the prober — a SEPARATE process; receives A's addr (down), dials, echoes the reply UP.
     prober (:wat::kernel::spawn-program' (:wat::spawn::process)
              (:wat::core::forms
                ;; the child evals in a FRESH world — it must re-declare the surface it dials
                ;; (deterministic derivation → wire-identical Op/Reply; arc-054 idempotent).
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
                     :Ok              [reply <- :wat::core::String]
                     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer
                            :wat::core::String
                            :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>)
                     addr (:wat::kernel::recv' self)
                     c    (:wat::kernel::connect' addr)
                     er   (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::kernel::send' self (:wat::core::match er -> :wat::core::String
                              ((:probe::Echo::EchoResponse::Ok reply) reply)
                              ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                                (:wat::kernel::assertion-failed! "prober dial: unexpected RequestTooLarge"
                                  :wat::core::None :wat::core::None))))]
                    nil))))
     ;; capture the prober's kernel pid and grant it into A's allow-set (ack'd: PeersAllowed).
     _   (:wat::core::match (:wat::kernel::peer-pid prober) -> :wat::core::nil
           ((:wat::core::Some p)
             (:probe::echo'/grant eh (:wat::core::Vector :wat::core::i64 p)))
           (:wat::core::None
             (:wat::kernel::assertion-failed! "peer-pid returned None on a process prober"
               :wat::core::None :wat::core::None)))
     ;; hand A's addr down; the prober dials — served ONLY because we granted its pid.
     _   (:wat::kernel::send' prober ea)
     out (:wat::kernel::recv' prober)]
    out))
