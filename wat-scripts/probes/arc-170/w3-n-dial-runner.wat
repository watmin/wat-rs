;; DISCONFIRMING PROBE (W3 / C2 N-runtime) — freeze-only (--check), no fork.
;; QUESTION: does the N-peer dial-runner TYPE-COMPOSE? i.e. can a runner recv a
;; PoolMsg carrying a HETEROGENEOUS (Tuple :- [(Address' :- [Echo]) (Address' :- [Kv])]), connect' EACH
;; component into its own typed Peer', hold the pair, and run a 2-peer work-fn?
;; C1's process-dial-runner :- [S R I O] (bracket.wat:82) is SINGLE-peer. This hand-writes
;; the N=2 generalization and asks the checker to type it. If it freezes clean, the
;; runtime is "generalize the codegen"; if it fails, the error names the exact gap.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable []  :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req))))])

(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest  [k <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure :Ok [v <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::kv
  :satisfies :probe::Kv  :durable []  :ephemeral []
  :impls [(get [s ctx req]
            (:wat::service::Outcome::Reply s
              (:probe::Kv::GetResponse::Ok (:probe::Kv::GetRequest/k req))))])

;; The hand-written N=2 dial-runner — the shape W3's codegen would emit. Item I = String,
;; O = String. The carrier D = (Tuple :- [(Address' :- [Echo]) (Address' :- [Kv])]); ctx holds the dialed pair.
(:wat::core::defn :probe::multi-dial-runner
  [self    <- (:wat::kernel::Peer :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) (:wat::bracket::PoolMsg :- [(:wat::core::Tuple :- [(:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]) (:wat::kernel::Address :- [:probe::Kv::Op :probe::Kv::Reply])]) :wat::core::String])])
   work-fn <- [(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply]) (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply]) :wat::core::String :-> :wat::core::String]
   ctx     <- (:wat::core::Option :- [(:wat::core::Tuple :- [(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply]) (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply])])])]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv self)
    ((:wat::kernel::RecvOutcome::Message m)
      (:wat::core::match m
        ((:wat::bracket::PoolMsg::Setup deps)
          ;; deps : (Tuple :- [(Address' :- [Echo]) (Address' :- [Kv])]) — connect' EACH component into its typed Peer'
          (:probe::multi-dial-runner self work-fn
            (:wat::core::Some
              (:wat::core::Tuple
                (:wat::core::match (:wat::kernel::connect (:wat::core::first deps)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                (:wat::core::match (:wat::kernel::connect (:wat::core::second deps)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))))
        ((:wat::bracket::PoolMsg::Work pair)
          (:wat::core::let
            [c   (:wat::core::Option/expect ctx "multi-dial-runner: Work before Setup")
             out (:wat::core::Tuple (:wat::core::first pair)
                   (work-fn (:wat::core::first c) (:wat::core::second c) (:wat::core::second pair)))
             ;; arc 278 #73 — discard-only send; the recv' at the top of the next iteration
             ;; faces a stop as its own outcome.
             _   (:wat::core::match (:wat::kernel::send self out) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
            (:probe::multi-dial-runner self work-fn ctx)))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    ;; arc 278 #73 — single self-peer worker (no "keep serving others" distinction from
    ;; "the world is ending"): either way this loop's one channel is done. Same body as
    ;; Closed, stated by name rather than folded together silently.
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))
