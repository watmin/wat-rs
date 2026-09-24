;; Excursus 003 stone H — measurement (a): a PROCESS-tier `after` whose msg is a handle.
;; The timer ships `msg` as a wire frame that `select` decodes as data. What arrives?
(:wat::core::defrecord :h::Box :- [T] [x <- :T])

(:wat::core::defn :h::arm :- [T] [x <- :T] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::select
      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil (:h::Box :- [:T])])]
        (:wat::kernel::after :wat::program::PeerKind.process (:wat::time::Millisecond 5) (:h::Box :x x))))
    [:wat::spawn::ServiceEvent.Message {:idx _i :msg _m} "Message"]
    [:wat::spawn::ServiceEvent.Closed {:idx _i} "Closed"]
    [:wat::spawn::ServiceEvent.Lost {:idx _i :cause c}
      (:wat::core::format "Lost: {m}" :m (:wat::kernel::Failure/message c))]
    [:wat::spawn::ServiceEvent.Malformed {:idx _i :cause c}
      (:wat::core::format "Malformed: {m}" :m (:wat::edn::write c))]
    [:wat::spawn::ServiceEvent.Rejected {:idx _i :cause _c} "Rejected"]
    [:wat::spawn::ServiceEvent.Shutdown {} "Shutdown"]
    [:wat::spawn::ServiceEvent.Connection {:peer _p} "Connection"]
    [:wat::spawn::ServiceEvent.Admin {:msg _m} "Admin"]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru")
     _ (:wat::kernel::println (:wat::core::format "pure:   {s}" :s (:h::arm 42)))
     _ (:wat::kernel::println (:wat::core::format "handle: {s}" :s (:h::arm h)))]
    nil))
