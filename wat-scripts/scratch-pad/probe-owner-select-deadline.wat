;; STOP-1 for the-owner-wait-has-a-deadline.
;; Coerced select on a REAL lineage handle (process-tier, transport pinned by /start).
;; Same mechanism as call-by-deadline: kind from peer-wire?, timer conj'd into a
;; Vector whose element type is Peer :- [Admin Status].
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-owner-select-deadline.wat
;; Prints the select event. Timer should fire (nothing was sent on the lineage peer).

(:wat::core::defsurface :hold::Hold :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :hold::Hold::PingRequest [])
   (:wat::core::defenum :hold::Hold::PingResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :hold::Hold  req <- :hold::Hold::PingRequest] -> :hold::Hold::PingResponse
     :max-request-bytes 65536)])

(:wat::service::defservice :hold::hold
  :satisfies :hold::Hold
  :durable []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:hold::Hold::Reply::Ping (:hold::Hold::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:hold::Hold::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:hold::hold::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     lineage (:hold::hold::Handle/handle h)
     ;; ⛔ peer-wire? REFUSES a lineage handle: runtime TypeMismatch
     ;; "expected peer (unified (Peer :- [S R])), got :wat::kernel::Process".
     ;; peer-process is the owner-handle predicate (grant-method-body uses it).
     kind (:wat::core::match (:wat::kernel::peer-process lineage)
             ((:wat::core::Some _) :wat::program::PeerKind::process)
             (:wat::core::None :wat::program::PeerKind::thread))
     inert :hold::hold::Status::PeersAllowed
     tmr (:wat::core::first
           (:wat::core::conj
             (:wat::core::Vector :- [(:wat::kernel::Peer :- [:hold::hold::Admin :hold::hold::Status])])
             (:wat::kernel::after kind (:wat::time::Milliseconds 200) inert)))
     ev (:wat::kernel::select [lineage tmr])
     _ (:wat::kernel::println ev)
     _ (:wat::service::stop-faced (:hold::hold/stop h))]
    nil))
