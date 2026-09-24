;; Excursus 003 stone H — measurement (b): a THREAD-tier `try-send` of values a thread peer may carry
;; but the EDN writer cannot encode (a source-macro holon) or can only render as a nil-bodied tag
;; (an Lru). A thread peer carries any value (RULING-purity-is-parametric) — does try-send raise?
(:wat::core::defn :h::try-holon [] -> :wat::core::String
  (:wat::core::let
    [bound (:wat::kernel::listener (:wat::spawn::thread) :wat::holon::HolonAST :wat::holon::HolonAST)
     lis   (:wat::spawn::Bound/listener bound)
     addr  (:wat::spawn::Bound/address bound)
     tx    (:wat::core::match (:wat::kernel::connect addr)
             [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
             [:wat::kernel::ConnectOutcome.Refused {:cause _c} (:wat::kernel::assertion-failed! :message "refused")]
             [:wat::kernel::ConnectOutcome.Rejected {:cause _c} (:wat::kernel::assertion-failed! :message "rejected")]
             [:wat::kernel::ConnectOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])
     _rx   (:wat::core::match (:wat::kernel::accept lis)
             [:wat::kernel::AcceptOutcome.Accepted {:peer p} p]
             [:wat::kernel::AcceptOutcome.Closed {} (:wat::kernel::assertion-failed! :message "closed")]
             [:wat::kernel::AcceptOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])]
    (:wat::core::match (:wat::kernel::try-send tx #holon [1 2 3])
      [:wat::kernel::TrySendOutcome.Sent {} "Sent"]
      [:wat::kernel::TrySendOutcome.WouldBlock {} "WouldBlock"]
      [:wat::kernel::TrySendOutcome.Closed {} "Closed"]
      [:wat::kernel::TrySendOutcome.Lost {:cause _c} "Lost"])))

(:wat::core::defn :h::send-holon [] -> :wat::core::String
  (:wat::core::let
    [bound (:wat::kernel::listener (:wat::spawn::thread) :wat::holon::HolonAST :wat::holon::HolonAST)
     lis   (:wat::spawn::Bound/listener bound)
     addr  (:wat::spawn::Bound/address bound)
     tx    (:wat::core::match (:wat::kernel::connect addr)
             [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
             [:wat::kernel::ConnectOutcome.Refused {:cause _c} (:wat::kernel::assertion-failed! :message "refused")]
             [:wat::kernel::ConnectOutcome.Rejected {:cause _c} (:wat::kernel::assertion-failed! :message "rejected")]
             [:wat::kernel::ConnectOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])
     _rx   (:wat::core::match (:wat::kernel::accept lis)
             [:wat::kernel::AcceptOutcome.Accepted {:peer p} p]
             [:wat::kernel::AcceptOutcome.Closed {} (:wat::kernel::assertion-failed! :message "closed")]
             [:wat::kernel::AcceptOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])]
    (:wat::core::match (:wat::kernel::send tx #holon [1 2 3])
      [:wat::kernel::SendOutcome.Sent {} "Sent"]
      [:wat::kernel::SendOutcome.Stopped {} "Stopped"]
      [:wat::kernel::SendOutcome.Closed {} "Closed"]
      [:wat::kernel::SendOutcome.Lost {:cause _c} "Lost"])))

(:wat::core::defrecord :h::Box :- [T] [x <- :T])

;; A generic fn: T is a parameter at the connect, so the compile-time wall lets it through.
(:wat::core::defn :h::try-box :- [T] [x <- :T] -> :wat::core::String
  (:wat::core::let
    [bound (:wat::kernel::listener (:wat::spawn::thread) (:h::Box :- [:T]) :wat::core::i64)
     lis   (:wat::spawn::Bound/listener bound)
     addr  (:wat::spawn::Bound/address bound)
     tx    (:wat::core::match (:wat::kernel::connect addr)
             [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
             [:wat::kernel::ConnectOutcome.Refused {:cause _c} (:wat::kernel::assertion-failed! :message "refused")]
             [:wat::kernel::ConnectOutcome.Rejected {:cause _c} (:wat::kernel::assertion-failed! :message "rejected")]
             [:wat::kernel::ConnectOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])
     _rx   (:wat::core::match (:wat::kernel::accept lis)
             [:wat::kernel::AcceptOutcome.Accepted {:peer p} p]
             [:wat::kernel::AcceptOutcome.Closed {} (:wat::kernel::assertion-failed! :message "closed")]
             [:wat::kernel::AcceptOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])]
    (:wat::core::match (:wat::kernel::try-send tx (:h::Box :x x))
      [:wat::kernel::TrySendOutcome.Sent {} "Sent"]
      [:wat::kernel::TrySendOutcome.WouldBlock {} "WouldBlock"]
      [:wat::kernel::TrySendOutcome.Closed {} "Closed"]
      [:wat::kernel::TrySendOutcome.Lost {:cause _c} "Lost"])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:wat::core::format "thread send holon:     {s}" :s (:h::send-holon)))
     _ (:wat::kernel::println (:wat::core::format "thread try-send Box<Lru>: {s}" :s (:h::try-box (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru"))))
     _ (:wat::kernel::println (:wat::core::format "thread try-send holon: {s}" :s (:h::try-holon)))]
    nil))
