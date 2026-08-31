;; Co-located fixture for probe_severed_reaches_the_client.rs.
;;
;; A service whose OWNER releases its handle is not the same event as a peer that closed cleanly,
;; and the client must be able to tell them apart. This drives the distinction end to end and
;; returns which one arrived, as a string, so a RED names the wrong outcome instead of just failing.
;;
;; The owner-drop is produced the way a caller actually trips it, not by a special teardown call:
;; `h` is bound in a `let` and the drive sits in the let's BODY, which is TAIL position. The scope
;; is released when the tail call leaves it, so the handle dies while still lexically in scope and
;; the serve loop severs. `:sched::severed-is-named` below is the SAME code with the drive moved
;; into a BINDING — the one-variable control proving this fixture measures the sever and not merely
;; "a service that stopped".

(:wat::core::defsurface :sev::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :sev::Echo::PingRequest [])
   (:wat::core::defenum :sev::Echo::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :sev::Echo  req <- :sev::Echo::PingRequest] -> :sev::Echo::PingResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :sev::echo
  :satisfies :sev::Echo
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :sev::echo::Record] -> :sev::echo::State
          (:sev::echo::State :durable record))
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Reply s (:sev::Echo::PingResponse::Pong)))])

(:wat::core::defn :sev::conn
  [h <- :sev::echo::Handle] -> (:wat::kernel::Peer :- [:sev::Echo::Op :sev::Echo::Reply])
  (:wat::core::match (:wat::kernel::connect (:sev::echo::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; name the outcome. Every LociDiedError variant is spelled out rather than wildcarded: a `_` here
;; would let a future variant silently pass this gate, which is the opposite of its purpose.
(:wat::core::defn :sev::ping-outcome
  [c <- (:wat::kernel::Peer :- [:sev::Echo::Op :sev::Echo::Reply])] -> :wat::core::String
  (:wat::core::match (:sev::Echo/ping c (:sev::Echo::PingRequest))
    ((:wat::kernel::RecvOutcome::Message __recv) "REPLIED")
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::core::match cause
        (:wat::kernel::LociDiedError::Severed "SEVERED")
        (:wat::kernel::LociDiedError::Disconnected "LOST:Disconnected")
        (:wat::kernel::LociDiedError::Stopped "LOST:Stopped")
        ((:wat::kernel::LociDiedError::Panic _m _f) "LOST:Panic")
        ((:wat::kernel::LociDiedError::RuntimeError _m) "LOST:RuntimeError")
        ((:wat::kernel::LociDiedError::StartupError _m) "LOST:StartupError")
        ((:wat::kernel::LociDiedError::EntryFormFailure _m) "LOST:EntryFormFailure")
        ((:wat::kernel::LociDiedError::MainSignature _m) "LOST:MainSignature")
        ((:wat::kernel::LociDiedError::BadReturn _m) "LOST:BadReturn")))
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED:MUTE")))

;; THE SUBJECT — the drive is in the let's BODY (tail position), so `h` is released before it runs.
;; Expect "SEVERED". Before this capability existed the client read "CLOSED:MUTE".
(:wat::core::defn :user::owner-drop-is-named [] -> :wat::core::String
  (:wat::core::let
    [h (:sev::echo/start :locus (:wat::spawn::thread) :record (:sev::echo::Record :n 0))
     c (:sev::conn h)]
    (:sev::ping-outcome c)))

;; THE CONTROL — byte-identical but for the drive sitting in a BINDING, so the handle outlives it.
;; Expect "REPLIED". If this ever returns "SEVERED" the fixture has stopped discriminating and the
;; subject above proves nothing.
(:wat::core::defn :user::held-handle-still-replies [] -> :wat::core::String
  (:wat::core::let
    [h (:sev::echo/start :locus (:wat::spawn::thread) :record (:sev::echo::Record :n 0))
     c (:sev::conn h)
     r (:sev::ping-outcome c)]
    r))
