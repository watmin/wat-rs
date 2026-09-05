;; probe-generated-method-against-silent-peer.wat — row 1 of "no client call can hang".
;; A generated SURFACE method (`:dp::Silent/wait`) against a peer that accepts
;; and never replies. Before: hangs forever. After: RecvOutcome::TimedOut ~10 s.

(:wat::config::set-redef! true)

(:wat::core::defsurface :dp::Silent :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :dp::Silent::WaitRequest [])
   (:wat::core::defenum :dp::Silent::WaitResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(wait [self <- :dp::Silent  req <- :dp::Silent::WaitRequest]
     -> :dp::Silent::WaitResponse :max-request-bytes 65536)])

(:wat::service::defservice :dp::silent
  :satisfies :dp::Silent
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :dp::silent::Record] -> :dp::silent::State
          (:dp::silent::State :durable record))
  :impls
  [(wait [s ctx req]
     (:wat::service::Outcome::Continue s
       :wat::core::None
       (:wat::core::Vector :- [(:wat::service::Directed :- [:dp::Silent::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:dp::silent::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:dp::silent/start :locus (:wat::spawn::process)
          :record (:dp::silent::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:dp::silent::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     r (:dp::Silent/wait p (:dp::Silent::WaitRequest))]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message _m) (:wat::kernel::println "UNEXPECTED-MESSAGE"))
      ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::println "UNEXPECTED-LOST"))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "UNEXPECTED-STOPPED"))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::println "UNEXPECTED-CLOSED"))
      (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::println "TIMED-OUT")))))
