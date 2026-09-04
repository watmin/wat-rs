;; Same durable shape as fanout::seen, always drop-after write.
(:wat::config::set-redef! true)

(:wat::core::defsurface :ps::S :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :ps::S::ClaimRequest [seq <- :wat::core::String])
   (:wat::core::defenum :ps::S::ClaimResponse :wat::enum::Pure
     :First []
     :Dup []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(claim [self <- :ps::S  req <- :ps::S::ClaimRequest] -> :ps::S::ClaimResponse :max-request-bytes 65536)])

(:wat::service::defservice :ps::s
  :satisfies :ps::S
  :durable   [firsts <- :wat::core::i64  dups <- :wat::core::i64
              drop-after? <- :wat::core::bool]
  :ephemeral [claimed <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])]
  :init (:wat::core::fn [record <- :ps::s::Record] -> :ps::s::State
          (:ps::s::State :durable record
            :claimed (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])))
  :impls
  [(claim [s ctx req]
     (:wat::core::let
       [rec (:ps::s::State/durable s)
        claimed (:ps::s::State/claimed s)
        key (:ps::S::ClaimRequest/seq req)
        rec' (:ps::s::Record :firsts (:wat::i64::+ (:ps::s::Record/firsts rec) 1)
               :dups (:ps::s::Record/dups rec) :drop-after? true)
        s' (:ps::s::State :durable rec' :claimed (:wat::hashmap::assoc claimed key true))
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:ps::S::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:ps::s::Op])])]
       (:wat::service::Outcome::Continue s'
         :wat::core::None
         sends none-alarms)))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:ps::s/start :locus (:wat::spawn::process)
          :record (:ps::s::Record :firsts 0 :dups 0 :drop-after? true))
     p (:wat::core::match (:wat::kernel::connect (:ps::s::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "dial" :wat::core::None :wat::core::None)))
     r1 (:wat::core::match (:ps::S/claim p (:ps::S::ClaimRequest :seq "a"))
          ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
          ((:wat::kernel::RecvOutcome::Message _) "MSG")
          (_ "OTHER"))
     r2 (:wat::core::match (:wat::kernel::connect (:ps::s::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected _) "REDIAL-OK")
          (_ "REDIAL-DEAD"))]
    (:wat::kernel::println (:wat::core::format "claim={c};redial={r}" :c r1 :r r2))))
