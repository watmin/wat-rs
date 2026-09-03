;; probe-frame-cap-severs-one-conn.wat — can a CONNECTION die while the SERVICE lives?
;;
;; The reconnect stone (ARC 278, connections are re-acquirable) wired 20 Lost arms to
;; reap + re-dial + do-not-ack. It shipped honestly labelled UNEXERCISED. But nobody has
;; established the path is even REACHABLE: if every Lost means the service is gone, then
;; re-dialing its address fails forever and the recovery is dead code.
;;
;; What the substrate says (read, not assumed):
;;   src/comms/process.rs:1147  "the peer is still alive"
;;   src/comms/process.rs:1183  "FrameTooLarge distinctly so callers can tear down the peer"
;;   src/kernel/spawn.rs:273    RecvError::FrameTooLarge -> PeerDeath::Lost
;;
;; So an oversized FRAME is a candidate: it should kill one connection and leave the
;; service serving. :max-frame-bytes (the deployment's limit) is separate from
;; :max-request-bytes (the contract cap, which replies RequestTooLarge instead).
;;
;; THE TEST: two clients. Client A sends a frame over the deployment limit but under the
;; contract cap. Then client B — a SEPARATE connection — pings.
;;   a=lost ; b=ok   -> the connection died, the service lived. RECOVERY IS REACHABLE.
;;   a=lost ; b=lost -> the service died too. The reconnect path is DEAD CODE and chaos
;;                      needs a substrate change before it can measure anything.

(:wat::core::defsurface :fc::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :fc::Echo::PingRequest [pad <- :wat::core::String])
   (:wat::core::defenum :fc::Echo::PingResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :fc::Echo  req <- :fc::Echo::PingRequest]
     -> :fc::Echo::PingResponse :max-request-bytes 524288)])

;; deployment frame limit far BELOW the contract cap, so a mid-sized request passes the
;; contract guard and trips the frame limit.
(:wat::service::defservice :fc::echo
  :satisfies :fc::Echo
  :max-frame-bytes 256
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:fc::Echo::Reply::Ping (:fc::Echo::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:fc::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:fc::echo::Op])])))])

(:wat::core::defn :fc::dial
  [a <- (:wat::kernel::Address :- [:fc::Echo::Op :fc::Echo::Reply])] -> :fc::Echo
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "fc: dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :fc::pad [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc "xxxxxxxxxx"))
    ""
    (:wat::core::range 0 n)))

(:wat::core::defn :fc::hit [c <- :fc::Echo  pad <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:fc::Echo/ping c (:fc::Echo::PingRequest :pad pad))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:fc::Echo::PingResponse::Ok) "ok")
        ((:fc::Echo::PingResponse::RequestTooLarge _b _c) "too-large-reply")
        (_ "other")))
    ((:wat::kernel::RecvOutcome::Lost _c) "lost")
    (:wat::kernel::RecvOutcome::Stopped "stopped")
    (:wat::kernel::RecvOutcome::Closed "closed")))

(:wat::core::defn :fc::run [] -> :wat::core::String
  (:wat::core::let
    [h  (:fc::echo/start :locus (:wat::spawn::process) :record (:fc::echo::Record))
     a  (:fc::dial (:fc::echo::Handle/addr h))
     b  (:fc::dial (:fc::echo::Handle/addr h))
     small (:fc::hit a "")                       ;; sanity: A works first
     big   (:fc::hit a (:fc::pad 200))           ;; ~2KB: over the 256B frame cap
     other (:fc::hit b "")                       ;; the OTHER connection — service alive?
     again (:fc::hit a "")                       ;; A again — connection really dead?
     out (:wat::core::format "a-small={s};a-big={g};b-other={o};a-again={r}"
           :s small :g big :o other :r again)]
    out))

(:wat::core::defn :user::compute [] -> :wat::core::String (:fc::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:fc::run)))
