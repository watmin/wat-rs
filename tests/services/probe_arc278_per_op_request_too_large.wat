;; Arc 278 #16 (c) — FEASIBILITY probe for the per-op `:max-request-bytes` tier.
;;
;; This proves 16.2's MECHANISM hand-rolled, BEFORE the checker+codegen strike (16.0/16.1/16.2):
;;   an op whose `<Op>Response` is an ENUM carrying `RequestTooLarge{bytes, cap}`; the `:impls`
;;   body measures the encoded request (`:wat::edn::write` length) and, over a small cap, returns
;;   the NAMED variant. The client MATCHES it (a matchable value, NOT a raise), and the SAME
;;   connection stays alive for a follow-up in-budget request — recoverable IN PLACE (the request
;;   arrived <= FOO, so the wire is synced; the opposite of the transport-FOO kick).
;;
;; GREEN here = the foundation (c) codegens into is sound (measure + construct-by-name + reply +
;; keep-connection all compose on the substrate). The strike then MOVES the measure+construct out
;; of this hand-written body into the serve-loop codegen (16.2), FORCED by the checker (16.1) —
;; turning hand-discipline (option a) into structure (option c).

(:wat::core::defsurface :probe::Op1 :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Op1::DoOpRequest [payload <- :wat::core::String])
   (:wat::core::defenum :probe::Op1::DoOpResponse :wat::enum::Pure
     :Ok              [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(do-op [self <- :probe::Op1  req <- :probe::Op1::DoOpRequest] -> :probe::Op1::DoOpResponse :max-request-bytes 524288)])

;; The satisfier: the body itself measures + returns the variant (hand-rolled = option a; the
;; mechanism the checker-forced serve-loop will codegen = option c).
(:wat::service::defservice :probe::op1svc
  :satisfies :probe::Op1
  :durable   []
  :ephemeral []
  :impls
  [(do-op [s ctx req]
     (:wat::core::let
       [enc (:wat::edn::write req)
        n   (:wat::string::length enc)
        cap 200]
       (:wat::core::if (:wat::core::> n cap)
         (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Op1::Reply::DoOp (:probe::Op1::DoOpResponse::RequestTooLarge n cap))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Op1::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::op1svc::Op])]))
         (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Op1::Reply::DoOp (:probe::Op1::DoOpResponse::Ok n))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Op1::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::op1svc::Op])])))))])

;; Build an ASCII string of n*32 bytes (byte-length == char-length for ASCII).
(:wat::core::defn :probe::pl [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc "0123456789ABCDEF0123456789ABCDEF"))
    ""
    (:wat::core::range 0 n)))

;; (1) an over-cap request returns the MATCHABLE `RequestTooLarge{bytes, cap}` variant — a value
;;     the client `match`es, not a raise. Returns `bytes` (a positive i64 > cap) on RequestTooLarge,
;;     else -1 (the Ok arm) so the harness can distinguish.
(:wat::core::defn :user::over-op-returns-matchable [] -> :wat::core::i64
  (:wat::core::let
    [big (:probe::pl 20)   ;; 20*32 = 640-byte payload → encoded request > the 200 cap
     h   (:probe::op1svc/start :locus (:wat::spawn::process) :record (:probe::op1svc::Record))
     c   (:wat::core::match (:wat::kernel::connect (:probe::op1svc::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r   (:probe::Op1/do-op c (:probe::Op1::DoOpRequest :payload big))]
    ;; arc 278 the recv'-outcome wall — `do-op` now returns a matchable
    ;; `(RecvOutcome :- [DoOpResponse])`; the happy-path Response comes through ::Message.
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::Op1::DoOpResponse::RequestTooLarge bytes cap) bytes)
          ((:probe::Op1::DoOpResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))
          ((:probe::Op1::DoOpResponse::Ok n) -1)))
      ((:wat::kernel::RecvOutcome::Lost _cause) -2)
      ;; arc 278 #73 — a stop is neither the death (-2) nor the close (-3) this probe already
      ;; distinguishes; -4 names it as its own terminal outcome rather than folding it into either.
      (:wat::kernel::RecvOutcome::Stopped -4)
      (:wat::kernel::RecvOutcome::Closed -3))))

;; (2) the SAME connection recovers IN PLACE: an over-cap request (→ RequestTooLarge, connection
;;     KEPT — the request arrived, so it is a normal reply, no eviction) then an in-budget request
;;     on the SAME peer `c` (→ Ok). Returns the Ok `n` (> 0) if the connection survived, else -1.
(:wat::core::defn :user::same-conn-recovers [] -> :wat::core::i64
  (:wat::core::let
    [big   (:probe::pl 20)     ;; > cap
     h     (:probe::op1svc/start :locus (:wat::spawn::process) :record (:probe::op1svc::Record))
     c     (:wat::core::match (:wat::kernel::connect (:probe::op1svc::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r1    (:probe::Op1/do-op c (:probe::Op1::DoOpRequest :payload big))    ;; RequestTooLarge; keep
     r2    (:probe::Op1/do-op c (:probe::Op1::DoOpRequest :payload "hi"))]  ;; SAME c → Ok
    ;; arc 278 the recv'-outcome wall — the in-budget Ok Response comes through ::Message.
    (:wat::core::match r2
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::Op1::DoOpResponse::Ok n) n)
          ((:probe::Op1::DoOpResponse::RequestTooLarge bytes cap) -1)
          ((:probe::Op1::DoOpResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost _cause) -2)
      ;; arc 278 #73 — same sentinel scheme as above: -4 is the stop, distinct from -2/-3.
      (:wat::kernel::RecvOutcome::Stopped -4)
      (:wat::kernel::RecvOutcome::Closed -3))))
