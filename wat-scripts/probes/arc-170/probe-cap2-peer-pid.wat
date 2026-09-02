;; probe-cap2-peer-pid.wat — arc 170 capability circuit, stone 2 DISCONFIRMING PROBE.
;;
;; Isolate the ONE gap: does (:wat::kernel::peer-pid p) lift the pid off a peer?
;;   process peer (connect' to a process service) -> (Some pid)
;;   thread  peer (connect' to a thread  service) -> :None
;; Pre-strike this fails on EXACTLY ":wat::kernel::peer-pid is undefined" — everything
;; around it (start the services, connect' → peers) is already green, so the whole
;; strike is "define :wat::kernel::peer-pid". EXPECT (post-strike): "(Some <pid>)" then ":None".

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req] (:wat::service::Outcome::Continue s
                          (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ── a PROCESS peer: its far end is a forked child → peer-pid should be (Some pid) ──
     ph  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     pc  (:wat::core::match (:wat::kernel::connect (:probe::echo::Handle/addr ph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _   (:wat::kernel::println "process-peer peer-pid:")
     _   (:wat::kernel::println (:wat::kernel::peer-pid pc))   ; ← THE GAP (undefined pre-strike)
     ;; ── a THREAD peer: its far end is a cell in THIS process → peer-pid should be :None ──
     th  (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))
     tc  (:wat::core::match (:wat::kernel::connect (:probe::echo::Handle/addr th)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _   (:wat::kernel::println "thread-peer peer-pid:")
     _   (:wat::kernel::println (:wat::kernel::peer-pid tc))]
    nil))
