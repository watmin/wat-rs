;; Arc 278 #16.2 — RED gate: the per-op `:max-request-bytes` ENFORCEMENT CODEGEN.
;;
;; The op DECLARES `:max-request-bytes 200` on the SURFACE; the `:impls` body does NOT
;; hand-roll the measure — it returns a bare `:Ok`. So an over-200 request must be flagged
;; `RequestTooLarge` BY THE SERVE-LOOP CODEGEN (16.2), not the body, and a follow-up in-budget
;; request on the SAME connection must still return `:Ok` (the per-op tier keeps the connection;
;; the request arrived <= FOO, so the wire is synced — the opposite of the transport-FOO kick).
;;
;; RED NOW: with no codegen enforcement, the body's `:Ok` comes back for the over-cap request
;; (test 1 returns -1). GREEN AFTER 16.2: the codegen measures + returns `RequestTooLarge` before
;; the body runs. This is the feasibility probe (per_op_request_too_large) with the measure MOVED
;; OUT of the body — proving 16.2 turns hand-discipline (option a) into structure (option c).

(:wat::core::defsurface :probe::Cap1 :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Cap1::DoOpRequest [payload <- :wat::core::String])
   (:wat::core::defenum :probe::Cap1::DoOpResponse :wat::enum::Pure
     :Ok              [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(do-op [self <- :probe::Cap1  req <- :probe::Cap1::DoOpRequest] -> :probe::Cap1::DoOpResponse
     :max-request-bytes 200)])

;; The satisfier: the body does NOT measure — it returns bare `:Ok`. Enforcement MUST come from
;; the serve-loop codegen (16.2), NOT the body (a hand-rolling body is the feasibility probe = a).
(:wat::service::defservice :probe::cap1svc
  :satisfies :probe::Cap1
  :durable   []
  :ephemeral []
  :impls
  [(do-op [s ctx req]
     (:wat::service::Outcome::Reply s (:probe::Cap1::DoOpResponse::Ok 0)))])

;; Build an ASCII string of n*32 bytes (byte-length == char-length for ASCII).
(:wat::core::defn :probe::pl [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc "0123456789ABCDEF0123456789ABCDEF"))
    ""
    (:wat::core::range 0 n)))

;; (1) an over-cap request must be flagged `RequestTooLarge` BY THE CODEGEN (the body returns
;;     bare `:Ok`). Returns `bytes` (> 200) on the RequestTooLarge arm, else -1 (the Ok arm) so
;;     the harness can distinguish. RED NOW: no codegen → the body's `:Ok 0` comes back → -1.
(:wat::core::defn :user::over-op-codegen-flags [] -> :wat::core::i64
  (:wat::core::let
    [big (:probe::pl 20)   ;; 20*32 = 640-byte payload → encoded request > the 200 cap, < FOO
     h   (:probe::cap1svc/start :locus (:wat::spawn::process) :record (:probe::cap1svc::Record))
     c   (:wat::core::match (:wat::kernel::connect (:probe::cap1svc::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r   (:probe::Cap1/do-op c (:probe::Cap1::DoOpRequest :payload big))]
    (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:probe::Cap1::DoOpResponse::RequestTooLarge bytes cap) bytes)
      ((:probe::Cap1::DoOpResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))
      ((:probe::Cap1::DoOpResponse::Ok n) -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; (2) the SAME connection recovers IN PLACE: an over-cap request (→ RequestTooLarge from the
;;     codegen, connection KEPT) then an in-budget request on the SAME peer `c` (→ Ok). Returns 1
;;     if the connection survived the flag, else -1. (Before 16.2 this trivially passes — both
;;     requests just `:Ok`; after 16.2 it guards that the RequestTooLarge path does not evict.)
(:wat::core::defn :user::same-conn-recovers-after-codegen [] -> :wat::core::i64
  (:wat::core::let
    [big (:probe::pl 20)     ;; > cap
     h   (:probe::cap1svc/start :locus (:wat::spawn::process) :record (:probe::cap1svc::Record))
     c   (:wat::core::match (:wat::kernel::connect (:probe::cap1svc::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r1  (:probe::Cap1/do-op c (:probe::Cap1::DoOpRequest :payload big))     ;; RequestTooLarge; keep
     r2  (:probe::Cap1/do-op c (:probe::Cap1::DoOpRequest :payload "hi"))]   ;; SAME c → Ok
    (:wat::core::match r2 ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:probe::Cap1::DoOpResponse::Ok n) 1)
      ((:probe::Cap1::DoOpResponse::RequestTooLarge bytes cap) -1)
      ((:probe::Cap1::DoOpResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
