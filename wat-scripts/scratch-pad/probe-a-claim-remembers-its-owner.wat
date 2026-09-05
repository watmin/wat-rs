;; probe-a-claim-remembers-its-owner.wat — can a claim ledger answer "is this MINE?"
;;
;; Arc 278. Under drop-after, six runs: seen-firsts=100 every time, total in {89..92}.
;; Every message was claimed First exactly once, and ~10 of those First-claimers never
;; emitted an outcome. No worker died in the completing runs.
;;
;; The mechanism: worker A claims -> First -> ledger written -> reply DROPPED. A times out
;; (200 ms), redials, retries. The ledger already holds the seq, so A is told Dup. A's rule is
;; `emit if first?` (circuit.wat:491), so A emits nothing -- and A was the only claimant.
;;
;; ⛔ THE INFORMATION LOSS IS ONE WORD. circuit.wat:82 --
;;      claimed <- (HashMap :- [String bool])
;; The ledger records THAT a seq was claimed and discards WHO claimed it, so "someone else
;; owns this" and "I own this and did not hear back" are the same answer. They are different
;; facts and the caller needs them apart.
;;
;; THE DISCONFIRMING QUESTION -- is the distinction EXPRESSIBLE at all?
;; A three-arm reply is only worth briefing if a service can compute it. The server knows the
;; stored owner and the requesting owner; it should return the ANSWER, not the data to compute
;; it -- otherwise a caller can ignore the owner field and we are back to today.
;;
;;   First    -- nobody held it; you do now
;;   DupSelf  -- YOU already hold it (your earlier claim landed; its reply did not)
;;   DupOther -- someone else holds it; stand down
;;
;; Cells, on one ledger:
;;   claim(seq=1, owner=A)  -> First
;;   claim(seq=1, owner=A)  -> DupSelf     <- the arm that does not exist today
;;   claim(seq=1, owner=B)  -> DupOther
;;   claim(seq=2, owner=B)  -> First
;;
;; ⚠ WHAT THIS DOES NOT PROVE. That the circuit's stranded messages take the DupSelf path is
;; INFERRED (seen-firsts=100 AND no worker died AND total<100), not observed. The EXPECTATIONS
;; row that converts it to observed is `total` returning to 100 under drop-after. If it does
;; not, this mechanism is wrong and the stone is refuted -- which is the point of saying so here.

(:wat::config::set-redef! true)

(:wat::core::defsurface :co::Ledger :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :co::Ledger::ClaimRequest
     [seq   <- :wat::core::String
      owner <- :wat::core::String])
   (:wat::core::defenum :co::Ledger::ClaimResponse :wat::enum::Pure
     :First    []
     :DupSelf  []
     :DupOther []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(claim [self <- :co::Ledger  req <- :co::Ledger::ClaimRequest]
     -> :co::Ledger::ClaimResponse :max-request-bytes 524288)])

(:wat::service::defservice :co::ledger
  :satisfies :co::Ledger
  :durable   [firsts <- :wat::core::i64]
  ;; the whole stone in one type: String, not bool. The owner IS the fact.
  :ephemeral [held <- (:wat::core::HashMap :- [:wat::core::String :wat::core::String])]
  :init (:wat::core::fn [record <- :co::ledger::Record] -> :co::ledger::State
          (:co::ledger::State :durable record
            :held (:wat::core::HashMap :- [:wat::core::String :wat::core::String])))
  :impls
  [(claim [s ctx req]
     (:wat::core::let
       [seq   (:co::Ledger::ClaimRequest/seq req)
        owner (:co::Ledger::ClaimRequest/owner req)
        held  (:co::ledger::State/held s)
        rec   (:co::ledger::State/durable s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:co::Ledger::Reply])])
        alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:co::ledger::Op])])
        prior (:wat::hashmap::get held seq)
        resp (:wat::core::match prior
               ((:wat::core::Some who)
                 (:wat::core::if (:wat::core::= who owner)
                   (:co::Ledger::ClaimResponse::DupSelf)
                   (:co::Ledger::ClaimResponse::DupOther)))
               (:wat::core::None (:co::Ledger::ClaimResponse::First)))
        held' (:wat::core::match prior
                ((:wat::core::Some _w) held)
                (:wat::core::None (:wat::hashmap::assoc held seq owner)))
        firsts' (:wat::core::match prior
                  ((:wat::core::Some _w) (:co::ledger::Record/firsts rec))
                  (:wat::core::None (:wat::i64::+ (:co::ledger::Record/firsts rec) 1)))
        s' (:co::ledger::State :durable (:co::ledger::Record :firsts firsts') :held held')]
       (:wat::service::Outcome::Continue s'
         (:wat::core::Some (:co::Ledger::Reply::Claim resp)) sends alarms)))])

(:wat::core::defn :co::say [l <- :co::Ledger  seq <- :wat::core::String  owner <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::match (:co::Ledger/claim l (:co::Ledger::ClaimRequest :seq seq :owner owner))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:co::Ledger::ClaimResponse::First) "First")
        ((:co::Ledger::ClaimResponse::DupSelf) "DupSelf")
        ((:co::Ledger::ClaimResponse::DupOther) "DupOther")
        (_ "malformed")))
    (_ "LOST")))

(:wat::core::defn :co::run [] -> :wat::core::String
  (:wat::core::let
    [h (:co::ledger/start :locus (:wat::spawn::thread)
         :record (:co::ledger::Record :firsts 0))
     l (:wat::core::match (:wat::kernel::connect (:co::ledger::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "co: dial failed" :wat::core::None :wat::core::None)))
     a1 (:co::say l "1" "A")
     a2 (:co::say l "1" "A")
     b1 (:co::say l "1" "B")
     b2 (:co::say l "2" "B")]
    (:wat::core::format "A-first={a};A-again={b};B-same-seq={c};B-new-seq={d};discriminates={e}"
      :a a1 :b a2 :c b1 :d b2
      :e (:wat::core::if
           (:wat::core::and (:wat::core::= a1 "First")
             (:wat::core::and (:wat::core::= a2 "DupSelf")
               (:wat::core::and (:wat::core::= b1 "DupOther") (:wat::core::= b2 "First"))))
           "yes" "NO"))))

(:wat::core::defn :user::compute [] -> :wat::core::String (:co::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:co::run)))
