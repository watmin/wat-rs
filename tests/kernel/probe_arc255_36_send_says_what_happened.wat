;; Stone 255.36 — a send to a far end that has left.
;; The client peer is minted, then the listener (holding the connect-request,
;; and therefore the server's receiver) is dropped with the Bound.
;; Pre-stone both loci printed Lost. A thread send never yields Stopped:
;; comms/thread.rs maps every crossbeam send error to Disconnected.
(:wat::core::defn :user::show
  [label <- :wat::core::String
   o <- :wat::kernel::SendOutcome]
  -> :wat::core::nil
  (:wat::core::match o
    [:wat::kernel::SendOutcome.Sent {}
      (:wat::kernel::println (:wat::string::concat label " Sent"))]
    [:wat::kernel::SendOutcome.HandleClosed {}
      (:wat::kernel::println (:wat::string::concat label " HandleClosed"))]
    [:wat::kernel::SendOutcome.Closed {:cause c}
      (:wat::kernel::println (:wat::string::concat label " Closed " (:wat::kernel::Failure/message c)))]
    [:wat::kernel::SendOutcome.Stopped {}
      (:wat::kernel::println (:wat::string::concat label " Stopped"))]
    [:wat::kernel::SendOutcome.Failed {:cause c}
      (:wat::kernel::println (:wat::string::concat label " Failed " (:wat::kernel::Failure/message c)))]))

(:wat::core::defn :user::thread-client [] -> (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [b (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     a (:wat::spawn::Bound/address b)]
    (:wat::core::match (:wat::kernel::connect a)
      [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
      [:wat::kernel::ConnectOutcome.Closed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
      [:wat::kernel::ConnectOutcome.Undialable {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
      [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
      [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::show "thread-far" (:wat::kernel::send (:user::thread-client) 1))
    (:wat::core::let
      [p (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil nil)))]
      (:wat::core::match (:wat::kernel::recv p)
        [:wat::kernel::RecvOutcome.Message {:msg _m} nil]
        [:wat::kernel::RecvOutcome.Lost {:cause _c} nil]
        [:wat::kernel::RecvOutcome.Stopped {} nil]
        [:wat::kernel::RecvOutcome.Closed {} nil])
      (:user::show "process-far" (:wat::kernel::send p 1)))))
