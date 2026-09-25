;; Stone 255.29 — a thread address sent to a process child.
;; The child dials it and must see Rejected (not the minting process).
;; The echoed copy dials in the parent and must see Connected.
;; Pre-stone the child dies decoding: unsupported substrate tag ##wat.kernel/Address.
(:wat::core::defn :p29::recv-back
  [p <- (:wat::kernel::Process :- [(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])
                                   (:wat::core::Tuple :- [:wat::core::String (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])])])]
  -> (:wat::core::Tuple :- [:wat::core::String (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])])
  (:wat::core::match (:wat::kernel::recv p)
    [:wat::kernel::RecvOutcome.Message {:msg m} m]
    [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message c))]
    [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "stopped")]
    [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "child closed")]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [b  (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     a  (:wat::spawn::Bound/address b)
     p  (:wat::test::spawn-peer (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::outcome-name
              [o <- (:wat::kernel::ConnectOutcome :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::String
              (:wat::core::match o
                [:wat::kernel::ConnectOutcome.Connected {:peer _p} "Connected"]
                [:wat::kernel::ConnectOutcome.Refused {:cause c} (:wat::string::concat "Refused: " (:wat::kernel::Failure/message c))]
                [:wat::kernel::ConnectOutcome.Rejected {:cause c} (:wat::string::concat "Rejected: " (:wat::kernel::Failure/message c))]
                [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::string::concat "Failed: " (:wat::kernel::Failure/message c))]))
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::let
                [sp (:wat::program::self-peer
                      (:wat::core::Tuple :- [:wat::core::String (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])])
                      (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]))]
                (:wat::core::match (:wat::kernel::recv sp)
                  [:wat::kernel::RecvOutcome.Message {:msg addr}
                    (:wat::core::match (:wat::kernel::send sp
                                         (:wat::core::Tuple (:user::outcome-name (:wat::kernel::connect addr)) addr))
                      [:wat::kernel::SendOutcome.Sent {} nil]
                      [:wat::kernel::SendOutcome.Closed {} nil]
                      [:wat::kernel::SendOutcome.Stopped {} nil]
                      [:wat::kernel::SendOutcome.Lost {:cause _c} nil])]
                  [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message c))]
                  [:wat::kernel::RecvOutcome.Stopped {} nil]
                  [:wat::kernel::RecvOutcome.Closed {} nil])))))
     _s (:wat::core::match (:wat::kernel::send p a)
          [:wat::kernel::SendOutcome.Sent {} nil]
          [:wat::kernel::SendOutcome.Closed {} (:wat::kernel::assertion-failed! :message "child closed")]
          [:wat::kernel::SendOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "stopped")]
          [:wat::kernel::SendOutcome.Lost {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message c))])
     back (:p29::recv-back p)
     parent (:wat::core::match (:wat::kernel::connect (:wat::core::second back))
              [:wat::kernel::ConnectOutcome.Connected {:peer _p} "Connected"]
              [:wat::kernel::ConnectOutcome.Refused {:cause c} (:wat::string::concat "Refused: " (:wat::kernel::Failure/message c))]
              [:wat::kernel::ConnectOutcome.Rejected {:cause c} (:wat::string::concat "Rejected: " (:wat::kernel::Failure/message c))]
              [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::string::concat "Failed: " (:wat::kernel::Failure/message c))])
     _keep (:wat::spawn::Bound/listener b)]
    (:wat::kernel::println
      (:wat::core::Vector :- [:wat::core::String] (:wat::core::first back) parent))))
