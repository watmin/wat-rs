;; SEVERITY: after a mistyped frame, does the service still serve OTHER clients?
;;
;; ARC 278 STONE 2 — THIS PROBE IS THE ACCEPTANCE GATE, AND IT FLIPPED.
;;
;; Its service opts into NOTHING. It declares a surface, a state, and a handler that uses the
;; request field at its declared type (`(string::length (nth items 0))` against
;; `items <- (Vector :- [String])`) — correct against the declaration, and undefended. Before Stone 2
;; the attacker's `#dos.Bag/PutRequest {:items [1 2 3]}` detonated inside that handler and the
;; service DIED FOR EVERY CLIENT:
;;
;;     "attacker good  => Ok"
;;     "attacker BAD   => LOST (peer gone)"
;;     victim: connect REFUSED — service is GONE          ← an innocent second client, killed
;;
;; Nothing about the service changed. What changed is that `defservice` now generates the
;; request-SHAPE guard into every op arm unconditionally — there is no clause to set, no default
;; to flip — and `:RequestMalformed` is checker-forced onto every serviceable op-Response, so the
;; refusal is a value this caller's exhaustive match CANNOT ignore. The two additions below (the
;; variant on the enum, the arm on the match) are that mandate landing, applied by the recorded
;; codemod `wat-scripts/fixes/mandate-request-malformed.wat` — not an opt-in.
;;
;;     "attacker good  => Ok"
;;     "attacker BAD   => MALFORMED at ["items" "[0]"] expected=:wat::core::String got=Integer"
;;     "victim   good  => Ok"                             ← served
;;
;; A bad caller, malicious or dumb, cannot crash anything. (Landed as a regression-proof
;; deftest in wat-tests/service-request-malformed.wat, both tiers.)
(:wat::core::defsurface :dos::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :dos::Bag::PutRequest [items <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :dos::Bag::PutResponse :wat::enum::Pure
     :Ok              [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :dos::Bag  req <- :dos::Bag::PutRequest]
     -> :dos::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :dos::bag-svc
  :satisfies :dos::Bag
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :impls
  [(put [s ctx req]
     ;; uses the field AT ITS DECLARED TYPE — correct against the declaration
     (:wat::service::Outcome::Reply s
       (:dos::Bag::PutResponse::Ok
         (:wat::core::string::length
           (:wat::core::nth (:dos::Bag::PutRequest/items req) 0)))))])

(:wat::core::defn :dos::try
  [c <- (:wat::kernel::Peer :- [:dos::Bag::Op :dos::Bag::Reply])  label <- :wat::core::String
   req <- :dos::Bag::PutRequest] -> :wat::core::nil
  (:wat::core::match (:dos::Bag/put c req)
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:dos::Bag::PutResponse::Ok n)
          (:wat::kernel::println (:wat::core::string::concat label " => Ok")))
        ((:dos::Bag::PutResponse::RequestTooLarge b cap)
          (:wat::kernel::println (:wat::core::string::concat label " => TooLarge")))
        ;; the codemod's default body for this arm is `assertion-failed!` (a terminal caller that
        ;; builds its own typed request cannot be malformed, so an unexpected refusal must be
        ;; loud). THIS probe is the one place that deliberately sends a malformed frame, so the
        ;; refusal is the expected observation and is printed with its full coordinate.
        ((:dos::Bag::PutResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::println
            (:wat::core::string::concat label
              (:wat::core::string::concat " => MALFORMED at "
                (:wat::core::string::concat (:wat::edn::write mpath)
                  (:wat::core::string::concat " expected="
                    (:wat::core::string::concat mexpected
                      (:wat::core::string::concat " got=" mgot))))))))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println (:wat::core::string::concat label " => LOST (peer gone)")))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::println (:wat::core::string::concat label " => Stopped")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println (:wat::core::string::concat label " => Closed")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:dos::bag-svc/start :locus (:wat::spawn::process) :record (:dos::bag-svc::Record :n 0))
     good (:dos::Bag::PutRequest :items (:wat::core::Vector :wat::core::String "abcd"))
     bad  (:wat::edn::read "#dos.Bag/PutRequest {:items [1 2 3]}")
     ;; ATTACKER connection
     a (:wat::core::match (:wat::kernel::connect (:dos::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))
     _ (:dos::try a "attacker good " good)
     _ (:dos::try a "attacker BAD  " bad)
     ;; a SECOND, INNOCENT client connects AFTER the bad frame
     b (:wat::core::match (:wat::kernel::connect (:dos::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "victim: connect REFUSED — service is GONE" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "victim: connect REJECTED — service is GONE" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "victim: connect FAILED — service is GONE" :wat::core::None :wat::core::None)))
     ;; The victim's call is a BINDING, not the let's tail expression. That is not cosmetic and it
     ;; is not about this stone: a service Handle bound in a `let` is dropped before the let's TAIL
     ;; body evaluates, so a request issued from tail position comes back `Closed` — the service is
     ;; being torn down under it. Reproduced on BOTH tiers against the untouched Stone-1 probe by
     ;; moving its victim call to tail position, so it is a pre-existing drop-order artifact of the
     ;; probe's own shape, NOT a failure of the wall. Kept out of the way here so the observation
     ;; below reads what it is actually measuring.
     _ (:dos::try b "victim   good " good)]
    nil))
